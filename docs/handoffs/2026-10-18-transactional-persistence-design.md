# ADR-0008 persistence design — review handoff

Date: October 18, 2026. Design-only follow-up to the accepted minimal REPL.
ADR-0008 is **Proposed**, not Accepted. Runtime remains MemoryOnly.

## Review packet

1. [Proposed ADR-0008](../adr/0008-transactional-persistence-and-recovery.md)
2. [Transactional architecture](../architecture/transactional-persistence.md)
3. [Provider-independent Rust API proposal](../architecture/storage-api-proposal.md)
4. [Versioned logical schema](../architecture/storage-schema-proposal.md)
5. [SQLite implementation design](../architecture/sqlite-persistence-design.md)
6. [Failure/recovery matrix](../architecture/persistence-recovery-matrix.md)
7. [Shared provider conformance plan](../architecture/storage-provider-conformance.md)
8. This handoff.

## Main decisions proposed

The database becomes authoritative; memory is a tagged replaceable projection.
State changes and terminal receipts commit in one provider transaction, including
the complete immutable publication package and lineage. Memory installation follows
commit. Failure/ambiguity freezes admission and recovers database truth instead
of reporting rollback or applying an operation again. Calculations remain outside
write transactions and database I/O stays outside the management memory mutex.

SQLite is first/default for explicitly configured local durability with a filesystem
path. PostgreSQL and MySQL remain planned compiled adapters, independently of
deployment mode. One active owner per store is required. SQLite proposes one owned
connection with lifetime exclusive locking, WAL/FULL and tested local filesystem
assumptions. Server adapters propose dedicated session locks plus persisted owner
fencing; the abstraction is unproven until a second provider passes shared tests.

Storage records separate original formula source from portable bound postfix
tokens and persisted parse/bind failure states. Rename/delete/name reuse cannot
rebind during restoration. Tombstones, exact decimal limbs/sign/scale, defaults,
interfaces, immutable published definitions and original typed intent are retained.
Storage-specific deterministic CBOR is proposed for versioned record payloads;
neither core Rust layouts nor transport messages become the database ABI.

Ordinary restart preserves store/resource/publication identities and durable logical
revision UUIDs, but allocates a fresh running ServiceInstanceId. Backup rollback
restore rotates a recovery epoch; it cannot turn absent post-backup receipts into
proof of noncommit. Logical fork import remaps identity closure and never imports
replay authority. Backup, migration and restore are privileged owned-store maintenance.

Initial receipt/revision/tombstone retention has no silent eviction. Explicit quotas
refuse new admission; acknowledgments do not authorize operation-ID reuse. Future
compaction must retain full promised outcomes/deduplication fences or explicitly
revoke an epoch. Deletion/acknowledgment/compaction commands are not added now.

## Important review gate: durable protocol scope

Existing v1.1 recovery is instance-scoped and its Durability enum exposes only
MemoryOnly (value 2 is reserved). Persisting receipts alone cannot truthfully extend
that wire contract across restart. The proposal therefore separates legacy and
durable replay dialects and requires a **separately reviewed negotiated extension**
before external durable mode can be enabled. Conceptual requirements are store/epoch
identity, durable expected heads, receipt namespaces and durability/failure reporting.
No tags or enum values are allocated, and reserved values must not be reused.

This gate should be reviewed with the storage design, not discovered after SQLite
implementation. A durable client also needs stable principal authority and eventual
durable pending-operation bookkeeping; the current session-local REPL does not
gain that feature from this proposal.

## Failure analysis and verification

The matrix covers crashes before/after commit, installation failure, lost responses
across restart, identical/changed intent, stale revisions, publication/receipt
failure, migration interruption, corruption/newer schemas, owner conflict/loss,
disk full/I/O and ambiguous commits, plus stable binding restoration after rename
and deletion/recreation. The conformance plan defines common semantic oracles,
provider-specific failure injection, SQLite platform tests, backup/restore and a
second-provider proof gate. None of those provider tests has been implemented or run.

Official SQLite documentation was consulted for WAL/exclusive locking, synchronization,
transactions and backup; official PostgreSQL/MySQL documentation for session locks;
RFC 8949 for deterministic CBOR. Supporting documents link the primary sources.
Engine versions and driver selection must be pinned/verified during implementation,
not inferred from a design-time minimum or from installed libraries.

Validation in this slice is documentation review: UTF-8/local links, ADR numbering,
failure/identity consistency, Markdown fences and `git diff --check`. No code changed,
so runtime CI was not rerun. The last implementation evidence remains 135 passing
Rust tests and the Windows/WSL checks in the minimal REPL handoff; it does not prove
persistence. macOS Stage B/REPL verification is still outstanding.

## Documentation cleanup and stopping point

Updated stale in-process-only/deferred statements in management/service, editing,
calculation/publication, snapshots and protocol foundation documents. ADR-0006/0007
retain their historical decisions, with explicit current implementation notes.
The ADR index, STATUS and provider-direction page now link the proposed design.

No database dependency, provider implementation, migration, database file, protocol
field, REPL feature, workbook/GUI change or Native AOT work is added. Submit this
packet for architecture review before implementation; no subsequent milestone has begun.
