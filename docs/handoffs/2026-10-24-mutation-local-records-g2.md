# G2 — mutation-local authoritative records

Date: 2026-10-24. **Feature-only Rust producer implemented; Windows/native Linux/native Mac
regressions and measurements pass. G2 is accepted/closed. No default mutation promotion or persisted journal integration.
G3 is authorized in principle but held, not started, during the infrastructure detour.
[Parent acceptance and owner hold](2026-10-24-g2-acceptance-g3-held.md).**
[Authorization](2026-10-24-g1-closure-g2-authorization.md),
[architecture](../architecture/history-production-readiness.md),
[ADR-0023](../adr/0023-document-transaction-history.md).

## Implementation boundary

`history_experiment::changes::PreparedRecordChanges` owns the private paired
candidate, deterministic semantic record operations, lifetime reservation additions,
source guards, metrics and admission charge. `Workbook::prepare_record_changes`
uses the normal private edit/Fill/Paste/duplication path. It does not export a full
checkpoint, clone a full record image or diff two complete roots.

Workbook transaction cell keys are combined with canonical definition/value,
selection descriptor/placement and validation-cache mutation capture. Selection
retirement at last consumer is captured at the descriptor write, not inferred from
touched cells. Persistent charged indexes collect keys while staging; final source/
candidate values are read only for those keys and fixed metadata. Repeated writes
coalesce against the original transaction state. Unchanged records disappear; a
changed batch has one final ContentStateId and one operation per stable record key.

`RecordKind` distinguishes workbook authority, ledger metadata/reservations,
canonical definitions, selection/placement metadata and records, validation/
invalidation, and workbook cell directory/content. Payloads use the existing typed
cold record encoders: already-bound tokens, exact decimal parts, source/provenance,
diagnostics, live/retired status and stable IDs. They are not authoring commands,
display-name lookups, Arc addresses or cached calculation results. No parser or
rebinder runs when applying records.

The full checkpoint's derived core-byte-length and outer digest records (tags 5/17)
are not authoritative operations. The independent reconstruction oracle recomputes
them; its complete resulting bytes must equal the actual candidate checkpoint.
Counts, identity floors, ContentStateId, validation and invalidation records remain
represented. No WB5, CB2, private v4, hosted v1 or existing cold format cut was made.
The record image is an in-memory experiment, not a new durable journal/container.

## Source authority and publication

Floor equality alone cannot certify the source: two private branches can share
session, revision and allocation floor while assigning different UUIDs to the same
ordinal. G2 now requires an ephemeral validated-content marker on the existing
ColdLedger. Full observation/identity validation or verified cold restoration
establishes it. It is not serialized, a durable witness, a timeline cache or an ACK.
Raw historical root activation requires one explicit existing `observe` at that
full-state baseline boundary before local edits. Cold reopen certifies the restored
source after its existing complete validation.

Successful feature-only `PreparedRecordChanges::publish` checks original Expected,
ContentStateId, ledger instance/source marker and old floors. It validates/adopts
only newly committed node/selection reservations, then moves both in-memory
authorities after all fallible admission. There is no ledger clone/full observation
between local edits. A 48-step test runs this path without any full observation
until the separate end-of-sequence oracle. This helper is not wired into the
persisted journal, Rust service/session or desktop, and makes no durable-decision
or acknowledgment claim.

The independent record-image application also guards original workbook/ledger/core
metadata. An old image of B with a smaller lifetime ledger is rejected even when
its ContentStateId matches current B. Stale publication, wrong ledger instance,
foreign same-floor private branches, no-op and failed candidates cannot advance
content, mutation authority, lifetime reservations or the retained historical root.
Existing captures remain independent during concurrent calculation and preparation.

## Independent oracle and coverage

The source image comes from the complete existing checkpoint encoder. Apply the
local operations to that independently built prior image, reconstruct a complete
checkpoint and compare **all bytes** with a separate full export of the actual
private candidate. The oracle independently clones/observes the ledger; that full
work is outside the producer and publication path. After paired publication, the
actual ledger export must also match that full oracle. Cold reconstruction compares
exact CellViews, provenance, diagnostics and valid-graph evaluation results.

| Coverage | Proof |
| --- | --- |
| Typed decimal/text/Blank, typed and text formulas | Exact checkpoint, signed/scaled zero, Unicode and bound-token/provenance comparison |
| Invalid/cyclic/copied-rejected/missing-reference drafts | Exact cold state and diagnostics; existing first-core-error qualification retained |
| Clear/Move/retire and same-coordinate new identity | Stable-key live/tombstone, placement and reservation reconstruction |
| Four-way Fill, repeated keys, duplication | One coalesced operation per final key, no-op Fill and original-state comparison |
| Native/external Paste and cross-document copy | Normal transformation, target identities, shape/format rejection and rollback |
| Mixed sequences | 160 deterministic mixed transitions checked against independent full checkpoints |
| Undo then branch / cold roots | Preserved future-only reservations, stale-image rejection, fresh namespace remapping |
| No-op/failure/cancellation/admission/stale requests | No committed transition; original complete checkpoint unchanged |
| Mid-canonical cancellation | Deterministic cancellation after private edits, no record/allocation publication |
| Publication exhaustion/wrong ledger | Paired admission/authority rejection preserves both authorities |
| Concurrent captures | Old calculations remain correct during preparation and after publication |
| Oracle independence | Removing each coupled operation from a range edit breaks complete-state equality |

There are nine focused workbook integration tests, one oracle-omission unit test
and one core private-cancellation unit test. Existing workspace regressions remain.
Repeated range duplication to one target retains intermediate selector identities
as live/retired lifetime reservations while coalescing the final cell/definition.
No persisted reconstruction or default-path mutation policy was replaced.

## Ownership and work measurements

Trace indexes and shared counters use the existing allocation domain. Key buffers,
old/final selective blobs, operation copies and reservation vectors are admitted
before allocation; the sealed result retains its own charge and paired roots.
Rejected candidates use existing reserved reclamation paths. All completed tests
and measurement children require explicit quiescence/finish with zero domain charge.
The source validation marker is one optional UUID, not growing per transaction.

Capture and oracle implementations are in
[core records](../../crates/pivot-core/src/model/structural/history/records.rs),
[workbook changes](../../crates/pivot-workbook/src/history_experiment/changes.rs),
[focused tests](../../crates/pivot-workbook/tests/history_mutation_records.rs),
[measurement worker](../../crates/pivot-workbook/examples/history_record_producer.rs).
Record-key reads, mutation marks, canonical reads/scans, membership work, bytes,
seal scratch, allocation traffic, staging/seal time, publication time and whole-domain
peak are separate. Logical record reads are not AVL traversal-node counts; the byte
counter exposes large metadata replacements. Whole-domain requested/reserved charge
is not process RSS. Membership counters are diagnostic isolated-run observations.

Final measurements use five cases × three root sizes (32/256/768) × three fresh
process samples: **45 zero-charge children per platform, 90 Windows/Linux total**.
The full oracle and its O(L) clone/observation/full-map fold run in a separately
measured phase. Publication is separately measured and performs no full export.

| Case | Emitted records at 32 → 768 | Selected core reads | Canonical scans at 32 → 768 |
| --- | --- | --- | --- |
| Existing input | 3 → 3 | 12 | 0 → 0 |
| Existing scalar formula | 3 → 3 | 12 | 0 → 0 |
| Input used by range | 4 → 4 | 12 | 0 → 0 |
| Range formula replacement | 10 → 10 | 16 | 35 → 771 |
| New input/reservation | 9 → 9 | 14 | 0 → 0 |

The input checkpoint grows from 139 to 3,083 records. Scalar staging/seal peak
increments at 768 are about 51–64 KB in the sampled runs. Range edits retain the
existing roughly 16.8 MB invalidation reservation; it was not raised or optimized.
Range-formula edits still perform existing whole-graph validation plus descriptor/
member work, explicitly counted above. This is not a constant-time edit claim.

Reservation publication appends only new keys, but deliberately keeps the existing
ColdLedger charging policy. Replacing its charge holds old/new ledger reservations
briefly: at 768 nodes, new-node/range-reservation publication adds roughly 398 KB
of requested-domain peak; value-only publication adds none in these samples.
There is no full-map clone in publication. This resource policy remains experimental;
no numeric production default, quota increase or G5 decision is implied. CPU timings
are three-sample diagnostics, not benchmarks or universal speedup guarantees.

## Gates, failures and frozen evidence

Evidence: `artifacts/history-records-g2/`; final candidates are `windows-current/`
and `linux-current/`. **173 matching inputs** cover 172 Rust files/manifests plus
the measurement runner. Initial source/input archives and earlier results remain
separate; they do not validate final source hardening.

| Final gate | Windows NTFS | Native Linux ext4 | Native Mac APFS |
| --- | --- | --- | --- |
| Default / all-feature Rust | 347 / 401 | 348 / 402 | 348 / 402 |
| Format and default/all-feature strict lint | Passed | Passed | Passed |
| Release worker / fresh-process samples | Passed / 45 | Passed / 45 | Passed / 45 |
| Domain charge after each completed sample | 0 | 0 | 0 |

Linux source/test data/report originals are `/var/tmp/pivot-records-g2-src`,
`/var/tmp/pivot-records-g2-tests` and `/var/tmp/pivot-records-g2-current`.
Only build cache and returned receipts use J:. Reports were hashed on native Linux
before copying. Windows source/data is local J: NTFS. Worker binaries, source/build
and evidence hashes identify the tested implementation. Original G1 and Stage 1–4
manifests remain unchanged.

The initial Linux all-feature run failed the existing Rust clipboard test's
`runtime.finish().is_ok()` assertion. Its unchanged-source targeted rerun passed;
final hardened-source complete gates passed. The cause remains unresolved. Do not
attribute it to the earlier UI clipboard failures or weaken the shutdown assertion.
Failed/initial logs are retained under `linux-initial/`. No runtime or clipboard
implementation/test fix was made for that failure.

Development compile corrections included a missing import, integer conversion in
the worker, test-module placement and trace visibility. Source/ledger hardening
followed adversarial review: equal floors were insufficient, so source validation
and guarded local paired publication were completed and retested before freezing.
The final worker also measures publication separately; earlier 36-sample runs are
intermediate evidence only. No confirmed semantic mismatch remains in the final
full-state differential coverage.

## Native Mac reconciliation and parent acceptance

[Native Mac instructions](2026-10-24-macos-mutation-local-records-g2-instructions.md)
cover the same feature-only Rust gates, 45 fresh-process samples, native APFS source/
target/data and hashes before SMB return. Fresh Mac coverage of this changed source passed: 348/402 Rust tests, all seven
gates and 45 zero-charge samples. [Independent reconciliation](2026-10-24-macos-mutation-local-records-g2-validation.md)
verifies 173 inputs, 483 frozen source entries and 85 transferred evidence hashes.
There are 135 fresh-process samples across three platforms. Prior G1 Mac results
are not reused as G2 evidence. No new manual UI, managed or AOT
run is required for this feature-only producer. Those gates remain required before
any separately approved default mutation promotion.

The parent accepts this implementation and evidence and closes G2. Preserve all
original receipts and reliability qualifications. The owner keeps the infrastructure
detour active: do not start G3. Its bounded feasibility authorization is recorded
separately; no production backend or persisted-journal promotion is selected.
G1/G2 remain closed; G4–G7 remain independently gated.

