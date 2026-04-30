#![forbid(unsafe_code)]

//! Protocol-first APRS engine core primitives.
//!
//! The codec boundary accepts untrusted bytes, preserves them exactly, and
//! fails closed when the packet shape is malformed.

mod diagnostic;
mod transport;

#[cfg(feature = "serde")]
pub mod serde_support;

pub use transport::{
    oversized_input_error, read_all_with_limit, LineTransport, PacketSink, PacketSource,
    TransportErrorCode, DEFAULT_TRANSPORT_READ_LIMIT,
};

/// Conservative upper bound for an APRS packet handled by this skeleton.
pub const MAX_PACKET_LEN: usize = 512;

/// Default parse options used by [`parse_packet`].
pub const DEFAULT_PARSE_OPTIONS: ParseOptions = ParseOptions {
    max_packet_len: MAX_PACKET_LEN,
};

/// Codec configuration for consumers that need a different envelope limit.
///
/// The parser remains fail-closed regardless of this setting. This value only
/// changes the maximum accepted packet length.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParseOptions {
    /// Maximum accepted packet length in bytes.
    pub max_packet_len: usize,
}

impl ParseOptions {
    /// Creates parse options with a custom maximum packet length.
    #[must_use]
    pub const fn new(max_packet_len: usize) -> Self {
        Self { max_packet_len }
    }
}

impl Default for ParseOptions {
    fn default() -> Self {
        DEFAULT_PARSE_OPTIONS
    }
}

/// Original packet bytes retained without normalization or lossy conversion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawPacket {
    bytes: Vec<u8>,
}

impl RawPacket {
    /// Returns the original packet bytes exactly as supplied to the parser.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Structured packet view backed by the preserved raw packet bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedPacket {
    raw: RawPacket,
    source_end: usize,
    path_start: usize,
    path_end: usize,
    path_components: Vec<(usize, usize)>,
    payload_start: usize,
}

impl ParsedPacket {
    /// Returns the preserved raw packet.
    #[must_use]
    pub fn raw(&self) -> &RawPacket {
        &self.raw
    }

    /// Returns the source callsign bytes before the `>` separator.
    #[must_use]
    pub fn source(&self) -> &[u8] {
        &self.raw.bytes[..self.source_end]
    }

    /// Returns the destination/path bytes between `>` and `:`.
    #[must_use]
    pub fn path(&self) -> &[u8] {
        &self.raw.bytes[self.path_start..self.path_end]
    }

    /// Returns the destination bytes, which are the first path component.
    #[must_use]
    pub fn destination(&self) -> &[u8] {
        let (start, end) = self.path_components[0];
        &self.raw.bytes[start..end]
    }

    /// Returns digipeater path component byte views after the destination.
    #[must_use]
    pub fn digipeaters(&self) -> Vec<&[u8]> {
        self.path_components[1..]
            .iter()
            .map(|(start, end)| &self.raw.bytes[*start..*end])
            .collect()
    }

    /// Returns all path component byte views, including destination first.
    #[must_use]
    pub fn path_components(&self) -> Vec<&[u8]> {
        self.path_components
            .iter()
            .map(|(start, end)| &self.raw.bytes[*start..*end])
            .collect()
    }

    /// Returns the payload bytes after the `:` separator.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.raw.bytes[self.payload_start..]
    }

    /// Returns the APRS data type identifier from the first payload byte.
    #[must_use]
    pub fn data_type_identifier(&self) -> DataTypeIdentifier {
        DataTypeIdentifier::from_byte(self.raw.bytes[self.payload_start])
    }

    /// Returns payload bytes after the data type identifier.
    #[must_use]
    pub fn information(&self) -> &[u8] {
        &self.raw.bytes[self.payload_start + 1..]
    }

    /// Returns a semantic view of the APRS information field.
    #[must_use]
    pub fn aprs_data(&self) -> AprsData<'_> {
        parse_aprs_data(
            self.data_type_identifier(),
            self.information(),
            self.destination(),
        )
    }

    /// Returns a structured diagnostic summary for observability.
    #[must_use]
    pub fn summary(&self) -> PacketSummary<'_> {
        PacketSummary::from_packet(self)
    }

    /// Serializes the parsed packet into a compact JSON diagnostic string.
    #[must_use]
    pub fn to_json(&self) -> String {
        diagnostic::packet_to_json(self)
    }
}

/// Structured packet diagnostic summary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PacketSummary<'a> {
    /// Source address bytes.
    pub source: &'a [u8],
    /// Destination address bytes.
    pub destination: &'a [u8],
    /// APRS data type identifier name.
    pub data_type: &'static str,
    /// APRS semantic kind name.
    pub semantic: &'static str,
    /// Decoded coordinates when the semantic family supports them.
    pub coordinates: Option<Coordinates>,
    /// NMEA checksum details when present.
    pub nmea_checksum: Option<NmeaChecksum>,
    /// Telemetry sequence number when present and numeric.
    pub telemetry_sequence: Option<u16>,
    /// Mic-E speed/course details when present and decodable.
    pub mic_e_speed_course: Option<MicESpeedCourse>,
}

impl<'a> PacketSummary<'a> {
    fn from_packet(packet: &'a ParsedPacket) -> Self {
        let data = packet.aprs_data();
        Self {
            source: packet.source(),
            destination: packet.destination(),
            data_type: packet.data_type_identifier().name(),
            semantic: data.kind_name(),
            coordinates: summary_coordinates(data),
            nmea_checksum: summary_nmea_checksum(data),
            telemetry_sequence: summary_telemetry_sequence(data),
            mic_e_speed_course: summary_mic_e_speed_course(data),
        }
    }
}

/// Stable diagnostic layer for parser, policy, and transport failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticLayer {
    /// Codec/parser validation at the APRS packet boundary.
    Parse,
    /// Operational policy after codec validation.
    Policy,
    /// Transport framing or I/O boundary before codec validation.
    Transport,
}

impl DiagnosticLayer {
    /// Returns a stable machine-readable layer code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::Policy => "policy",
            Self::Transport => "transport",
        }
    }
}

/// Structured diagnostic metadata for parser, policy, and transport errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ErrorDiagnostic {
    /// Layer that produced the diagnostic.
    pub layer: DiagnosticLayer,
    /// Stable fully-qualified diagnostic code.
    pub code: &'static str,
    /// Stable short diagnostic name within the layer.
    pub name: &'static str,
    /// Human-readable diagnostic description for operators.
    pub description: &'static str,
    /// Human-readable remediation guidance.
    pub remediation: &'static str,
}

/// Support status for documented APRS capabilities and integration surfaces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SupportStatus {
    /// Supported by parser semantics or adapter helpers.
    Supported,
    /// Supported partially; callers should inspect documentation for limits.
    Partial,
    /// Intentionally unsupported in the current release line.
    Unsupported,
}

impl SupportStatus {
    /// Returns a stable machine-readable support status.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Partial => "partial",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Support-matrix item exposed for documentation and machine-readable CLI output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SupportItem {
    /// Stable item kind.
    pub kind: &'static str,
    /// Current support status.
    pub status: SupportStatus,
    /// Short operational note.
    pub notes: &'static str,
}

/// Transport adapter support entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportSupport {
    /// Published crate name.
    pub crate_name: &'static str,
    /// Boundary handled by the adapter.
    pub boundary: &'static str,
    /// Current support status.
    pub status: SupportStatus,
    /// Short operational note.
    pub notes: &'static str,
}

/// Machine-readable support matrix for operator tooling and docs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SupportMatrix {
    /// Schema version for machine-readable output.
    pub schema_version: u8,
    /// Documented APRS semantic families.
    pub semantic_families: &'static [SupportItem],
    /// Optional transport adapter crates.
    pub transport_adapters: &'static [TransportSupport],
    /// Diagnostic layers emitted by the project.
    pub diagnostic_layers: &'static [DiagnosticLayer],
}

/// Returns the current support matrix for CLI and documentation consumers.
#[must_use]
pub const fn support_matrix() -> SupportMatrix {
    SupportMatrix {
        schema_version: 1,
        semantic_families: SEMANTIC_SUPPORT,
        transport_adapters: TRANSPORT_SUPPORT,
        diagnostic_layers: DIAGNOSTIC_LAYERS,
    }
}

const DIAGNOSTIC_LAYERS: &[DiagnosticLayer] = &[
    DiagnosticLayer::Parse,
    DiagnosticLayer::Policy,
    DiagnosticLayer::Transport,
];

const SEMANTIC_SUPPORT: &[SupportItem] = &[
    SupportItem {
        kind: "status",
        status: SupportStatus::Supported,
        notes: "status text bytes are preserved",
    },
    SupportItem {
        kind: "position",
        status: SupportStatus::Supported,
        notes: "uncompressed and compressed coordinates are decoded where valid",
    },
    SupportItem {
        kind: "message",
        status: SupportStatus::Supported,
        notes:
            "messages, acknowledgements, rejections, bulletins, and announcements are classified",
    },
    SupportItem {
        kind: "object",
        status: SupportStatus::Supported,
        notes: "object name, liveness, timestamp, body, and supported coordinates are exposed",
    },
    SupportItem {
        kind: "item",
        status: SupportStatus::Supported,
        notes: "item name, liveness, body, and supported coordinates are exposed",
    },
    SupportItem {
        kind: "weather",
        status: SupportStatus::Partial,
        notes: "common weather fields are extracted; empty weather reports are malformed",
    },
    SupportItem {
        kind: "telemetry",
        status: SupportStatus::Supported,
        notes: "sequence, analogue values, digital bits, and metadata packets are exposed",
    },
    SupportItem {
        kind: "nmea",
        status: SupportStatus::Supported,
        notes: "sentence identifiers and checksum diagnostics are exposed",
    },
    SupportItem {
        kind: "mic_e",
        status: SupportStatus::Partial,
        notes: "destination-derived status, latitude digits, speed, and course helpers are exposed",
    },
    SupportItem {
        kind: "third_party",
        status: SupportStatus::Partial,
        notes: "nested packet bytes must pass the codec envelope before explicit caller parsing",
    },
    SupportItem {
        kind: "unsupported",
        status: SupportStatus::Supported,
        notes: "unknown identifiers remain explicit and byte-preserving",
    },
    SupportItem {
        kind: "malformed",
        status: SupportStatus::Supported,
        notes: "codec-valid but semantically malformed packets remain visible to policy",
    },
];

const TRANSPORT_SUPPORT: &[TransportSupport] = &[
    TransportSupport {
        crate_name: "aprs-transport-file",
        boundary: "newline-separated files and stdin-style byte streams",
        status: SupportStatus::Supported,
        notes: "bounded file and packet-line reads",
    },
    TransportSupport {
        crate_name: "aprs-transport-tcp",
        boundary: "blocking TCP or Read packet streams",
        status: SupportStatus::Supported,
        notes: "caller owns socket timeouts and reconnect behavior",
    },
    TransportSupport {
        crate_name: "aprs-transport-aprs-is",
        boundary: "APRS-IS login framing and server line filtering",
        status: SupportStatus::Supported,
        notes: "authentication and reconnect loops stay application-owned",
    },
    TransportSupport {
        crate_name: "aprs-transport-kiss",
        boundary: "KISS frame encoding and decoding",
        status: SupportStatus::Supported,
        notes: "invalid escapes and oversized frames fail closed",
    },
    TransportSupport {
        crate_name: "aprs-transport-serial",
        boundary: "serial-like byte readers",
        status: SupportStatus::Supported,
        notes: "serial configuration stays application-owned",
    },
    TransportSupport {
        crate_name: "aprs-transport-udp",
        boundary: "UDP datagram payloads",
        status: SupportStatus::Supported,
        notes: "datagram length is bounded before parsing",
    },
    TransportSupport {
        crate_name: "aprs-transport-http",
        boundary: "HTTP request body bytes",
        status: SupportStatus::Supported,
        notes: "body and packet-line limits are enforced by helpers",
    },
    TransportSupport {
        crate_name: "aprs-transport-file-watch",
        boundary: "append-only packet logs",
        status: SupportStatus::Supported,
        notes: "appended byte ranges and packet lines are bounded",
    },
    TransportSupport {
        crate_name: "aprs-transport-mqtt",
        boundary: "MQTT topics and payload copies",
        status: SupportStatus::Supported,
        notes: "broker sessions, authentication, and reconnects stay application-owned",
    },
    TransportSupport {
        crate_name: "aprs-transport-ax25",
        boundary: "AX.25 UI frames",
        status: SupportStatus::Supported,
        notes: "oversized UI frames fail closed before payload extraction",
    },
    TransportSupport {
        crate_name: "aprs-transport-corpus",
        boundary: "fixture and corpus replay",
        status: SupportStatus::Supported,
        notes: "stable ordering and per-file limits for tests",
    },
    TransportSupport {
        crate_name: "aprs-transport-channel",
        boundary: "in-process packet channels",
        status: SupportStatus::Supported,
        notes: "caller-owned channel capacity controls backpressure",
    },
    TransportSupport {
        crate_name: "aprs-transport-async",
        boundary: "runtime-neutral async byte splitting",
        status: SupportStatus::Supported,
        notes: "runtime, timeouts, and cancellation stay caller-owned",
    },
];

/// Parser and policy orchestration engine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Engine {
    policy: Policy,
    counters: Counters,
}

impl Engine {
    /// Creates an engine with the provided policy.
    #[must_use]
    pub fn new(policy: Policy) -> Self {
        Self {
            policy,
            counters: Counters::default(),
        }
    }

    /// Processes one packet through codec, semantics, and policy.
    pub fn process(&mut self, input: &[u8]) -> EngineResult {
        match parse_packet(input) {
            Ok(packet) => {
                let semantic = packet.aprs_data();
                match self.policy.evaluate(&packet, &semantic) {
                    PolicyDecision::Accept => {
                        self.counters.accepted = self.counters.accepted.saturating_add(1);
                        EngineResult::Accepted { packet }
                    }
                    PolicyDecision::Reject(reason) => {
                        self.counters.rejected = self.counters.rejected.saturating_add(1);
                        EngineResult::Rejected { packet, reason }
                    }
                }
            }
            Err(error) => {
                self.counters.malformed = self.counters.malformed.saturating_add(1);
                EngineResult::ParseError(error)
            }
        }
    }

    /// Processes a caller-provided packet batch in order.
    pub fn process_packets<I, P>(&mut self, packets: I) -> Vec<EngineResult>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<[u8]>,
    {
        packets
            .into_iter()
            .map(|packet| self.process(packet.as_ref()))
            .collect()
    }

    /// Reads one bounded batch from a packet source and processes it in order.
    pub fn process_source<S>(&mut self, source: &mut S) -> Result<Vec<EngineResult>, S::Error>
    where
        S: PacketSource,
    {
        Ok(self.process_packets(source.recv_packets()?))
    }

    /// Returns engine counters.
    #[must_use]
    pub fn counters(&self) -> Counters {
        self.counters
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(Policy::default())
    }
}

/// Engine processing result.
#[derive(Clone, Debug, PartialEq)]
pub enum EngineResult {
    /// Packet parsed and passed policy.
    Accepted {
        /// Parsed packet.
        packet: ParsedPacket,
    },
    /// Packet parsed but failed policy.
    Rejected {
        /// Parsed packet.
        packet: ParsedPacket,
        /// Rejection reason.
        reason: PolicyRejection,
    },
    /// Packet failed the codec boundary.
    ParseError(ParseError),
}

/// Runtime counters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Counters {
    /// Accepted packet count.
    pub accepted: u64,
    /// Policy-rejected packet count.
    pub rejected: u64,
    /// Codec-malformed packet count.
    pub malformed: u64,
}

/// Policy options applied after parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Policy {
    /// Allow semantic packets represented as unsupported.
    pub allow_unsupported: bool,
    /// Allow semantic packets represented as malformed.
    pub allow_malformed_semantics: bool,
    /// Reject NMEA sentences when a present checksum does not match.
    pub reject_invalid_nmea_checksum: bool,
    /// Maximum allowed path component count including destination.
    pub max_path_components: usize,
}

impl Policy {
    /// Strict policy: reject malformed semantics, unsupported formats, and long paths.
    #[must_use]
    pub fn strict() -> Self {
        Self::default()
    }

    /// Permissive policy: accept unsupported and malformed semantic packets.
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            allow_unsupported: true,
            allow_malformed_semantics: true,
            reject_invalid_nmea_checksum: false,
            max_path_components: 9,
        }
    }

    /// Evaluates a parsed packet and semantic view.
    #[must_use]
    pub fn evaluate(&self, packet: &ParsedPacket, semantic: &AprsData<'_>) -> PolicyDecision {
        if packet.path_components.len() > self.max_path_components {
            return PolicyDecision::Reject(PolicyRejection::PathTooLong);
        }

        if self.reject_invalid_nmea_checksum
            && matches!(
                semantic,
                AprsData::Nmea(nmea) if nmea.checksum().is_some_and(|checksum| !checksum.valid)
            )
        {
            return PolicyDecision::Reject(PolicyRejection::InvalidNmeaChecksum);
        }

        match semantic {
            AprsData::Malformed { .. } if !self.allow_malformed_semantics => {
                PolicyDecision::Reject(PolicyRejection::MalformedSemantics)
            }
            AprsData::Unsupported { .. } if !self.allow_unsupported => {
                PolicyDecision::Reject(PolicyRejection::UnsupportedSemantics)
            }
            _ => PolicyDecision::Accept,
        }
    }
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            allow_unsupported: false,
            allow_malformed_semantics: false,
            reject_invalid_nmea_checksum: false,
            max_path_components: 9,
        }
    }
}

/// Policy decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyDecision {
    /// Packet is accepted.
    Accept,
    /// Packet is rejected with a reason.
    Reject(PolicyRejection),
}

/// Policy rejection reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyRejection {
    /// Path contains too many components.
    PathTooLong,
    /// Semantic payload is malformed.
    MalformedSemantics,
    /// Semantic payload is unsupported.
    UnsupportedSemantics,
    /// NMEA sentence has a present checksum that does not match.
    InvalidNmeaChecksum,
}

impl PolicyRejection {
    /// Returns a stable policy rejection code for logs and external systems.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::PathTooLong => "policy.path_too_long",
            Self::MalformedSemantics => "policy.malformed_semantics",
            Self::UnsupportedSemantics => "policy.unsupported_semantics",
            Self::InvalidNmeaChecksum => "policy.nmea_checksum_mismatch",
        }
    }

    /// Returns structured policy rejection metadata for operator diagnostics.
    #[must_use]
    pub fn diagnostic(self) -> ErrorDiagnostic {
        match self {
            Self::PathTooLong => ErrorDiagnostic {
                layer: DiagnosticLayer::Policy,
                code: self.code(),
                name: "path_too_long",
                description: "packet path contains more components than policy permits",
                remediation: "raise Policy::max_path_components only after reviewing path abuse risk",
            },
            Self::MalformedSemantics => ErrorDiagnostic {
                layer: DiagnosticLayer::Policy,
                code: self.code(),
                name: "malformed_semantics",
                description: "packet passed codec validation but the APRS semantic payload is malformed",
                remediation: "inspect the preserved raw bytes and keep strict policy enabled for untrusted inputs",
            },
            Self::UnsupportedSemantics => ErrorDiagnostic {
                layer: DiagnosticLayer::Policy,
                code: self.code(),
                name: "unsupported_semantics",
                description: "packet uses an unsupported APRS semantic family or identifier",
                remediation: "use permissive policy only for corpus collection or add explicit support before accepting",
            },
            Self::InvalidNmeaChecksum => ErrorDiagnostic {
                layer: DiagnosticLayer::Policy,
                code: self.code(),
                name: "nmea_checksum_mismatch",
                description: "NMEA sentence has a present checksum that does not match the calculated value",
                remediation: "treat the packet as untrusted and investigate upstream data corruption or spoofing",
            },
        }
    }
}

/// Semantic APRS information-field data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AprsData<'a> {
    /// Status report.
    Status {
        /// Status text bytes.
        text: &'a [u8],
    },
    /// Uncompressed position report.
    Position(Position<'a>),
    /// Timestamped uncompressed position report.
    TimestampedPosition(TimestampedPosition<'a>),
    /// Compressed position report.
    CompressedPosition(CompressedPosition<'a>),
    /// Message, bulletin, or announcement.
    Message(Message<'a>),
    /// Object report.
    Object(Object<'a>),
    /// Item report.
    Item(Item<'a>),
    /// Weather report without position.
    Weather(Weather<'a>),
    /// Telemetry report.
    Telemetry(Telemetry<'a>),
    /// Telemetry metadata carried as an APRS message.
    TelemetryMetadata(TelemetryMetadata<'a>),
    /// Query packet.
    Query(Query<'a>),
    /// Station capabilities packet.
    Capability(Capability<'a>),
    /// NMEA sentence packet.
    Nmea(Nmea<'a>),
    /// Mic-E packet.
    MicE(MicE<'a>),
    /// Maidenhead locator packet.
    Maidenhead(Maidenhead<'a>),
    /// User-defined data packet.
    UserDefined(UserDefined<'a>),
    /// Third-party traffic packet.
    ThirdParty(ThirdParty<'a>),
    /// Data format is validly framed but not implemented yet.
    Unsupported {
        /// Original data type identifier byte.
        identifier: u8,
        /// Remaining information-field bytes.
        information: &'a [u8],
    },
    /// Data type is known, but its information bytes are malformed.
    Malformed {
        /// Original data type identifier byte.
        identifier: u8,
        /// Remaining information-field bytes.
        information: &'a [u8],
    },
}

impl AprsData<'_> {
    /// Returns a stable semantic kind name for diagnostics.
    #[must_use]
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Status { .. } => "status",
            Self::Position(_) => "position",
            Self::TimestampedPosition(_) => "timestamped_position",
            Self::CompressedPosition(_) => "compressed_position",
            Self::Message(_) => "message",
            Self::Object(_) => "object",
            Self::Item(_) => "item",
            Self::Weather(_) => "weather",
            Self::Telemetry(_) => "telemetry",
            Self::TelemetryMetadata(_) => "telemetry_metadata",
            Self::Query(_) => "query",
            Self::Capability(_) => "capability",
            Self::Nmea(_) => "nmea",
            Self::MicE(_) => "mic_e",
            Self::Maidenhead(_) => "maidenhead",
            Self::UserDefined(_) => "user_defined",
            Self::ThirdParty(_) => "third_party",
            Self::Unsupported { .. } => "unsupported",
            Self::Malformed { .. } => "malformed",
        }
    }
}

fn summary_coordinates(data: AprsData<'_>) -> Option<Coordinates> {
    match data {
        AprsData::Position(position) => position.coordinates(),
        AprsData::TimestampedPosition(position) => position.position.coordinates(),
        AprsData::CompressedPosition(position) => position.coordinates(),
        AprsData::MicE(mic_e) => mic_e.coordinates(),
        _ => None,
    }
}

fn summary_nmea_checksum(data: AprsData<'_>) -> Option<NmeaChecksum> {
    match data {
        AprsData::Nmea(nmea) => nmea.checksum(),
        _ => None,
    }
}

fn summary_telemetry_sequence(data: AprsData<'_>) -> Option<u16> {
    match data {
        AprsData::Telemetry(telemetry) => telemetry.sequence_number(),
        _ => None,
    }
}

fn summary_mic_e_speed_course(data: AprsData<'_>) -> Option<MicESpeedCourse> {
    match data {
        AprsData::MicE(mic_e) => mic_e.speed_course(),
        _ => None,
    }
}

/// Uncompressed APRS position fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position<'a> {
    /// Whether the data type identifier indicates APRS messaging support.
    pub messaging: bool,
    /// Latitude bytes in APRS `DDMM.mmN/S` form.
    pub latitude: &'a [u8],
    /// Symbol table identifier byte.
    pub symbol_table: u8,
    /// Longitude bytes in APRS `DDDMM.mmE/W` form.
    pub longitude: &'a [u8],
    /// Symbol code byte.
    pub symbol_code: u8,
    /// Optional comment bytes after the symbol code.
    pub comment: &'a [u8],
}

impl Position<'_> {
    /// Returns decimal latitude and longitude if both coordinate fields decode.
    #[must_use]
    pub fn coordinates(&self) -> Option<Coordinates> {
        Some(Coordinates {
            latitude: decode_latitude(self.latitude)?,
            longitude: decode_longitude(self.longitude)?,
        })
    }
}

/// Decimal coordinates in signed degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coordinates {
    /// Latitude in signed decimal degrees.
    pub latitude: f64,
    /// Longitude in signed decimal degrees.
    pub longitude: f64,
}

/// Timestamped uncompressed APRS position fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimestampedPosition<'a> {
    /// Whether the data type identifier indicates APRS messaging support.
    pub messaging: bool,
    /// Seven-byte timestamp field.
    pub timestamp: &'a [u8],
    /// Position fields after the timestamp.
    pub position: Position<'a>,
}

/// Compressed APRS position fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompressedPosition<'a> {
    /// Whether the data type identifier indicates APRS messaging support.
    pub messaging: bool,
    /// Symbol table identifier byte.
    pub symbol_table: u8,
    /// Four-byte compressed latitude.
    pub compressed_latitude: &'a [u8],
    /// Four-byte compressed longitude.
    pub compressed_longitude: &'a [u8],
    /// Symbol code byte.
    pub symbol_code: u8,
    /// Two-byte compressed extension field.
    pub extension: &'a [u8],
    /// Compression type byte.
    pub compression_type: u8,
    /// Optional comment bytes after the compression type byte.
    pub comment: &'a [u8],
}

impl CompressedPosition<'_> {
    /// Returns decoded compressed-position coordinates.
    #[must_use]
    pub fn coordinates(&self) -> Option<Coordinates> {
        let y = decode_base91(self.compressed_latitude)?;
        let x = decode_base91(self.compressed_longitude)?;

        Some(Coordinates {
            latitude: 90.0 - (y as f64 / 380_926.0),
            longitude: -180.0 + (x as f64 / 190_463.0),
        })
    }
}

/// APRS message fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Message<'a> {
    /// Nine-byte addressee field.
    pub addressee: &'a [u8],
    /// Classified message subtype.
    pub kind: MessageKind,
    /// Message text bytes before an optional message ID.
    pub text: &'a [u8],
    /// Optional message ID bytes after `{`.
    pub id: Option<&'a [u8]>,
}

/// APRS message subtype.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageKind {
    /// Regular addressed message.
    Message,
    /// Message acknowledgement.
    Ack,
    /// Message rejection.
    Reject,
    /// Bulletin.
    Bulletin,
    /// Announcement.
    Announcement,
}

/// APRS object report fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Object<'a> {
    /// Nine-byte object name.
    pub name: &'a [u8],
    /// Whether the object is live (`*`) rather than killed (`_`).
    pub live: bool,
    /// Seven-byte object timestamp.
    pub timestamp: &'a [u8],
    /// Remaining object body bytes.
    pub body: &'a [u8],
}

impl Object<'_> {
    /// Returns object coordinates when the object body starts with a supported
    /// APRS position encoding.
    #[must_use]
    pub fn coordinates(&self) -> Option<Coordinates> {
        coordinates_from_position_body(self.body)
    }
}

/// APRS item report fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Item<'a> {
    /// Item name bytes.
    pub name: &'a [u8],
    /// Whether the item is live (`!`) rather than killed (`_`).
    pub live: bool,
    /// Remaining item body bytes.
    pub body: &'a [u8],
}

impl Item<'_> {
    /// Returns item coordinates when the item body starts with a supported APRS
    /// position encoding.
    #[must_use]
    pub fn coordinates(&self) -> Option<Coordinates> {
        coordinates_from_position_body(self.body)
    }
}

/// APRS weather report bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Weather<'a> {
    /// Weather report bytes after the `_` data type identifier.
    pub report: &'a [u8],
}

impl Weather<'_> {
    /// Extracts common numeric weather fields when present.
    #[must_use]
    pub fn fields(&self) -> WeatherFields<'_> {
        WeatherFields {
            timestamp: self
                .report
                .get(..6)
                .filter(|value| value.iter().all(u8::is_ascii_digit)),
            wind_direction_degrees: parse_tagged_u16(self.report, b'c', 3),
            wind_speed_mph: parse_tagged_u16(self.report, b's', 3),
            wind_gust_mph: parse_tagged_u16(self.report, b'g', 3),
            temperature_fahrenheit: parse_tagged_i16(self.report, b't', 3),
            rain_last_hour_hundredths_inch: parse_tagged_u16(self.report, b'r', 3),
            rain_last_24_hours_hundredths_inch: parse_tagged_u16(self.report, b'p', 3),
            rain_since_midnight_hundredths_inch: parse_tagged_u16(self.report, b'P', 3),
            humidity_percent: parse_tagged_u16(self.report, b'h', 2).map(|value| {
                if value == 0 {
                    100
                } else {
                    value
                }
            }),
            pressure_tenths_hpa: parse_tagged_u16(self.report, b'b', 5),
            luminosity_watts_per_square_meter: parse_tagged_u16(self.report, b'L', 3),
            luminosity_1000_plus_watts_per_square_meter: parse_tagged_u16(self.report, b'l', 3)
                .map(|value| value + 1000),
            snow_last_24_hours_inches: parse_tagged_u16(self.report, b'S', 3),
            raw_rain_counter: parse_tagged_u16(self.report, b'#', 3),
        }
    }
}

/// Extracted numeric weather fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WeatherFields<'a> {
    /// Optional six-byte timestamp prefix.
    pub timestamp: Option<&'a [u8]>,
    /// Wind direction in degrees.
    pub wind_direction_degrees: Option<u16>,
    /// Sustained wind speed in miles per hour.
    pub wind_speed_mph: Option<u16>,
    /// Wind gust speed in miles per hour.
    pub wind_gust_mph: Option<u16>,
    /// Temperature in degrees Fahrenheit.
    pub temperature_fahrenheit: Option<i16>,
    /// Rain in the last hour, in hundredths of an inch.
    pub rain_last_hour_hundredths_inch: Option<u16>,
    /// Rain in the last 24 hours, in hundredths of an inch.
    pub rain_last_24_hours_hundredths_inch: Option<u16>,
    /// Rain since midnight, in hundredths of an inch.
    pub rain_since_midnight_hundredths_inch: Option<u16>,
    /// Relative humidity percent.
    pub humidity_percent: Option<u16>,
    /// Barometric pressure in tenths of hPa.
    pub pressure_tenths_hpa: Option<u16>,
    /// Luminosity in watts per square meter from `Lnnn`.
    pub luminosity_watts_per_square_meter: Option<u16>,
    /// Luminosity in watts per square meter from `lnnn`, representing 1000+.
    pub luminosity_1000_plus_watts_per_square_meter: Option<u16>,
    /// Snowfall in the last 24 hours, in inches.
    pub snow_last_24_hours_inches: Option<u16>,
    /// Raw rain counter value from `#nnn`.
    pub raw_rain_counter: Option<u16>,
}

/// APRS telemetry report fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Telemetry<'a> {
    /// Telemetry sequence bytes.
    pub sequence: &'a [u8],
    /// Five analog telemetry value fields.
    pub analog: [&'a [u8]; 5],
    /// Optional eight-bit digital telemetry field.
    pub digital: Option<&'a [u8]>,
}

impl Telemetry<'_> {
    /// Returns the numeric telemetry sequence number.
    #[must_use]
    pub fn sequence_number(&self) -> Option<u16> {
        parse_u16(self.sequence)
    }

    /// Returns the five numeric analog telemetry values.
    #[must_use]
    pub fn analog_values(&self) -> Option<[u16; 5]> {
        Some([
            parse_u16(self.analog[0])?,
            parse_u16(self.analog[1])?,
            parse_u16(self.analog[2])?,
            parse_u16(self.analog[3])?,
            parse_u16(self.analog[4])?,
        ])
    }

    /// Returns eight digital telemetry bits.
    #[must_use]
    pub fn digital_bits(&self) -> Option<[bool; 8]> {
        let digital = self.digital?;
        if digital.len() != 8 {
            return None;
        }

        let mut bits = [false; 8];
        for (index, byte) in digital.iter().enumerate() {
            bits[index] = match byte {
                b'0' => false,
                b'1' => true,
                _ => return None,
            };
        }

        Some(bits)
    }
}

/// APRS telemetry metadata packet carried in an APRS message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TelemetryMetadata<'a> {
    /// Nine-byte telemetry metadata addressee field.
    pub addressee: &'a [u8],
    /// Classified telemetry metadata subtype.
    pub kind: TelemetryMetadataKind,
    /// Metadata body bytes after the message separator.
    pub body: &'a [u8],
}

impl<'a> TelemetryMetadata<'a> {
    /// Returns comma-separated metadata fields without lossy conversion.
    #[must_use]
    pub fn fields(&self) -> Vec<&'a [u8]> {
        self.body.split(|byte| *byte == b',').collect()
    }
}

/// APRS telemetry metadata subtype.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryMetadataKind {
    /// `PARM.` parameter-name metadata.
    ParameterNames,
    /// `UNIT.` unit metadata.
    Units,
    /// `EQNS.` calibration/equation metadata.
    Equations,
    /// `BITS.` bit-sense/project metadata.
    BitSense,
}

/// APRS query packet bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Query<'a> {
    /// Query bytes after the `?` data type identifier.
    pub query: &'a [u8],
}

/// APRS station capabilities packet bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Capability<'a> {
    /// Capability body bytes after the `<` data type identifier.
    pub body: &'a [u8],
}

/// APRS NMEA packet bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Nmea<'a> {
    /// NMEA sentence bytes after the `$` data type identifier.
    pub sentence: &'a [u8],
}

impl Nmea<'_> {
    /// Returns the NMEA talker ID from the sentence address field.
    #[must_use]
    pub fn talker_id(&self) -> Option<&[u8]> {
        let address = self.address_field()?;
        (address.len() >= 2).then_some(&address[..2])
    }

    /// Returns the NMEA sentence formatter ID from the sentence address field.
    #[must_use]
    pub fn sentence_id(&self) -> Option<&[u8]> {
        let address = self.address_field()?;
        (address.len() >= 5).then_some(&address[2..5])
    }

    /// Returns data fields after the NMEA address field without the checksum.
    #[must_use]
    pub fn data_fields(&self) -> Vec<&[u8]> {
        let body = self.body_without_checksum();
        let mut fields = body.split(|byte| *byte == b',');
        let _address = fields.next();
        fields.collect()
    }

    /// Returns checksum validation details when the sentence has `*HH` syntax.
    #[must_use]
    pub fn checksum(&self) -> Option<NmeaChecksum> {
        let separator = self.sentence.iter().rposition(|byte| *byte == b'*')?;
        let checksum = self.sentence.get(separator + 1..separator + 3)?;
        if checksum.len() != 2 || self.sentence.get(separator + 3).is_some() {
            return None;
        }

        let expected = parse_hex_byte(checksum)?;
        let calculated = self.sentence[..separator]
            .iter()
            .fold(0u8, |accumulator, byte| accumulator ^ byte);

        Some(NmeaChecksum {
            expected,
            calculated,
            valid: expected == calculated,
        })
    }

    fn address_field(&self) -> Option<&[u8]> {
        let body = self.body_without_checksum();
        let end = body
            .iter()
            .position(|byte| *byte == b',')
            .unwrap_or(body.len());
        let address = &body[..end];
        (address.len() >= 5 && address.iter().all(u8::is_ascii_alphanumeric)).then_some(address)
    }

    fn body_without_checksum(&self) -> &[u8] {
        match self.sentence.iter().rposition(|byte| *byte == b'*') {
            Some(separator) => &self.sentence[..separator],
            None => self.sentence,
        }
    }
}

/// NMEA checksum validation details.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NmeaChecksum {
    /// Checksum value supplied by the packet.
    pub expected: u8,
    /// Checksum calculated over bytes before `*`.
    pub calculated: u8,
    /// Whether supplied and calculated checksums match.
    pub valid: bool,
}

/// APRS Mic-E packet bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MicE<'a> {
    /// Original Mic-E data type identifier byte.
    pub identifier: u8,
    /// Destination address bytes that carry Mic-E latitude/status data.
    pub destination: &'a [u8],
    /// Mic-E body bytes.
    pub body: &'a [u8],
    /// Destination-derived Mic-E status bits when the destination permits decoding.
    pub status: Option<MicEStatus>,
    /// Destination-derived six latitude digit nibbles when decodable.
    pub latitude_digits: Option<[u8; 6]>,
}

impl MicE<'_> {
    /// Returns decoded Mic-E coordinates when destination and body bytes permit it.
    #[must_use]
    pub fn coordinates(&self) -> Option<Coordinates> {
        Some(Coordinates {
            latitude: decode_mic_e_latitude(self.destination)?,
            longitude: decode_mic_e_longitude(self.destination, self.body)?,
        })
    }

    /// Returns decoded Mic-E speed and course when body bytes permit it.
    #[must_use]
    pub fn speed_course(&self) -> Option<MicESpeedCourse> {
        decode_mic_e_speed_course(self.body)
    }

    /// Returns the Mic-E destination-derived message code when decodable.
    #[must_use]
    pub fn message_code(&self) -> Option<MicEMessageCode> {
        decode_mic_e_message_code(self.destination)
    }
}

/// Mic-E destination-derived status bits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MicEStatus {
    /// Standard/custom status bit tuple from the first three destination bytes.
    Custom([bool; 3]),
}

/// Mic-E destination-derived message code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MicEMessageCode {
    /// Standard Mic-E message code.
    Standard(MicEStandardMessage),
    /// Custom Mic-E message code number from 0 through 6.
    Custom(u8),
    /// Emergency message code.
    Emergency,
}

/// Standard Mic-E message code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MicEStandardMessage {
    /// M0: Off Duty.
    OffDuty,
    /// M1: En Route.
    EnRoute,
    /// M2: In Service.
    InService,
    /// M3: Returning.
    Returning,
    /// M4: Committed.
    Committed,
    /// M5: Special.
    Special,
    /// M6: Priority.
    Priority,
}

/// Mic-E speed/course extension.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MicESpeedCourse {
    /// Speed in knots.
    pub speed_knots: u16,
    /// Course in degrees as encoded by Mic-E.
    pub course_degrees: u16,
}

/// APRS Maidenhead locator packet bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Maidenhead<'a> {
    /// Six-byte Maidenhead locator.
    pub locator: &'a [u8],
    /// Remaining comment bytes.
    pub comment: &'a [u8],
}

/// APRS user-defined packet fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserDefined<'a> {
    /// One-byte user ID.
    pub user_id: u8,
    /// One-byte user-defined packet type.
    pub packet_type: u8,
    /// User-defined body bytes.
    pub body: &'a [u8],
}

/// APRS third-party traffic packet bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThirdParty<'a> {
    /// Encapsulated third-party traffic bytes.
    pub body: &'a [u8],
}

impl ThirdParty<'_> {
    /// Explicitly parses the encapsulated packet through the same codec boundary.
    pub fn nested_packet(&self) -> Result<ParsedPacket, ParseError> {
        parse_packet(self.body)
    }
}

/// APRS data type identifier from the first payload byte.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DataTypeIdentifier {
    /// `!`: position without timestamp, no APRS messaging.
    PositionNoTimestamp,
    /// `=`: position without timestamp, APRS messaging supported.
    PositionNoTimestampMessaging,
    /// `/`: position with timestamp, no APRS messaging.
    PositionWithTimestamp,
    /// `@`: position with timestamp, APRS messaging supported.
    PositionWithTimestampMessaging,
    /// `>`: status.
    Status,
    /// `?`: query.
    Query,
    /// `<`: station capabilities.
    Capability,
    /// `:`: message, bulletin, or announcement.
    Message,
    /// `;`: object.
    Object,
    /// `)`: item.
    Item,
    /// `_`: weather report without position.
    Weather,
    /// `T`: telemetry.
    Telemetry,
    /// `$`: NMEA sentence.
    Nmea,
    /// ``` ` ```: current Mic-E data.
    MicECurrent,
    /// `'`: old Mic-E data.
    MicEOld,
    /// `[`: Maidenhead locator.
    Maidenhead,
    /// `{`: user-defined data.
    UserDefined,
    /// `}`: third-party traffic.
    ThirdParty,
    /// Any currently unclassified identifier byte.
    Unknown(u8),
}

impl DataTypeIdentifier {
    fn from_byte(byte: u8) -> Self {
        match byte {
            b'!' => Self::PositionNoTimestamp,
            b'=' => Self::PositionNoTimestampMessaging,
            b'/' => Self::PositionWithTimestamp,
            b'@' => Self::PositionWithTimestampMessaging,
            b'>' => Self::Status,
            b'?' => Self::Query,
            b'<' => Self::Capability,
            b':' => Self::Message,
            b';' => Self::Object,
            b')' => Self::Item,
            b'_' => Self::Weather,
            b'T' => Self::Telemetry,
            b'$' => Self::Nmea,
            b'`' => Self::MicECurrent,
            b'\'' => Self::MicEOld,
            b'[' => Self::Maidenhead,
            b'{' => Self::UserDefined,
            b'}' => Self::ThirdParty,
            other => Self::Unknown(other),
        }
    }

    fn as_byte(self) -> u8 {
        match self {
            Self::PositionNoTimestamp => b'!',
            Self::PositionNoTimestampMessaging => b'=',
            Self::PositionWithTimestamp => b'/',
            Self::PositionWithTimestampMessaging => b'@',
            Self::Status => b'>',
            Self::Query => b'?',
            Self::Capability => b'<',
            Self::Message => b':',
            Self::Object => b';',
            Self::Item => b')',
            Self::Weather => b'_',
            Self::Telemetry => b'T',
            Self::Nmea => b'$',
            Self::MicECurrent => b'`',
            Self::MicEOld => b'\'',
            Self::Maidenhead => b'[',
            Self::UserDefined => b'{',
            Self::ThirdParty => b'}',
            Self::Unknown(value) => value,
        }
    }

    /// Returns a stable data type identifier name for diagnostics.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::PositionNoTimestamp => "position_no_timestamp",
            Self::PositionNoTimestampMessaging => "position_no_timestamp_messaging",
            Self::PositionWithTimestamp => "position_with_timestamp",
            Self::PositionWithTimestampMessaging => "position_with_timestamp_messaging",
            Self::Status => "status",
            Self::Query => "query",
            Self::Capability => "capability",
            Self::Message => "message",
            Self::Object => "object",
            Self::Item => "item",
            Self::Weather => "weather",
            Self::Telemetry => "telemetry",
            Self::Nmea => "nmea",
            Self::MicECurrent => "mic_e_current",
            Self::MicEOld => "mic_e_old",
            Self::Maidenhead => "maidenhead",
            Self::UserDefined => "user_defined",
            Self::ThirdParty => "third_party",
            Self::Unknown(_) => "unknown",
        }
    }
}

fn parse_aprs_data<'a>(
    identifier: DataTypeIdentifier,
    information: &'a [u8],
    destination: &'a [u8],
) -> AprsData<'a> {
    match identifier {
        DataTypeIdentifier::Status => AprsData::Status { text: information },
        DataTypeIdentifier::PositionNoTimestamp => parse_position(false, b'!', information),
        DataTypeIdentifier::PositionNoTimestampMessaging => parse_position(true, b'=', information),
        DataTypeIdentifier::PositionWithTimestamp => {
            parse_timestamped_position(false, b'/', information)
        }
        DataTypeIdentifier::PositionWithTimestampMessaging => {
            parse_timestamped_position(true, b'@', information)
        }
        DataTypeIdentifier::Message => parse_message(information),
        DataTypeIdentifier::Object => parse_object(information),
        DataTypeIdentifier::Item => parse_item(information),
        DataTypeIdentifier::Weather => parse_weather(information),
        DataTypeIdentifier::Telemetry => parse_telemetry(information),
        DataTypeIdentifier::Query => AprsData::Query(Query { query: information }),
        DataTypeIdentifier::Capability => AprsData::Capability(Capability { body: information }),
        DataTypeIdentifier::Nmea => AprsData::Nmea(Nmea {
            sentence: information,
        }),
        DataTypeIdentifier::MicECurrent | DataTypeIdentifier::MicEOld => {
            parse_mic_e(identifier, information, destination)
        }
        DataTypeIdentifier::Maidenhead => parse_maidenhead(information),
        DataTypeIdentifier::UserDefined => parse_user_defined(information),
        DataTypeIdentifier::ThirdParty => parse_third_party(information),
        other => AprsData::Unsupported {
            identifier: other.as_byte(),
            information,
        },
    }
}

fn parse_weather(information: &[u8]) -> AprsData<'_> {
    if information.is_empty() {
        return AprsData::Malformed {
            identifier: b'_',
            information,
        };
    }

    AprsData::Weather(Weather {
        report: information,
    })
}

fn parse_third_party(information: &[u8]) -> AprsData<'_> {
    if parse_packet(information).is_err() {
        return AprsData::Malformed {
            identifier: b'}',
            information,
        };
    }

    AprsData::ThirdParty(ThirdParty { body: information })
}

fn parse_mic_e<'a>(
    identifier: DataTypeIdentifier,
    information: &'a [u8],
    destination: &'a [u8],
) -> AprsData<'a> {
    if information.len() < 3 {
        return AprsData::Malformed {
            identifier: identifier.as_byte(),
            information,
        };
    }

    AprsData::MicE(MicE {
        identifier: identifier.as_byte(),
        destination,
        body: information,
        status: decode_mic_e_status(destination),
        latitude_digits: decode_mic_e_latitude_digits(destination),
    })
}

fn parse_position(messaging: bool, identifier: u8, information: &[u8]) -> AprsData<'_> {
    if is_compressed_position(information) {
        return parse_compressed_position(messaging, identifier, information);
    }

    if information.len() < 19 {
        return AprsData::Malformed {
            identifier,
            information,
        };
    }

    let latitude = &information[..8];
    let symbol_table = information[8];
    let longitude = &information[9..18];
    let symbol_code = information[18];
    let comment = &information[19..];

    if !is_latitude(latitude)
        || !is_symbol_table_identifier(symbol_table)
        || !is_longitude(longitude)
        || !is_printable_ascii(symbol_code)
    {
        return AprsData::Malformed {
            identifier,
            information,
        };
    }

    AprsData::Position(Position {
        messaging,
        latitude,
        symbol_table,
        longitude,
        symbol_code,
        comment,
    })
}

fn coordinates_from_position_body(body: &[u8]) -> Option<Coordinates> {
    if is_compressed_position(body) {
        let AprsData::CompressedPosition(position) = parse_compressed_position(false, b'!', body)
        else {
            return None;
        };
        return position.coordinates();
    }

    let AprsData::Position(position) = parse_position(false, b'!', body) else {
        return None;
    };
    position.coordinates()
}

fn parse_timestamped_position(messaging: bool, identifier: u8, information: &[u8]) -> AprsData<'_> {
    if information.len() < 8 {
        return AprsData::Malformed {
            identifier,
            information,
        };
    }

    let timestamp = &information[..7];
    if !is_timestamp(timestamp) {
        return AprsData::Malformed {
            identifier,
            information,
        };
    }

    match parse_position(messaging, identifier, &information[7..]) {
        AprsData::Position(position) => AprsData::TimestampedPosition(TimestampedPosition {
            messaging,
            timestamp,
            position,
        }),
        AprsData::CompressedPosition(position) => AprsData::CompressedPosition(position),
        _ => AprsData::Malformed {
            identifier,
            information,
        },
    }
}

fn parse_compressed_position(messaging: bool, identifier: u8, information: &[u8]) -> AprsData<'_> {
    if information.len() < 13 {
        return AprsData::Malformed {
            identifier,
            information,
        };
    }

    let symbol_table = information[0];
    let compressed_latitude = &information[1..5];
    let compressed_longitude = &information[5..9];
    let symbol_code = information[9];
    let extension = &information[10..12];
    let compression_type = information[12];
    let comment = &information[13..];

    if !is_symbol_table_identifier(symbol_table)
        || !compressed_latitude.iter().all(|byte| is_base91(*byte))
        || !compressed_longitude.iter().all(|byte| is_base91(*byte))
        || !is_printable_ascii(symbol_code)
        || !extension.iter().all(|byte| is_base91(*byte))
        || !is_base91(compression_type)
    {
        return AprsData::Malformed {
            identifier,
            information,
        };
    }

    AprsData::CompressedPosition(CompressedPosition {
        messaging,
        symbol_table,
        compressed_latitude,
        compressed_longitude,
        symbol_code,
        extension,
        compression_type,
        comment,
    })
}

fn parse_object(information: &[u8]) -> AprsData<'_> {
    if information.len() < 17
        || !matches!(information[9], b'*' | b'_')
        || !is_timestamp(&information[10..17])
    {
        return AprsData::Malformed {
            identifier: b';',
            information,
        };
    }

    AprsData::Object(Object {
        name: &information[..9],
        live: information[9] == b'*',
        timestamp: &information[10..17],
        body: &information[17..],
    })
}

fn parse_item(information: &[u8]) -> AprsData<'_> {
    let Some(separator) = information
        .iter()
        .position(|byte| matches!(*byte, b'!' | b'_'))
    else {
        return AprsData::Malformed {
            identifier: b')',
            information,
        };
    };

    if separator == 0 || separator > 9 {
        return AprsData::Malformed {
            identifier: b')',
            information,
        };
    }

    AprsData::Item(Item {
        name: &information[..separator],
        live: information[separator] == b'!',
        body: &information[separator + 1..],
    })
}

fn parse_message(information: &[u8]) -> AprsData<'_> {
    if information.len() < 10 || information[9] != b':' {
        return AprsData::Malformed {
            identifier: b':',
            information,
        };
    }

    let addressee = &information[..9];
    let body = &information[10..];
    if let Some(kind) = classify_telemetry_metadata_kind(addressee) {
        return AprsData::TelemetryMetadata(TelemetryMetadata {
            addressee,
            kind,
            body,
        });
    }

    let (text, id) = match body.iter().position(|byte| *byte == b'{') {
        Some(separator) => (&body[..separator], Some(&body[separator + 1..])),
        None => (body, None),
    };
    let kind = classify_message_kind(addressee, text);

    AprsData::Message(Message {
        addressee,
        kind,
        text,
        id,
    })
}

fn parse_telemetry(information: &[u8]) -> AprsData<'_> {
    if !information.starts_with(b"#") {
        return AprsData::Malformed {
            identifier: b'T',
            information,
        };
    }

    let fields: Vec<&[u8]> = information[1..].split(|byte| *byte == b',').collect();
    if fields.len() < 6 || fields[..6].iter().any(|field| field.is_empty()) {
        return AprsData::Malformed {
            identifier: b'T',
            information,
        };
    }

    AprsData::Telemetry(Telemetry {
        sequence: fields[0],
        analog: [fields[1], fields[2], fields[3], fields[4], fields[5]],
        digital: fields.get(6).copied().filter(|field| !field.is_empty()),
    })
}

fn parse_maidenhead(information: &[u8]) -> AprsData<'_> {
    if information.len() < 6 || !is_maidenhead_locator(&information[..6]) {
        return AprsData::Malformed {
            identifier: b'[',
            information,
        };
    }

    AprsData::Maidenhead(Maidenhead {
        locator: &information[..6],
        comment: &information[6..],
    })
}

fn parse_user_defined(information: &[u8]) -> AprsData<'_> {
    if information.len() < 2 {
        return AprsData::Malformed {
            identifier: b'{',
            information,
        };
    }

    AprsData::UserDefined(UserDefined {
        user_id: information[0],
        packet_type: information[1],
        body: &information[2..],
    })
}

fn classify_telemetry_metadata_kind(addressee: &[u8]) -> Option<TelemetryMetadataKind> {
    match addressee.get(..5)? {
        b"PARM." => Some(TelemetryMetadataKind::ParameterNames),
        b"UNIT." => Some(TelemetryMetadataKind::Units),
        b"EQNS." => Some(TelemetryMetadataKind::Equations),
        b"BITS." => Some(TelemetryMetadataKind::BitSense),
        _ => None,
    }
}

fn classify_message_kind(addressee: &[u8], text: &[u8]) -> MessageKind {
    if text.starts_with(b"ack") {
        MessageKind::Ack
    } else if text.starts_with(b"rej") {
        MessageKind::Reject
    } else if addressee.starts_with(b"BLN") && addressee.get(3).is_some_and(u8::is_ascii_digit) {
        MessageKind::Bulletin
    } else if addressee.starts_with(b"BLN") && addressee.get(3).is_some_and(u8::is_ascii_uppercase)
    {
        MessageKind::Announcement
    } else {
        MessageKind::Message
    }
}

fn is_latitude(value: &[u8]) -> bool {
    if !(value.len() == 8
        && value[0].is_ascii_digit()
        && value[1].is_ascii_digit()
        && value[2].is_ascii_digit()
        && value[3].is_ascii_digit()
        && value[4] == b'.'
        && value[5].is_ascii_digit()
        && value[6].is_ascii_digit()
        && matches!(value[7], b'N' | b'S'))
    {
        return false;
    }

    coordinate_in_range(&value[..2], &value[2..7], 90)
}

fn is_longitude(value: &[u8]) -> bool {
    if !(value.len() == 9
        && value[0].is_ascii_digit()
        && value[1].is_ascii_digit()
        && value[2].is_ascii_digit()
        && value[3].is_ascii_digit()
        && value[4].is_ascii_digit()
        && value[5] == b'.'
        && value[6].is_ascii_digit()
        && value[7].is_ascii_digit()
        && matches!(value[8], b'E' | b'W'))
    {
        return false;
    }

    coordinate_in_range(&value[..3], &value[3..8], 180)
}

fn coordinate_in_range(degrees: &[u8], minutes: &[u8], max_degrees: u16) -> bool {
    let Some(degrees) = parse_u16(degrees) else {
        return false;
    };
    let Some(minutes) = parse_fixed_minutes(minutes) else {
        return false;
    };

    degrees < max_degrees || (degrees == max_degrees && minutes == 0.0)
}

fn is_symbol_table_identifier(value: u8) -> bool {
    matches!(value, b'/' | b'\\') || value.is_ascii_alphanumeric()
}

fn is_printable_ascii(value: u8) -> bool {
    (0x20..=0x7e).contains(&value)
}

fn is_base91(value: u8) -> bool {
    (b'!'..=b'{').contains(&value)
}

fn is_compressed_position(information: &[u8]) -> bool {
    information
        .first()
        .is_some_and(|byte| !byte.is_ascii_digit() && is_symbol_table_identifier(*byte))
        && information
            .get(1..13)
            .is_some_and(|bytes| bytes.iter().all(|byte| is_base91(*byte)))
}

fn is_timestamp(value: &[u8]) -> bool {
    value.len() == 7
        && value[..6].iter().all(u8::is_ascii_digit)
        && matches!(value[6], b'z' | b'/' | b'h')
}

fn is_maidenhead_locator(value: &[u8]) -> bool {
    value.len() == 6
        && is_ascii_alpha_range(value[0], b'A', b'R')
        && is_ascii_alpha_range(value[1], b'A', b'R')
        && value[2].is_ascii_digit()
        && value[3].is_ascii_digit()
        && is_ascii_alpha_range(value[4], b'A', b'X')
        && is_ascii_alpha_range(value[5], b'A', b'X')
}

fn is_ascii_alpha_range(value: u8, start: u8, end: u8) -> bool {
    let uppercase = value.to_ascii_uppercase();
    (start..=end).contains(&uppercase)
}

fn decode_latitude(value: &[u8]) -> Option<f64> {
    if !is_latitude(value) {
        return None;
    }

    let degrees = parse_u16(&value[..2])? as f64;
    let minutes = parse_fixed_minutes(&value[2..7])?;
    let sign = match value[7] {
        b'N' => 1.0,
        b'S' => -1.0,
        _ => return None,
    };

    Some(sign * (degrees + minutes / 60.0))
}

fn decode_longitude(value: &[u8]) -> Option<f64> {
    if !is_longitude(value) {
        return None;
    }

    let degrees = parse_u16(&value[..3])? as f64;
    let minutes = parse_fixed_minutes(&value[3..8])?;
    let sign = match value[8] {
        b'E' => 1.0,
        b'W' => -1.0,
        _ => return None,
    };

    Some(sign * (degrees + minutes / 60.0))
}

fn parse_fixed_minutes(value: &[u8]) -> Option<f64> {
    if value.len() != 5 || value[2] != b'.' || !value[..2].iter().all(u8::is_ascii_digit) {
        return None;
    }

    let whole = parse_u16(&value[..2])? as f64;
    let fraction = parse_u16(&value[3..])? as f64 / 100.0;
    Some(whole + fraction)
}

fn decode_base91(value: &[u8]) -> Option<u32> {
    if value.len() != 4 || !value.iter().all(|byte| is_base91(*byte)) {
        return None;
    }

    let mut decoded = 0u32;
    for byte in value {
        decoded = decoded * 91 + u32::from(byte - b'!');
    }

    Some(decoded)
}

fn parse_u16(value: &[u8]) -> Option<u16> {
    if value.is_empty() || !value.iter().all(u8::is_ascii_digit) {
        return None;
    }

    let mut parsed = 0u16;
    for digit in value {
        parsed = parsed.checked_mul(10)?;
        parsed = parsed.checked_add(u16::from(digit - b'0'))?;
    }

    Some(parsed)
}

fn parse_i16(value: &[u8]) -> Option<i16> {
    if value.is_empty() {
        return None;
    }

    let (sign, digits) = match value[0] {
        b'-' => (-1, &value[1..]),
        b'+' => (1, &value[1..]),
        _ => (1, value),
    };

    let unsigned = parse_u16(digits)?;
    i16::try_from(unsigned).ok()?.checked_mul(sign)
}

fn parse_hex_byte(value: &[u8]) -> Option<u8> {
    if value.len() != 2 {
        return None;
    }

    Some(hex_value(value[0])? * 16 + hex_value(value[1])?)
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'A'..=b'F' => Some(value - b'A' + 10),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}

fn parse_tagged_u16(report: &[u8], tag: u8, width: usize) -> Option<u16> {
    parse_tagged(report, tag, width).and_then(parse_u16)
}

fn parse_tagged_i16(report: &[u8], tag: u8, width: usize) -> Option<i16> {
    parse_tagged(report, tag, width).and_then(parse_i16)
}

fn parse_tagged(report: &[u8], tag: u8, width: usize) -> Option<&[u8]> {
    let start = report.iter().position(|byte| *byte == tag)? + 1;
    report.get(start..start + width)
}

fn decode_mic_e_status(destination: &[u8]) -> Option<MicEStatus> {
    if destination.len() != 6 {
        return None;
    }

    let bytes = destination.get(..3)?;
    Some(MicEStatus::Custom([
        mic_e_status_bit(bytes[0])?,
        mic_e_status_bit(bytes[1])?,
        mic_e_status_bit(bytes[2])?,
    ]))
}

fn decode_mic_e_message_code(destination: &[u8]) -> Option<MicEMessageCode> {
    if destination.len() != 6 {
        return None;
    }

    let mut bits = [MicEMessageBit::Zero; 3];
    for (index, byte) in destination[..3].iter().copied().enumerate() {
        bits[index] = mic_e_message_bit(byte)?;
    }

    let code = message_code_number([
        !matches!(bits[0], MicEMessageBit::Zero),
        !matches!(bits[1], MicEMessageBit::Zero),
        !matches!(bits[2], MicEMessageBit::Zero),
    ]);

    if code == 7 {
        return Some(MicEMessageCode::Emergency);
    }

    let has_standard = bits
        .iter()
        .any(|bit| matches!(bit, MicEMessageBit::StandardOne));
    let has_custom = bits
        .iter()
        .any(|bit| matches!(bit, MicEMessageBit::CustomOne));

    if has_standard && !has_custom {
        return standard_mic_e_message(code).map(MicEMessageCode::Standard);
    }

    if has_custom && !has_standard {
        return Some(MicEMessageCode::Custom(code));
    }

    None
}

#[derive(Clone, Copy)]
enum MicEMessageBit {
    Zero,
    StandardOne,
    CustomOne,
}

fn mic_e_message_bit(byte: u8) -> Option<MicEMessageBit> {
    match byte {
        b'0'..=b'9' | b'L' => Some(MicEMessageBit::Zero),
        b'A'..=b'K' => Some(MicEMessageBit::StandardOne),
        b'P'..=b'Z' => Some(MicEMessageBit::CustomOne),
        _ => None,
    }
}

fn message_code_number(bits: [bool; 3]) -> u8 {
    match bits {
        [true, true, true] => 0,
        [true, true, false] => 1,
        [true, false, true] => 2,
        [true, false, false] => 3,
        [false, true, true] => 4,
        [false, true, false] => 5,
        [false, false, true] => 6,
        [false, false, false] => 7,
    }
}

fn standard_mic_e_message(code: u8) -> Option<MicEStandardMessage> {
    match code {
        0 => Some(MicEStandardMessage::OffDuty),
        1 => Some(MicEStandardMessage::EnRoute),
        2 => Some(MicEStandardMessage::InService),
        3 => Some(MicEStandardMessage::Returning),
        4 => Some(MicEStandardMessage::Committed),
        5 => Some(MicEStandardMessage::Special),
        6 => Some(MicEStandardMessage::Priority),
        _ => None,
    }
}

fn mic_e_status_bit(byte: u8) -> Option<bool> {
    match byte {
        b'0'..=b'9' | b'L' => Some(false),
        b'A'..=b'K' | b'P'..=b'Z' => Some(true),
        _ => None,
    }
}

fn decode_mic_e_latitude_digits(destination: &[u8]) -> Option<[u8; 6]> {
    if destination.len() != 6 {
        return None;
    }

    let mut digits = [0u8; 6];
    for (index, byte) in destination.iter().copied().enumerate() {
        digits[index] = mic_e_latitude_digit(byte)?;
    }

    Some(digits)
}

fn mic_e_latitude_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'J' => Some(byte - b'A'),
        b'P'..=b'Y' => Some(byte - b'P'),
        b'K' | b'L' | b'Z' => Some(0),
        _ => None,
    }
}

fn decode_mic_e_latitude(destination: &[u8]) -> Option<f64> {
    let digits = decode_mic_e_latitude_digits(destination)?;
    let degrees = u16::from(digits[0]) * 10 + u16::from(digits[1]);
    let minutes = u16::from(digits[2]) * 10 + u16::from(digits[3]);
    let hundredths = u16::from(digits[4]) * 10 + u16::from(digits[5]);
    if degrees > 90 || minutes > 59 {
        return None;
    }

    let sign = if mic_e_north(destination[3])? {
        1.0
    } else {
        -1.0
    };
    Some(sign * (f64::from(degrees) + (f64::from(minutes) + f64::from(hundredths) / 100.0) / 60.0))
}

fn decode_mic_e_longitude(destination: &[u8], body: &[u8]) -> Option<f64> {
    if destination.len() != 6 || body.len() < 3 {
        return None;
    }

    let mut degrees = i16::from(mic_e_body_value(body[0])?);
    if mic_e_longitude_offset(destination[4])? {
        degrees += 100;
    }
    if (180..=189).contains(&degrees) {
        degrees -= 80;
    } else if (190..=199).contains(&degrees) {
        degrees -= 190;
    }

    let minutes = mic_e_body_value(body[1])?;
    let hundredths = mic_e_body_value(body[2])?;
    if !(0..=179).contains(&degrees) || minutes > 59 || hundredths > 99 {
        return None;
    }

    let sign = if mic_e_west(destination[5])? {
        -1.0
    } else {
        1.0
    };
    Some(sign * (f64::from(degrees) + (f64::from(minutes) + f64::from(hundredths) / 100.0) / 60.0))
}

fn decode_mic_e_speed_course(body: &[u8]) -> Option<MicESpeedCourse> {
    if body.len() < 6 {
        return None;
    }

    let speed_tens = u16::from(mic_e_body_value(body[3])?);
    let speed_units_course_hundreds = u16::from(mic_e_body_value(body[4])?);
    let course_remainder = u16::from(mic_e_body_value(body[5])?);
    let mut speed_knots = speed_tens * 10 + speed_units_course_hundreds / 10;
    if speed_knots >= 800 {
        speed_knots -= 800;
    }

    Some(MicESpeedCourse {
        speed_knots,
        course_degrees: (speed_units_course_hundreds % 10) * 100 + course_remainder,
    })
}

fn mic_e_body_value(byte: u8) -> Option<u8> {
    let value = byte.checked_sub(28)?;
    (value <= 99).then_some(value)
}

fn mic_e_north(byte: u8) -> Option<bool> {
    match byte {
        b'0'..=b'9' | b'A'..=b'L' => Some(false),
        b'P'..=b'Z' => Some(true),
        _ => None,
    }
}

fn mic_e_longitude_offset(byte: u8) -> Option<bool> {
    match byte {
        b'0'..=b'9' | b'A'..=b'L' => Some(false),
        b'P'..=b'Z' => Some(true),
        _ => None,
    }
}

fn mic_e_west(byte: u8) -> Option<bool> {
    match byte {
        b'0'..=b'9' | b'A'..=b'L' => Some(false),
        b'P'..=b'Z' => Some(true),
        _ => None,
    }
}

/// Fail-closed packet parse errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// No bytes were supplied.
    Empty,
    /// Packet exceeds [`MAX_PACKET_LEN`].
    Oversized,
    /// Packet does not contain the required APRS `>` and `:` separators.
    MissingSeparator,
    /// Packet contains an empty source, path, or payload segment.
    EmptySegment,
    /// Packet source or path contains bytes outside the conservative address set.
    InvalidAddress,
}

impl ParseError {
    /// Returns a stable parse error code for logs and external systems.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Empty => "parse.empty",
            Self::Oversized => "parse.oversized",
            Self::MissingSeparator => "parse.missing_separator",
            Self::EmptySegment => "parse.empty_segment",
            Self::InvalidAddress => "parse.invalid_address",
        }
    }

    /// Returns structured parse error metadata for operator diagnostics.
    #[must_use]
    pub fn diagnostic(&self) -> ErrorDiagnostic {
        match self {
            Self::Empty => ErrorDiagnostic {
                layer: DiagnosticLayer::Parse,
                code: self.code(),
                name: "empty",
                description: "no packet bytes were supplied to the codec boundary",
                remediation: "drop empty transport records before calling parse_packet",
            },
            Self::Oversized => ErrorDiagnostic {
                layer: DiagnosticLayer::Parse,
                code: self.code(),
                name: "oversized",
                description: "packet exceeds the configured parser byte limit",
                remediation: "reject the input or lower upstream batch sizes before parsing",
            },
            Self::MissingSeparator => ErrorDiagnostic {
                layer: DiagnosticLayer::Parse,
                code: self.code(),
                name: "missing_separator",
                description: "packet is missing the required source>path:payload separators",
                remediation: "only send source>path:payload APRS packet bytes into the codec",
            },
            Self::EmptySegment => ErrorDiagnostic {
                layer: DiagnosticLayer::Parse,
                code: self.code(),
                name: "empty_segment",
                description: "packet contains an empty source, path, or payload segment",
                remediation: "reject the input and inspect upstream framing before retrying",
            },
            Self::InvalidAddress => ErrorDiagnostic {
                layer: DiagnosticLayer::Parse,
                code: self.code(),
                name: "invalid_address",
                description:
                    "packet source or path contains bytes outside the conservative address set",
                remediation:
                    "preserve the raw bytes for review and reject malformed address metadata",
            },
        }
    }
}

/// Parses an APRS packet from untrusted bytes.
///
/// This parser intentionally validates only the minimal frame shape for the
/// skeleton: `source>path:payload`. Payload bytes are opaque and may be invalid
/// UTF-8.
pub fn parse_packet(input: &[u8]) -> Result<ParsedPacket, ParseError> {
    parse_packet_with_options(input, ParseOptions::default())
}

/// Parses an APRS packet from untrusted bytes with explicit codec options.
pub fn parse_packet_with_options(
    input: &[u8],
    options: ParseOptions,
) -> Result<ParsedPacket, ParseError> {
    if input.is_empty() {
        return Err(ParseError::Empty);
    }

    if input.len() > options.max_packet_len {
        return Err(ParseError::Oversized);
    }

    let source_end = input
        .iter()
        .position(|byte| *byte == b'>')
        .ok_or(ParseError::MissingSeparator)?;
    let payload_separator = input[source_end + 1..]
        .iter()
        .position(|byte| *byte == b':')
        .map(|offset| source_end + 1 + offset)
        .ok_or(ParseError::MissingSeparator)?;

    let path_start = source_end + 1;
    let path_end = payload_separator;
    let payload_start = payload_separator + 1;

    if source_end == 0 || path_start == path_end || payload_start == input.len() {
        return Err(ParseError::EmptySegment);
    }

    let Some(path_components) = path_component_ranges(input, path_start, path_end) else {
        return Err(ParseError::InvalidAddress);
    };

    if !is_ax25_like_source(&input[..source_end])
        || !path_components
            .iter()
            .all(|(start, end)| is_ax25_like_path_component(&input[*start..*end]))
    {
        return Err(ParseError::InvalidAddress);
    }

    Ok(ParsedPacket {
        raw: RawPacket {
            bytes: input.to_vec(),
        },
        source_end,
        path_start,
        path_end,
        path_components,
        payload_start,
    })
}

fn path_component_ranges(
    input: &[u8],
    path_start: usize,
    path_end: usize,
) -> Option<Vec<(usize, usize)>> {
    let mut components = Vec::new();
    let mut component_start = path_start;

    for (offset, byte) in input[path_start..path_end].iter().enumerate() {
        if *byte == b',' {
            let index = path_start + offset;
            if component_start == index {
                return None;
            }
            components.push((component_start, index));
            component_start = index + 1;
        }
    }

    if component_start == path_end {
        return None;
    }

    components.push((component_start, path_end));
    Some(components)
}

fn is_ax25_like_source(source: &[u8]) -> bool {
    is_ax25_like_address(source, false)
}

fn is_ax25_like_path_component(component: &[u8]) -> bool {
    is_ax25_like_address(component, true)
}

fn is_ax25_like_address(address: &[u8], allow_repeated_marker: bool) -> bool {
    let address = if allow_repeated_marker {
        address.strip_suffix(b"*").unwrap_or(address)
    } else {
        address
    };

    if address.is_empty() || address.contains(&b'*') {
        return false;
    }

    let (callsign, ssid) = match address.iter().position(|byte| *byte == b'-') {
        Some(separator) => (&address[..separator], Some(&address[separator + 1..])),
        None => (address, None),
    };

    is_ax25_like_callsign(callsign) && ssid.map_or(true, is_ax25_like_ssid)
}

fn is_ax25_like_callsign(callsign: &[u8]) -> bool {
    (1..=6).contains(&callsign.len())
        && callsign
            .iter()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

fn is_ax25_like_ssid(ssid: &[u8]) -> bool {
    if ssid.is_empty() || ssid.len() > 2 || !ssid.iter().all(u8::is_ascii_digit) {
        return false;
    }

    let mut value = 0u8;
    for digit in ssid {
        value = value * 10 + (digit - b'0');
    }

    value <= 15
}
