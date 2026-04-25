//! Protocol-first APRS engine core primitives.
//!
//! The codec boundary accepts untrusted bytes, preserves them exactly, and
//! fails closed when the packet shape is malformed.

/// Conservative upper bound for an APRS packet handled by this skeleton.
pub const MAX_PACKET_LEN: usize = 512;

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

    /// Serializes the parsed packet into a compact JSON diagnostic string.
    #[must_use]
    pub fn to_json(&self) -> String {
        format!(
            "{{\"raw\":\"{}\",\"source\":\"{}\",\"destination\":\"{}\",\"path\":\"{}\",\"payload\":\"{}\",\"data_type\":\"{}\",\"semantic\":\"{}\"}}",
            escape_json_bytes(self.raw().as_bytes()),
            escape_json_bytes(self.source()),
            escape_json_bytes(self.destination()),
            escape_json_bytes(self.path()),
            escape_json_bytes(self.payload()),
            self.data_type_identifier().name(),
            self.aprs_data().kind_name(),
        )
    }
}

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
                        self.counters.accepted += 1;
                        EngineResult::Accepted { packet }
                    }
                    PolicyDecision::Reject(reason) => {
                        self.counters.rejected += 1;
                        EngineResult::Rejected { packet, reason }
                    }
                }
            }
            Err(error) => {
                self.counters.malformed += 1;
                EngineResult::ParseError(error)
            }
        }
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

/// Line-oriented packet source for file/stdin style transports.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineTransport<'a> {
    input: &'a [u8],
}

impl<'a> LineTransport<'a> {
    /// Creates a transport over newline-separated packet bytes.
    #[must_use]
    pub fn new(input: &'a [u8]) -> Self {
        Self { input }
    }

    /// Iterates packet lines without trailing CR/LF bytes.
    #[must_use]
    pub fn packets(&self) -> Vec<&'a [u8]> {
        self.input
            .split(|byte| *byte == b'\n')
            .map(trim_trailing_carriage_return)
            .filter(|line| !line.is_empty())
            .collect()
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
            max_path_components: 9,
        }
    }

    /// Evaluates a parsed packet and semantic view.
    #[must_use]
    pub fn evaluate(&self, packet: &ParsedPacket, semantic: &AprsData<'_>) -> PolicyDecision {
        if packet.path_components.len() > self.max_path_components {
            return PolicyDecision::Reject(PolicyRejection::PathTooLong);
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

/// Mic-E destination-derived status bits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MicEStatus {
    /// Standard/custom status bit tuple from the first three destination bytes.
    Custom([bool; 3]),
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
        DataTypeIdentifier::Weather => AprsData::Weather(Weather {
            report: information,
        }),
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
        DataTypeIdentifier::ThirdParty => AprsData::ThirdParty(ThirdParty { body: information }),
        other => AprsData::Unsupported {
            identifier: other.as_byte(),
            information,
        },
    }
}

fn parse_mic_e<'a>(
    identifier: DataTypeIdentifier,
    information: &'a [u8],
    destination: &'a [u8],
) -> AprsData<'a> {
    AprsData::MicE(MicE {
        identifier: identifier.as_byte(),
        destination,
        body: information,
        status: decode_mic_e_status(destination),
        latitude_digits: decode_mic_e_latitude_digits(destination),
    })
}

fn parse_position<'a>(messaging: bool, identifier: u8, information: &'a [u8]) -> AprsData<'a> {
    if is_compressed_position(information) {
        return parse_compressed_position(messaging, identifier, information);
    }

    if information.len() < 18 {
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

fn parse_timestamped_position<'a>(
    messaging: bool,
    identifier: u8,
    information: &'a [u8],
) -> AprsData<'a> {
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

fn parse_compressed_position<'a>(
    messaging: bool,
    identifier: u8,
    information: &'a [u8],
) -> AprsData<'a> {
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
    if information.len() < 17 || !matches!(information[9], b'*' | b'_') {
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
    if information.len() < 6 {
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
    value.len() == 8
        && value[0].is_ascii_digit()
        && value[1].is_ascii_digit()
        && value[2].is_ascii_digit()
        && value[3].is_ascii_digit()
        && value[4] == b'.'
        && value[5].is_ascii_digit()
        && value[6].is_ascii_digit()
        && matches!(value[7], b'N' | b'S')
}

fn is_longitude(value: &[u8]) -> bool {
    value.len() == 9
        && value[0].is_ascii_digit()
        && value[1].is_ascii_digit()
        && value[2].is_ascii_digit()
        && value[3].is_ascii_digit()
        && value[4].is_ascii_digit()
        && value[5] == b'.'
        && value[6].is_ascii_digit()
        && value[7].is_ascii_digit()
        && matches!(value[8], b'E' | b'W')
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

fn escape_json_bytes(bytes: &[u8]) -> String {
    let mut escaped = String::new();
    for byte in bytes {
        match byte {
            b'"' => escaped.push_str("\\\""),
            b'\\' => escaped.push_str("\\\\"),
            b'\n' => escaped.push_str("\\n"),
            b'\r' => escaped.push_str("\\r"),
            b'\t' => escaped.push_str("\\t"),
            0x20..=0x7e => escaped.push(char::from(*byte)),
            _ => {
                escaped.push_str("\\u00");
                escaped.push(hex_digit(byte >> 4));
                escaped.push(hex_digit(byte & 0x0f));
            }
        }
    }
    escaped
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + value - 10),
        _ => '0',
    }
}

fn trim_trailing_carriage_return(line: &[u8]) -> &[u8] {
    line.strip_suffix(b"\r").unwrap_or(line)
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

/// Parses an APRS packet from untrusted bytes.
///
/// This parser intentionally validates only the minimal frame shape for the
/// skeleton: `source>path:payload`. Payload bytes are opaque and may be invalid
/// UTF-8.
pub fn parse_packet(input: &[u8]) -> Result<ParsedPacket, ParseError> {
    if input.is_empty() {
        return Err(ParseError::Empty);
    }

    if input.len() > MAX_PACKET_LEN {
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
