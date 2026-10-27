# G3 bounded verified-open research result

Parent accepted this research result and corrected admission behavior on 2026-10-27.
G3 remains incomplete; index-cost/reachability research is authorized next.
[Acceptance and next scope](2026-10-27-g3-verified-open-acceptance.md).

2026-10-27. Research result accepted; original measurements and qualifications follow. **Bounded logical verification and
authenticated receipt absence are demonstrated at 1K/10K under the stated trust
boundary. G3 is not complete and no production backend/index layout is selected.**

## Design and failure contract

[Design, completeness argument and failure matrix](../architecture/history-verified-open-experiment.md).
The experiment adds a hash-addressed compressed trie inside the existing SQLite
transaction, coupling receipt/state proofs with payload references, complete
lifetime authority and publication/head/cursor/Redo. It does not infer completeness
from a sequence, count or unchecked cached marker. There is no separate index
publication or second storage engine.

Normal `verified_open` validates current authority and indexed structure, current
receipt/content proofs, latest lifetime authority and a bounded current decode
chain. `audit` explicitly traverses retained history. A missing historical receipt
may not be visited during opening, but its subsequent lookup is Unknown rather
than a negative answer. Missing/corrupt current roots, required indexes or current
content prevent activation. Old content validates lazily before activation.

Verified absence is terminalized under writer exclusion, so delayed identical
requests cannot author an edit afterward. Stale horizons and COMMIT/ACK ambiguity
clear admission. Known proof/row/payload integrity faults quarantine the handle
until a full audit succeeds; maintenance inspection alone cannot release it.
This is per-handle state, not a durable corruption service or production session
coordinator. A fresh process can rediscover latent damage on the affected lookup;
carrying higher-level unresolved-operation state remains a later integration gate.

The argument assumes correct atomic writer transitions, collision resistance,
SQLite/VFS atomic recovery and current publication authority that has not been
coherently replaced.
It does not defend against coherent entire-store/root rewriting or rollback, nor
certify every historical byte without reading it. No witness is introduced.

## Validation and source reconciliation

The final source passed **26 adversarial tests on each of Windows x64/NTFS,
Ubuntu x64/ext4 and native Mac ARM64/APFS: 78/78**. Each also passed five fresh
1K and five fresh 10K open/positive/terminal-negative probes, plus a matched
zero-delta checkpoint profile on separate native database copies.

Tests cover missing receipt/state rows, payload/index/root disagreements, required
index loss, corrupt old proofs, pending/incomplete/oversized recovery tails, wrong
instance/epoch, duplicate/conflicting transaction identities, stale horizons, lost
ACKs, terminal negatives and delayed edits, actual SQLite exhaustion and real
process death before/after atomic index publication. They explicitly distinguish
latent damage from opening-path damage and verify the post-Unknown admission barrier.

The full workload measurements were completed before the final admission-barrier
correction. Review found that an old missing receipt could return Unknown without
clearing admission. An isolated negative counterexample against that preserved
source actually advanced the head **3 → 4** after the failure; the corrected source
blocked the same edit and retained head **3 → 3**. Errors remain visible and no
failed case was relabeled. A further assertion verifies that maintenance inspection
cannot bypass the barrier.

The final correction changes failure admission, not successful payload/index
construction. Rather than replay all completed workload sweeps, final-source tests
and fresh 1K/10K open/receipt profiles validate the changed boundary. The earlier
workload/audit/resource timings below are explicitly observations of the pre-barrier
source; they are not mislabeled as measurements of the final implementation.

## Final-source open measurements

Median of five warm-cache observations, milliseconds. Different hosts/engines
prevent a normalized cross-platform ranking.

| Measurement, 1K / 10K | Windows x64 | Ubuntu x64 | Mac ARM64 |
| --- | ---: | ---: | ---: |
| Constructor including SQLite connect | 1.798 / 1.590 | 0.917 / 0.792 | 0.672 / 0.634 |
| Current-cursor verified activation | 19.897 / 14.556 | 13.147 / 7.925 | 9.750 / 5.775 |
| Matched zero-delta verified activation | 3.518 / 4.582 | 1.966 / 2.374 | 1.457 / 1.849 |
| Matched proof-node reads | 88 / 107 | 89 / 106 | 89 / 113 |

The ordinary 1K head needs 40 deltas; the 10K head needs 16. Its faster activation
is therefore **not** evidence that larger histories open faster. Matched profiles
move the cursor to checkpoint 960 or 9984 on native backup copies while retaining
the same head/latest lifetime authority. Both decode zero deltas and approximately
49.55 KB of payload. Their SQL statement counts are 104/123, 105/122 and 105/129
respectively. No historical decision scan occurs.

The ordinary final-source profiles fetch roughly 329–564 proof nodes and decode
181,145–382,239 bytes. These are authenticated key-probe/decode bounds, not constant
physical page I/O. The compressed proof path is capped at 257 nodes; each node is
still located through a SQLite B-tree. Exact page-cache/page-visit behavior is not
exposed by this Python binding and has not been proven logarithmic end-to-end.

## Earlier full workload and maintenance observations

Each platform ingested actual 1K then 10K mixed G2 streams; 1K gates passed before
10K was attempted. At each size, scalar/formula/Paste/Fill-seed/Fill cases each
represent one fifth of the stream. Rectangles contain 32 cells; this is not a
large-workbook or formula-heavy workload. Producer/oracle work remains separate.
The unchanged accepted native Rust fixture executable was reused only after its
example source matched; all six producer processes printed `FINISH 0`.

| Measurement, 1K / 10K | Windows x64 | Ubuntu x64 | Mac ARM64 |
| --- | ---: | ---: | ---: |
| COMMIT p95, ms | 3.049 / 6.207 | 5.366 / 6.317 | 1.272 / 1.444 |
| Complete publication p95, ms | 30.806 / 49.643 | 21.888 / 27.991 | 14.549 / 19.060 |
| Positive receipt p50, microseconds | 532.0 / 765.7 | 364.3 / 387.7 | 192.8 / 244.6 |
| Terminal negative p50, ms | 2.965 / 5.657 | 4.606 / 4.701 | 1.235 / 1.485 |
| Explicit full audit, seconds | 13.462 / 204.897 | 8.680 / 121.220 | 7.117 / 93.693 |

Publication includes validation/reconstruction, proof maintenance, SQL and commit.
ACK is the local post-COMMIT interval, not a transport/UI acknowledgment. COMMIT
includes backend synchronization wall time; independent sync syscall timing was
not instrumented. Five opens are repeated samples of one generated store, not five
independent workload trials. Earlier first-baseline audit timings used the simpler
layout and are not substituted for this prototype's more expensive full audit.

Real post-COMMIT process termination left 103–194 KB WAL envelopes in the earlier
1K/10K profiles. Verified recovery activation took about 6–20 ms without historical
decision traversal. This is process-death evidence, not power-loss certification.
Preflight refuses incomplete or over-128-MiB WAL input before SQLite recovery.
Application pending markers remain Unknown, never presumed absent.

Held-reader tests forced a one-byte WAL threshold and observed checkpoint refusal
after about 202–285 ms, unchanged authority, then successful admission after release.
This exercises backpressure without inventing a retention cap. General production
reader leases/deadlines remain unimplemented. Passive checkpoint and allocation
observations are separately retained in each raw summary.

## Space, memory and engine caveats

The prototype has a material cost: databases were about **18.1–18.2 MB at 1K and
195.7–195.9 MB at 10K** after query/audit phases. Linux/Mac dbstat attributes about
75 MB at 10K to proof nodes alone. Superseded path nodes are retained; no GC is
implemented or authorized by this experiment. The 10K pre-checkpoint WAL occupied
about 14–19 MB at the sampled instant, not total WAL traffic over the run. Database,
WAL, allocated blocks, payload bytes and index pages are distinct measurements.

Supplementary native Windows file-allocation queries report 18,219,008 and
195,952,640 bytes after recovery/close. Linux/Mac st_blocks/dbstat measurements are
in the raw summaries; Windows dbstat was unavailable. These are filesystem allocation
observations, not device-level writes. Ubuntu's own-process `write_bytes` grew by
roughly 4.4 GB during 10K publication, showing an amplification concern; it is not a
normalized VFS/device measurement. No production layout recommendation follows.

Publication-phase process RSS was approximately 23–34 MB across these observations.
Measured final-process peak RSS was roughly 27–38 MB; traced Python activation
scratch peaked around 185–211 KB. RSS includes Python, SQLite and measurement buffers;
the harness retains timing samples proportional to transaction count. Rust domain
peaks, measured independently by the producer, were about 10.53–10.56 MB with zero
final charge. The configured SQLite cache is approximately 2 MiB, not an observed
whole-process or Pivot-domain cap. Native cache hits/misses, complete transient
native scratch and OS-cache ownership remain unqualified. Own-process faults/IO
are retained without unrelated process inspection.

Exact engines remain Windows 3.50.4, Ubuntu 3.46.1 and Apple 3.54.0, with source IDs
and compile options preserved. Supplementary public-C-API queries matched those
source IDs and reported default VFS **win32 / unix / unix**; the experiment supplies
no VFS override. This is default-VFS identification, not syscall certification.

All runs use WAL and synchronous FULL. Primary Mac runs explicitly request/read
back fullfsync=1. The same 1K fixture with fullfsync=0 measured COMMIT p95 0.564 ms
versus 1.272 ms with it enabled. This is a connection-policy comparison, not proof
of equal physical durability across systems. No VFS sync trace or power-cut test
was performed. The older Windows/Linux builds' WAL-reset fix status remains
unqualified; no competing checkpoint writers were tested. The
[official SQLite WAL notes](https://sqlite.org/wal.html#walreset) identify that engine
qualification requirement. No system SQLite/SDK was replaced.

## Evidence and reproduction

Under `artifacts/g3-verified/`:

- `g3-verified-20261027-a/`: original workload reports, transaction logs, real G2
  fixture streams, engine/allocation observations, earlier matched profiles and
  native-filesystem receipts. Each target has returned `evidence.zip` and extracted
  `evidence/{1000,10000}/summary.json`; Mac also has `1000-sync-off`.
- `g3-verified-20261027-c/`: final-source 26-test logs and fresh 1K/10K open/receipt
  samples in each target's `evidence/`, final matched samples in `matched-open.json`,
  and the preserved old/fixed barrier counterexample (`barrier-a.json`, `barrier-c.json`).
- Earlier probe logs and intermediate `b` verification remain as development history.
  Final source and claims use `c`; the original accepted G3 baseline is unchanged.

Native files remain under each target's `~/PivotValidation/<run>/<rid>/`; Windows
uses User's home. Copies and backup measurements stayed on NTFS/ext4/APFS, while
results returned to shared storage. Controller source scripts for collection,
matched copies, engine/allocation queries and the counterexample are retained
alongside the artifacts. Git revision/dirty state and ordinary commands are recorded.

For an independent native-local reproduction, build the unchanged Rust fixture
with `cargo build --locked --release -p pivot-workbook --features history-experiment
--example history_index_fixture`, then invoke:

```text
python scripts/research_history_verified_run.py <new-native-output> --fixture-binary <absolute-native-executable> --fixture-source <absolute-example-source>
```

Use python3 where appropriate. The runner executes failure tests, gates 10K on 1K,
keeps producer/oracle work separate and performs the Mac sync comparison. The
supplementary final probes use native SQLite backup copies so original evidence
databases are not rewritten.

## Review request and remaining boundaries

Review the authenticated completeness argument, explicit lazy-corruption boundary
and corrected admission barrier. The experiment removes the normal full-history
scan under those assumptions, but its proof-index write/space amplification,
unqualified engine/cache/physical-I/O bounds and later coordinator/epoch authority
prevent a production choice or complete G3 acceptance claim. Keep SQLite as the
candidate while deciding the next bounded research work; do not silently replace
it with a custom backend or adopt this index layout as a product requirement.

No 100K run or scale claim was made. G1 ordinary WB5 Save, G2, production core,
session/UI/protocol/persistence and six-RID GUI/AOT acceptance are unchanged.
Clipboard ISSUE-0001 remains separate. G4–G7 and production Undo/Redo stay gated.
