# Transactional persistence architecture

Status: Proposed, ADR-0008; design only. Runtime remains MemoryOnly.

## Authority and boundaries

The durable database owns committed resource heads, portable identity allocations,
revision records, published definitions/lineage and terminal operation receipts.
The service owns authorization, staging, binding and execution. A provider does
not parse formulas or decide graph semantics. The management service consumes a
portable prepared candidate and owns the transaction coordinator; the current
in-memory API must not simply call a database after its existing memory commit.

Memory holds a projection tagged by StoreId, RecoveryEpochId and CommitSequence.
Install a complete candidate projection atomically after database commit, including
the one publication registry. No successful response or new state-dependent
request observes a partially installed generation. Derived dependency indexes and
calculation caches are rebuildable and never outrank stored definitions.

One commit gate serializes state transitions and ownership-sensitive maintenance.
It is distinct from the brief memory mutex. Stage/calculate outside database write
transactions. A bounded storage worker owns provider I/O; never await disk/network
I/O while holding the management memory mutex. Existing captured calculations may
finish on their immutable generation; suspend delivery while store outcome is
uncertain. An unrecoverable ownership loss terminates that serving session.

## Identity and restart

| Identity | Proposed lifetime |
| --- | --- |
| StoreId | Persistent store identity; a logical fork/export import gets a new one |
| RecoveryEpochId | Stable across ordinary restart; changes on rollback restore or explicit replay-contract reset |
| ServiceInstanceId | Fresh on every running process, including reopening the same store |
| WorkingModelId, node/port UUIDs | Retained, including allocation tombstones; never derived from names |
| LogicalRevisionId | New persistent UUID per committed working revision; immutable identity |
| Working sequence | Checked u64 per model, starts at zero; never used alone as a durable comparison token |
| PublishedModelId/revision ID | Preserve lineage, immutable references and external port IDs |
| Published snapshot/capture provenance | Persist its IDs and original logical working revision |
| Attachments, cursors, ordinary evaluation captures | Ephemeral; never restored |
| Invocation IDs | Diagnostic; process namespace remains fresh, never a replay key |

A durable expected head includes store/epoch/model/revision UUID/sequence. Ordinary
restart preserves it. Runtime handles are newly allocated and connected through
portable identity maps. Existing v1.1 reads of a restored model would project its
sequence with the **new** running instance; old v1.1 requests still fail StaleInstance.
Do not rewrite archived receipt bytes to pretend their original instance survived.

An offline restore may roll back sequence/history. Rotate RecoveryEpochId before
serving, retaining restored resource/publication identities and historical revision
records. Old epoch requests are rejected even if their expected head equals the
restored head. Data omitted by the backup is not silently described as never committed.

## Replay domains and required protocol follow-up

Current instance-scoped and future durable operations are distinct dialects:

- Legacy v1.1: key includes origin ServiceInstanceId, trusted principal and operation
  ID. Persisted records may support audit but do not grant cross-restart wire replay.
- Durable: key includes StoreId, RecoveryEpochId, PrincipalAuthorityId,
  PrincipalId and operation ID. Operation kind and target are part of compared
  intent, so reusing an ID against a different model conflicts.

Intent stores exact decoded field presence/order/source/decimal representation and
durable expected head; equivalent protobuf encodings compare equally. Running
connection/request IDs are routing context, not durable intent. Explicit dialect
separation prevents stripping the instance field from legacy intent to broaden replay.

Before durable external authoring, separately review a negotiated capability with
store/epoch identity, durable expected revisions, receipt lookup namespace and
durability/failure reporting. No tags or enum values are allocated here. Existing
Durability only exposes MemoryOnly and reserves value 2; **do not reuse 2**.
Until that gate passes, public durable mode must be refused rather than mislabeled
MemoryOnly or silently advertised as durable. Internal provider tests may precede it.
Legacy clients can later invoke restored published references without durable
authoring semantics, subject to fresh authorization and normal v1.0 negotiation.

Stable principal identity includes a configured authority namespace, not a display
username. SID and UID claims remain OS-derived; a UID from another machine is not
automatically the same principal. Moving/restoring a store requires an explicit
authority mapping or preserves receipts inaccessible to unmapped users. Mapping
must never collapse different principals into one deduplication namespace.

## Commit algorithm

1. Validate shape/budgets and authorize request and recovery access. Check retained
   receipt before expected-head/quota checks. Same intent returns its original
   outcome; changed intent conflicts. Unauthorized callers learn no receipt content.
2. Capture a head and stage the candidate using existing semantics. Produce portable
   storage records and the original intent; reserve response/projection capacity.
3. Under the commit gate recheck head, admission, authorization and owner fence.
   Provider transaction repeats receipt lookup and expected-head comparison under
   serialization. This check is authoritative even with one service owner.
4. Atomically commit revision records, head/directory/identity changes, global
   CommitSequence and terminal receipt. A publication additionally commits the
   complete snapshot, public interface, lineage and immutable reference.
5. Install that committed generation without further fallible external callbacks.
   Final delivery authorization still applies. If installation fails or commit is
   ambiguous, enter RecoveryRequired and withhold success/new work.

After admission, deterministic edit/publication rejections can commit a terminal
rejection receipt without advancing the model head. Shape, authorization and
capacity failures before admission retain no receipt, matching current boundaries.
No persistent InProgress receipt is required: the database transaction either
installs a terminal record with its state or installs neither. A provider cannot
return a committed rejection until that receipt itself commits durably.

No authorization system is magically transactional with a database. In-process
policy updates must share the commit gate/generation fence. External policy
revocation can occur after a check; the guaranteed boundary remains final commit
check plus delivery recheck, not retrospective rollback. Provider code never invokes
arbitrary policy callbacks while holding database locks.

## Failures, retention and maintenance

Use states Opening, Recovering, Ready, RecoveryRequired, OwnershipLost and Failed.
Only Ready admits ordinary work. The [failure matrix](persistence-recovery-matrix.md)
defines transitions. An uncertain commit is resolved from authoritative receipt and
head state under exclusive ownership, not by regenerating an operation ID.
An absent receipt after proven rollback permits an explicit same-intent retry;
absence during inaccessible storage remains Unknown. Never infer failure from timeout.

Initial policy retains receipts, revision records, tombstones and published
snapshots for the store lifetime. Apply configured byte/count quotas before new
admission, reserving space for the entire terminal outcome. Full storage refuses
new mutations; existing receipt reconciliation remains available when readable.
No disk-pressure eviction. Backups remain necessary; quotas are not a disk-full guarantee.

Acknowledgment is future metadata, not permission to reuse operation IDs. A future
compactor may remove bulky detail only while retaining an intent/outcome fence that
returns the promised original outcome. Otherwise require an explicit epoch reset
and advertise the revoked recovery scope. Do not implement acknowledgment/deletion
commands now. Future resource deletion is a transaction with a tombstone, receipt
and reachability checks; never cascade-delete referenced publication or receipt data.

Physical backup includes one consistent store generation and its receipts, codec
versions and authority metadata. Restore is offline, validated in a staging target,
and rotates the recovery epoch before accepting work. Original-epoch receipts are
archived, not replayable into the new epoch. Logical export excludes active sessions
and secrets; import is an explicit fork with new store/resource identities and
remapped bound references, never source rebinding. It cannot import replay authority.
An export must include its complete referenced identity/tombstone closure or mark
explicit unsupported external references and refuse import; it never resolves
missing UUIDs by matching names in the destination. Backup/export are privileged
maintenance operations, not ordinary client draft downloads.
Neither backup nor export pretends to preserve external authentication credentials.

Proposed API/schema, SQLite lifecycle and conformance are detailed in the linked
[ADR](../adr/0008-transactional-persistence-and-recovery.md). No provider is implemented.
