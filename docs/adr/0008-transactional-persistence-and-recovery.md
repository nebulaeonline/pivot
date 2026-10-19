# ADR-0008: Transactional persistence and recovery

Status: Proposed
Date: 2026-10-18

## Context

ADR-0006/0007 and the minimal REPL now support cross-process authoring and
immutable publication. Everything remains MemoryOnly. A durable implementation
must preserve bindings and transactional receipts, not merely reload source text.
The owner selected configurable providers: SQLite first/default locally with a
filesystem path, PostgreSQL/MySQL planned, independently of deployment mode.

## Decision

Propose the database as authoritative for durable graphs, identities, published
snapshots and operation receipts. Memory is a versioned, replaceable projection.
Use one bounded commit coordinator and one active service owner per store.
Providers expose atomic domain commits, never independent save-state/save-receipt
methods. Calculations run against immutable captures outside write transactions.

Persist versioned portable bound expressions separately from original source,
including invalid binding states, exact decimals, interface defaults, tombstones
and immutable published definitions. Restoration never rebinds names. Runtime
Rust handles, dependency indexes and compiled evaluator layouts are not storage ABI.

Allocate persistent StoreId, RecoveryEpochId and logical revision UUIDs. Preserve
resource/node/port/publication IDs across ordinary restart; allocate a fresh
ServiceInstanceId per process. New durable replay uses a stable principal authority,
epoch and operation ID. Keep current v1.1 replay instance-scoped: do not reinterpret
old fields, reuse reserved enum values, or claim cross-restart recovery from them.
A separately reviewed negotiated protocol extension is required before exposing
durable guarantees externally. This slice allocates no fields and changes no code.

Commit candidate state plus its original typed intent and terminal receipt in one
transaction, using expected-head comparison and ownership fencing. Publication
commits its complete immutable snapshot, lineage/ref and receipt together.
Respond success only after durable commit and coherent memory installation.
If commit succeeds but installation fails, stop admitting work and recover from
the database; never report rollback or apply the mutation again.

Initially retain terminal receipts, identity tombstones and published revisions
without silent eviction, subject to explicit store quotas. Acknowledgments do not
authorize deletion. Future compaction must retain deduplication fences or rotate
an explicitly revoked recovery epoch. Store deletion and restore are administrative
boundaries, not ordinary edits or implicit retries.

SQLite uses one owned connection, lifetime exclusive database locking, WAL and
FULL synchronization on supported local filesystems. No network filesystem,
external live database writer or live raw-file copy is supported. Provider-specific
SQL, ownership mechanisms and migrations remain behind the common contract.
PostgreSQL is the proposed second conformance target, then MySQL; neither is proven.

## Alternatives considered

- Source-only persistence: loses stable bindings after rename/name reuse.
- Serialize core structs or generated wire blobs as the whole database contract:
  couples recovery to runtime layouts and transport versions.
- Memory authoritative with asynchronous database flush: cannot promise durable
  state/receipt atomicity on success.
- Reuse running ServiceInstanceId indefinitely: confuses dead sessions and restart
  identities, weakening stale-instance checks.
- Treat driver errors as rollback: unsafe when commit acknowledgment is lost.
- Automatically evict old receipts or transparently fail over to memory: weakens
  advertised replay/durability guarantees.
- Multi-owner stores now: requires registry/cache/policy coordination beyond this scope.

## Consequences

Durable commits require a coordinator and explicit recovery states. Initial full
revision records favor correctness over space efficiency; quotas and later measured
compaction are necessary. The provider abstraction remains provisional until a
second implementation passes shared tests. Existing MemoryOnly operation is
unchanged until design acceptance and a separately authorized implementation.

## Verification / reconsideration trigger

Acceptance requires review of the linked API, identity/protocol gate, portable
expression codec, failure matrix and provider conformance plan. Implementation
must pass crash and ambiguous-commit tests on supported platforms before any
durable capability is advertised. Reconsider for multi-instance serving, receipt
expiry, new numeric semantics, large revision histories or filesystem changes.

Supporting designs: [architecture](../architecture/transactional-persistence.md),
[API](../architecture/storage-api-proposal.md),
[logical schema](../architecture/storage-schema-proposal.md),
[SQLite](../architecture/sqlite-persistence-design.md),
[failure matrix](../architecture/persistence-recovery-matrix.md),
[conformance](../architecture/storage-provider-conformance.md).
