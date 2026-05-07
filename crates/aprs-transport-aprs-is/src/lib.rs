#![forbid(unsafe_code)]

//! APRS-IS transport helpers.
//!
//! This crate keeps APRS-IS connection framing outside the core parser crate.
//! It provides login-line construction and reader-backed packet splitting while
//! preserving packet bytes.

use std::io::{self, Read};

use libaprs_engine::{read_all_with_limit, LineTransport, MAX_PACKET_LEN};

/// Default maximum APRS-IS reader batch size.
pub const DEFAULT_MAX_APRS_IS_READ_BYTES: usize = 1024 * 1024;

/// APRS-IS login settings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AprsIsLogin<'a> {
    /// Login callsign.
    pub callsign: &'a str,
    /// APRS-IS passcode, or `-1` for receive-only/no-auth use.
    pub passcode: i32,
    /// Client software/version identifier.
    pub software: &'a str,
    /// Optional APRS-IS filter expression.
    pub filter: Option<&'a str>,
}

impl AprsIsLogin<'_> {
    /// Builds the APRS-IS login line terminated with CRLF.
    ///
    /// Values containing CR, LF, or other ASCII control bytes are rejected to
    /// prevent control-line injection. Prefer [`Self::profile_line`] when
    /// fields may come from untrusted input.
    pub fn line(&self) -> Result<String, AprsIsLoginError> {
        validate_login_field("callsign", self.callsign)?;
        validate_login_field("software", self.software)?;
        if let Some(filter) = self.filter {
            validate_login_field("filter", filter)?;
        }

        let mut line = format!(
            "user {} pass {} vers {}",
            self.callsign, self.passcode, self.software
        );
        if let Some(filter) = self.filter {
            line.push_str(" filter ");
            line.push_str(filter);
        }
        line.push_str("\r\n");
        Ok(line)
    }

    /// Builds a profile-validated APRS-IS login line terminated with CRLF.
    ///
    /// This stricter helper enforces uppercase AX.25-like login callsigns and
    /// conservative filter syntax while leaving authentication and connection
    /// ownership to the caller.
    pub fn profile_line(&self) -> Result<String, AprsIsProfileError> {
        validate_aprs_is_callsign(self.callsign)?;
        validate_profile_field("software", self.software)?;
        if let Some(filter) = self.filter {
            validate_aprs_is_filter(filter)?;
        }

        let mut line = format!(
            "user {} pass {} vers {}",
            self.callsign, self.passcode, self.software
        );
        if let Some(filter) = self.filter {
            line.push_str(" filter ");
            line.push_str(filter);
        }
        line.push_str("\r\n");
        Ok(line)
    }
}

/// APRS-IS login line validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AprsIsLoginError {
    /// A login field contains CR, LF, or another ASCII control byte.
    LineInjection { field: &'static str },
}

impl AprsIsLoginError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::LineInjection { .. } => "aprs_is_login_line_injection",
        }
    }
}

impl std::fmt::Display for AprsIsLoginError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LineInjection { field } => write!(
                formatter,
                "APRS-IS login field contains CR, LF, or control byte: {field}"
            ),
        }
    }
}

impl std::error::Error for AprsIsLoginError {}

/// APRS-IS profile validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AprsIsProfileError {
    /// A profile field contains CR, LF, or another ASCII control byte.
    LineInjection { field: &'static str },
    /// A callsign does not fit the conservative AX.25-like login shape.
    InvalidCallsign,
    /// A callsign contains lowercase letters.
    LowercaseCallsign,
    /// A filter expression is empty, malformed, or contains unsupported bytes.
    InvalidFilter,
}

impl AprsIsProfileError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::LineInjection { .. } => "aprs_is_profile_line_injection",
            Self::InvalidCallsign => "aprs_is_profile_invalid_callsign",
            Self::LowercaseCallsign => "aprs_is_profile_lowercase_callsign",
            Self::InvalidFilter => "aprs_is_profile_invalid_filter",
        }
    }
}

impl std::fmt::Display for AprsIsProfileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LineInjection { field } => {
                write!(
                    formatter,
                    "APRS-IS profile field contains CR, LF, or control byte: {field}"
                )
            }
            Self::InvalidCallsign => formatter.write_str("APRS-IS profile callsign is invalid"),
            Self::LowercaseCallsign => {
                formatter.write_str("APRS-IS profile callsign must be uppercase")
            }
            Self::InvalidFilter => formatter.write_str("APRS-IS profile filter is invalid"),
        }
    }
}

impl std::error::Error for AprsIsProfileError {}

/// Validated APRS-IS server-side filter expression.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AprsIsFilter<'a> {
    expression: &'a str,
}

impl<'a> AprsIsFilter<'a> {
    /// Validates an APRS-IS filter expression without normalizing it.
    pub fn new(expression: &'a str) -> Result<Self, AprsIsProfileError> {
        validate_aprs_is_filter(expression)?;
        Ok(Self { expression })
    }

    /// Returns the original filter expression exactly as provided.
    #[must_use]
    pub const fn as_str(&self) -> &'a str {
        self.expression
    }
}

/// APRS-IS q-construct classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AprsIsQConstructKind {
    /// `qAC`: verified bidirectional login.
    VerifiedLogin,
    /// `qAX`: unverified login; deprecated on APRS-IS.
    UnverifiedLoginDeprecated,
    /// `qAU`: direct via UDP.
    DirectUdp,
    /// `qAo`: gated packet via client-only port.
    GatedClientOnly,
    /// `qAO`: non-gated/send-only packet.
    SendOnlyOrNonGated,
    /// `qAS`: server-generated or server-forwarded packet.
    Server,
    /// `qAr`: gated packet from a remote IGate.
    RemoteIgate,
    /// `qAR`: verified IGate or client-generated gated RF packet.
    VerifiedIgate,
    /// `qAZ`: server/client command packet.
    Command,
    /// `qAI`: trace packet.
    Trace,
    /// A syntactically valid but currently unknown q construct.
    Unknown,
}

impl AprsIsQConstructKind {
    /// Stable machine-readable q-construct code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::VerifiedLogin => "qac",
            Self::UnverifiedLoginDeprecated => "qax",
            Self::DirectUdp => "qau",
            Self::GatedClientOnly => "qao_client_only",
            Self::SendOnlyOrNonGated => "qao_send_only_or_non_gated",
            Self::Server => "qas",
            Self::RemoteIgate => "qar_remote_igate",
            Self::VerifiedIgate => "qar_verified_igate",
            Self::Command => "qaz",
            Self::Trace => "qai",
            Self::Unknown => "q_unknown",
        }
    }
}

/// APRS-IS q-construct diagnostic over raw TNC2 monitor bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AprsIsQConstruct<'a> {
    /// The q-construct path component bytes.
    pub component: &'a [u8],
    /// The path component after the q construct, usually a server or IGate.
    pub next_component: Option<&'a [u8]>,
    /// Classified q-construct kind.
    pub kind: AprsIsQConstructKind,
}

/// Validates a conservative uppercase APRS-IS login callsign.
pub fn validate_aprs_is_callsign(callsign: &str) -> Result<(), AprsIsProfileError> {
    let bytes = callsign.as_bytes();
    if bytes.iter().any(u8::is_ascii_lowercase) {
        return Err(AprsIsProfileError::LowercaseCallsign);
    }

    let (base, ssid) = match bytes.iter().position(|byte| *byte == b'-') {
        Some(separator) => (&bytes[..separator], Some(&bytes[separator + 1..])),
        None => (bytes, None),
    };

    if !(1..=6).contains(&base.len())
        || !base
            .iter()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    {
        return Err(AprsIsProfileError::InvalidCallsign);
    }

    if let Some(ssid) = ssid {
        if ssid.is_empty() || ssid.len() > 2 || !ssid.iter().all(u8::is_ascii_digit) {
            return Err(AprsIsProfileError::InvalidCallsign);
        }

        let mut parsed = 0u8;
        for digit in ssid {
            parsed = parsed * 10 + (digit - b'0');
        }
        if parsed > 15 {
            return Err(AprsIsProfileError::InvalidCallsign);
        }
    }

    Ok(())
}

/// Validates an APRS-IS server-side filter expression without normalization.
pub fn validate_aprs_is_filter(filter: &str) -> Result<(), AprsIsProfileError> {
    validate_profile_field("filter", filter)?;

    let bytes = filter.as_bytes();
    if bytes.is_empty()
        || bytes.first() == Some(&b' ')
        || bytes.last() == Some(&b' ')
        || bytes.windows(2).any(|pair| pair == b"  ")
        || bytes
            .iter()
            .any(|byte| !(0x21..=0x7e).contains(byte) && *byte != b' ')
    {
        return Err(AprsIsProfileError::InvalidFilter);
    }

    Ok(())
}

/// Finds the first APRS-IS q construct in raw TNC2 monitor-format bytes.
#[must_use]
pub fn q_construct_from_tnc2(input: &[u8]) -> Option<AprsIsQConstruct<'_>> {
    let source_end = input.iter().position(|byte| *byte == b'>')?;
    let payload_separator = input[source_end + 1..]
        .iter()
        .position(|byte| *byte == b':')
        .map(|offset| source_end + 1 + offset)?;
    if source_end + 1 >= payload_separator {
        return None;
    }

    let path = &input[source_end + 1..payload_separator];
    let mut components = path.split(|byte| *byte == b',').peekable();
    while let Some(component) = components.next() {
        if component.len() == 3 && component[0] == b'q' {
            return Some(AprsIsQConstruct {
                component,
                next_component: components.peek().copied(),
                kind: classify_q_construct(component),
            });
        }
    }

    None
}

/// Reads newline-separated APRS-IS packet bytes from a generic reader.
pub fn read_packet_lines_from_reader(reader: impl Read) -> io::Result<Vec<Vec<u8>>> {
    read_packet_lines_from_reader_with_limit(reader, DEFAULT_MAX_APRS_IS_READ_BYTES)
}

/// Reads APRS-IS packet bytes from a reader with an explicit byte limit.
pub fn read_packet_lines_from_reader_with_limit(
    reader: impl Read,
    max_bytes: usize,
) -> io::Result<Vec<Vec<u8>>> {
    let input = read_all(reader, max_bytes)?;
    try_read_packet_lines(&input)
}

/// Splits newline-separated APRS-IS packet bytes into owned packet lines.
#[must_use]
pub fn read_packet_lines(input: &[u8]) -> Vec<Vec<u8>> {
    LineTransport::new(input)
        .packets()
        .into_iter()
        .filter(|line| !line.starts_with(b"#"))
        .map(<[u8]>::to_vec)
        .collect()
}

/// Splits APRS-IS packet bytes while enforcing the APRS packet limit.
pub fn try_read_packet_lines(input: &[u8]) -> io::Result<Vec<Vec<u8>>> {
    Ok(LineTransport::new(input)
        .packets_with_limit(MAX_PACKET_LEN)?
        .into_iter()
        .filter(|line| !line.starts_with(b"#"))
        .map(<[u8]>::to_vec)
        .collect())
}

fn read_all(reader: impl Read, max_bytes: usize) -> io::Result<Vec<u8>> {
    read_all_with_limit(reader, max_bytes)
}

fn validate_profile_field(field: &'static str, value: &str) -> Result<(), AprsIsProfileError> {
    if value.as_bytes().iter().any(u8::is_ascii_control) {
        return Err(AprsIsProfileError::LineInjection { field });
    }
    Ok(())
}

fn validate_login_field(field: &'static str, value: &str) -> Result<(), AprsIsLoginError> {
    if value.as_bytes().iter().any(u8::is_ascii_control) {
        return Err(AprsIsLoginError::LineInjection { field });
    }
    Ok(())
}

fn classify_q_construct(component: &[u8]) -> AprsIsQConstructKind {
    match component {
        b"qAC" => AprsIsQConstructKind::VerifiedLogin,
        b"qAX" => AprsIsQConstructKind::UnverifiedLoginDeprecated,
        b"qAU" => AprsIsQConstructKind::DirectUdp,
        b"qAo" => AprsIsQConstructKind::GatedClientOnly,
        b"qAO" => AprsIsQConstructKind::SendOnlyOrNonGated,
        b"qAS" => AprsIsQConstructKind::Server,
        b"qAr" => AprsIsQConstructKind::RemoteIgate,
        b"qAR" => AprsIsQConstructKind::VerifiedIgate,
        b"qAZ" => AprsIsQConstructKind::Command,
        b"qAI" => AprsIsQConstructKind::Trace,
        _ => AprsIsQConstructKind::Unknown,
    }
}
