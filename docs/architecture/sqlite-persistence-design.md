# SQLite implementation design

Status: Proposed under ADR-0008. No driver, database, migration or service flag is
implemented. Settings here define the intended profile, not current behavior.

## Deployment and ownership

SQLite is the first provider and default when local durable storage is explicitly
configured. Accept an absolute filesystem path owned by the service user; do not
silently switch an existing MemoryOnly session to durable storage. Deployment mode
and provider choice remain independent. A network-facing service could own a local
SQLite store; clients never mount or open the database.

Use a synchronous Rust SQLite driver on one dedicated bounded storage worker;
`rusqlite` with a pinned bundled SQLite build is the proposed implementation choice.
Select and audit versions during implementation, including known WAL fixes and
target VFS behavior. Do not depend on an arbitrary system SQLite version.

Acquire **database-level lifetime exclusive ownership** through EXCLUSIVE locking
mode on that single connection, before first WAL access. Hold the connection and
lock until shutdown. A sidecar PID file may aid diagnostics but is not the ownership
authority. No TTL takeover: a paused owner still owns the store until its OS/SQLite
locks release. After acquisition, atomically increment a persisted owner generation;
all commits check it. This catches stale handles in addition to physical exclusion.

SQLite documents exclusive WAL access, including rejection of other connections
while an exclusive owner is active. Its WAL mode requires same-host filesystem
support. This design therefore excludes network filesystems and additional database
connections, even for monitoring. See [SQLite WAL locking](https://www.sqlite.org/wal.html#use_of_wal_without_shared_memory).
Test path aliases/hard links and process crashes; do not rely only on canonicalized
path strings for exclusion. Replacing, copying over or unlinking an owned database
is unsupported administrative interference. Reject untrusted/symlinked paths and
unsafe parent permissions rather than weaken ownership checks.

## Startup sequence

1. Validate path/parent, engine version/features and configured budgets. Create a
   new store only under explicit create policy; missing existing paths do not
   silently create an empty replacement. Credentials/path details stay server-side.
2. Set exclusive locking mode before database/WAL reads. Acquire a bounded exclusive
   transaction to establish ownership and inspect schema identity/version; retain
   the connection's exclusive ownership after ending this inspection transaction.
   Another owner or unverifiable filesystem/lock support fails startup.
3. For an existing database, reject unsupported schema/codecs before any schema or
   journal conversion. Inspect only under ownership. For a new/compatible store,
   configure/verify the profile below outside a transaction where necessary.
4. Initialize a new header or perform an explicitly permitted offline migration.
   Create/verify backup before upgrading an existing schema. Commit owner generation
   only with a consistent header; unfinished migration cannot become Ready.
5. Verify database integrity, foreign keys and logical codec/graph invariants.
   Load a single consistent projection under recovery budgets; allocate fresh
   runtime handles without binding source names again.
6. Install heads and the complete published registry, allocate a fresh running
   ServiceInstanceId, verify the future durability-negotiation gate, then listen.

No fallback to an empty graph, MemoryOnly, a different journal mode or a different
store path on failure. Startup recovery does not replay client mutations.

## Proposed durable profile

| Setting | Required policy |
| --- | --- |
| Journal | WAL; verify returned effective mode |
| Synchronization | FULL for commits; reject NORMAL/OFF for this durable profile |
| Locking | EXCLUSIVE, acquired and retained by one connection |
| Foreign keys | Enabled and verified on the owned connection |
| Busy wait | Bounded configurable startup/transaction timeout; fail visibly on exhaustion |
| Checkpoints | Controlled on the storage worker; bound WAL growth and maintenance duration |
| macOS flush | Request/verify fullfsync and checkpoint_fullfsync where supported; test the selected VFS |
| Query safety | Bound parameters and record sizes; defensive configuration; no extension loading or arbitrary client SQL |

In WAL mode, FULL adds synchronization at transaction commit; NORMAL does not
provide the same power-loss boundary. Pragmas must be read back because unknown
ones may be ignored. See [SQLite synchronization policy](https://www.sqlite.org/pragma.html#pragma_synchronous).
The public durability claim is conditional on a tested local filesystem/VFS and
storage stack honoring synchronization. It is not immunity to device failure,
operator file deletion or corruption. Do not claim at-rest encryption from this profile.

Pin a supported engine incorporating the documented WAL-reset correction (3.51.3
or an appropriate fixed later release/backport), even though this design avoids
multiple live connections. Check the actual bundled runtime and release advisories
before implementation; this is not a recommendation to pin an old minimum forever.
See [SQLite WAL-reset notice](https://www.sqlite.org/wal.html#walreset).

## Transactions and failures

Use short explicit write transactions, beginning with writer admission before
receipt/head checks. BEGIN IMMEDIATE is the proposed transaction policy within
the already exclusive owner. SQLite documents writer admission and transaction
error behavior in [transactions](https://www.sqlite.org/lang_transaction.html).
Stage expressions/snapshots and validate budgets beforehand. Within the transaction
repeat receipt lookup, expected-head comparison and complete state/receipt writes.
Bind all data; do not interpolate formula text, paths or user labels into SQL.

No calculation, socket write, policy callback or waiting for a client acknowledgment
occurs inside a database write transaction. Storage worker queues and transactions
have separate budgets. Long backup/migration operations require an explicit
maintenance state rather than quietly extending normal mutation latency.

Classify busy/constraint/rollback errors only after inspecting transaction state.
Disk full/I/O errors or commit interruptions may require reopening and probing.
Do not assume every failed COMMIT means rollback. On ambiguity, freeze serving,
preserve the operation key/intent and resolve the receipt/head after authoritative
recovery. If a receipt exists, install its committed generation; if proven absent,
do not fabricate a rejection or a replacement operation ID.

Post-commit checkpoint failure is a maintenance/failure-state event, not proof that
an already durable transaction rolled back. Do not delete WAL files to recover.
If continued readiness cannot be established, close admission and recover offline.

## Migration, backup and restore

Initial migrations must fit one bounded atomic schema/data transaction with a
version/checksum/completion ledger. Run under exclusive ownership with verified
backup and post-migration checks. A transactional failure leaves the prior schema;
unknown/inconsistent ledger state blocks startup. No automatic downgrade or
best-effort repair. Large shadow-copy migrations require a separate reviewed design.

Back up through the owned connection using SQLite's backup API into a separate
destination connection while mutation admission is paused. Existing immutable
calculations may continue; no memory-only dump substitutes for database backup.
Finalize and validate the target before marking the backup manifest complete.
The API supplies a consistent database copy; see [SQLite online backup](https://www.sqlite.org/backup.html).
A raw copy of the main live database alone is not the backup procedure.

Initial restore is offline to a **new, explicitly selected empty destination**,
not an in-place overwrite. Reject an existing target or target sidecars. Validate
the backup and restored schema/records, rotate its epoch transactionally, and close/
checkpoint the restored target into a verified self-contained database before
activating its configuration. Synchronize the completed target and directory under
the supported platform profile. Keep the old database and all its WAL/journal
sidecars untouched as one offline recovery set; never pair a restored main file
with old sidecars. Interrupted activation leaves no completed target manifest and
must not start an empty service. In-place replacement requires a separate tested
multi-file recovery procedure and is not initially supported.
Preserve restored IDs but rotate RecoveryEpochId before serving, so a
rolled-back backup cannot replay absent post-backup mutations as new work.
Imported authority mappings require explicit administrative validation.

Windows NTFS, Linux local filesystems and macOS local APFS must each pass lifecycle,
locking, crash, flush/backup and replacement tests. SMB/NFS-mounted stores are not
supported by this initial profile. PostgreSQL/MySQL do not inherit SQLite settings;
each planned provider must prove equivalent transaction/durability/ownership semantics.
