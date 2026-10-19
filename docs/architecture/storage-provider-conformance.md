# Shared storage-provider conformance plan

## Current scope and priority

This is the accepted ADR-0008 design for **optional durable hosted storage**.
Further hosted implementation, including the SQLite feasibility gate, is deferred
following the portable-foundation review. It is not required for ordinary local
files and does not select their container. See proposed
[ADR-0009](../adr/0009-portable-documents-and-local-lifecycle.md). Requirements below
remain applicable if hosted work resumes; they are not authorization to proceed.

Status: Accepted ADR-0008 plan. No database tests are implemented or claimed to
pass. Portable codec/import tests now cover their bounded subset; existing
MemoryOnly CI does not prove database durability.

## Harness and oracle

Build one provider-independent test harness around OwnedStorage and the actual
management coordinator, plus adapter-specific fault controls. Run identical
logical operations/records against every provider; compare committed generations,
full typed receipts, portable identities and immutable publications. Use a
reference semantic ledger, not SQL statement counts or driver return strings.

Test both direct provider commits and real daemon restart/reconciliation once
the separately reviewed durable protocol exists. Distinguish process kill from
OS/power-loss simulation: a successful process-crash test alone does not prove
flush behavior. Persist reproducible seeds, engine/driver versions, settings,
filesystem/VFS identity and every fault cut point. Bound process waits, database
budgets and cleanup; retain failed stores as artifacts rather than deleting evidence.

## Required common cases

| Area | Required cases and assertion |
| --- | --- |
| Exact values | Decimal limits, signed zero, all scales/trailing zeros and malformed flags; every authoritative value/default/intent round-trips exactly; no SQL numeric coercion |
| Portable graph identity | New runtime handles after restart, same portable IDs, no tombstone reuse; cross-model/store references rejected |
| Binding preservation | Rename A; delete A/recreate its label as B; originally unresolved name later present; repeated reload never rebinds |
| Invalid drafts | Empty/incomplete source, parse/bind failure, dangling target, cycle, invalid public target; source and authoritative failure state survive |
| State/receipt atomicity | Kill at each write/commit/install boundary: candidate and terminal receipt appear together or neither; deterministic rejection advances no model head |
| Intent equivalence | Field-order/valid wire representation differences compare equal; changed edit order, source, presence, decimal scale/sign, target or expected head conflicts |
| Replay | Same key across restart returns original success/rejection before quota/head checks; different principal/epoch/dialect cannot impersonate it |
| Conflict isolation | Competing expected heads yield one commit; no dirty reads, lost updates, partial identities or partial public interfaces |
| Publication | Full snapshot/interface/provenance plus receipt atomic; immutable old revision returns unchanged results after working edits/restart |
| Projection failure | Force allocation/install failure after confirmed commit; no false rollback/success from stale memory; complete registry rebuild |
| Ambiguous commit | Drop commit acknowledgment before/after actual durable boundary; resolve under ownership without fresh operation ID or duplicate mutation |
| Retention | Fill count/byte quotas; replay still works; no receipt/tombstone/publication eviction; acknowledgment cannot enable ID reuse |
| Ownership | Second process, aliases, paused owner, killed owner, disconnected owner session and stale fence; no overlap or unfenced write |
| Compatibility | Old/new schema and record versions, unsupported numeric/operator codecs, malformed CBOR, u64 values above 2^63 and sequence exhaustion |
| Migrations | Crash before/during/after each step; ledger/schema coherent; transaction rollback or explicit blocked recovery, never partial Ready |
| Backup/restore | Consistent heads/publications/receipts at one generation; interrupted target rejected; rollback restore rotates epoch and rejects old operations |
| Authority | OS principal namespace preserved; administrative mapping cannot merge principals or disclose prior receipts without rights |
| Execution separation | Slow evaluation/slow client never holds DB write transaction; policy revocation after commit retains truth |
| Failure reporting | Disk full, I/O, corruption, unavailable provider and unsupported settings yield explicit state; no MemoryOnly fallback |

Include a representation corpus spanning all existing decimal parity categories
and authoring edit alternatives. Cross-version readers must preserve old record
semantics without rerunning a changed formula binder. Semantic migrations, if
needed, require explicit fixtures and cannot silently rewrite published results.

## SQLite-specific proof

Run Windows local NTFS, Linux local storage and macOS local APFS with the selected
bundled engine/VFS. Verify effective WAL/FULL/exclusive/foreign-key settings and
rejected weaker settings. Cross-process probes must fail while the owner lives
and succeed only after its connection releases. Test hard-link/path aliases and
replacement interference under documented supported-path rules.

Inject busy timeouts, full filesystem, write/sync failures, torn/corrupt records,
interrupted checkpoints and migration/backup/restore activation failures. Exercise WAL
recovery after abrupt process death. Demonstrate that copying a live main file is
not used as backup, and that the owned-connection backup is coherent. Test macOS
flush settings, Windows handle lifetime, existing-target refusal and stale-sidecar
restore rejection explicitly. Unsupported
network filesystems/configurations must fail configuration checks, not silently
weaken the promise. Record which crash assumptions are fault-injected rather than
physically power-tested.

## Second provider and release gates

Before **any database migration**, freeze and execute an independent versioned
storage-codec corpus, including exact valid bytes and malformed/noncanonical cases.
Before full SQLite schema/coordinator work, prove ownership/WAL/checkpoint/backup
feasibility with the selected driver and VFS on each supported platform. Require
independent count/byte quota tests for receipts, retained revisions, tombstones and
publications, and actionable authorized category reporting. Include failure of one
object in an otherwise valid startup catalog: Ready must remain blocked globally.

SQLite passing is necessary but does not prove the abstraction. PostgreSQL is the
proposed second target: demonstrate transaction serialization, ownership-session
loss, commit acknowledgment ambiguity, binary/collation equality and offline
migrations using the same suite. MySQL remains a subsequent planned adapter;
DDL atomicity and ownership behavior require its own proof, not borrowed claims.

Gate order: ADR acceptance; bounded portable import/codec tests; provider contract
and SQLite internal conformance; separately reviewed durable protocol/capability;
real daemon crash/restart tests; supported-platform validation; explicit durable
enablement. No partial adapter or driver enables durable advertising. Keep v1.0
invocation, current v1.1 strict preflight, numeric parity and MemoryOnly CI intact.

Performance checks are later additions: record commit/recovery/backup size and
latency to catch pathological growth, but do not set invented speed targets or
weaken atomicity/retention for a benchmark. Large-history compaction and clustered
ownership require separate designs and test gates.
