# Bounded disk-history feasibility plan

Date: 2026-10-23. **Inventory and Stage 1 cold reconstruction accepted;
Stage 2 representation feasibility accepted; Stage 3 Windows/Linux decision/restart research implemented, review pending. Production integration deferred.**
[Stage 1 acceptance and Stage 2 resource qualifications](../handoffs/2026-10-23-cold-history-stage1-review.md).
[Stage 1 implementation and executed evidence](../handoffs/2026-10-23-cold-history-restoration-stage1.md).
[Stage 2 implementation, measurements and qualifications](../handoffs/2026-10-23-history-representation-stage2.md).
[Stage 2 acceptance and Stage 3 safeguards](../handoffs/2026-10-23-history-representation-stage2-review.md).
The stages below remain the experiment plan. Stages 1 and 2 are accepted research evidence. Additional adaptive cumulative-work checkpoint/fallback tests and measurements are
reported separately from the fixed-interval baseline.
[Stage 3 persisted decision/fault/restart evidence](../handoffs/2026-10-23-history-durability-stage3.md)
is ready for review, with explicit process-crash versus power-loss and scaling limits.
File association remains unproven; Stage 4 is deferred until Stage 3 review.
[Inventory acceptance and cold-restoration gate](../handoffs/2026-10-23-cold-history-inventory-review.md).
[Inventory, identity-boundary probes and allocation measurements](cold-history-state-inventory.md).
[Resolved logical/runtime/authority contract](../handoffs/2026-10-23-cold-history-identity-review.md).
[Architecture approval and safeguards](../handoffs/2026-10-23-tiered-history-design-review.md).
[ADR-0023](../adr/0023-document-transaction-history.md),
[parent direction](../handoffs/2026-10-23-undo-redo-feasibility-review.md),
[accepted hot-root foundation](../handoffs/2026-10-23-undo-redo-feasibility.md).

## Question and experiment boundary

Can Pivot retain an extensible recoverable timeline on disk, with bounded live
memory and bounded cold restoration, while preserving canonical definitions,
identities, historical drafts, atomic edits and independent file-save authority?

Build a feature-only research adapter/harness after architecture authorization.
Reuse accepted hot activation and cleanup. Do not implement a production coordinator,
public provider framework, session RPC, desktop UI, JIT/SIMD, history-aware WB5
format or compatibility reader. Use local temporary stores; hosted databases and
network storage are out of scope. Physical production packaging remains undecided.

The work is staged: stop at a failed semantic/resource gate before expanding the
prototype. Measurements do not authorize changing graph/workbook/allocation limits.
Do not make current-document Save write history into WB5.

## Stage 1 — complete cold-state inventory and oracle

Map each retained field to canonical root, workbook root, lifetime ledger, historical
provenance or transient authority. Establish a small versioned logical research image
and an independently inspectable semantic comparison, not a byte copy of Rust memory.

| Preserve as authoritative state | Rebuild/issue fresh, with verification | Never restore from history |
| --- | --- | --- |
| Exact typed Decimal96/text/blank content; source/drafts and errors | Persistent lookup/reverse/selection indexes derived from stored authoritative records | Request/session authority, locks and save tickets |
| Scalar/range bindings, stable IDs, retired references, positions and axis metadata | Current Expected/generation and history-control authority | Calculation results, source contexts, viewport and OS clipboard |
| Content/transaction identities, parent links and original provenance | Admitted allocations and bounded cleanup ownership | Old allocation charges or process addresses |
| Current committed lifetime floors/reservations, independent of active content | Runtime-private representation with explicit identity mapping | Cached history roots as the only durable truth |

Use logical state/identity comparison plus existing calculations where valid.
Compare original diagnostics and bound targets directly; evaluating alone cannot
prove preservation of invalid or equivalent-looking states. Include opened WB5
baselines without changing the document codec. WB5 is only a fixture source and file
oracle, never the implementation of historical activation.

Cold-load into a private owner, enforce lengths/versions/reference integrity and
existing quotas before publication. Checksums detect accidental damage, not trust;
a structurally valid but malicious length/reference must not cause uncontrolled
allocation or bypass validation. A cold representation must not reparse/rebind or
run Fill's stricter candidate-cycle validator on accepted historical drafts.

Use three separate Stage 1 gates:

1. A → B → evict A → load A → branch C within one live session, retaining B's
   capture. Preserve exact runtime NodeIds as well as durable IDs, bindings, drafts,
   errors and ContentStateId; stale authority rejects and C cannot reuse B-only IDs.
2. Restart in a real new process with no surviving roots. Allocate a fresh runtime
   graph namespace and preserve durable identities, reservations and exact logical
   bound-target relationships. Compare logical equivalence, not old NodeId bits.
   Repeat A/B/C loads using one coherent reopened-session mapping, including after
   unrelated models have already been allocated. Preserve original provenance as
   non-authoritative data and issue fresh session/request authority.
3. Open independent copies simultaneously with overlapping historical logical IDs;
   demonstrate isolated runtime graph namespaces, request authority and captures.

Do not change core NodeId representation or add persistent model-number ownership.
Do not modify ordinary portable transfer to serve as a history codec; preserve its
accepted import/export semantics and build a separate narrow reconstruction seam.
If same-session cold activation cannot preserve runtime IDs without broad redesign,
stop and report before weakening the invariant or expanding storage work. Fresh
restart namespaces do not reset lifetime identity ceilings or discarded-branch
logical reservations. Existing probes do not establish these cold-restoration gates.

## Stage 2 — representation comparison

Stage 1's complete checkpoint is the accepted semantic baseline, not an approved
memory policy. Replace loose fixed-buffer admission where appropriate with checked
bounds tied to actual serialization/reconstruction work. Report encoded bytes,
working-buffer capacities, required and reserved scratch, reconstructed domain
allocations and peak simultaneous owners separately. Retain existing hard limits.
Compare exact identities, bound targets, diagnostics/order, provenance, invalidation
and ledger reservations independently of numerical output equality. Keep the
same-session and restart/copy identity contracts in the representation tests.

Implement the smallest independently verifiable baseline first, then bounded
comparators. These are research encodings over the same logical inventory, not three
production storage providers.

| Comparator | Storage/retrieval experiment | Specific risks to measure |
| --- | --- | --- |
| Complete checkpoint per changed transaction | Indexed independently loadable state; correctness oracle | Bytes per transaction, repeated-definition amplification, sync latency |
| Checkpoint plus canonical record delta | Store changed/deleted stable-key records, bindings and metadata; replay internal data deltas from a checkpoint | Missing coupled edits, tombstones, diagnostics, branch ledger; chain/work bound |
| Immutable objects plus root/commit journal | Store verified content objects and root descriptors with shared references | Unique bytes, object/index overhead, schema coupling, partial object publication, reachable-object collection |

Internal delta application is a trusted data reconstruction step, not ordinary user
command replay. A journal records commit decisions/cursor/ledger for any comparator;
it cannot substitute for a complete state representation. Hash/object references
require integrity verification and bounds, not unchecked deduplication assumptions.

For the delta prototype, compare checkpoint intervals 1, 16 and 64 committed changes
as experimental settings only. Start a new checkpoint sooner if the cumulative
encoded/decoded-work budget would be exceeded. If a single bounded delta cannot fit,
use an independently loadable checkpoint; never silently drop its transaction.
Measure checkpoint scheduling cost, not only delta-only steady state. Bound index
cache/pages and retrieval depth independently of total timeline length. A flat in-RAM
Vec of all transactions is not an acceptable scalability demonstration.

Keep storage mechanics minimal and replaceable. Compare representation outcomes
before recommending SQLite, a framed append log, embedded content or a sidecar.
If a physical API becomes necessary to test flush/recovery, describe its limited
experimental role and guarantees rather than declaring it the production container.

## Workload and measurement matrix

Use existing quotas and both sparse/formula-heavy states. Reuse the accepted cases:
16, 256 and 1,024 live cells; 256 mostly-formula cells; one live cell plus retired
identities up to actual existing admission. Never assume all 8,192 allocations fit
other limits. Long timelines replace existing content rather than increasing the
node ledger beyond current ceilings.
Separately measure repeated create/Undo/branch cycles under the existing lifetime
identity ceilings. Record attempted/committed transactions, new/retired identities,
ledger floors and the first actual limiting admission reason. Compare that result
with fixed-identity 1,000/10,000-edit runs. If finite identity allocation prevents
realistic authoring, report the product constraint; do not claim that disk history
removes it or increase quotas to reach the transaction target.

| Workload | Required history behavior |
| --- | --- |
| 1,000 then 10,000 changed transactions on fixed live cells | More recoverable entries do not require more resident roots/catalog entries; verify oldest/middle/newest reachable states |
| Repeated edits and 256-cell Fill/Paste | One user batch is one transition; include checkpoint and index update costs |
| Text, blank/zero, exact decimal, mixed-dollar/reversed aggregate ranges | Exact content and bindings survive cold restoration |
| Moved/retired targets, invalid/cyclic drafts, original diagnostics | No text reconstruction, ID regeneration or loss of accepted bad drafts |
| Undo cold A, branch C; failed/no-op candidate with Redo present | Committed branch replacement only; no lost future on failure/no-op |
| Capture B, evict/collect its store representation, activate A/C | B remains evaluable; store GC and domain ownership do not assume its memory was freed |
| Prepared Save B, evict/clear B, finish native install while live A | Exact B bytes/checkpoint remain usable; live A stays independently identified |
| Reopen after ledger-heavy branch removal | Previously committed B-only IDs cannot be allocated again |

For each representation/case record:

- Logical transactions retained, reachable branches/cursor and states loaded;
  checkpoint/delta/object/index/metadata bytes, physical live/garbage bytes,
  write amplification and compaction transient space.
- Actual distinct domain allocated/charged/peak bytes, resident-root/cache/index
  counts, transient buffers, cleanup reservations and separate capture ownership.
  Report disk bytes and conservative hot-root weights separately.
- Hot/cold Undo/Redo and commit latency distributions (p50/p95/max), bytes read,
  decoded records, chain depth, index pages visited and validation work. Separate
  write, flush, decode, activation and cleanup time; report cold versus warm OS cache
  honestly rather than assuming a dropped root produces a cold filesystem cache.
- Weight/metadata maintenance visits per edit, no full-history scan; full checkpoint
  work explicitly attributed and bounded by current document size.
- Zero-charge finish after roots/captures/store readers are released, bounded open
  file descriptors and no temporary-file/lease leak.

No latency or compression target is asserted before measurement. For a fixed
checkpoint-work/cache configuration, increasing 1,000 to 10,000 transactions must
not increase resident catalog/root count or linear replay work. Disk index depth
may grow logarithmically and must be measured. Do not call a single-state load bound
independent of document size. Retrieval budgets and defaults return for review with
the data; the 16 MiB research window is not reused as a disk-history retention cap.

## Stage 3 — commit, cursor and restart fault harness

Checkpoint-plus-canonical-record-deltas is the preferred research candidate; complete
checkpoints remain the independent reference/fallback. Start experimentally at interval
16; retain selected interval-64/adaptive comparisons. No threshold or production
container is selected. Reuse full export if needed to isolate durability, with a
separate authoritative transaction-record construction seam for later mutation-local
encoding. Measure full export, delta construction, writes, sync and acknowledgment
separately. Recovery must bind TransactionId, ContentStateId, state, head/cursor/Redo,
lifetime ledger and acknowledged outcome coherently. A valid decision checksum cannot
substitute for durable referenced data: test missing payloads and actual sync ordering,
including directory metadata where relevant. Stop for parent review after this stage.

Use a controlled I/O seam and child-process termination harness. Commit identities
and reachability determine results, not whether the client received an acknowledgment.
Fault injection must exercise the actual research persistence/recovery path; test-local
booleans or mocking out the complete commit are not evidence of atomic recovery.

| Failure/kill point | Recovery invariant |
| --- | --- |
| Partial payload, before references are committed | Old committed timeline intact; incomplete payload unreachable and safely collectible |
| Payload written but not synced; decision absent | No claimed committed transition; old head/cursor/ledger remain authoritative |
| Decision write/flush outcome unknown | Reconcile exact TransactionId; block further mutation until committed/uncommitted is established |
| Decision durable, before live publish/ack | Recover committed new state/branch/ledger; report potentially unacknowledged work; no command retry |
| Undo/Redo cursor decision interrupted | One complete committed cursor, fresh session/activation authority; no half-switched workbook/core |
| Branch switch, baseline reset or GC interrupted | Old or new complete reachable metadata, no deletion of still-reachable state |
| Disk full/permission/I/O error before commit | No partial live edit or premature Redo deletion; distinct actionable failure |
| Truncated/corrupt/unsupported records and broken object references | Reject/quarantine safely with bounded work; never silently skip an undoable action |
| Store reader/capture alive during eviction and collection | Reader safety and capture isolation; bounded deferred cleanup |

Prove process-crash behavior separately from power-loss durability. Record actual
filesystem/flush semantics and limitations, including directory-entry updates where
relevant to the chosen experimental I/O. Never infer network-share guarantees from
native local storage results. Test real store reopen, not reconstruction from a
surviving parent process's in-memory catalog.

## Stage 4 — association, save and retention harness

Keep workbook Save and history commit separate; inject interruption around verified
native installation and its history receipt. Verify the saved checkpoint/content
association against file observations; never equate journal head with on-disk saved
content. The real production Session ticket/dirty integration remains a later gate;
use the existing native adapter and clearly identify what this harness can prove.

Exercise matching reopen, journal newer than saved file, document without store,
store without document, move/rename, Save As, identical-content copy, external
replacement and two processes trying to attach the same history instance. All
association decisions must use more than path/DocumentId. A valid independent workbook
must remain openable if history is absent or mismatched; report history unavailable,
do not merge histories or destroy quarantined data.
Explicitly decline recovery of a newer journal head, reopen the saved workbook
normally, then restart and accept recovery later. Assert that declining did not
delete or truncate the journal and that later activation restores its exact content,
timeline and ledger. Recovery decline and history deletion must be separate decisions.

Exercise explicit retain across close/restart, session-only successful close,
cancelled/failed close, Clear History, relocation and disk unavailable during each.
Clear/reset preserves content, saved marker, path and lifetime ledger; it cannot
invalidate prepared bytes or claim secure erasure. Disk loss must not silently change
policy to no-history. An explicit unretained edit must break the old timeline visibly
and cannot bypass an unresolved commit outcome or uncertain workbook installation.

## Artifacts, platforms and stop gates

Keep new evidence separate from the accepted hot-root manifests. Freeze source and
build inputs, record toolchains/commands/experimental parameters, raw measurements,
semantic comparison results, injected failure positions and child exit/recovery
receipts. Hash evidence at its original execution location; native Mac evidence
must originate on local APFS before copying to the share.

First establish semantic round trips and representation measurements on one local
filesystem, then Windows and Linux fault/reopen tests. Native macOS/APFS storage
validation is required before accepting cross-platform disk-history durability
claims. No recurring manual keyboard checks or desktop/.NET/AOT run is required
for a Rust-only research adapter. Run relevant Rust regression/feature/lint gates
on the actual changed source; later production integration keeps its full normal
three-platform managed/Native AOT requirements. A narrower executed platform set
must be reported as such, not extrapolated.

Stop and return for review on any lost identity/binding/draft, stale authority
acceptance, transaction/ledger disagreement, unbounded history-dependent RAM/replay,
ordinary first edit rejected merely by a full cache, required quota increase,
broad core redesign, inability to reconcile uncertain commits or unsupported
filesystem assumptions. Do not weaken assertions or conceal fallback paths.

Deliver a comparison and explicit recommendation: selected logical representation,
measured retrieval/cache/checkpoint parameters, disk-space/failure behavior,
association/recovery contract and unresolved risks. Container/placement selection
must be separately justified; defer it if the evidence is inconclusive. Stop for
parent feasibility review before production Undo/Redo, session/save/protocol/UI work.
