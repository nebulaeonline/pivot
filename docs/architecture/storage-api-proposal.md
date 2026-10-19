# Provider-independent storage API proposal

Status: Proposed under ADR-0008. Signatures below are design notation, not Rust
implementation or a promise of source-compatible public APIs.

## Ownership and execution

A configured factory opens an owned store. MemoryOnly remains a distinct existing
service configuration; it does not masquerade as a durable provider. SQLite takes
an absolute configured path; future PostgreSQL/MySQL configurations remain private
to their adapters. Providers are compiled Rust implementations, not dynamic plugins.
Reject unknown providers, unsupported features and insecure configuration explicitly.

All provider methods run on a bounded storage worker and receive cancellation and
I/O deadlines where meaningful. Cancellation before submission can prevent work;
after commit submission it cannot prove rollback. Never drop an uncertain commit
and report it as cancelled. One exclusive OwnedStore handle carries the ownership
generation; losing its session/lock makes all derived commit permits unusable.

```rust
// Illustrative domain interface; owned records, no borrowed core graph handles.
trait StorageProvider {
    fn open_owned(config: ProviderConfig, budget: OpenBudget)
        -> Result<OwnedStore, OpenFailure>;
}
trait OwnedStorage {
    fn capabilities(&self) -> VerifiedCapabilities;
    fn load_projection(&mut self, budget: RecoveryBudget)
        -> Result<CommittedProjection, RecoveryFailure>;
    fn lookup(&mut self, key: ReceiptKey, access: RecoveryPermit)
        -> Result<ReceiptLookup, ReadFailure>;
    fn commit(&mut self, prepared: PreparedCommit, permit: CommitPermit)
        -> CommitResolution;
    fn resolve(&mut self, key: ReceiptKey, intent: StoredIntent)
        -> Result<ResolvedCommit, RecoveryFailure>;
    fn backup(&mut self, destination: BackupTarget, budget: BackupBudget)
        -> Result<BackupManifest, BackupFailure>;
    fn close(self) -> Result<(), CloseFailure>;
}
```

Permits are service-internal authorization decisions tied to policy generation
and owner fence; possession of an external resource ID cannot manufacture one.
Provider transactions still enforce expected heads, domain ownership and receipt
uniqueness. No generic save-model, save-receipt, delete-row or raw SQL method is
part of this contract. Startup migration is an explicit open policy, never hidden
inside a normal commit.

## Inputs and results

| Record | Required content |
| --- | --- |
| VerifiedCapabilities | Actual durability profile, supported codec/schema versions, ownership mode, retention promises and operational limits |
| ReceiptKey | Explicit dialect; store/epoch or legacy origin instance; principal authority/principal; operation ID |
| StoredIntent | Versioned complete typed intent and exact representation; digest is only an index accelerator |
| ExpectedHead | Store/epoch/model/logical revision UUID/sequence, or explicit absent-resource expectation for creation |
| PreparedCommit | Intent/key, expected head, bounded candidate revision/identity delta or publication package, terminal outcome, byte/count accounting |
| CommitPermit | Current owner generation and authorization-policy generation checked by the coordinator |
| CommittedProjection | One consistent CommitSequence, active heads, identity ledgers, complete publication registry and retention metadata |
| CommitResolution | Committed(receipt, sequence), Replayed(original receipt), RejectedWithReceipt(receipt), NotAdmitted(reason), or Indeterminate(key) |
| ReceiptLookup | Found(original terminal record), AbsentInActiveEpoch, or RevokedEpoch; storage unavailable is an error, never Absent |

Expected-head conflict after admission is a stable rejection receipt. A changed
intent for an existing key returns operation-ID conflict without replacing the
original receipt. Replayed includes previously committed rejection outcomes.
Unauthorized and malformed operations must fail before receipt access/admission.
Provider helpers may not broaden that access to make recovery convenient.

Comparison uses the stored version's full typed intent, not only a hash, raw wire
bytes or normalized decimal values. Ordered edits, temporary IDs, partial-port
presence, defaults, source whitespace, exact decimals and expected revisions are
significant. Unknown intent codec versions fail closed. Legacy instance-scoped
and future durable-domain records are never implicitly compared as one dialect.

## Atomic commit obligations

Within one serializable domain transaction:

1. Verify store/epoch/owner generation and permit validity. Lookup the receipt key.
   Equal intent returns its outcome before head or quota checks; unequal intent conflicts.
2. Lock/compare the expected head, and check identity uniqueness, immutable published
   keys and complete store quotas. Storage validates record structure and ownership,
   not formula source semantics. The service prepares executable/binding checks.
3. For a valid candidate, insert immutable revision/snapshot data and update the
   head/directory/identity ledger. Otherwise insert only its terminal rejection.
4. Insert exact intent and terminal receipt and advance CommitSequence atomically.
   A publication inserts its complete package and lineage/ref in this same transaction.
5. Return Committed only after the configured durability boundary is confirmed.

No caller can commit the graph and then call a separate receipt method. The
provider cannot write a visible success receipt for an uninstalled database state.
The coordinator may install memory only from Committed/Replayed database records,
and must not reinstall a replayed old revision over a newer head.

Definite rollback is a provider assertion backed by engine state, not the default
interpretation of an error. Connection loss, interrupted commit acknowledgment,
or uncertain engine state returns Indeterminate. Freeze mutation admission and
projection-dependent delivery until resolve/reopen establishes the authoritative
receipt/head under exclusive ownership. A retry of a database transaction is
permitted only after proven rollback, using the same key/intent and fresh checks.
No adapter invents a replacement client operation ID.

## Projection installation and policy

Prepare bounded owned projection data before commit when possible. After commit,
install the returned generation under brief memory access and release the commit
gate. If allocation, invariant verification or installation fails, transition to
RecoveryRequired, invalidate current projections and rebuild from storage. A fresh
process can recover if rebuilding in place is unsafe. The durable receipt remains
the source of truth; success must never be relabeled as a rejected mutation.

Authorization still applies to receipt recovery and response delivery. Denial after
commit leaves the receipt untouched. The API cannot make an external identity
provider transactional; policy updates controlled by Pivot share its commit gate.
Provider errors redact credentials, database paths and SQL data from client failures.

## Provider obligations and limits

SQLite initially serializes all storage use through its one owned connection.
Planned server providers may implement serializable behavior with appropriate
isolation/row locks and conditional updates; engine isolation names alone are not
proof of the contract. A dedicated ownership session and fenced transactions must
prevent a disconnected former owner from continuing to commit. Initially prefer
one writer connection tied to that ownership session, not an unfenced connection pool.

Open, recovery, records, transactions, retained bytes, backup and queues have
explicit budgets. Exceeding a recovery budget fails startup without silently
discarding models. Read-only calculations hold no database write transaction.
Provider conformance must include attempted misuse of independent SQL sessions,
ambiguous commit and lock loss. PostgreSQL is the proposed second proof target;
MySQL remains planned. Neither is declared supported by this proposal.

For planned server adapters, use a store-scoped session advisory lock on the same
dedicated connection that performs writes: PostgreSQL session advisory locking
or MySQL GET_LOCK, with bounded acquisition and a stable store-derived lock key.
These are cooperative ownership mechanisms, not protection against arbitrary SQL
administrators. PostgreSQL describes its session lock lifecycle in
[advisory locks](https://www.postgresql.org/docs/current/explicit-locking.html#ADVISORY-LOCKS);
MySQL describes connection-scoped named locks in
[locking functions](https://dev.mysql.com/doc/refman/8.4/en/locking-functions.html#function_get-lock).
Keep all owner writes on that connection and verify the persisted generation in
each transaction; never return it to a general pool or continue writes after lock
loss. Reacquisition is a new recovery/owner-generation boundary. A lock-key collision
may conservatively reject startup, never permit two owners. Server failover/topology
changes must close admission and prove ownership again; multi-primary deployment
is outside the initial contract.
