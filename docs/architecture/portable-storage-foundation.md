# Portable storage records and restoration foundation

Implemented bounded subset of accepted ADR-0008 in `pivot-storage`. It provides
owned portable graph/publication records, deterministic encoding, validated import
and isolated in-memory use after restoration. It does **not** implement a database,
durable management coordinator, Ready lifecycle or cross-restart client recovery.
The daemon and REPL remain MemoryOnly and have no dependency on this crate.

## Boundaries and identities

`Record` owns codec/numeric/expression versions; store, originating epoch, working
model and logical revision UUIDs; sequence/name; complete allocated node/port
identity ledgers; live nodes/ports; and optional publication metadata. Publication
metadata owns lineage/revision/snapshot UUIDs and exact external-port mappings.
Retired identities are ledger entries absent from the live definitions, not reused
slots or names. Runtime core IDs never occur in codec records.

`capture` and `capture_snapshot` are Rust-internal bridges taking caller-owned
portable-to-core maps. They validate complete allocated ledgers and export actual
pre-bound state, including parse/bind failures. No UUID is inferred from a label.
Capturing an immutable snapshot ignores subsequent edits to its source working model.
Exporting a bare runtime decimal cannot reconstruct representation already normalized
before export; exact incoming representation must originate in a portable record.
The future coordinator must retain that portable authority alongside its projection.

The core's doc-hidden `model::transfer` module is an internal cross-crate bridge,
not a storage ABI or client API. Its owned Image uses temporary slot indices.
The installer validates limits, names, duplicates, reference ranges, token stack
arity and failure codes before installing private model fields. It never calls
the formula parser/binder. Fresh runtime model/node/port handles are allocated on
every restore, including handles for retired identities. Invalid draft semantics
(missing targets, parse/bind failures, cycles, computed public-input targets) survive.

`RestoredModel` keeps the graph and all runtime maps private. Its portable edit
helpers delegate to existing core operations, stage changes atomically and retain
untouched exact record representations. They are an import/editability test surface,
not a durable coordinator or additional REPL semantics. Published records reject
editing. Evaluation/public invocation delegates to the existing engine and preserves
public required/default behavior and external-port mappings.

`restore_all` returns the entire bounded collection or an error, never a partial
result. It verifies one store, identity ownership/kinds, publication/snapshot key
uniqueness, lineage ownership and stable external-port mappings. Records may carry
different originating epochs: historical snapshots are not rewritten when a store's
active replay epoch changes. The active store epoch/commit generation and complete
catalog membership are future provider/coordinator responsibilities.

No live registry installation occurs here. In particular, future cold registry
rebuild must compare **portable** public-port identities across independently restored
snapshots, not equate their newly allocated core PortIds. This foundation verifies
portable consistency; integration with ModelHost is a later coordinator task.

## Bounded deterministic codec

Codec 1 is a deliberately restricted deterministic-CBOR profile. Only unsigned
integers, definite byte/text strings and definite arrays are accepted. Integers
and lengths use the shortest encoding. Tags, maps, negative integers, floats,
simple values and indefinite lengths are rejected. Fixed array arity makes unknown
fields fail closed. UTF-8 is checked before constructing text. Trailing data and
unsupported codec/numeric/operator versions are rejected. UUIDs are canonical
16-byte network order; unsigned sequences support the entire u64 range.

Defaults are 1 MiB encoded bytes, 16 container/value levels, 100,000 CBOR items,
1,024 live nodes, 256 live ports, 8,192 allocated identities and 1,048,576 mixed work
units (identity/live counts, source bytes and tokens). Each source is at most
4,096 bytes; names/identifiers are at most 128 bytes. Validation work is also capped
by the byte budget. Count, depth, declared lengths and item/work budgets are checked
before corresponding unbounded processing or runtime construction. Core transfer
has independent hard guardrails. These are implementation budgets, not protocol fields.

Decimal96 stores lo/mid/hi plus flags in 16 little-endian bytes. Reserved flag bits
and scales outside 0–28 fail. Exact sign/coefficient/scale, including signed zero
and trailing scale, remain authoritative in the owned record. Runtime projection
uses the existing decimal backend; arithmetic behavior is unchanged. Unrelated
edits and re-encoding preserve original values/defaults/literals even if constructing
the corresponding runtime value normalized its representation.

Definition states remain distinct: input, bound tokens with original source,
parse-failed source/offset/code, and bind-failed source/unresolved identifier.
Bound node tokens contain owner-model UUID and node UUID. A reference must belong
to that model's allocated ledger; it may point at a retired node in an editable
draft. Stack validation requires one final result and valid unary/binary arity.
Dependencies, reverse dependencies, ordering and structural diagnostics are rebuilt
by the core, with no lookup of historical formula names.

Working drafts can fail structural validation while remaining restorable/editable.
Published records additionally require a globally valid core snapshot and complete
unique public-port maps; malformed or unsupported published data cannot be treated
as a valid partial catalog. Historical receipt records/codecs are not implemented.

## Frozen v1 layout and compatibility

The top-level definite array has exactly 14 positions:

| Index | Value |
| --- | --- |
| 0, 1, 2 | codec, numeric-semantics, expression versions (all 1) |
| 3, 4, 5, 6 | StoreId, originating RecoveryEpochId, WorkingModelId, LogicalRevisionId bytes |
| 7, 8 | u64 sequence, UTF-8 display name |
| 9, 10 | strictly UUID-sorted allocated node/port ledgers |
| 11, 12 | live node/port arrays, each strictly UUID-sorted |
| 13 | empty array or publication array |

Node = `[uuid, name, definition]`. Definitions are `[0, decimal16]`,
`[1, source, tokens]`, `[2, source, byte_offset, parse_code]`, or
`[3, source, unresolved_name]`. Tokens are `[0, decimal16]`,
`[1, owner_model_uuid, node_uuid]`, `[2, unary_code]`, or `[3, binary_code]`.
Unary codes 0/1 mean plus/minus; binary 0/1/2/3 mean add/subtract/multiply/divide.

Parse codes 0–5 respectively preserve expression-too-long, trailing-input,
nesting-too-deep, missing-closing-parenthesis, unrepresentable-decimal and
expected-operand failures. These numbers do not come from Rust enum discriminants.
Offsets must be within source at a UTF-8 boundary. Unknown codes/operators fail;
they are never coerced into a default operation or reparsed as repair.

Port = `[uuid, name, owner_model_uuid, target_node_uuid, requirement]`.
Requirements are `[0]` required input, `[1, decimal16]` optional input or `[2]` output.
Publication = `[lineage_uuid, revision_uuid, snapshot_uuid, public_ports]`;
public_ports is UUID-sorted `[authoring_port_uuid, external_id]` pairs, covering
every live port exactly once. Publication and snapshot IDs are unique within a
restored collection; one external ID cannot change authoring-port identity within
a publication lineage.

[Frozen fixtures](../../crates/pivot-storage/tests/fixtures/v1/README.md) define exact
valid bytes and decode-versus-restore rejection phases. Tests never regenerate
them. Maintain old-version readers/fixtures when introducing a new record version;
do not edit old bytes to make a new encoder pass. A future field or semantic change
requires an explicit version/compatibility decision before database migrations.

## Verification and remaining gates

Tests cover source-model independence, fresh handles, renames/name reuse, retired
nodes/ports, invalid interface repair, unresolved/incomplete source, all operators,
exact decimals, immutable publication/defaults, malformed/cross-model records,
truncation/noncanonical bytes, bounded arbitrary data and all-or-error collection
restoration. CI executes these through workspace Rust tests.

Next, after review: SQLite owner-lock/WAL/checkpoint/backup feasibility and provider
foundation. SQL domain commits, database-authoritative coordinator/startup completeness,
durable protocol negotiation, actual daemon restart recovery and cross-platform
durability proof remain separate slices. No driver or service flag has been added.
