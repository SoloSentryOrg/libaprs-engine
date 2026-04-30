# APRS Conformance Matrix

The parser is protocol-first and byte-preserving. This matrix tracks current
semantic coverage and known gaps for APRS101 packet families.

| Family | Status | Typed interpretation | Notes |
| --- | --- | --- | --- |
| Status | Supported | Byte-preserving text | Invalid UTF-8 allowed |
| Uncompressed position | Supported | Decimal coordinates, embedded weather report when symbol code is `_` | Conservative coordinate validation |
| Timestamped position | Supported | Timestamp bytes, decimal coordinates, embedded weather report when symbol code is `_` | Timestamp shape validated |
| Compressed position | Supported | Decimal coordinates | Extension bytes preserved |
| Message | Supported | Kind classification, message ID | Addressee content remains bytes |
| Bulletin | Supported | Message kind | Classified via addressee |
| Announcement | Supported | Message kind | Classified via addressee |
| Acknowledgement | Supported | Message kind | `ack` prefix |
| Reject | Supported | Message kind | `rej` prefix |
| Object | Supported | Name, liveness, timestamp, body, coordinates/weather when body starts with a supported position | Timestamp shape validated; body preserved |
| Item | Supported | Name, liveness, body, coordinates/weather when body starts with a supported position | Body preserved |
| Weather | Supported | Positionless and weather-symbol position reports, common numeric fields, luminosity, snow, raw rain counter | Empty reports are malformed; invalid optional fields are ignored |
| Telemetry | Supported | Sequence, analog, digital bits | Report values preserved and numerically decoded when safe |
| Telemetry metadata | Supported | Parameter names, units, equations, bit sense | `PARM.`, `UNIT.`, `EQNS.`, and `BITS.` message packets |
| Query | Supported | Query bytes | Query body preserved |
| Capability | Supported | Body bytes | Capability fields not split |
| NMEA | Supported | Sentence bytes, talker ID, sentence ID, data fields, checksum details | Invalid checksums are reported; policy can reject checksum mismatches |
| Mic-E | Supported | Status bits, message code, latitude digits, coordinates, speed/course | Short bodies are malformed; values decode only when destination/body bytes permit it |
| Maidenhead | Supported | Locator bytes | Six-character locator syntax is validated |
| User-defined | Supported | User ID, packet type, body | Body preserved |
| Third-party traffic | Supported | Encapsulated bytes, explicit nested parser | Nested envelope must pass codec validation |
| Unknown identifier | Supported | Explicit unsupported variant | Not guessed |
| Malformed semantic payload | Supported | Explicit malformed variant | Policy rejects by default |

## Future Conformance Work

- Additional semantic consistency policies beyond malformed semantic rejection
  and NMEA checksum mismatch.
- More real-world corpus fixtures for Mic-E, NMEA, object/item positions, and
  third-party nested traffic.
- More policy-rejection fixtures as semantic policies are added.

## Known Unsupported Edge Cases

- Compressed-position weather extraction is not exposed yet.
- Mic-E altitude, ambiguity, and telemetry extension decoding are not exposed
  yet.
- Capability body fields remain byte-preserving and are not split into typed
  key/value data.
- Third-party traffic validates the nested codec envelope but does not apply a
  separate nested policy decision unless callers explicitly parse and evaluate
  the nested packet.

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
classification, malformed semantic visibility, malformed semantic identifier
preservation, and strict-policy rejection for malformed semantic payloads.
`aprs101_malformed_semantics.aprs` stores codec-valid but semantically invalid
golden packets for position, timestamped position, compressed position,
message, object, item, weather, telemetry, Mic-E, Maidenhead, user-defined, and
third-party payloads.

## Compatibility Coverage

The conformance corpus verifies packet-family behavior. The separate
`api_compat` test verifies documented integration APIs, including parser entry
points, parse options, stable error codes, engine/policy flow, line transport,
and typed helper methods.
