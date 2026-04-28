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
| Object | Supported | Name, liveness, timestamp, body, coordinates when body starts with a supported position | Body preserved |
| Item | Supported | Name, liveness, body, coordinates when body starts with a supported position | Body preserved |
| Weather | Supported | Common numeric fields, luminosity, snow, raw rain counter | Invalid optional fields are ignored |
| Telemetry | Supported | Sequence, analog, digital bits | Report values preserved and numerically decoded when safe |
| Telemetry metadata | Supported | Parameter names, units, equations, bit sense | `PARM.`, `UNIT.`, `EQNS.`, and `BITS.` message packets |
| Query | Supported | Query bytes | Query body preserved |
| Capability | Supported | Body bytes | Capability fields not split |
| NMEA | Supported | Sentence bytes, talker ID, sentence ID, data fields, checksum details | Invalid checksums are reported; policy can reject checksum mismatches |
| Mic-E | Supported | Status bits, message code, latitude digits, coordinates, speed/course | Values decode only when destination/body bytes permit it |
| Maidenhead | Supported | Locator bytes | Locator syntax is minimally framed |
| User-defined | Supported | User ID, packet type, body | Body preserved |
| Third-party traffic | Supported | Encapsulated bytes, explicit nested parser | Nested parsing is caller-controlled via API |
| Unknown identifier | Supported | Explicit unsupported variant | Not guessed |
| Malformed semantic payload | Supported | Explicit malformed variant | Policy rejects by default |

## Future Conformance Work

- Additional semantic consistency policies beyond NMEA checksum mismatch.
- More real-world corpus fixtures for Mic-E, NMEA, object/item positions, and
  third-party nested traffic.
- More policy-rejection fixtures as semantic policies are added.

## Fixture Coverage

The checked-in fixture corpus covers representative examples for status,
positions, compressed positions, messages, acknowledgements, rejects,
bulletins, announcements, objects, items, weather, telemetry, telemetry
metadata, query, capability, NMEA with checksum, Mic-E, Maidenhead,
user-defined data, third-party traffic, and unsupported identifier handling.
The APRS101-oriented fixture set stores packet bytes separately from fixture
IDs and requires source-reference entries for every case.
Tests assert raw-byte preservation, source-reference coverage, minimum semantic
family coverage, expected semantic family classification, selected subtype
classification, and malformed semantic visibility.

## Compatibility Coverage

The conformance corpus verifies packet-family behavior. The separate
`api_compat` test verifies documented integration APIs, including parser entry
points, parse options, stable error codes, engine/policy flow, line transport,
and typed helper methods.
