# G3 indexed-store experiment: first bounded slice

Parent accepted this first research baseline on 2026-10-27. G3 remains incomplete;
bounded verified-open/completeness research is authorized next.
[Acceptance and next assignment](../handoffs/2026-10-27-g3-first-result-acceptance.md).

2026-10-27. Research only, behind the existing history-experiment fixture feature.
G1/G2 remain closed; no production history integration or backend choice is made.

## Experiment boundary

The Rust example `history_index_fixture` produces a fixed-identity workbook stream
using accepted G2 `PreparedRecordChanges`. Every mutation is checked against an
independent full checkpoint export before and after paired in-memory publication.
It exports resolved stable-key before/after bytes, content IDs, oracle digests,
and a logical checkpoint every 64 edits. It does not export commands to replay.
The JSONL/hex interchange is a research convenience, not a file-format commitment.

`research_history_index.py` consumes those records using Python's bundled SQLite.
No canonical-core SQLite dependency, production Rust binding, provider abstraction
or second backend is introduced. The store has:

- Hash-addressed payloads for deltas, checkpoints and complete lifetime authority.
- Indexed content states with parent and checkpoint-base references.
- Immutable exact-transaction decisions binding intent, receipt chain, state row,
  head/cursor/Redo and full lifetime-ledger payload reference.
- One current-authority row, updated atomically with payload/state/decision rows.

One writer uses `BEGIN IMMEDIATE`, WAL and `synchronous=FULL`. Terminal negative
reconciliation happens under that same writer lock. Reusing a transaction with a
different intent fails. An identical retry returns its existing decision without
replaying authoring; a terminal negative prevents a late identical edit.

The fixture is linear. Cursor movement tests adjacent Undo/Redo relationships
while retaining the newest lifetime authority, but it is not production Undo/Redo
and does not establish arbitrary branch, instance/epoch, retirement or save-ticket
authority. Existing G2 tests independently retain their prior coverage.

## Conservative verification and resource limits

On reopen, no negative answer is allowed before SQLite integrity/foreign-key checks
and a complete semantic receipt/state/payload audit establish a horizon. A stale
connection must audit again after another writer changes authority. Missing or
inconsistent required state/receipt indexes fail verification. Cold retrieval
checks before-images and the independent Rust oracle before any cursor change.

**The audit scans history and does not satisfy bounded startup.** It is a safe
baseline, not a claim that SQLite inherently requires full scans. A later G3
experiment must establish the reviewed bounded verification/completeness argument
without substituting a high-water counter for proof. This gap does not justify
silently building another backend.

Cold content retrieval follows at most 63 deltas from a direct checkpoint lookup.
The SQLite cache is configured to approximately 2 MiB, mmap is disabled, automatic
WAL checkpointing is disabled, and a 64 MiB WAL threshold attempts a checkpoint
before more admission. A pinned reader causes backpressure instead of deleting
logical history. This threshold can be exceeded by one transaction and is not an
exact physical quota or a retention cap. General reader leases, resident-root
accounting and production Pivot-domain memory admission are not implemented here.

The runner checks native NTFS/ext4/APFS roots and available disk space before the
SQLite workload. No network-share database execution or GUI activation is used.
Actual engine versions/source IDs, compile options, SQL settings and query plans
are recorded. The default platform VFS is used; its actual name is not exposed by
this Python binding. No production engine/VFS/package selection is approved.

## First result and remaining gates

[The first packet](../handoffs/2026-10-27-g3-indexed-store-first-results.md) has
1,000 real mixed transactions and fault tests on NTFS, ext4 and native APFS.
It establishes initial atomic/indexed mechanics, not complete G3 acceptance.
10K/100K histories, larger/formula-heavy workloads, independently instrumented
sync, bounded verified startup/tail work, long-reader policy, full memory and
physical allocation accounting, explicit instance/epoch/control invariants and
further corruption/interruption cases remain work for reviewed follow-up.

Process termination and SQL page exhaustion are tested; neither simulates power
loss. The experiment does not establish durability against lying storage or
rollback of a database and colocated authority. macOS also needs an explicit
sync-policy decision: SQLite documents its separate `fullfsync` setting and its
default OFF behavior in the [official PRAGMA reference](https://sqlite.org/pragma.html#pragma_fullfsync).
The faster Mac measurements must not be read as equivalent durability or hardware
comparisons across the three different engine builds and hosts.
