# G3 bounded verified-open experiment

Parent accepted this research result and corrected admission behavior on 2026-10-27.
G3 remains incomplete; index-cost/reachability research is authorized next.
[Acceptance and next scope](../handoffs/2026-10-27-g3-verified-open-acceptance.md).

2026-10-27. Research only. This extends the accepted SQLite fixture; it is not a
production coordinator, a second backend, a workbook-format change or an external
witness. The original full-audit fixture and its evidence remain unchanged.

## Why SQL absence is insufficient

A successful SQL query returning no row does not distinguish never-committed from
loss of a previously committed row. Counting rows or checking the latest sequence
does not repair that distinction. The experiment therefore binds an authenticated
map of every retained receipt and state into the same atomic publication as the
ordinary SQLite rows. It does not rebuild that map by scanning at normal open.

`research_history_verified.py` stores immutable, SHA-256-addressed compressed binary
trie nodes in a SQLite table. Keys hash the expected fixture instance, fixed research
epoch 0, namespace and transaction/state identity. A leaf holds a digest of the full
typed SQL row; a branch holds a binary prefix and two child hashes. Prefixes strictly
lengthen along a path, so a proof needs at most 257 node fetches for a 256-bit key.
An absent key ends at a verified differing leaf or differing compressed prefix.
Missing/corrupt nodes never count as an empty branch.

This is a deliberately simple authenticated secondary index *inside SQLite*, not
a custom log or alternative storage engine. It duplicates some indexing work and
retains superseded path nodes without GC. Its write/space costs are part of the
experiment, not approved production requirements. Normal SQL indexes remain useful
for locating rows, but a queried row must agree with its authenticated proof.

## Atomic invariant and trust boundary

The seed transaction creates its state, full lifetime authority, receipt, map and
publication together. Each subsequent writer begins under SQLite writer exclusion
against a verified authority horizon. One SQLite transaction then publishes:

- Resolved immutable delta/checkpoint and complete lifetime payload references.
- The new state row when applicable, including parent/content/checkpoint identity.
- An immutable transaction/intent/decision/receipt row.
- Corresponding authenticated state/receipt leaves and replacement path nodes.
- Head/cursor/Redo/lifetime authority and its current publication seal/root.

There is no separate post-commit optional-index update. The current publication
binds its instance/epoch, authority tuple, authenticated root and recovery marker.
The map binds the full current receipt, whose body independently agrees with head,
cursor, Redo and lifetime authority. The publication checksum is not an external
authenticity anchor; completeness follows from the seed/update invariant, verified
lookup paths, collision resistance and SQLite atomicity together, not that checksum
or the high-water sequence alone.

Assumptions: the experiment's writer constructs correct transitions; SQLite/VFS
honor their documented transaction/recovery semantics; hash collisions are absent;
and current publication authority is not coherently forged/replaced or rolled back
with the entire database. An actor able to rewrite all hashes and current authority
can forge a different history. SQLite alone cannot detect that or coherent rollback.
No witness, anti-rollback guarantee or comprehensive latent-media guarantee is
claimed. This is consistent with the existing G3 boundary, not a substitute for G4.

The expected instance is supplied from the independent fixture context, not adopted
from an unchecked persisted marker. Other instances/epochs/formats fail. Research
epoch 0 is fixed; general epoch lifecycle and retired-receipt semantics remain out
of scope. There is no reader for the earlier baseline database layout.

## Normal open versus maintenance audit

Before creating a SQLite connection, preflight bounds the physical WAL to 128 MiB,
checks header/frame alignment and refuses unexpected rollback journals. Oversize or
incomplete tails require explicit maintenance, not truncation or an inferred
negative outcome. The file-size envelope bounds admitted recovery input; it is not
a measured count of frames actually replayed, nor a hard bound on COMMIT wall time.
SQLite performs its own WAL checks/recovery. This prototype's one writer/checkpoint
owner and ordinary 64 MiB admission threshold keep normal WAL work small; an oversized
recovery input is Blocked even if it might be valid. This is a temporary research
resource bound, not history retention or a product capacity increase.

Normal `verified_open()` takes a consistent writer-excluded snapshot and verifies:

1. Exact required schema/index definitions, the current publication and its instance,
   epoch, checksum, root node and empty application recovery marker.
2. An authenticated current receipt matching its SQL row and authority tuple.
3. Authenticated current head/cursor/Redo state rows, linear control relationships,
   and head-to-latest-lifetime binding.
4. Current cursor content from one authenticated logical checkpoint plus at most
   63 authenticated deltas, original-state checks and the independent Rust oracle.
   The complete latest lifetime payload is also hash-validated.

No historical decision traversal, global integrity_check or historical row count
is used here. There is bounded logical proof/decode work, not a claim of constant
physical I/O: each node is itself located through a SQLite B-tree whose height/cache
behavior can change. The Python binding exposes SQL/proof/VM work, not exact SQLite
page-cache hits/misses or physical page visits. Desired production page-access and
memory bounds remain unqualified.

`audit()` remains explicit full maintenance: SQLite integrity/foreign-key checks,
all decisions and state reconstruction, receipt/state proof agreement, and reachable
map-node validation. Superseded unreachable proof nodes are non-authoritative research
garbage; GC and comprehensive auditing of those unused bytes are not implemented.
An audit failure clears admission. Lazily loaded historical state always validates
its row proof, payloads, before-images and oracle before cursor activation.

## Reconciliation and failure cases

The same writer exclusion protects reconciliation. A stale horizon refuses admission.
After verified open, an uncertain prior request resolves against the recovered map:
an exact positive returns its immutable receipt; a conflicting intent fails; verified
absence is atomically recorded as a terminal negative. A delayed identical authoring
request then returns that negative receipt rather than executing. COMMIT/ACK ambiguity
clears local admission until verified reopening; no authoring replay is used.

Discovery of an authenticated path/row/payload integrity fault also quarantines
that handle: normal reopen cannot clear it, and even maintenance inspection cannot
admit mutations before a complete audit succeeds. Errors are rethrown, not converted
to success/absence. This is per-handle quarantine, not a durable corruption service;
a fresh process may again discover latent damage only when its affected path is
visited. Carrying higher-level unresolved-operation state across application restart
and general coordinator/lifecycle authority remain separate production gates.

| Damage/condition | Bounded open | Later access / full audit |
| --- | --- | --- |
| Missing required SQL index/schema change | Unknown | Maintenance required |
| Corrupt publication/root/current receipt or current content | Unknown | No activation |
| Missing old receipt row outside the opening paths | May open | Its lookup is Unknown, never inferred uncommitted; audit fails |
| Corrupt proof node on a required lookup path | Unknown if visited | Any affected lookup is Unknown; audit fails |
| Missing/corrupt old state/payload outside current decode chain | May open | Cold load fails before activation; audit fails |
| Incomplete/oversized physical WAL or application pending marker | Unknown/Blocked | No terminal negative until recovery is verified |
| Process death during publication | SQLite exposes old or new atomic publication | Reopen/reconcile; no half-index success |
| Coherent entire-store rollback or root forgery | Not guaranteed detected | Outside this unanchored experiment |

The tests make the lazy-versus-opening distinction explicit. A successful bounded
open is not a certificate that every historical byte is intact.

## Engine and sync qualification

Keep WAL and synchronous FULL, with connection-local fullfsync requested explicitly
for the primary Mac runs and an OFF comparison on the same 1K fixture. A PRAGMA
readback is not proof of actual VFS syscalls or storage power-loss behavior.
Compile options, source IDs, settings and native filesystems are recorded. No system
engine or SDK is replaced, and no production binding is selected.

SQLite's [atomicity documentation](https://sqlite.org/atomiccommit.html) describes
the backend guarantee and its hardware assumptions; [WAL documentation](https://sqlite.org/wal.html)
describes commit/recovery/checkpoint behavior. Its documented WAL-reset race affects
older builds under overlapping checkpoint/write connections. The Windows/Linux
research engines' fix status is unqualified; this experiment uses one writer and
checkpoint owner and does not test competing checkpoint writers. These runs do not
qualify those builds for production. See the official
[WAL-reset notes](https://sqlite.org/wal.html#walreset) and
[fullfsync reference](https://sqlite.org/pragma.html#pragma_fullfsync).

## Measurement boundary

Compare actual 1K and 10K G2 streams only after the 1K tests/open gate pass. Keep
producer/oracle logs separate, retain failed attempts and perform no 100K run.
Record constructor versus activation, proof/SQL/decode work, positive/terminal
negative queries, independent cold samples, full audit, a real post-COMMIT process
termination/recovery, checkpoint refusal under a held reader and release/admission.
The forced reader test uses a one-byte WAL threshold to exercise backpressure
without generating an artificial 64 MiB workload; it is not a general reader-lease
policy. Record COMMIT/local ACK intervals, process RSS, traced Python activation
scratch, configured cache and available filesystem/dbstat allocation. Timing samples
are warm-cache observations; SQL/proof counts are not physical page measurements.
G1/G2, GUI/AOT acceptance and clipboard ISSUE-0001 remain unchanged.

[Measured results, counterexample and final-source validation](../handoffs/2026-10-27-g3-verified-open-results.md).
