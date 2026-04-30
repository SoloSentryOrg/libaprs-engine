//! Optional serde-backed diagnostic structures.
//!
//! This module is available with the `serde` feature. It intentionally exposes
//! owned byte vectors so serialization cannot depend on UTF-8 validity.

use serde::Serialize;

use crate::ParsedPacket;

/// Current packet diagnostic schema version.
pub const PACKET_DIAGNOSTIC_SCHEMA_VERSION: u8 = 1;

/// Stable diagnostic packet representation for serde-based integrations.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PacketDiagnostic {
    /// Version of this diagnostic structure.
    pub schema_version: u8,
    /// Original raw packet bytes.
    pub raw: Vec<u8>,
    /// Source address bytes.
    pub source: Vec<u8>,
    /// Destination address bytes.
    pub destination: Vec<u8>,
    /// Complete destination/path bytes.
    pub path: Vec<u8>,
    /// Complete payload bytes.
    pub payload: Vec<u8>,
    /// Data type identifier name.
    pub data_type: &'static str,
    /// APRS semantic kind name.
    pub semantic: &'static str,
}

impl PacketDiagnostic {
    /// Builds an owned diagnostic value from a parsed packet.
    #[must_use]
    pub fn from_packet(packet: &ParsedPacket) -> Self {
        Self {
            schema_version: PACKET_DIAGNOSTIC_SCHEMA_VERSION,
            raw: packet.raw().as_bytes().to_vec(),
            source: packet.source().to_vec(),
            destination: packet.destination().to_vec(),
            path: packet.path().to_vec(),
            payload: packet.payload().to_vec(),
            data_type: packet.data_type_identifier().name(),
            semantic: packet.aprs_data().kind_name(),
        }
    }
}
