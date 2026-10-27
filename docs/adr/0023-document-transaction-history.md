# ADR-0023: Tiered recoverable document transaction history

Status: **Four-stage feasibility accepted/closed; G0 approved; G1 ordinary native
Save and feature-only G2 mutation-local authoritative records accepted/closed.
G3 indexed-store feasibility active: first baseline accepted; bounded verified-open
1K/10K result accepted; compact storage and metadata/growth research accepted;
hierarchical authentication/receipt normalization and the owner-authorized dictionary
comparison accepted. Physical-storage accounting and scalable admission proposal are
next, not started. The unmet 20% figure is an unproven aspiration, not a mandatory gate.
G4–G7 and production history integration remain gated (2026-10-27)**.
[Research acceptance and scalability/storage-efficiency assignment](../handoffs/2026-10-27-g3-hierarchy-dictionary-acceptance.md).
[Hierarchical authentication and normalization results](../handoffs/2026-10-27-g3-hierarchy-normalization-results.md).
[Separate trained-dictionary investigation](../handoffs/2026-10-27-g3-dictionary-results.md).
[Layout/growth acceptance and next research boundaries](../handoffs/2026-10-27-g3-layout-growth-acceptance.md).
[Page-layout and authenticated-growth results and remaining limits](../handoffs/2026-10-27-g3-layout-growth-results.md).
[Compact-storage acceptance and metadata/growth scope](../handoffs/2026-10-27-g3-compact-storage-acceptance.md).
[Storage-efficiency results and remaining targets](../handoffs/2026-10-27-g3-storage-efficiency-results.md).
[Revised storage-efficiency targets and compact-comparison scope](../handoffs/2026-10-27-g3-storage-efficiency-reset.md).
[Verified-open acceptance and index-cost research scope](../handoffs/2026-10-27-g3-verified-open-acceptance.md).
[Verified-open design and measured result](../handoffs/2026-10-27-g3-verified-open-results.md).
[First G3 acceptance and bounded verified-open assignment](../handoffs/2026-10-27-g3-first-result-acceptance.md).
[First G3 result and remaining feasibility gates](../handoffs/2026-10-27-g3-indexed-store-first-results.md).
[G2 acceptance and owner scheduling hold](../handoffs/2026-10-24-g2-acceptance-g3-held.md).
[Accepted G2 implementation](../handoffs/2026-10-24-mutation-local-records-g2.md).
[G1 closure and exact G2 authorization](../handoffs/2026-10-24-g1-closure-g2-authorization.md).
[Final promotion validation and closure recommendation](../handoffs/2026-10-24-macos-g1-promotion-validation.md).
[Ordinary Save promotion implementation and remaining gates](../handoffs/2026-10-24-historical-save-g1-promotion.md).
[Internal G1 acceptance and bounded promotion assignment](../handoffs/2026-10-24-historical-save-g1-internal-review.md).
[Mac validation and G1 review readiness](../handoffs/2026-10-24-macos-historical-save-g1-validation.md).
[G1 internal implementation and remaining gates](../handoffs/2026-10-24-historical-native-save-g1.md).
[G0 approval and bounded G1 authorization](../handoffs/2026-10-24-history-readiness-g1-authorization.md).
[Production-readiness proposal: decisions P1–P9 and gates G0–G7](../architecture/history-production-readiness.md).
The proposal recommends a separate validated WB5 projection, not a lower allocation
floor; generic transfer and Redo remain intact. G1/G2 are closed. Bounded feature-only
G3 feasibility has resumed; no production backend/UI
choice or G4–G7 implementation is authorized.
[Final feasibility acceptance and next deliverable](../handoffs/2026-10-23-disk-history-final-feasibility-review.md).
Historical native Save is a mandatory design gate: preserve lifetime floors and
Redo, compare content/ledger separation with a validated export projection, and
return a concrete recommendation/test plan before correction. One readiness packet
must also cover durable indexing/startup, mutation-local records, resources,
filesystem/retention/reclamation contracts and staged three-platform acceptance.
[Architecture approval and experiment boundary](../handoffs/2026-10-23-tiered-history-design-review.md).
[Stage-one inventory and identity-boundary findings](../architecture/cold-history-state-inventory.md).
[Parent inventory acceptance and required next proofs](../handoffs/2026-10-23-cold-history-inventory-review.md).
The inventory/probes are accepted. A separate feature-only complete-checkpoint adapter
now has accepted Stage 1 Windows/Linux cold-restoration evidence:
[implementation](../handoffs/2026-10-23-cold-history-restoration-stage1.md),
[acceptance and Stage 2 qualifications](../handoffs/2026-10-23-cold-history-stage1-review.md).
[Stage 2 representation measurements](../handoffs/2026-10-23-history-representation-stage2.md)
are accepted as research evidence: 100 baseline/final-policy Windows/Linux runs compare full checkpoints, record deltas
and shared objects. Work-scaled admission replaces loose fixed reservations;
checkpoint-plus-record-deltas is the preferred Stage 3 research candidate. Fixed
intervals and the additional adaptive byte/work checkpoint thresholds are experimental.
[Stage 2 acceptance and Stage 3 authorization](../handoffs/2026-10-23-history-representation-stage2-review.md)
require durable decision/restart proofs before file-association or production work.
Preserve an encoder boundary for later mutation-local records; Stage 3 may reuse full
export but must report its cost separately. Loose prototype buffer/ledger reservations are
not approved production defaults. Measure actual work/ownership separately and
retain exact semantic comparisons rather than numerical-output-only checks.
The
[parent identity clarification](../handoffs/2026-10-23-cold-history-identity-review.md)
approves restart remapping while requiring exact same-session runtime IDs.
[Stage 3 decision/restart research](../handoffs/2026-10-23-history-durability-stage3.md)
is accepted as Windows/Linux process-crash consistency evidence.
[Stage 3 acceptance and Stage 4 authorization](../handoffs/2026-10-23-history-durability-stage3-review.md)
require non-destructive recovery and independently verified workbook/history association.
[Stage 4 Windows/Linux/native APFS association research](../handoffs/2026-10-23-history-association-stage4.md)
is accepted within the three tested native filesystem scopes; the complete
feasibility investigation is closed. This feature-only journal is not production packaging.
Full export/reconstruction and linear disk-ID lookup remain explicit costs. Physical
power-loss, general filesystem/cross-platform guarantees and production checkpoint
policy remain unproven. [Mac evidence review](../handoffs/2026-10-23-macos-history-association-validation.md)
records the native APFS scope without approving production integration. New WB5 export from an older historical root with later identity floors
still rejects; the successful late-save proof uses an earlier prepared export.
No production history integration is claimed.
[Superseding parent direction](../handoffs/2026-10-23-undo-redo-feasibility-review.md),
[accepted foundation](../handoffs/2026-10-23-undo-redo-feasibility.md),
[review packet](../handoffs/2026-10-23-undo-redo-design.md),
[bounded disk-history feasibility plan](../architecture/disk-history-feasibility-plan.md).
Predecessors: [document lifecycle](0009-portable-documents-and-local-lifecycle.md),
[accepted Fill](../handoffs/2026-10-23-four-direction-fill-final-review.md).

## Decision proposed

Provide effectively unlimited recoverable Undo/Redo transactions, constrained by
available storage and explicit user retention policy. Rust owns document-history
semantics; the core remains independent of files and storage providers. A bounded
cache of immutable roots accelerates recent Undo/Redo. Older states remain
recoverable from a document-layer history store after resident eviction.

The accepted experiment establishes fast hot-root activation, not a cold-state
representation or restart protocol. Its 32-transition/33-state and 16 MiB limits
are experimental settings, not product retention limits. A full cache never by
itself rejects a valid edit or deletes recoverable history. Existing live-workbook,
identity, allocation-domain, staging and cleanup quotas still apply; this decision
does not promise infinite storage or increase current workbook capacity.

Current workbook content, recoverable history and compiled execution artifacts
are separate. This proposal selects logical contracts and an experiment, not a
container, database, sidecar placement, codec or public storage-plugin framework.
WB5, CB2, private v4 and frozen hosted v1 remain unchanged in this design round.

## Transaction and identity contracts retained

One changed committed authoring transaction creates one history transition: cell
entry (including accepted invalid/cyclic drafts), clear, move/retire where exposed,
Paste, trusted duplication or an entire directional Fill. Preserve the authoritative
mutator's typed change result. Equal displayed values do not establish a no-op;
explicit formula resubmission retains existing rebinding/provenance semantics.
No implicit keystroke coalescing. Failed/no-op/pre-commit-cancelled requests create
no transition and preserve Redo.

| Identity | Contract |
| --- | --- |
| Durable logical cell/node/range UUIDs | Serialized identity preserved across hot/cold restoration and restart, including original bound-target relationships |
| Runtime NodeId | Process-local; preserved exactly within one live session, including cold restoration after eviction; remapped coherently across restart |
| ContentStateId | Identifies committed content, preserved on hot/cold restoration and restart; not a byte hash or saved-file checkpoint |
| TransactionId | Stable identity of one committed transition, with before/after ContentStateIds, parent and fixed action kind; not a request revision |
| HistoryInstanceId and epoch | Scope a store lineage and control/cursor authority; document ancestry or path alone cannot attach it |
| Session/Expected/generation | Fresh authority on every activation; a new session namespace on reopen prevents old requests becoming valid |
| Allocation ledger | Monotonic committed identity floors/reservations across branches, disk eviction and restart; not undoable content |
| Calculation capture | Independent immutable invocation owner; never a history representation |

Changed content gets a fresh ContentStateId. Exact historical activation restores
its original identity but mints new request authority. Returning to A never makes
requests from the original A valid. Historical diagnostic/binding provenance remains
data. No pointers, sessions, tickets or old request handles become authority on load.

After restart, allocate a fresh runtime graph namespace and consistently map durable
identities for all history loaded into that reopened session. Do not assign a new,
conflicting mapping independently per retrieved state. Independent copies may share
historical logical UUIDs within their lineages but must have isolated runtime graph
namespaces and request authority. Referential equivalence, rather than old runtime
bitwise equality, is the cross-process oracle. Preserve historical provenance as data.
Do not change core NodeId representation or add persistent model-number ownership.

The allocation ledger covers canonical node/port IDs, portable cell/node identities
and selection/range identities, including retired/discarded-branch reservations.
Persist durable logical reservations and allocation authority, not old runtime
namespaces as authority. Restart remapping must preserve those reservations even
for objects absent from the current root or reachable timeline.
Committing a new branch cannot reuse B-only identities even after B is evicted or
unreachable. Existing captures of B remain isolated. Failed, definitely uncommitted
candidates change no committed ledger. A committed-but-unacknowledged transaction
retains its ledger advance. Current allocation ceilings are not reset by disk paging,
Clear History or restart of the same attached history instance. Standalone document
imports remain a different lifecycle and must not reconnect old runtime handles.
Unlimited transaction retention is not unlimited identity allocation. Measure
existing-cell edits separately from create/Undo/branch cycles and report finite
identity exhaustion as a distinct product scaling constraint. Do not raise limits
or weaken identity guarantees without separate review.
The inventory measurements are accepted. Identity-space scalability is a separate
architectural prerequisite for removing experimental workbook limits, not a problem
solved by extensible disk history. No identity recycling or rebasing is authorized.

## Hot and cold state boundary

A resident DocumentState owns canonical definitions/index/value roots and workbook
records, positions, blank markers, exact typed literals, draft/source text, bound
references, diagnostics, range identities/geometry and worksheet/axis/graph metadata.
Canonical and workbook halves activate together under fresh guards and current
ledger authority. Reuse the accepted activation and bounded cleanup foundation.

A cold representation must preserve those logical definitions and identity mappings,
including accepted invalid/cyclic drafts, without authoring-command replay, name
resolution, formula rebind, recalculation or invented IDs. Cold loading may decode
and reconstruct private persistent structures and derived indexes; it must verify
integrity, references and resource bounds before publication. Logical bindings and
historical errors cannot be changed by reconstruction. Do not serialize memory
addresses, allocation charges, locks or calculation caches. Restoring native root
layout verbatim is not required. Reconstructing a bounded complete state may be
necessary; unlike a hot switch it is not claimed constant-time.

WB5 is the current-document format, not the history codec. Do not add compatibility
readers or use WB5 Open as a history activation shortcut. Existing portable transfer
is reference material, not assumed sufficient: it does not itself establish preserved
same-session runtime IDs, coherent restart mapping, durable floors or fresh-authority
cold restoration. Stop if a narrow
validated cold adapter requires broad graph redesign or cannot fit existing quotas.
In particular, report inability to preserve exact same-session runtime NodeIds
after cold eviction before weakening that contract. Test same-session restoration,
fresh-process remapping and simultaneous copies separately.

## Resident cache and retrieval

Store content/transition IDs in a disk-addressable index; do not retain the full
transaction catalog, every checkpoint or every root in RAM. Cache size, index pages,
decoder buffers, prefetch, candidate roots and cleanup tickets are bounded. Root
weights can guide cache admission conservatively, but do not measure disk retention
or actual distinct allocation. Shared allocations remain charged once by the domain.

Before detaching a resident representation, verify that its canonical state is
recoverable from a committed checkpoint/delta chain. A newly durable candidate may
be committed with no resident history pin if needed. Plan eviction before staging
where safe, but do not assume that captures or deferred cleanup have freed memory.
Existing hard live-memory failure still rejects atomically, with a distinct reason
from disk/cache retention. Eviction changes no logical cursor, saved marker or Redo.

Cold retrieval has explicit byte/work/depth bounds independent of total transaction
count. Checkpoint scheduling must bound chain depth and decoded bytes; the experiment
will select measured settings, not a product Undo count. Reject an oversized or corrupt
record before unbounded allocation. If restoration cannot fit existing staging limits,
leave the current document/cursor intact and return for architecture review.

## Logical timeline and durable commit

Expose one linear Undo/Redo cursor. A changed edit after Undo replaces the reachable
Redo branch only when that edit commits. Failed/no-op edits preserve the branch.
Discarded futures cease to be user-reachable but may remain physical garbage until
safe collection; no branch browser or permanent revision archive is promised.
Undo/Redo activates the exact predecessor/successor; it never re-executes its command.
Empty Undo/Redo is a validated no-op. Presentation state and captures do not move.

Serialize session mutations and give each prepared transaction a durable identity.
Proposed sequence for a history-retaining operation:

1. Validate exact authority; privately stage content, before/after identities,
   ledger, branch/cursor change, resource ownership and bounded cold records.
2. Persist all referenced payload, then a verifiable commit decision containing
   the new head/cursor/ledger. Until this decision, new records are unreachable.
3. Once the store confirms the decision, publish the already prepared live state
   and status without further fallible semantic work; then acknowledge its ID.

A cursor move without changed content also commits its cursor/ledger metadata for
restart consistency before publishing fresh live authority. Reads/evaluation create
no journal transactions. A failed prepare or definitely uncommitted write preserves
live state/history. Cancellation before the durable decision aborts; cancellation
after it must drain/reconcile the committed result, never replay or pretend rollback.

An ambiguous write/flush result is not an ordinary failure. Freeze further mutation
of that history instance, retain current visible state and reconcile the transaction
by ID before claiming success/failure or offering a destructive bypass. Recovery
may find a durable transaction that was never acknowledged; surface it as recovered
work, not an automatic retry. Exact durability primitives/platform guarantees are a
feasibility gate; do not claim power-loss guarantees from a successful process test.

## Saved content, recovery and file authority

Clean means current ContentStateId equals the last verified saved ContentStateId,
with no installation uncertainty. An untouched new document has its clean baseline
without a file association. Saved markers/checkpoints do not pin history roots.
Returning to the exact saved identity can be clean under fresh guards; independently
recreating equal values does not make it the saved state.

PrepareSave owns immutable bytes, captured content identity/checkpoint and existing
ticket/location authority independently of resident/disk history retention. Completing
Save S records S, not current T. Evicting/clearing S does not invalidate a valid ticket.
Keep one-ticket bounds and existing no-replay rules. Save As/path/fingerprints are
not undoable. Undo/Redo/Clear cannot clear uncertain file installation or restore an
old filesystem observation. Failed Save preserves its prior successful marker.

A history commit does not Save the workbook. Conversely, verified workbook installation
and history metadata acknowledgment are not assumed one atomic filesystem transaction.
Persist a save-intent/receipt association between checkpoint, ContentStateId and verified
file identity as needed; recovery must reconcile a crash between these operations.
Never mark clean solely because the journal says Save succeeded. If file observation
and checkpoint cannot prove the saved baseline, keep recovery detached/uncertain and
require existing conflict/reopen handling. History I/O uncertainty is distinct from
workbook installation uncertainty; neither resolves the other.

On open, validate current document bytes independently before considering history.
Attach only after checking an explicit association: history instance/lineage, exact
saved checkpoint/content correspondence and applicable file observation. DocumentId,
path and numeric state counter alone are insufficient. Current WB5 does not acquire
new metadata in this round; proving an external association is part of feasibility.

| Situation | Proposed behavior |
| --- | --- |
| Matching retained history and saved baseline | Open the saved content clean with its proven identity; restore reachable history/cursor consistently |
| Journal head newer than saved file | Offer explicit recovered unsaved work; do not silently replace the workbook's normal saved-open content |
| Workbook copied without history | Open a valid standalone baseline, report unavailable prior history; never fetch an unrelated store by shared DocumentId |
| Workbook moved/renamed | Offer verified reassociation; preserve history only on a proven match, never by filename |
| Save As | Keep the live instance; commit the new association only after verified file installation; old copy cannot share a writable history instance automatically |
| External replacement or mismatched history | Open/retain the valid workbook independently; quarantine/detach mismatched history without deleting it |
| Two independent copies/windows | Exclusive writer per history instance; independent history instances or explicit read-only access, no merge or last-writer-wins attachment |
| Failed/cancelled Open/New/close | Preserve the existing live session, history and pending authority |

Declining immediate recovery does not authorize deletion of the recoverable journal.
Preserve it for later explicit recovery or separately authorized discard; prove that
decline followed by restart and later recovery retains exact content and history.

A clean close may retain disk history; closing releases live roots, not necessarily
the store. Recovery/open issues fresh session authority; persisted revisions never
resurrect handles from a previous process. Association, leases and save reconciliation
require explicit executable evidence before across-session history is advertised.

## Retention and storage failures

Support explicit policies for retaining history across sessions and for session-only
disk history removed on a successfully completed close. Choose the default and
placement after review; the application must disclose the selected policy. Save
alone does not discard history. Crash-orphan cleanup must not silently erase retained
or potentially recoverable work. History contains deleted cell data; disclose that
retention consequence. No automatic age/count/size deletion is proposed.

Clear History requires an explicit destructive decision: establish a durable baseline
for current content and ledger, then remove reachable Undo/Redo metadata atomically.
It is not undoable and changes history-control epoch, not content, mutation revision,
saved marker, dirty state or path. Keep the allocation ledger and any save-ticket
ownership. Physical reclamation follows reachability and active reader pins; this
is not secure erasure. Failure to establish the baseline preserves old history.

| Condition | Required default and explicit recovery choices |
| --- | --- |
| Cache full | Evict recoverable resident state; preserve logical history |
| Actual domain/cleanup capacity exhausted | Reject before commit without partial state change; no quota increase |
| Disk full/unavailable before commit | Preserve content/history/Redo; offer retry/free space, relocation or explicit clear after a safe baseline can be established |
| User chooses an unretained operation | After reconciliation, explicitly end/detach the old timeline and establish a new baseline; never leave Undo implying continuity across the missing operation |
| Commit outcome uncertain | Suspend mutation and reconcile by transaction ID; no automatic retry, truncation or bypass |
| Corruption/incompatible history | Fail the affected history load safely; retain valid current content and evidence; offer deliberate detach/recovery, not silent fallback |
| Retained history relocation | Copy/verify committed reachable content and ledger, switch association atomically, keep old data until confirmed; failure leaves original authoritative |

Continuing without history cannot evade existing edit-memory or uncertain-file
safeguards. It requires explicit disclosure of unavailable Undo across the boundary;
if no safe baseline/decision can be recorded, close the old instance and start a
fresh explicit standalone baseline rather than claiming durable continuity.

Disk garbage collection may delete only records proven unreachable from current
retained timeline, recovery/save metadata and active store readers. It must never
affect resident calculation captures or rely on their memory being released. Compact
physical storage without deleting logical transactions; destructive history compaction
is a separate explicit choice. Bound work and transient disk overhead, including
failure when insufficient space exists to compact.

## Mode, editor and protocol boundaries

Ordinary workbook history remains the first scope. External publications, captures,
producer registries and presentation/selection/focus changes are not undoable content.
Proof-mode entry cannot silently clear retained history: require explicit history
boundary consent or preserve the ordinary timeline detached and unavailable while
in proof mode. Failed entry preserves it. The specific UI is deferred. Future permanent
structured documents need a separately reviewed partitioned history contract.

Keep editor-local undo local; disable document history during drafts/drag/pending
lifecycle work. Activation preserves current logical selection/viewport and invalidates
late authoring/viewport/clipboard callbacks with fresh guards. Calculation captures
retain their own IDs/contexts. No implicit draft commit/discard on Undo.

Rust will own coherent cursor/status/action labels. History controls require both
current document authority and a fresh history epoch; Clear may change only the epoch.
Do not allocate protocol opcodes/fields or choose count widths in this revision.
Do not inherit old fixed-count status assumptions; disk-index queries must remain
bounded. Private protocol compatibility/version cuts need later review and actual
older-peer evidence. Desktop shortcuts/menu integration remains deferred.

## Alternatives and next gate

| Candidate | Benefit to investigate | Main correctness/cost question |
| --- | --- | --- |
| Independent logical checkpoint per transaction | Simple bounded retrieval and oracle | Duplicate payload/disk amplification and durable write latency |
| Checkpoints plus canonical record deltas | Small local edits; bounded chains via checkpoints | Complete delta semantics, indices, drafts, ledger and atomic mixed state |
| Content-addressed persistent-state objects plus root journal | Reuse immutable state/subtrees | Object identity, schema coupling, bounded indexing/GC and integrity |
| Journal of ordinary authoring commands | Superficially compact | Rejected: replay/rebinding and changed semantics cannot restore authoritative history |

Journaling is a commit/recovery mechanism usable with any accepted representation,
not itself proof of content correctness. Evaluate physical containers only after
logical needs and workload measurements; no SQLite versus bespoke format decision
is made here. The experiment must include a simple checkpoint baseline and compare
local-delta and shared-object approaches before selecting one.

The parent authorizes the bounded plan linked above with the three safeguards in
the approval receipt. No disk experiment is claimed executed. Execute its staged
gates; after experimentation, stop with evidence before production container selection,
a production coordinator, saves,
session/protocol or desktop work. Preserve the accepted hot-root tests and manifests.

Optimize Worksheet remains separate future design: compiled/fused execution and
possible SIMD must preserve exact Decimal96 rounding, overflow, error and dependency
semantics. Canonical computational definitions remain authoritative. Compilation does
not imply history deletion; destructive compaction requires explicit user approval.
No JIT, SIMD, format migration or optimization implementation is included.
