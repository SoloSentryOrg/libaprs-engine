//! Runtime-neutral service building blocks.
//!
//! These helpers keep storage, networking, clocks, and task runtimes
//! application-owned while covering common ingestion policy needs.

use crate::AprsData;

/// Duplicate suppression decision for a packet byte sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DuplicateDecision {
    /// Packet bytes were not seen in the retained window.
    New,
    /// Packet bytes match a retained packet in the window.
    Duplicate,
}

impl DuplicateDecision {
    /// Stable machine-readable decision code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::New => "duplicate.new",
            Self::Duplicate => "duplicate.duplicate",
        }
    }
}

/// Bounded exact-byte duplicate window.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DuplicateWindow {
    capacity: usize,
    packets: Vec<Vec<u8>>,
}

impl DuplicateWindow {
    /// Creates a duplicate window retaining at most `capacity` packet byte
    /// sequences. A zero-capacity window never retains packets.
    #[must_use]
    pub const fn new(capacity: usize) -> Self {
        Self {
            capacity,
            packets: Vec::new(),
        }
    }

    /// Observes packet bytes and reports whether the exact bytes were recently
    /// retained.
    pub fn observe(&mut self, packet: &[u8]) -> DuplicateDecision {
        if self.packets.iter().any(|existing| existing == packet) {
            return DuplicateDecision::Duplicate;
        }

        if self.capacity > 0 {
            if self.packets.len() == self.capacity {
                self.packets.remove(0);
            }
            self.packets.push(packet.to_vec());
        }

        DuplicateDecision::New
    }

    /// Returns the number of packet byte sequences currently retained.
    #[must_use]
    pub fn retained_len(&self) -> usize {
        self.packets.len()
    }

    /// Returns the configured retention capacity.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Packet-rate budget decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RateLimitDecision {
    /// The caller may process this packet.
    Allowed,
    /// The caller-owned budget has been exhausted.
    Limited,
}

impl RateLimitDecision {
    /// Stable machine-readable decision code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Allowed => "rate.allowed",
            Self::Limited => "rate.limited",
        }
    }
}

/// Caller-reset packet-rate budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PacketRateBudget {
    limit: u64,
    remaining: u64,
}

impl PacketRateBudget {
    /// Creates a packet-rate budget with `limit` allowed packets per
    /// caller-owned window.
    #[must_use]
    pub const fn new(limit: u64) -> Self {
        Self {
            limit,
            remaining: limit,
        }
    }

    /// Attempts to consume one packet from the current budget.
    pub fn try_consume(&mut self) -> RateLimitDecision {
        if self.remaining == 0 {
            return RateLimitDecision::Limited;
        }

        self.remaining -= 1;
        RateLimitDecision::Allowed
    }

    /// Resets the current budget back to the configured limit.
    pub fn reset(&mut self) {
        self.remaining = self.limit;
    }

    /// Returns the configured packet limit.
    #[must_use]
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    /// Returns the remaining packet budget in the current caller-owned window.
    #[must_use]
    pub const fn remaining(&self) -> u64 {
        self.remaining
    }
}

/// Semantic packet family for service-level blocklists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticFamily {
    /// Status report.
    Status,
    /// Position without timestamp.
    Position,
    /// Timestamped position.
    TimestampedPosition,
    /// Compressed position.
    CompressedPosition,
    /// Message, bulletin, announcement, acknowledgement, or rejection.
    Message,
    /// Object report.
    Object,
    /// Item report.
    Item,
    /// Weather report.
    Weather,
    /// Telemetry report.
    Telemetry,
    /// Telemetry metadata message.
    TelemetryMetadata,
    /// Query packet.
    Query,
    /// Station capabilities packet.
    Capability,
    /// NMEA sentence packet.
    Nmea,
    /// Mic-E packet.
    MicE,
    /// Maidenhead locator packet.
    Maidenhead,
    /// User-defined packet.
    UserDefined,
    /// Third-party traffic packet.
    ThirdParty,
    /// Unsupported data type identifier.
    Unsupported,
    /// Codec-valid but semantically malformed packet.
    Malformed,
}

impl SemanticFamily {
    /// Classifies a semantic packet view into a stable family.
    #[must_use]
    pub const fn from_aprs_data(data: &AprsData<'_>) -> Self {
        match data {
            AprsData::Status { .. } => Self::Status,
            AprsData::Position(_) => Self::Position,
            AprsData::TimestampedPosition(_) => Self::TimestampedPosition,
            AprsData::CompressedPosition(_) => Self::CompressedPosition,
            AprsData::Message(_) => Self::Message,
            AprsData::Object(_) => Self::Object,
            AprsData::Item(_) => Self::Item,
            AprsData::Weather(_) => Self::Weather,
            AprsData::Telemetry(_) => Self::Telemetry,
            AprsData::TelemetryMetadata(_) => Self::TelemetryMetadata,
            AprsData::Query(_) => Self::Query,
            AprsData::Capability(_) => Self::Capability,
            AprsData::Nmea(_) => Self::Nmea,
            AprsData::MicE(_) => Self::MicE,
            AprsData::Maidenhead(_) => Self::Maidenhead,
            AprsData::UserDefined(_) => Self::UserDefined,
            AprsData::ThirdParty(_) => Self::ThirdParty,
            AprsData::Unsupported { .. } => Self::Unsupported,
            AprsData::Malformed { .. } => Self::Malformed,
        }
    }

    /// Stable machine-readable family code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Position => "position",
            Self::TimestampedPosition => "timestamped_position",
            Self::CompressedPosition => "compressed_position",
            Self::Message => "message",
            Self::Object => "object",
            Self::Item => "item",
            Self::Weather => "weather",
            Self::Telemetry => "telemetry",
            Self::TelemetryMetadata => "telemetry_metadata",
            Self::Query => "query",
            Self::Capability => "capability",
            Self::Nmea => "nmea",
            Self::MicE => "mic_e",
            Self::Maidenhead => "maidenhead",
            Self::UserDefined => "user_defined",
            Self::ThirdParty => "third_party",
            Self::Unsupported => "unsupported",
            Self::Malformed => "malformed",
        }
    }
}

/// Runtime-neutral semantic-family blocklist helper.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticBlocklist<'a> {
    families: &'a [SemanticFamily],
}

impl<'a> SemanticBlocklist<'a> {
    /// Creates a blocklist over caller-owned semantic-family storage.
    #[must_use]
    pub const fn new(families: &'a [SemanticFamily]) -> Self {
        Self { families }
    }

    /// Returns true when the semantic packet family is blocklisted.
    #[must_use]
    pub fn rejects(&self, data: &AprsData<'_>) -> bool {
        let family = SemanticFamily::from_aprs_data(data);
        self.families.contains(&family)
    }

    /// Returns the caller-owned blocked families.
    #[must_use]
    pub const fn families(&self) -> &'a [SemanticFamily] {
        self.families
    }
}
