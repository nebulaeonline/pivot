# Versioned logical storage schema proposal

Status: Accepted ADR-0008 design. This is a logical database/record contract, not
DDL, migrations or an implemented `.proto` schema. Providers choose physical SQL
types/indexes while preserving these keys and invariants.

## Encoding and versions

Use storage-specific typed records with explicit schema, record, expression,
intent and numeric-semantics versions. Propose deterministic CBOR encoding for
record payloads, using only integers, byte strings, UTF-8 text, booleans, arrays
and fixed-key maps initially. Arrays retain order, optional fields retain presence,
duplicate/unknown record fields fail closed, and bounded decoders reject excess
depth/count/bytes. A storage digest is not a substitute for typed comparison.
The deterministic encoding rules are defined in [RFC 8949 section 4.2](https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2).
No generic Rust serde layout or generated transport struct is the stored ABI.

The first [codec/import implementation](portable-storage-foundation.md) freezes a
restricted deterministic-CBOR v1 array layout and an independent compatibility
corpus. Only portable graph/publication records are implemented; logical database
tables, receipt encoding and migrations below remain future work. Full immutable
working-revision copies are the initial strategy, not a permanent physical layout.
Future structural sharing must preserve this logical contract and retained outcomes.

UUIDs are 16 bytes in canonical network order, never platform GUID memory layouts.
u64 values use fixed eight-byte big-endian logical SQL keys, with checked arithmetic,
so SQLite signed integer limits do not truncate public sequences. Payload integers
use the record codec. Strings preserve exact UTF-8 bytes with binary equality;
display names are not identity keys or globally unique model names.

Decimal96 v1 is 16 bytes: little-endian lo/mid/hi u32 limbs followed by .NET-style
flags (sign bit 31, scale bits 16–23, all other bits zero; scale 0–28). Preserve
coefficient/sign/scale, including signed zero and trailing scale. No REAL or SQL
DECIMAL coercion. Authoritative portable values retain their representation even
if the execution backend normalizes a runtime value. Receipts independently retain
the exact submitted representation. Future IEEE values need a new explicit type
codec; unknown value kinds are unsupported, never coerced to decimal.

## Logical tables

All subordinate keys include StoreId. Domain writes also verify the active epoch
and owner generation in StoreHeader. Foreign keys and application checks enforce
same-store/model ownership; deliberately dangling draft references use the identity
ledger rather than a foreign key requiring a live node.

| Relation | Logical key and content |
| --- | --- |
| StoreHeader | StoreId; active RecoveryEpochId, schema/codec compatibility, CommitSequence, owner generation, authority namespace, verified profile |
| MigrationLedger | Migration ID/source/target/checksum/state; backup generation and completion metadata |
| WorkingResource | WorkingModelId; display metadata, current LogicalRevisionId/sequence, publication lineage ID, live/deleted state |
| WorkingRevision | ModelId + LogicalRevisionId; checked sequence, predecessor, originating epoch/commit, immutable full graph/interface payload and digest |
| IdentityLedger | ModelId + kind + portable node/port ID; first allocation revision, live/retired metadata; never reuse |
| RevisionNode | RevisionId + node UUID; name, value or computed definition record |
| RevisionPort | RevisionId + port UUID; name, direction, target UUID and required/optional typed default |
| PublishedRevision | PublishedModelId + PublishedRevisionId; captured name, logical working origin, snapshot UUID, immutable validated graph/interface payload and numeric version |
| PublishedPort | PublishedRevision key + external port ID; portable authoring port/node identity and captured interface requirement |
| OperationReceipt | Dialect + replay domain + principal authority/principal + operation ID; full intent, terminal outcome, referenced identities/revision, commit sequence |
| RecoveryEpoch | Epoch UUID; active/revoked/archive status, creation/restore reason; no silent epoch replacement |
| MaintenanceManifest | Consistent backup/export generation, schema/codec/durability profile and provenance; no provider credentials |

The header has a singleton active owner generation and epoch. Published keys and
logical revision UUIDs cannot be replaced in place. Operation IDs are unique across
operation kinds/targets within their explicit replay namespace. A receipt refers
to committed revision/publication records, not merely the latest head. Rejection
receipts contain no new head. There is no persistent pending-receipt state.

Initial storage retains complete immutable working revisions, including invalid
drafts. This deliberately avoids event-log replay and delta-chain recovery.
Physical providers may place a revision's nodes/ports in one bounded payload or
normalized child rows within its transaction; semantic equality, completeness and
backup behavior are identical. Indexes are derived and may be rebuilt. Capacity
limits apply to retained history, not only live graph counts.

## Portable computed definitions

Store exact source bytes separately from binding state. A definition has exactly one:

| State | Persisted authoritative content |
| --- | --- |
| BoundV1 | Versioned postfix tokens and numeric semantics; node tokens carry portable UUIDs, never names/core NodeIds |
| ParseFailedV1 | Source plus stable diagnostic code/byte offset and failure state; no automatic parse-on-load |
| BindFailedV1 | Source plus unresolved-name/failure data from submission; do not bind newly available names on load |

BoundV1 token kinds are typed decimal literal, node UUID, unary operator and binary
operator, with an explicitly versioned operator vocabulary and arity. Decode checks
stack depth, final single result, literal validity, resource bounds and node/model
identity membership. Rebuild dependencies from node tokens; any stored dependency
index must match. A retired/missing target remains a missing-reference draft error;
it never resolves to a replacement sharing its old name.

For example, `Revenue - Costs` remains bound to node A after A is renamed Sales.
Deleting A and creating a new Revenue node B leaves the old token referencing A.
An unresolved Revenue at submission remains unresolved even if a Revenue node
exists at restart. Only an explicit formula edit requests a new parse/bind attempt.

Restore uses a future internal validated graph-import boundary: allocate fresh
runtime handles, install portable mappings, then install recorded binding states.
Existing add-computed/source APIs are insufficient because they rebind. This
proposal does not implement that import boundary. Recompute structural diagnostics
from recorded semantics; retain versioned submission diagnostics where required.
Unknown codecs/operators or malformed stored tokens are storage incompatibility/
corruption, not an ordinary editable parse error to silently repair.

Published snapshots must decode as globally valid, with complete public-port maps,
exact defaults, external IDs such as existing `port-<uuid>` strings and captured
provenance. They do not point at mutable working definitions. Historical receipt
summaries are stored, not reconstructed from a newer draft. Detailed historical
diagnostic paging remains outside the contract.

## Migration, deletion and restoration

Maintain schema minimum-reader/minimum-writer and required codec versions, not just
one integer. Newer unsupported schema/required codec fails startup before writes;
no automatic downgrade, unknown-field stripping or schema guessing. Initial
migrations are offline with exclusive ownership, a verified consistent backup,
transactional change+ledger completion and post-migration integrity checks.
An interrupted migration either rolls back atomically or leaves the store blocked
for explicit recovery; never resume a partial schema as Ready.

Future migrations too large for one transaction require an independently reviewed
shadow-copy/swap protocol with a durable completion marker. This is not permitted
implicitly, especially on providers with nontransactional DDL. Every provider must
prove migration atomicity or implement such a protocol before supporting upgrades.

No resource deletion/acknowledgment/compaction API is implemented initially.
Tombstones and receipt roots prevent identity reuse. Future collection must trace
publication, receipt and live-head roots before deleting unreachable bodies and
must retain the promised original outcomes. Offline restore preserves data IDs,
rotates the active replay epoch and archives old receipts. Logical fork import
allocates/remaps identities and never imports replay authority.
