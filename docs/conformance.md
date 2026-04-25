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
| Telemetry | Supported | Sequence, analog, digital bits | Report values preserved and numerically decoded when safe |
| Telemetry metadata | Supported | Parameter names, units, equations, bit sense | `PARM.`, `UNIT.`, `EQNS.`, and `BITS.` message packets |
| Query | Supported | Query bytes | Query body preserved |
| Capability | Supported | Body bytes | Capability fields not split |
| NMEA | Supported | Sentence bytes, checksum details | Invalid checksums are reported, not rejected |
| Mic-E | Supported | Status bits, latitude digits, coordinates, speed/course | Values decode only when destination/body bytes permit it |
| Maidenhead | Supported | Locator bytes | Locator syntax is minimally framed |
| User-defined | Supported | User ID, packet type, body | Body preserved |
| Third-party traffic | Supported | Encapsulated bytes, explicit nested parser | Nested parsing is caller-controlled via API |
| Unknown identifier | Supported | Explicit unsupported variant | Not guessed |
| Malformed semantic payload | Supported | Explicit malformed variant | Policy rejects by default |

## Future Conformance Work

- Broader APRS101 fixture corpus with source references.
- More exhaustive weather grammar coverage.
- Optional strict policies for semantic checksum failures.

## Fixture Coverage

The checked-in fixture corpus covers representative examples for status,
positions, compressed positions, messages, objects, items, weather, telemetry,
telemetry metadata, query, capability, NMEA with checksum, Mic-E, Maidenhead,
user-defined data, and third-party traffic. Fixtures are intentionally small and
byte-preserving; future additions should include source references when they are
derived from APRS101 examples or real packet captures.
