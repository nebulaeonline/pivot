# G3 storage accounting, scalable admission proposal and readiness assessment

2026-10-27. **Immediate deliverable complete, ready for parent review.** This packet
stops at the physical assessment and admission proposal, as directed. No expanded
index workload, full 100K G2 run, backend selection or production integration occurred.
The accepted 1,024-leaf runtime profile and zlib-6 baseline are unchanged.

[Assignment and explicit stop boundary](2026-10-27-g3-hierarchy-dictionary-acceptance.md) ·
[Physical storage model](../architecture/history-storage-accounting.md) ·
[Scalable admission proposal](../architecture/history-scalable-admission-proposal.md).

## A — Physical accounting: complete within the stated representation

The read-only native collector reconstructs SQLite's table/index record encodings
from actual values. Ubuntu and Mac match all nine dbstat objects exactly; Windows
has a modeled record size because dbstat is unavailable. Source database/WAL hashes
are unchanged and all canonical receipt manifests match accepted source snapshots.
The snapshot includes 10,033 receipts after the standard probes, not a new workload.

| Existing 10K fixture, bytes | Windows | Ubuntu | Mac |
| --- | ---: | ---: | ---: |
| Compressed authoritative payload | 7,931,306 | 7,931,326 | 7,932,104 |
| Total file | 12,554,240 | 12,521,472 | 12,525,568 |
| Serialized table/index records | 11,199,793 | 11,199,591 | 11,200,591 |
| Unused space | Unknown | 1,059,588 | 1,062,627 |
| Page/cell overhead | Unknown | 262,293 | 262,350 |

**20% is unattainable by page packing alone with this schema/encoding held fixed.**
Its existing serialized records already cost **41.21% beyond compressed payload**, even
with all page overhead and unused bytes treated as free. Ignoring record headers too
still leaves **38.00%**. The 20% total would be about 9.518 MB, roughly 1.68 MB below
the existing records themselves. This is a conditional representation lower bound,
not a claim about all possible authenticated stores or information-theoretic entropy.

The accounting separates payload, identifiers/intents/oracles, authentication, repeated
hash/key copies, required index records, integer/record headers, page structure, slack
and native allocation. It identifies potential alternate placements/derivations without
assuming their lookup/recovery costs are free. One independently retained fixed-width
copy of the fixture's distinct IDs/intents/commitments/oracles totals 1,606,912 bytes
(20.26%); that is a field budget, not proof that each must be stored independently.

The current 12.5 MB cost is roughly 1.25 kB per retained decision. Nothing here
establishes the same ratio for larger or broad-mutation models. The evidence supports
the parent's decision to treat 20% as an unproven aspiration rather than forcing
semantics changes to satisfy it. No further codec or tiny packing experiment ran.

## B — Admission proposal: logical model validated, runtime qualification pending

Propose fresh research databases with **4,096 leaves / depth 24 / 60 entries / eight
splits**, retaining the exact identities, proof rules and canonical receipts. The
current writer still rejects 4,096 before database creation. No default was raised.

For one insertion, successful splits follow one overflow chain: `d+s <= 24`, `s<=8`.
The tight maximum is **33 node writes**, **5,473 node bytes**, or **5,537 index bytes**
with a conservative root-envelope allowance. One proof uses at most **25 nodes /
5,401 bytes**. The proposed directory has at most 8,191 current nodes and an aggregate
245,760-entry ceiling; adversarial key concentration may refuse much earlier.

Ten pure adversarial tests passed on each native target (**30 passes**): maximal
successful split chain, depth/split/leaf exhaustion, last-leaf admission, non-splitting
insertion at a full leaf cap, duplicate/wrong-interval refusal, malformed configuration,
arithmetic bounds and proof that the larger profile is not enabled in the store.
At most 61 keys enter a case; no larger history is generated. These are model tests,
not a claim of new SQLite publication/fault qualification.

The proposal requires planning under the verified writer horizon, reserving resources
before dirty writes, and atomic rollback/refusal without pruning, identity changes or
permanent path copies. It distinguishes logical work from heap, dirty pages, WAL,
payload decode and physical disk costs. Proposed pilot budgets include a 64 MiB DB
cap, 1 MiB instrumented index-scratch provision, 512 MiB RSS safety stop, 256 MiB disk
planning per point and a 512 MiB free-space floor. These are review proposals, not
validated production limits; physical reservation/enforcement remains an explicit gap.

The 64 MiB WAL checkpoint trigger and 128 MiB recovery preflight are not hard growth
quotas. A transaction may overshoot the trigger; logical node bounds do not prove
its frame count. Native allocator/cache, pager/frame and pinned-reader qualification
must precede conditional 32,768/65,536-control proposals. Full G2 scaling needs its
own payload/cold-root budgets and separate parent authorization.

## D — Comparison with original G3 production-readiness requirements

| Original requirement | Established evidence | Remaining gap and gate ownership |
| --- | --- | --- |
| SQLite-first indexed mechanics | Transaction and state indexes, exact intent/replay, authenticated presence/absence; path-local current nodes | Research-only Python binding; no pinned production engine, Rust binding or package selected. G3 selection obligation |
| Bounded verified startup/reconciliation | Current root plus bounded path/state decoding; no ordinary history scan in tested fixture; stale horizons refused | Valid near-limit WAL recovery work and physical cache/page behavior unmeasured; only linear epoch-0 research contract qualified. G3 evidence gap |
| Incremental authoritative payloads | G2 accepted complete mutation-local records and independent checkpoint/oracle comparison; canonical receipts preserved by normalization | Do not reopen G2. Broad/full-G2 scale and content-size admission are not established by control probes |
| Predictable storage growth | No persistent-path accumulation; 10K retained storage/accounting and 16K-control growth accepted | Larger profile not enabled or physically admitted; no 100K/broad-mutation extrapolation. G3 gap |
| Actual SQLite/VFS configuration | Exact versions/source IDs and FULL/WAL/4 KiB settings recorded; Mac fullfsync/checkpoint_fullfsync enabled | Actual VFS identity, engine fix qualification and distributable build/config pin are unresolved. G3 blocker before stronger backend claims |
| Physical synchronization and acknowledgement cost | Publication/COMMIT timings, WAL footprint and checkpoint samples; local process-death tests | No direct xWrite/xSync/frame tracing or measured physical-sync contract; process death is not power loss. G3 selection evidence, with witness publication deferred to G4 |
| WAL/tail/read bounds | Pre-open size/framing checks; checkpoint/backpressure experiments; old snapshots/stale writers covered | Trigger can overshoot; no complete dirty-page/frame reservation or read-duration policy. G3 experimental bounds needed; product lifecycle/read leases remain G5 |
| Resident memory and scratch admission | 2 MiB configured cache; roughly 25–33 MB ingest RSS; model bounds on raw index paths | No native allocator/cache high-water or measured per-domain reservations; configured cache is not total resident memory. G3 instrumentation gap; production pool/ticket budgets remain G5 |
| Fault recovery and explicit maintenance | Split/commit process death, stale readers, corruption/missing coverage, canonical reconstruction and earlier exhaustion tests | New profile/quota enforcement not crash-qualified; valid large recovery tails and physical I/O failures need reviewed cases. G4 witness/epoch/suffix-loss implementation remains excluded |
| Independence from external witness assumptions | Current publication trust/collision/SQLite atomicity boundary stated; coherent rollback explicitly unsupported | No witness implemented or claimed. Selection must document the future contract; do not require G4 implementation to complete this G3 assessment |

The original requirements are in
[production readiness](../architecture/history-production-readiness.md#2-durable-indexed-lookup-and-verified-startup)
and the [first indexed-store experiment](../architecture/history-indexed-store-experiment.md).
G3 remains incomplete for concrete resource/engine evidence reasons, not because the
unproven 20% percentage failed.

### Engine qualification that must not be lost in the storage discussion

The accepted runs used Windows SQLite **3.50.4**, Ubuntu **3.46.1** and Apple **3.54.0**,
with full source IDs in their evidence. SQLite documents a WAL-reset corruption fix
in **3.51.3**, with backports **3.44.6** and **3.50.7**. Windows/Ubuntu are in the older
version families; no vendor backport proof has been collected. A version string alone
is insufficient to infer vendor patch status or approve the Apple private build.
[SQLite's WAL-reset advisory](https://www.sqlite.org/wal.html#wal_reset_bug).

Before stronger concurrent writer/checkpoint stress, select or verify a fixed research
build and its actual VFS, preserving all earlier evidence for its tested inputs.
No engine was upgraded here and no accepted pass was retroactively relabelled. This
packet uses read-only connections to the accepted databases. Configuration still
records WAL/FULL, mmap off, a 2 MiB cache and automatic checkpoints off; Mac additionally
records both fullfsync settings enabled. Those flags are not observed sync guarantees.

## Recommendation and stopping point

**Do not close G3 or select a production backend/configuration yet.** The SQLite
architecture remains viable within the accepted scope; the remaining priority is one
narrow research gate for **fixed-engine/VFS instrumentation and explicit experimental
resource admission**, not compression or a custom replacement engine.

That proposed gate should implement the reviewed pre-write planner and guards, expose
the actual connection's native cache/allocator and WAL/frame/sync telemetry, and test
refusal/rollback/recovery at small bounds. Only after that evidence passes should the
documented larger control probes be considered. If safe resource admission cannot be
established, return rather than quietly increasing ceilings. Any full-G2 scaling
proposal remains separate and needs disk/content-memory/stop budgets and parent approval.

This packet is the requested stop after the physical assessment and scalable admission
proposal. Assignment C's expanded scaling runs have **not started**. The next gate
above is a recommendation for review, not newly inferred authorization to change an
engine, enable larger bounds or run full 100K transactions.

## Evidence and checks

`artifacts/g3-accounting/g3-accounting-20261027-b/<rid>/` contains the native filesystem,
commands, source revision/dirty state, exact new input hashes, read-only accounting,
canonical-equivalence assertion, proposed bounds and ten-test log. All three native
collections passed; Mac/Ubuntu exact byte identities close, Windows dbstat absence
is explicit. Original native source DB/WAL fingerprints are unchanged. This is
additional read-only/model evidence, not a repeat of accepted application/CI gates.
`review-data.json` reconciles exact byte identities, current tested-input hashes and
the 30 model passes. The first collection (`g3-accounting-20261027-a`) is preserved;
the final collection refines cap-boundary examples to globally realizable depth-12
leaves and confirms byte-identical accounting/source fingerprints. No scale test or
accepted performance gate was rerun as part of that test refinement.

Reproduce with `research_history_accounting.py <existing-db> <new-report> --scripts
<frozen-source/scripts>` and `test_admission_model.py`. The pure bounds command is
`python scripts/research_history_admission_model.py`. No 32K/65K/100K execution is
part of these commands. Native reports remain under
`~/PivotValidation/g3-accounting-20261027-b/<rid>/` and are copied to shared artifacts.

No production integration, codec adoption, witness, retention/GC, G4–G7 implementation,
protocol/workbook changes, identity recycling, product-limit or VM/Defender changes.
G1/G2 and six-RID acceptance stay closed; clipboard ISSUE-0001 remains separate.
