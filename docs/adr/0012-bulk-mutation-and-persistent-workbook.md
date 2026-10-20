# ADR-0012: Atomic bulk mutation and opt-in persistent workbook integration

Status: Accepted. Date: 2026-10-19. Implementation: foundation implemented;
verification evidence is recorded separately in the handoff. Production workbook
backend selection remains reference pending review.

## Context

The parent accepted ADR-0011 and the standalone persistent default, while promoting
per-node construction dispatch overhead to the critical path. Opening/populating
models and pasting cells need one atomic candidate, not one dispatch per cell.
Existing worksheet identities, sequential edit provenance, invalid drafts and
frozen storage/protocol contracts must survive migration.

## Decision

Use one canonical graph and existing persistent indexes. Internal doc-hidden
Batch builders reserve request metadata, text and typed-token storage before core
installation. New IDs are private planned identities until success. New names are
predeclared before symbolic formula binding, so formulas can reference later new
names in the same candidate. Definitions and reverse indexes use the existing
canonical mutations, parser, binder, typed-transfer validation and arithmetic.
A shared memo validates overlapping changed closures once. The first invalid-draft
witness conservatively prevents global certification; complete diagnostics still
use the accepted explicitly requested complete validator.

The batch contract supports new inputs/formulas, input replacement, named or typed
formula replacement, rejected parse drafts and retirement. Rename and public-port
batching are not added. Names of new nodes must be distinct from existing names;
retirement/name reuse in one core batch is not a rename/swap facility. Worksheet
coordinate reuse is independent and uses fresh diagnostic names and identities.
Existing individually authored APIs retain their semantics.

Actual core revisions advance by accepted logical operations, not physical dispatch
count. Creating a formula is one operation despite an internal placeholder. A
worksheet transaction advances its public state once; its core revision advances
by the actual core operations prepared in that transaction. Metadata-only moves
leave core revision unchanged. Empty/net-noop worksheet transactions install
nothing. Hosted service publication/revision policy is unchanged.

Batch application checks both the scoped revision and exact root identity, then
installs once. Preparation-limit errors poison a builder; it cannot accidentally
commit its accepted prefix. Hard malformed typed bindings, stale roots, invalid
identities, admission/work/quota failures and cancellation reject the whole
candidate. Lexical/name/cycle/missing-reference drafts remain editable, following
the accepted model contract, and never imply whole-graph validity.

## Workbook boundary

`pivot-workbook/persistent-core` enables explicit `Workbook::new_in` with an
application-owned shared IntegrationRuntime. Ordinary Workbook::new, host, storage,
document and daemon paths retain their reference selections. No production cap or
format changes. The new path uses canonical charged AVL indexes for positions,
records, cell-to-node bindings and inverse node lookup; it maintains cached logical
quota totals. It holds no populated shadow reference graph or worksheet.

Coordinate resolution follows the accepted sequential private worksheet candidate,
including demand-created blank anchors and moves. This is distinct from the core
symbolic-name final-directory phase. Original source, spans, submitted positions,
stable target IDs, decimal representations, transaction base, edit ordinal and
logical core revision at resolution are preserved. WorksheetId, CellId, portable
GraphNodeId and runtime NodeId remain independent.

Ordinary interactive calculation uses explicit capture_local and pinned metadata.
Existing capture remains the complete globally validated compatibility contract.
Bounded row-seeking rectangle reads avoid scanning unrelated columns/rows. Returned
CellViews have transient copy-out admission and become caller-owned afterward.
Old captures retain original metadata and graph definitions after new transactions.

## Runtime and resource policy

Keep ADR-0011's runtime/budgets and fixed lanes. A batch uses at most one dispatch:
large preparation (over 256 operations, over 64 KiB text, or a typed submission over
256 tokens) uses the existing heavy lane; small batches use mutation workers.
Cancellation polls occur between worksheet edits, graph operations and validation
steps. A completed installation is not subsequently rolled back by cancellation.

Core candidate guards, batch payloads, worksheet metadata roots and journals reserve
reclamation ownership before work. Rejected/canceled partial generations and
last owners retire through the existing bounded cleanup/recovery machinery.
Request storage, persistent allocations, traversal reservations and copy-out are
charged. Worker stacks/metadata follow the existing admission policy; caller-owned
requests/results after transfer, allocator/OS overhead and the fixed process
recovery worker remain outside exact engine accounting. No RSS ceiling is claimed.

Internal limits: 262,144 core operations, 16 MiB declared text budget, 4,096 bytes
per formula, existing typed-token and graph/work limits. Production worksheet caps
remain unchanged (4,096 live / 8,192 allocated / 1 MiB configured maxima). Larger
fixtures use explicitly gated research constructors, not expanded product limits.

## Alternatives considered

Repeated public mutations retain dispatch overhead. A separate bulk engine would
duplicate binding/identity semantics. Full materialization per edit would lose the
accepted localized-edit gains. Global revalidation of each changed root separately
would make overlapping chains quadratic. These alternatives are not selected.

## Consequences

Bulk work is bounded and atomic but still synchronous at the Rust API boundary.
Worksheet parsing/metadata preparation is O(request work) on the submitting thread;
applications must schedule large authoring requests appropriately. Core preparation
and reclamation use the shared runtime; no thread-per-cell or new runtime is added.
Complete diagnostics/materialization remain explicit broader operations. Reference
fallback/oracles remain while migration continues. A trusted experimental reference
coordinator checks quotas at the end for monotonic benchmark fixtures; production
reference per-edit checks are unchanged and remain the correctness oracle.

## Verification / reconsideration trigger

Require accepted suites on both paths, exact differential metadata/revision tests,
independent arithmetic oracles, rollback/cancellation/admission/cleanup tests, and
matched construction/new-existing/batch/selected-calculation evidence. Reconsider
before production workbook selection, any public general batch API commitment,
cap increase, new formula language, file container or hosted integration.
See the [handoff](../handoffs/2026-10-19-persistent-workbook-bulk.md).


## Three-platform verification (2026-10-19)

Full CI now passes on Windows/Linux/native macOS, with all-feature test counts
296/297/297, true fallback 21 each and workbook-only 48 each. All 335 matched Release
and diagnostic children pass with zero final domain charges. The native run used a
fresh internal APFS copy; all 290 source hashes matched before documentation updates,
without source fixes or reruns. See the
[native report](../../artifacts/macos-validation-20261019T220845Z/REPORT.md) and
[performance evidence](../performance/persistent-workbook-bulk.md). Native large cases
recorded warning-level memory pressure; separate sparse RSS samples are not child
high-water, and the runner RSS guard remains unavailable on macOS. These results
support review of the opt-in foundation; they do not select the production Workbook
backend or expand the accepted language, formats or product limits.


## Acceptance and readiness follow-up

The parent review accepted the opt-in foundation and this structural direction;
the owner authorized the next readiness milestone. Production Workbook selection
remains reference. The [readiness assessment](../architecture/persistent-workbook-readiness.md)
profiles construction and evaluates a bounded private-index optimization before
any backend selection change. It does not reopen the graph representation decision.

The subsequent readiness optimization now has source-matched Windows/Linux/native
macOS validation; see the [readiness review handoff](../handoffs/2026-10-19-persistent-workbook-readiness.md).
That follow-up awaits owner review and does not change production constructor policy.
