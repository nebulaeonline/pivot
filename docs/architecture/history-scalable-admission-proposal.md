# G3 scalable directory admission proposal

2026-10-27. **Proposal for review, not an enabled profile.** The accepted store still
admits at most 1,024 leaves. Its constructor rejects 4,096; the new pure model tests
explicitly check that rejection occurs before a database is created. No larger
native history or index-only workload was executed for this packet.

## Proposed index-only profile

Retain 60 entries/leaf, depth 24, eight splits/publication, fixed key/digest widths,
separate branch/leaf tables, rowid payloads, zlib 6 and existing identity semantics.
Propose **4,096 leaves** for fresh, explicitly identified research databases only.
The full configuration remains authenticated in publication metadata and must match
on reopen. Do not reinterpret a qualified 1,024-leaf database or silently increase
defaults, product limits or process-wide settings.
The constructor's generic parameter parser currently permits split limits up to 24;
the accepted measured profile uses eight. Parser acceptance is not qualification of
every tuple. The proposed run must pin the reviewed tuple, not infer permission to
raise the split/depth budgets from available constructor arguments.

This gives an aggregate capacity ceiling of 245,760 entries, not a guarantee that
any chosen set of that size is admissible. Concentrated keys can hit depth or split
bounds while most aggregate space remains free. Hash routing does not make such
inputs impossible. Refusal preserves all old receipts and authority; it never drops
history, fabricates a negative result or retries with a new transaction identity.

The existing controls probe reaches 421–427 leaves at 16,385 receipts. Multiplying
that occupancy by four is a planning observation only, not proof that a 65K workload
will fit. The proposed leaf ceiling provides room to test that question without
changing path-depth work or returning to permanent superseded path copies.

## Structural and logical-work bounds

`research_history_admission_model.py` plans a single insertion into at most 61 scoped
keys without IO. Ten small adversarial tests validate the model on all three native
targets. It is not wired into the writer and does not establish SQLite enforcement.

| Bound for the proposed profile | Value |
| --- | ---: |
| Current leaves | 4,096 |
| Current branch + leaf nodes | 8,191 |
| Maximum prefix-address integer | 33,554,431 |
| Maximum entries across all full leaves | 245,760 |
| Raw node bytes at that full capacity | 15,998,911 |
| Nodes on one authenticated path | 25 |
| Node bytes on one path | 5,401 |
| Logical nodes written by one successful insertion | 33 |
| Logical node bytes written | 5,473 |
| Index bytes including a conservatively bounded 64-byte root envelope | 5,537 |
| Node INSERT/DELETE statements in the separate-table implementation | 57 |

The full-capacity raw total is `4096*(1+60*64) + 4095*65`. It is persistent logical
data, not resident memory, SQL page allocation or the size of a 65K control database.

For an existing leaf at depth d and s successful splits, node writes are
`d + 2*s + 1`. One insertion can introduce only 61 entries. Both children cannot
overflow a 60-entry leaf, so recursive overflow follows one chain and `d+s <= 24`.
With `s <= 8`, the tight node-write maximum is 33, at d=16/s=8. The same constructed
case writes `65*(d+s) + 64*61 + (s+1) = 5473` node bytes. An unsplit full leaf writes
at most `65*24 + 3841 = 5401` bytes. Adding at most 64 root-envelope bytes gives 5537.
The envelope bound assumes admitted nonnegative SQLite signed-64-bit sequences,
32-byte commitments and this fixed configuration; it is not an arbitrary-format bound.

These bounds exclude canonical receipt/payload SQL, indexes, SQLite page balancing,
cache spills, WAL framing, sync calls and temporary allocations. In particular,
**33 logical nodes do not imply 33 dirty SQLite pages or WAL frames**. Branch updates
also issue a leaf-table DELETE; most ancestor deletes find no leaf. Those extra index
searches are included in the statement bound, not misreported as data bytes written.

## Staged admission and refusal contract to implement after review

1. Verify the exact engine/profile, local filesystem, current authority and reader
   context. Acquire the existing writer-excluded horizon before planning.
2. Authenticate the selected path and existing leaf; reject a conflicting/duplicate
   binding using existing exact-intent rules. Plan all splits using the bounded
   61-entry scratch set, without altering SQL rows or authority.
3. Check depth, split count and `current_leaves + planned_splits`. A full leaf cap
   still permits insertion into a non-full leaf. Missing/corrupt coverage is an
   integrity fault, not a resource rejection or evidence of absence.
4. Admit scratch/native allocation and disk/transaction resources **before** dirty
   writes. A logical node-byte bound alone is not sufficient physical admission.
5. Apply the plan, receipt and publication in one SQLite transaction. On any refusal
   or write error, roll back; preserve exact IDs, all retained receipts and old root.
   Reopen/verification must precede further mutations where authority is uncertain.
6. Reader snapshots remain SQLite-owned. Old verified writers remain stale; no
   application-level historical root/path collection is introduced.

The pure model covers an eight-split extreme, depth exhaustion with free leaf budget,
split exhaustion before depth exhaustion, last-leaf success, cap refusal, non-split
success at the cap, duplicates, wrong intervals and invalid configurations. Those
tests validate arithmetic/refusal planning, not new crash recovery or larger SQLite
transactions. The accepted writer's original rollback and stale-reader evidence is
preserved; future wiring needs its own tests.

## Proposed pilot resources and remaining admission proof

The following are **requested research guardrails**, not approved product defaults
or already proven hard resource bounds:

| Resource | Proposed pilot provision | Qualification needed before expanded execution |
| --- | --- | --- |
| SQLite page/cache profile | 4,096-byte pages, existing configured 2 MiB cache, mmap off | Verify actual settings and native cache/allocator use; configuration is not measured ownership |
| Index scratch | Provision 1 MiB separately from content/oracle scratch | Instrument and refuse allocations exceeding the admitted reserve; 5,537 logical bytes is not a heap bound |
| Private worker RSS | Abort ceiling 512 MiB | Monitoring is a safety stop, not reservation or prevention of transient overshoot |
| Database | Proposed `max_page_count=16384` (64 MiB) for a fresh control-only database | Verify enforcement/SQLITE_FULL rollback on the chosen engine; not a payload/history product quota |
| WAL/recovery | Retain reviewed 64 MiB checkpoint trigger and 128 MiB recovery preflight | These are not a hard growth cap; establish transaction-frame/dirty-page work and pinned-reader behavior first |
| Disk planning | 256 MiB per retained point, plus a 512 MiB free-space floor | Two proposed points require at least 1 GiB initially; check before each point and during the run |
| Observer/log memory | Stream raw events; calculate quantiles after closing the measured writer | Avoid hiding sample arrays in claimed store-resident memory |

The per-point disk provision includes 64 MiB main DB, 128 MiB WAL, 32 MiB auxiliary/
evidence and 32 MiB transient space. A free-space check is not an OS reservation or
a guarantee against other users or thin-provisioned backing-store exhaustion. Do not
claim that provision as a validated maximum until actual transaction/WAL bounds and
failure behavior support it. If they cannot be established, return before scaling.

Current admission checkpoints only when the existing WAL has already reached 64 MiB.
One transaction may overshoot. `journal_size_limit` or a DB page cap must not be
misrepresented as a hard WAL bound. A single transaction can write multiple frames
for physical pages; no one-frame-per-logical-node assumption is permitted. Measure
the real pager/dirty-cache/frame work on the actual selected engine.
See SQLite's [journal-size limit](https://www.sqlite.org/pragma.html#pragma_journal_size_limit),
[database page cap](https://www.sqlite.org/pragma.html#pragma_max_page_count) and
[WAL checkpoint behavior](https://www.sqlite.org/wal.html).

Index-only publications have fixed small receipt bodies and the unchanged seed,
unlike full G2 deltas/checkpoints with independently large decode, encoding and core
allocation work. Neither this profile nor its scratch estimates admit full G2 edits.
Cold reconstruction still has content-size costs; a constant root does not bound them.

Test-owned readers may intentionally pin a WAL, but must have explicit release and
experiment-stop handling. No product ticket/read-lease expiry or destructive retention
rule is selected; G5 remains held. Checkpoint refusal must produce backpressure or
stop the experiment, never discard receipts or terminate unrelated readers.

## Conditional scaling sequence, not execution authorization

First implement and qualify explicit planning/resource guards on the reviewed engine
with small/adversarial cases and at most the already tested receipt count. Record
actual cache/native allocation, WAL frames/bytes, sync behavior, quota failures and
crash/reopen evidence. Do not load a different SQLite library merely to observe the
Python binding and then attribute its counters to the tested connection.

Only after that gate passes should fresh **32,768** and then **65,536 terminal-negative
control publications** be considered, each plus its genuine G2 seed. Keep the original
scoped ID/intents for the common prefix and compare canonical receipts. Report every
point separately; stop on any refusal, coverage/oracle disagreement, unexpected scan,
resource overshoot, sync uncertainty or recoverability failure. Do not raise bounds,
retry around failures or combine the controls into a full-G2 claim.

Measure database/WAL EOF and allocation, logical index writes, indexed queries,
verified startup including any valid recovery tail, seed cold reconstruction,
publication/COMMIT and process memory. Hold baseline configuration and no-codec-change
policy fixed. Preserve failed attempts. Engine fixes, instrumentation and these
conditional runs are proposals for the next review, not performed in this packet.

Any full-G2 scale proposal—including 100K—needs separate parent authorization with
payload/cold-root, resident-memory, disk and stop budgets. This packet stops at the
[physical assessment and readiness comparison](../handoffs/2026-10-27-g3-storage-admission-assessment.md).
