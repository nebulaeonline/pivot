# ADR-0023 production-readiness architecture packet

Date: 2026-10-24. **G0 approved; G1 and feature-only G2 accepted/closed.
G3 first baseline accepted; bounded verified-open 1K/10K research accepted; compact storage accepted; metadata/growth research
is accepted; hierarchical authentication/receipt normalization and owner-authorized
dictionary research are accepted. Physical accounting and scalable admission proposal
are next, not started. G4–G7 and production Undo/Redo remain gated.**
[Acceptance, revised storage objective and next gate](../handoffs/2026-10-27-g3-hierarchy-dictionary-acceptance.md).
Retain the separate-table normalized hierarchy, rowid payloads and zlib 6 as the
experimental baseline. The 20% aspiration is not a mandatory gate pending defensible
minimum-storage accounting. Return with physical accounting, an explicit admission
proposal and the original-G3 readiness comparison; no full 100K G2 authorization.
[Hierarchical authentication and normalization results](../handoffs/2026-10-27-g3-hierarchy-normalization-results.md):
12.52–12.55 MB at 10K, bounded affected-path updates without retained superseded nodes,
exact canonical receipt restoration, 249 passes/three skips across native targets.
Broader overhead remains 57.8–58.2%; no production backend adoption or full 100K gate.
[Separate trained-dictionary results](../handoffs/2026-10-27-g3-dictionary-results.md):
fixed external dictionaries verified across all targets; 18 additional native codec
tests pass. Logical payload savings do not solve metadata/page overhead. Production
codec selection remains open; the original zlib evidence is unchanged.
[Page-layout and authenticated-growth results](../handoffs/2026-10-27-g3-layout-growth-results.md):
about 13.6 MB at 10K, atomic splitting beyond 256 buckets and 147 test passes/three
skips across three native targets. The 20% metadata target remains unmet; flat
directory bounds and linear costs preclude claiming permanent scale feasibility.
[Acceptance and hierarchical authentication/receipt-normalization assignment](../handoffs/2026-10-27-g3-layout-growth-acceptance.md).
The authorized research is now complete as linked above. Rowid payload storage and
all authoritative semantics remain; retained allocation and verification strength
are reported alongside logical writes. Full 100K G2 work remains unauthorized.
[Compact-storage acceptance and metadata/growth scope](../handoffs/2026-10-27-g3-compact-storage-acceptance.md).
[Storage-efficiency results and remaining targets](../handoffs/2026-10-27-g3-storage-efficiency-results.md).
[Revised storage-efficiency targets and compact-comparison scope](../handoffs/2026-10-27-g3-storage-efficiency-reset.md).
[Verified-open acceptance and index-cost research scope](../handoffs/2026-10-27-g3-verified-open-acceptance.md).
[Verified-open research design](history-verified-open-experiment.md) and
[1K/10K result with final-source validation](../handoffs/2026-10-27-g3-verified-open-results.md).
[First G3 acceptance and bounded verified-open assignment](../handoffs/2026-10-27-g3-first-result-acceptance.md).
[First G3 result](../handoffs/2026-10-27-g3-indexed-store-first-results.md) and
[research design](history-indexed-store-experiment.md). Historical held-G3 notes
below are superseded by the 2026-10-27 infrastructure closure and resumption.
[G2 acceptance and owner scheduling hold](../handoffs/2026-10-24-g2-acceptance-g3-held.md).
[G2 implementation, independent oracle and measurement evidence](../handoffs/2026-10-24-mutation-local-records-g2.md).
[G1 closure, reliability qualifications and exact G2 scope](../handoffs/2026-10-24-g1-closure-g2-authorization.md).
[Final ordinary Save validation and closure recommendation](../handoffs/2026-10-24-macos-g1-promotion-validation.md).
[Internal G1 acceptance and ordinary Save promotion scope](../handoffs/2026-10-24-historical-save-g1-internal-review.md).
[Parent approval, clarifications and G1 promotion gate](../handoffs/2026-10-24-history-readiness-g1-authorization.md).
[Assignment and feasibility acceptance](../handoffs/2026-10-23-disk-history-final-feasibility-review.md),
[ADR-0023](../adr/0023-document-transaction-history.md),
[Stage 4 evidence](../handoffs/2026-10-23-history-association-stage4.md),
[native APFS evidence](../handoffs/2026-10-23-macos-history-association-validation.md).

[G1 implementation, exact experimental boundary and validation](../handoffs/2026-10-24-historical-native-save-g1.md).
[Ordinary Save promotion](../handoffs/2026-10-24-historical-save-g1-promotion.md)
now routes `Document::export_document` directly through validated current-content
projection in default builds, with no fallback from strict transfer. The separate
experimental historical exporter was removed; the real Rust session and research
association call the same ordinary exporter. Generic full-ledger transfer remains
strict. Windows/Linux/Mac full gates pass with source-matched managed/desktop/AOT
validation; the parent accepts promotion and closes G1. No production history integration.
[Mac evidence reconciliation and three-platform review readiness](../handoffs/2026-10-24-macos-historical-save-g1-validation.md)
record 207 native cases, 2,652 original-file hashes and matching 168-input source.
Internal G1 acceptance is recorded under the linked review; ordinary Save promotion
is implemented with final three-platform managed/session/desktop/AOT gates passed.
The final promotion source has 236 matched inputs, 207 native cases and 2,652 hashes;
the earlier 168-input internal correction remains separately accepted evidence.

## Approved direction and decisions still requiring approval

First resolve historical native Save with an exhaustive, validated **current-root
document projection into existing WB5 slots**. Keep lifetime allocation authority
separate and generic full-ledger transfer strict. Save changes saved-file authority,
not the history cursor, Redo or document content.

Then produce authoritative records during private mutation and replace linear history
scans with a durable bounded-memory index. Evaluate SQLite before engineering a custom
index, without choosing a production container now. A separately placed acknowledgment
witness is needed for conditional detection of lost committed suffixes; an index in
the same rolled-back store cannot supply that evidence.

| Decision | Recommendation | Approval/evidence still required |
| --- | --- | --- |
| P1 | Validated current-root projection; existing WB5 | Historical Save tests below; no assumed format change or weaker transfer validator |
| P2 | Complete ledger stays with the live/attached history instance | Standalone WB5 is a fresh authority, never a replacement allocation ledger for old history |
| P3 | Evaluate SQLite first, custom indexed log second | Backend, Rust dependency, VFS/configuration and packaging remain undecided |
| P4 | Publish decision, receipt index, cursor and ledger atomically | Bounded verified startup/recovery; no ordinary full-history lookup scan |
| P5 | Mutation-local typed authoritative records | Completeness against full-image oracles; no full-root diff presented as incremental |
| P6 | Separate memory pools and work-bounded checkpoints | Numeric cache/scratch/checkpoint defaults require measured approval |
| P7 | Local process-crash contract and conditional suffix-loss detection | Independent witness placement/trust, sync/ack cost and platform contract |
| P8 | Explicit retention choices and measured reclamation | Product default, disk budgets, deletion/privacy and discovery policies |
| P9 | Independent implementation gates | No session/protocol/desktop/container implementation during this design phase |

Checkpoint-plus-canonical-record-deltas remains the leading logical representation.
No hosted database/provider framework, identity recycling, limit increase, WB
migration, Optimize Worksheet, JIT/SIMD or unrelated reliability work is proposed.

Component boundary: the core produces canonical definitions/changes without filesystem
or database dependencies. The workbook pairs them with stable document identities and
its own root. The document layer constructs/validates WB5. An outer Rust history owner
manages storage/index, witness, retention and native-file association. Clients never
resolve engine formulas through mutable display names or access backend internals.
No backend dependency or general history coordinator is introduced into the core.


## 1. Historical-state native Save

### Source boundary and alternatives

The current path is `Document::export_document` → `Workbook::export_portable` →
selection materialization → `transfer::export_image`. The workbook supplies identities
in the selected root's bindings directory. Historical activation retains later
allocation floors; the materialized GraphImage carries those floors. The strict helper
requires `nodes.len() == next_node` and corresponding port coverage. Its rejection
is correct for a full-ledger request. After branching, runtime ordinals can also be
sparse: changing a count alone cannot fix the mapping.

Source evidence: [workbook transfer](../../crates/pivot-workbook/src/transfer.rs),
[materialization](../../crates/pivot-core/src/model/structural/materialization.rs),
[strict transfer](../../crates/pivot-core/src/model/transfer.rs),
[historical floors](../../crates/pivot-core/src/model/structural/history.rs).
The existing snapshot exporter is not an unmodified workaround: its capability and
ID assumptions must not silently become a historical-document export contract.

| Authority | Meaning | Effect of Save B |
| --- | --- | --- |
| P: content B | Paired workbook/canonical root, current identity directory, definitions/bindings/provenance and ContentStateId B | Project exactly this snapshot |
| L: lifetime | All committed reservations/floors, including later/discarded D-only identities | Remain monotonic in the same history instance |
| G: request/revision | Fresh mutation/session authority after activation/restart | Validate capture without advancing live Expected; never substitute G for B |
| Saved checkpoint/file observation | Actual verified WB5 installation | Update only after the exact captured bytes/checkpoint are installed |

| Alternative | Assessment |
| --- | --- |
| Put current content and full L into a revised native persistence model | Possible longer-term option, but couples standalone documents to history lineage and may require a separate format decision. Not necessary to prove Save of visible B. |
| Strict current-root document projection into existing slots | Recommended: preserve UUIDs and content, translate serialization coordinates, retain L outside the projection. |
| Supply all D-only reservations to the existing exporter | Not a general fix: B need not have workbook records for them. Adding future identities to B's directory can alter its content semantics. |
| Lower floors, fabricate active placeholders, replay/reparse or write cold-history bytes as WB5 | Rejected. Violates identity, draft/binding or native-file contracts. |

### Proposed projection algorithm

Add a narrow internal core/workbook projection seam and route native export through
it. Keep generic full-ledger transfer unchanged. The core remains independent of WB5
and filesystem APIs; cross-crate projection types should be sealed/internal rather
than a general arbitrary-subset export/provider API.

1. Capture a coherent immutable P and ContentStateId B under the expected guard.
   Pin the two authoritative halves and their mapping. Do not look up names, positions
   or definitions in the later editable model while encoding.
2. Enumerate all identities already in P's workbook bindings directory, including
   its genuine retired entries. Preserve CellId/GraphNodeId UUIDs and every live
   canonical definition. Do not restrict export to an evaluated dependency closure.
   D-only reservations absent from P are not document content.
3. Build a bijection from those runtime IDs to deterministic portable-UUID-ordered
   wire slots. Translate scalar targets, selection/aggregate formulas, points and
   blank markers through it. Range endpoints/members and any supported port/selection
   metadata must use the same verified mapping. Slots are encoding coordinates, not rebased durable IDs
   or rewritten live NodeIds. Do not edit any source allocator or materialized floor.
4. Preserve real retired slots as holes without active definitions. Preserve exact
   Decimal96 parts/text, rejected/cyclic drafts, source/provenance and dollar/range
   semantics. Do not evaluate or reparse to reconstruct definitions. Required targets
   must already have authoritative identities; missing mappings are integrity errors,
   not permission to create a node or copy a newer D definition. Existing blank-marker
   encoding is used only for genuine blanks, never as a D-only placeholder.
5. Validate exhaustive live coverage, workbook/core correspondence, retirement,
   reference/selection coverage, ordering, uniqueness and existing quotas. Accept
   existing draft semantics; Fill's stricter cycle rejection is not a Save rule.
6. Construct a logical selection image with its own slot extent N, then use existing
   image/native validation and encoding. Do not build a fake mutable WorkingModel
   whose floor has been reduced to pass the old full-ledger helper.
7. Capture an ordinary checkpoint and immutable WB5 bytes bound to B. Verified
   installation updates saved-file authority only. Cursor, Redo and L are unchanged,
   including when the ticket completes after a newer edit or Clear.

This is exhaustive document projection, not relaxed subset transfer. Current cold
validation already requires cell-reference coverage in the root bindings directory.
A supposedly legitimate root violating that invariant requires investigation, not
invented content. Current unsupported structured/captured-source native capabilities
remain unsupported; this correction does not broaden them.

WB5 already represents local slots, retired identities and portable UUIDs separately
from an original runtime namespace. Recommend **no format cut**, conditional on exact
representability and existing-reader validation. If a legitimate state cannot be
represented, stop for a separately approved format decision. No WB4 reader/migration.

### Ledger, reopen and save semantics

Saving undone B must not discard Redo D or lower L. Reopening the original verified
history uses its complete ledger and coherent fresh-process mapping. Clear and file
Save cannot grant permission to reuse previously committed identities.

Standalone WB5 Open remains a fresh document/history authority under the independent
file contract. It preserves UUIDs present in the file, but cannot recover omitted
D-only reservations or assert an old timeline. Fresh runtime/session authority rejects
stale requests even if numeric NodeId bits coincide across processes. Attaching old
history still requires its verified lineage and full L; file contents cannot replace
that ledger. This is not a promise that a standalone workbook contains discarded history.

Constructing the Save candidate must not mutate B, live Expected, original runtime
IDs, L, cursor/Redo or independent calculation captures. The existing WB5 reader must
reconstruct equivalent durable relationships, not merely accept the encoded image.

A new Save takes B's identity from its captured content, never from the live model at
completion. Fresh native checkpoint/revision metadata are not ContentStateId. Save
metadata can need its own durable control decision but creates no document Undo step.
Recovery compares verified saved content with the committed cursor/head, not just
sequence timestamps: a late-installed historical B can have a newer save receipt
while different live A remains unsaved.

### Adversarial acceptance before correction implementation

These are proposed tests, not newly executed evidence. Compare exact typed content,
IDs, bindings, diagnostics/order and ledger as well as native bytes/checkpoints.

| Sequence or failure | Required outcome |
| --- | --- |
| A → B → D-only allocations → Undo B → initiate a new Save | B exports/opens as WB5; D stays Redo; source floors/runtime IDs unchanged |
| Same sequence with cold eviction and real restart before Save | Same logical B/full L, fresh guards, coherent mapping; no authoring replay |
| Undo A, branch C at sparse high ordinals, Save C | Valid wire slots and unchanged source UUIDs/IDs; no live rebasing |
| Retired referenced target versus later-only unreferenced reservation | Genuine retired hole retained; no D-only content inserted into B |
| Equal-valued distinct targets; invalid/cyclic/rejected formulas; text; exact signed-zero/scale; reversed mixed-dollar aggregates | Preserve definitions/provenance/bindings/order without evaluation substitution |
| Prepare B, then edit D/Undo A/Clear, then install B | Exact original B/checkpoint; live A independently dirty; ticket survives |
| Save B, Redo D, Undo B | No lost Redo, extra Undo step or saved-marker confusion |
| Cancellation, admission/codec failure, stale guard, inconsistent mapping | No root, ledger, Redo, file or saved-receipt mutation |
| Partial/uncertain install or lost receipt | Native uncertainty/reopen safeguards remain; journal cannot declare file success |
| Independent open/copies versus attachment of original history | Fresh authorities for copies; only verified original attachment restores L |
| Foreign/duplicate/missing IDs, extra live node or missing range point | Projection rejects before publication; quotas and generic transfer stay strict |
| Independent captures alive across preparation, Save and subsequent activation | Capture definitions/results/authority remain isolated; completed owners finish at zero charge |
| Existing full-ledger historical-floor negative test | Still rejects that generic request; add a separate native-Save positive proof |

Preserve Stage 4's original failed-export source/artifacts. Update the current native
regression to prove the correction only after approval; never reinterpret old negative
evidence as a successful new export.

## 2. Durable indexed lookup and verified startup

One atomic publication must bind operation identity/intent, content payload/root,
parent/head/cursor/Redo, full ledger root, receipt index and changed recovery/save/
lifecycle authority. Referenced payloads must be committed before the publication
root can claim them. A checksum-valid decision alone is insufficient.

| Logical key | Persisted indexed information |
| --- | --- |
| Instance + epoch + TransactionId | Intent digest, immutable outcome/receipt locator |
| Commit/control sequence | Decision digest, prior link, publication root |
| Content/root ID | Payload/base location, parent, checkpoint/decode-work bounds |
| Current authority | Cursor/head/Redo, ledger, save/recovery metadata and fence |
| Stable identity | Monotonic committed reservation, never undoable content |
| Retention/lifecycle root | Epoch retirement, reachability and reader fences |

Normal lookup targets logarithmic page access with a fixed resident page cache, not
an all-history RAM map or scan on every edit. Cursor/Redo traversal needs direct links.
Do not commit decisions first and call a missing optional index update “uncommitted.”
An inconsistent required index is an integrity/unknown outcome, not permission to replay.

Startup verifies instance/epoch, selected index/metadata roots, publication/witness
fences and a bounded recovery tail. Payloads validate before activation; historical
payload auditing can be lazy. Full-history integrity audit/index rebuild is explicit
maintenance, not ordinary startup/reconciliation. A corrupt older state cannot change
live content when its load fails. A high-water counter alone is not proof of receipt
index completeness: the approved backend must establish coverage through atomic
indexed decisions and verified roots/integrity rules. If negative lookup cannot be
trusted without a full scan, return unknown or fail the design gate, not uncommitted.

Reconciliation has committed, definitely-uncommitted and unknown outcomes. Committed
retry verifies identical intent and returns the same receipt without parsing/rebinding/
executing again. Index absence is negative evidence only at a verified current horizon
with the recovery tail resolved and no matching in-flight publication. Unknown blocks
mutation. Proposed negative rule: before releasing a post-uncertainty barrier with a
final uncommitted result, atomically index that operation ID as terminal under the
current epoch/fence. A delayed original request then cannot commit after reconciliation.
This is a control outcome, not a document edit. Pre-admission rejection is distinct
from registering a retained operation; protocol wording needs its own review at G6.
Retired/pruned receipts report stale/not-retained, never fabricated uncommitted status. Clear cannot bypass unknown operations. Prepared
native-save tickets retain their separate authority across history epochs.

Indexed/witness adversarial requirements include: crash between payload and index/root
publication; index/root disagreement; stale reader horizon during status query; a
late request after terminal negative reconciliation; wrong instance/epoch; pruned
receipt lookup; store ahead of witness; witness ahead of store; witness update failure
after known commit; and rollback of both. The last must be reported outside detection
unless a further trusted anchor survives, not passed as a guaranteed detection case.

| Candidate | Benefit | Obligation |
| --- | --- | --- |
| SQLite transactional indexed store | Reuse database indexing/recovery and atomic payload/receipt/head/ledger updates | Pin/test engine, VFS/configuration, cache/WAL/read bounds, startup, quotas, backup and witness |
| Payload log + copy-on-write disk index + manifest | Direct record/I/O control | Engineer/review index pages, atomic roots, bounded tail recovery, torn writes, GC and relocation |
| Existing stream + RAM index rebuilt by scanning | Simple prototype | Rejected as normal production startup/admission; preserves the scaling problem |

Recommend **evaluating SQLite first**, not selecting a dependency or container.
Use one exclusive history writer and bounded short read transactions; materialized
calculations/tickets must not hold database readers indefinitely. SQLite documents
that readers can prevent completion of WAL checkpoints, so application backpressure
and read-lease bounds require tests. [SQLite WAL documentation](https://www.sqlite.org/wal.html).

WAL with `synchronous=FULL` is the initial configuration candidate: SQLite documents
an additional per-commit WAL sync compared with NORMAL. Pin/review the actual engine,
VFS and platform behavior before approval. [SQLite synchronous documentation](https://www.sqlite.org/pragma.html#pragma_synchronous).
SQLite still depends on storage/filesystem assumptions and cannot detect joint rollback
of itself and a colocated witness. [Atomic-commit assumptions](https://www.sqlite.org/atomiccommit.html).

Database WAL checkpoints and logical history checkpoints are different operations.
Bound pending WAL/tail work and reader duration; apply explicit backpressure rather
than discard history when a reader prevents progress. Measure complete verified
startup, not just database-open time.

After approval, compare 1,000/10,000/100,000 fixed-identity transactions, broad Fill/Paste,
indexed cold queries and fault/restart. Reserve test disk first. Report page visits,
bytes, cache/RSS/OS-cache distinctions, tail work, sync/ack latency and physical versus
logical storage. If SQLite misses reviewed bounds, return with evidence before building
the custom alternative. No dual production backend/provider abstraction is presumed.

## 3. Mutation-local authoritative record production

The feature-only G2 implementation extends the private candidate path with owned `PreparedRecordChanges`
alongside paired canonical/workbook roots. Use resolved stable identities, not client
names or commands to replay. Starting seams are the workbook's
[Transaction/apply_mode](../../crates/pivot-workbook/src/persistent.rs) and core
[Batch](../../crates/pivot-core/src/model/integration/batch.rs).

G2 records mutation keys at the canonical and workbook seams, then reads original/
final values only for those keys and fixed metadata. The source must have a validated
ledger/content pairing; an ephemeral marker established by existing observation or
cold validation avoids a full source-directory scan in the local edit path. Guarded
feature-only in-memory publication advances only new reservations and the marker.
It creates no persisted decision, journal entry, durable witness or production
coordinator. Whole-state oracle work is separate; a local 48-step chain requires no
full observation between edits. Existing persisted reconstruction remains unchanged.

Track semantic changes at mutation primitives, coalesce by stable key within the
batch and seal after normal candidate validation. Include typed definition/payload,
bindings, positions, blank/tombstone status, selection descriptors/points, provenance,
diagnostic ordering, invalidation, ContentStateId and newly committed reservations.
Retirement never removes a lifetime reservation. A whole rectangular edit is one
candidate/decision. A no-op means unchanged authoritative semantics, not equal values.

Existing touched cell/position sets are not a complete history delta. Core bindings,
selection data and ledger changes need coverage. Do not enumerate/export both full
roots or calculate a full-root diff for an ordinary small edit. Dependency/range work
can legitimately affect many records; an ordered aggregate record may initially be
replaced in full. Charge/count that work, not a claim that every edit is O(1).

Persist typed/versioned logical records, not Arc addresses, collection layouts or
cached calculations. Decimal96 parts stay exact. Future IEEE values need separately
approved semantics/encoding, not conversion through decimal or assumed WB5 support.

Future production sequence (not persisted by G2): validate private candidate once → seal record/reservation changes
→ admit buffers/index/write capacity → persist atomic decision/index → establish receipt
eligibility → publish paired roots/fresh guard → acknowledge. Pre-commit failure preserves
old content/Redo; uncertainty blocks continuation. Existing allocator behavior does not
authorize recycling IDs. A required broader allocator redesign returns for review.

Keep the full checkpoint encoder as an independent oracle. Apply produced changes to
a separate prior record image and compare the complete authoritative result, including
drafts, bindings/order and L. Differential/property sequences include failed/no-op and
cancelled batches, branch replacement, rename/retire, blanks, copy/Fill/Paste and cold
roots. Only after these proofs remove repeated cold reconstruction from normal commit;
retain it for tests, audits and cold activation validation.

Expected benefit is work proportional to changed authoritative records plus actual
semantic dependency/range work. It is a hypothesis to measure: report full-oracle
export separately and do not infer incremental CPU savings from smaller disk deltas.

## 4. Resource, filesystem and lifecycle contracts

### Memory and checkpoint work

Use separate pools/owners for live roots/calculations, hot pins, index pages, encoding,
decoding/reconstruction, native tickets and deferred cleanup. Report actual capacities,
distinct allocations, conservative scratch and simultaneous owners separately from RSS,
OS cache and disk bytes. Existing document/identity limits are independent of cache weight.

Recent roots are an evictable soft cache, not disk-retention limits. Evict/flush cleanup
before rejecting staging merely because the cache is full. Captures/tickets keep their
own pins; eviction cannot pretend they were freed. A genuine hard admission failure
must be actionable and precede publication, not silently lose Redo or skip retention.
The policy must support a complete checkpoint for admitted supported models or reject
enabling that policy before promising history. No quota increase is assumed.

Checkpoint scheduling bounds chain edges, cumulative decoded bytes, record/index work
and peak scratch; checked readers enforce the same limits. Oversized deltas require
an independently loadable checkpoint for that transaction, not a dropped edit. Include
checkpoint construction/transient disk in admission. Streaming private staging may be
needed. The 16/64 intervals, 64 KiB and 256-record research values remain comparators,
not product defaults. Return measured budgets, including broad-edit amplification.

History length must not grow resident catalogs or turn ordinary retrieval into replay
of all history. Finite lifetime allocation ceilings remain separate; the former
32-transition/16 MiB prototype settings are not retention caps. No rebasing/recycling.

### Durability and acknowledgment witness

Recommend an initial reviewed-local-filesystem process-crash contract, with conditional
suffix-loss detection **when an independently placed witness survives**. Do not promise
general physical power-loss durability. Placement/trust require parent approval.

Propose a private local registry excluded from routine per-history copy/restore,
binding instance/epoch, eligible sequence, TransactionId and decision/root digest.
Separate placement does not imply independent physical failure domains. Coordinated
rollback/loss of store and witness remains undetectable without another trusted anchor.

1. Commit/synchronize payload, index and decision as one publication.
2. Advance/synchronize the witness, including required namespace metadata.
3. Publish/acknowledge only after the selected guarantee is established. Witness
   failure leaves potentially committed/unacknowledged work and blocks later mutation
   until reconciliation; do not replay or claim failure.

Keep one unresolved retained publication per instance; even a committed store
operation cannot be followed by further mutation while its witness outcome is unknown.
Previously published calculation captures remain independent owners.

The witness means *eligible to acknowledge*, not proof of delivered ACK. A received
ACK must imply a covered horizon; the converse is false. A store ahead of its witness
may contain committed unacknowledged work and must not be truncated to the older
witness. A witness ahead of/inconsistent with the store indicates loss/mismatch:
block that history instance and preserve independent WB5 Open. Never lower a witness
to hide loss. Arbitrary client receipt text is not a trusted anchor.

Cover cursor, Clear/retirement and relocation decisions too, or suffix loss could
resurrect retired history. A witnessed save receipt still cannot make an uncertain
WB5 clean without actual installation verification.

The platform contract must define file/directory sync, writer fencing, namespace and
external-operation failure windows and actual flush limitations. Validate local NTFS,
native Linux and APFS. Do not silently enable SMB/NFS or foreign mounts based on path
syntax; unsupported history fails explicitly while valid WB5 Open stays independent.
No automatic downgrade or unretained fallback. Power-cut claims require OS/VFS review
and evidence beyond process kills. Physical identity tokens are evidence, not cryptographic
ownership or a universal defense against reuse and hostile rewriting.

### Retention, reclamation and file independence

Recommend retained history after explicit enablement, with an explicit session-only
alternative; defaults and disk budgets remain approval decisions. Retain reachable
logical Undo/Redo without a fixed transaction cap, constrained by real storage/user
policy. New branches may discard Redo only through committed decisions, not erase
its receipt semantics or lifetime reservations.

Clear publishes a baseline/epoch preserving content, full L, saved authority and
valid tickets. Logical Clear, physical reclamation and secure erasure need distinct
statuses. The research's archives are not production reclamation. Session-only close
may retire after successful completion; cancellation, failed close or declined recovery
does not authorize deletion. Witness retirement before acknowledging it.

GC roots include retained timeline/cursor, declined recovery, uncertain operations,
ledger/receipt fences and actual dependencies of pending tickets/read leases. Immutable
WB5 ticket bytes can avoid pinning an entire old root. Calculation captures own their
definitions independently. Publish durable reachability before removing obsolete data;
pruned receipts cannot become apparently uncommitted operations.

Physical compaction preserves logical states, Redo, identities and save authority.
Admit transient disk/memory first; exhaustion leaves the old store authoritative.
Database free pages do not prove physical file shrinkage: measure database/WAL and
filesystem allocation. Log compaction requires verified new segments/index roots and
reader-safe retirement. Neither deletion nor rewriting proves secure erasure across
snapshots, caches and backups.

Same-filesystem relocation uses persisted, verified/fenced handoff. Cross-filesystem
or copied-history migration needs a separate transfer/witness/single-writer contract;
do not implement it as a raw open-store copy. SQLite's live state can include WAL
data, so its backup/transfer mechanism needs explicit selection/testing.
[SQLite WAL documentation](https://www.sqlite.org/wal.html).

Save As rebinds only after verified installation; the old file remains independent
and copies cannot inherit a shared writable history automatically. Discovery/fork
policy requires approval. Decline never deletes unsaved work. Explicit unretained
editing must visibly end old continuity and cannot bypass uncertain history, witness
or file installation outcomes.

## 5. Implementation ordering and independent stop gates

Each row needs approval of the preceding design/results, not a blanket implementation
authorization inferred from this packet.

| Gate | Bounded deliverable after approval | Acceptance and stop |
| --- | --- | --- |
| G0 — approved | Review P1–P9 and adversarial plan | Resolve Save/ledger semantics and policy choices; no runtime edits yet |
| G1 — accepted/closed | One explicit validated native export path for ordinary and historical roots; existing WB5 | Preserve strict generic transfer/IDs/Redo/L; full three-platform Rust/managed/session/desktop/AOT gates accepted; preserve original evidence |
| G2 — accepted/closed, feature-only | Mutation-local PreparedRecordChanges from staged core/workbook candidates; no persisted journal integration | Independently reconstructed prior-state/full-checkpoint oracle; complete coupled changes, deterministic original-state coalescing, failed/no-op isolation and ownership/work measures; three-platform evidence accepted |
| G3 — active; hierarchy/normalization and dictionary research accepted; accounting/admission proposal next | Bounded SQLite-first indexed-store feasibility, no production backend adoption | Establish minimum physical costs and explicit scalable admission, compare original readiness requirements, and return for review; 20% is an unproven aspiration; full G2 scaling needs separate authorization |
| G4 | Durable index/decision publication and independent witness | Child restart at payload/index/root/witness windows; lost ACK/suffix, branch/cursor/ledger and wrong-instance checks; no production integration |
| G5 | Resource, lifecycle and reclamation contracts | Real native Save/decline/Clear/close with tickets/readers, exhaustion and interrupted compaction/relocation; approve numeric budgets/filesystem profiles |
| G6 | Separately approved Rust session ownership slice | Keep guards, saved content, tickets and uncertainty distinct; review any private protocol cut separately; hosted v1 unchanged |
| G7 | Separately approved desktop slice/release acceptance | Automated edit/clipboard/selection/save/close/recovery plus native three-platform CI/AOT; no recurring manual matrix |

G1 can start as an experimental projection. Promotion into the ordinary native Save
path also requires existing managed/native-session/AOT Save round-trip gates on all
three platforms, even if C# source is unchanged. Do not infer that a Rust-only research
proof alone certifies that production path.

Do not batch all work into one final approval. Stop before weakening assertions,
identity/semantic coverage, admission limits or filesystem guarantees. Preserve Stage
1–4 artifacts and add new milestone evidence; do not rewrite accepted research history.

Final acceptance freezes source/build inputs and native-origin evidence on local
Windows NTFS, native Linux and Mac APFS, with real restarts, zero-charge completed
workers, intended kill receipts and separately reported setup failures. Rust-only
slices use appropriate Rust gates; session/desktop slices also run managed/protocol/
native/AOT and automated interactions. Manual inspection is optional unless an actually
changed native interaction requires it.

## Accepted G2 boundaries and held G3 authorization

G0 is approved; G1 and G2 are accepted/closed. The owner keeps infrastructure work
active and holds G3, notwithstanding the parent's bounded SQLite-first feasibility
authorization. No G3 design, dependency, implementation or experiment has begun.
The accepted G2 contract below remains feature-only and does not promote a backend.
Produce
resolved stable-key semantic changes from private mutation, not command replay or
touched coordinates. Coalesce repeated stable keys relative to the transaction's
original state; failed/no-op candidates publish neither a transition nor ledger
advance. Include definitions, workbook and range/selection metadata, provenance,
invalidation, ContentStateId and lifetime reservations.

Keep complete checkpoints as an independent semantic oracle: apply local records
to an independently constructed prior authoritative image, then compare the complete
result with the candidate's full checkpoint. Cover edits/Clear/Move/copy, four-way
Fill, native/external Paste, mixed repeated batches, branch-after-Undo, cancellation,
failed/no-op and cold-restored roots. Measure local versus full-oracle visits/work,
emitted records, scratch admission, whole-domain peak and zero-charge cleanup.

Do not replace the accepted journal's persisted reconstruction with unverified
records. No coordinator, indexed store, SQLite/provider framework, witness, new
journal, protocol fields, UI history, changed production semantics/limits, recycling,
migration, JIT/SIMD or compaction. Stop on incomplete coverage, mismatched semantics,
stale authority, lost reservations, broad redesign or a required capacity increase.
The parent has accepted the G2 evidence. G3 remains held for infrastructure;
G4–G7 and production backend/session/UI choices still require separate reviews.
