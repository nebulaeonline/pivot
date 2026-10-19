# Experimental model-file envelope v1

Implemented for ADR-0009 Document Foundation Slice B, 2026-10-19. This is a
provisional headless proof format, **not the production .pivot format**. No file
extension is registered; tests use `.experimental` as an illustrative suffix.
Future experimental changes require explicit version handling, but this slice
does not promise long-term document-format compatibility or automatic migration.

## Envelope

| Byte range | Field |
| --- | --- |
| 0–7 | ASCII `PIVXFILE` magic |
| 8–11 | Envelope version, unsigned big-endian u32; currently 1 |
| 12–19 | Payload byte length, unsigned big-endian u64 |
| 20–35 | SavedFileVersion UUID, 16 network-order bytes, non-nil |
| 36–67 | SHA-256 digest of bytes 0–35 followed by the payload |
| 68 onward | Exactly the declared payload bytes; trailing content is rejected |

The digest covers the header's version, length and saved-file identity as well as
content. It detects corruption; it does not authenticate the author or make a file
trusted. A fresh saved-file UUID identifies each export written to a file. It is
distinct from DocumentId, DocumentStateId, GraphId and GraphRevisionId. Save As
preserves logical graph/document identity while creating another saved-file version.

The file limit is independent of IPC frames: default payload cap 4 MiB, configurable
up to a 16 MiB hard ceiling, plus the 68-byte header. Graph validation separately
uses its logical budgets, normally approximately 1 MiB, 1,024 live nodes and 256
ports. A larger file cap neither raises those graph ceilings nor advertises large
workbook support. The current protocol/frame limits are unchanged.

## Bounded deterministic payload

The payload reuses the storage foundation's restricted CBOR primitives: shortest
unsigned integers, definite arrays, byte strings and UTF-8 text. No maps, tags,
floats, indefinite lengths, duplicate implicit fields or trailing values are allowed.
Bytes, item count, nesting, string lengths and graph validation work are bounded.
Declared file lengths are checked before payload decoding; filesystem reads also
have a separate cap in case a file grows while being read.

```text
document = [logical_format, document_uuid, state_uuid, graph]
graph = [payload_version, numeric_version, expression_version,
         graph_uuid, graph_revision_uuid, display_name,
         allocated_node_ids, allocated_port_ids, live_nodes, live_ports]
```

All four logical/semantic versions above are 1. UUID ledgers and live definitions
are strictly sorted and unique. Retired IDs remain in their ledgers. Node, definition,
bound token, source-error and port entries reuse the
[portable storage field semantics](portable-storage-foundation.md#frozen-v1-layout-and-compatibility),
including exact Decimal96 bytes and explicit operator/failure codes. This reuse
does not include the hosted 14-field Record or its store/epoch/publication fields.

The implementation factors graph-body writing and reading out of the existing
codec rather than duplicating graph parsing. Hosted-v1 record field order and all
11 frozen compatibility fixtures remain unchanged.

## Restore rules

Validate envelope, digest, payload shape/versions and portable graph integrity before
installing a DocumentSession. The existing core transfer creates fresh runtime
handles and restores bound tokens directly. It never resolves historical formula
names again. Inputs, literals, defaults, exact sign/scale/coefficient, original
source and invalid draft states survive. A cycle or incomplete formula is editable
content; corrupt references to never-allocated/foreign IDs are malformed input.

Unsupported versions and malformed data fail closed without exposing a partial
session. File Open produces a new private SessionId. Separately opened copies may
share logical IDs but cannot share mutable runtime state or calculation handles.
The envelope contains no registry, publication, receipt, authorization policy,
StoreId or RecoveryEpochId.

## Scope and implementation

Code: [experimental_file.rs](../../crates/pivot-storage/src/experimental_file.rs),
[shared codec](../../crates/pivot-storage/src/codec.rs),
[document session](../../crates/pivot-storage/src/document.rs).
The file path is absent from the payload; association and physical fingerprints
belong to the broker. No SQLite/ZIP container, compression, encryption, workbook,
external-link fetching, document RPC or new Protobuf operation is introduced.
Production container selection remains open for later workbook requirements.

See [broker behavior and platform limits](experimental-file-broker.md) and the
[Slice B review handoff](../handoffs/2026-10-19-document-foundation-slice-b.md).
