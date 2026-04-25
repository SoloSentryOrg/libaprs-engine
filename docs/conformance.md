# APRS Conformance Matrix

The parser is protocol-first and byte-preserving. This matrix tracks current
semantic coverage and known gaps for APRS101 packet families.

| Family | Status | Typed interpretation | Notes |
| --- | --- | --- | --- |
| Status | Supported | Byte-preserving text | Invalid UTF-8 allowed |
| Uncompressed position | Supported | Decimal coordinates | Conservative coordinate validation |
| Timestamped position | Supported | Timestamp bytes, decimal coordinates | Timestamp shape validated |
| Compressed position | Supported | Decimal coordinates | Extension bytes preserved |
| Message | Supported | Kind classification, message ID | Addressee content remains bytes |
| Bulletin | Supported | Message kind | Classified via addressee |
| Announcement | Supported | Message kind | Classified via addressee |
| Acknowledgement | Supported | Message kind | `ack` prefix |
| Reject | Supported | Message kind | `rej` prefix |
| Object | Supported | Name, liveness, timestamp, body | Body preserved |
| Item | Supported | Name, liveness, body | Body preserved |
| Weather | Supported | Common numeric fields | Full weather grammar not exhaustive |
| Telemetry | Supported | Sequence, analog, digital bits | Calibration/equation packets not yet modeled |
| Query | Supported | Query bytes | Query body preserved |
| Capability | Supported | Body bytes | Capability fields not split |
| NMEA | Supported | Sentence bytes | NMEA checksum not validated |
| Mic-E | Partial | Status bits, latitude digits | Full Mic-E position decoding remains future work |
| Maidenhead | Supported | Locator bytes | Locator syntax is minimally framed |
| User-defined | Supported | User ID, packet type, body | Body preserved |
| Third-party traffic | Supported | Encapsulated bytes | Nested packet not recursively parsed |
| Unknown identifier | Supported | Explicit unsupported variant | Not guessed |
| Malformed semantic payload | Supported | Explicit malformed variant | Policy rejects by default |

## Future Conformance Work

- Full Mic-E position, speed, course, and telemetry decoding.
- Telemetry metadata, calibration, and unit/equation packet modeling.
- Optional NMEA checksum validation.
- Broader APRS101 fixture corpus with source references.
- Recursive third-party packet parsing behind an explicit API.
