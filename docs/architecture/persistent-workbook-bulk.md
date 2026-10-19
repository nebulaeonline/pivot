# Persistent workbook integration and bulk foundation

Date: 2026-10-19. [ADR-0012](../adr/0012-bulk-mutation-and-persistent-workbook.md)
defines the internal batch/revision/resource contract. This is an explicit opt-in
worksheet path, not a production backend switch or document format migration.

## Application-owned workflow

Enable pivot-workbook's persistent-core feature. Create one IntegrationRuntime per
application/session and call Workbook::new_in(name, limits, &runtime). Multiple
worksheets can share runtime admission and workers. Supply expected() and a bounded
slice of Edit to apply/apply_cancelled. The method prepares private metadata and
core commands, validates and installs one coherent version, or installs nothing.
Release workbooks/captures before deterministic runtime finish.

Workbook::new remains reference. Storage/host/daemon selections and all frozen
codecs remain unchanged. No new IPC, file container, persistence, ranges, fill/copy
semantics or value types are introduced. Scalar spreadsheet parsing/lowering and
the core parser/binder/postfix arithmetic remain canonical.

## State and atomic installation

Persistent worksheet metadata consists of four canonical indexes: position→CellId,
CellId→record, CellId→portable/runtime binding, and NodeId→CellId. Records are charged
shared allocations. A candidate clones index headers and changes only affected
paths. Incremental logical statistics avoid full worksheet scans during editing.
The old reference containers in the facade stay empty on the new path.

A bounded journal tracks touched records/positions. Net-noop detection compares
only that journal, not all cells. Metadata-only moves and move-away/move-back noops
preserve accepted graph/revision behavior. Private core IDs/names are reserved by
the batch builder, while coordinate lookup uses the sequential worksheet candidate.
Demanded blanks receive zero-backed inputs and independent stable identities.

Core publication and metadata installation occur under exclusive mutable workbook
ownership. State exhaustion, malformed/stale/foreign targets, limits and cancellation
are checked before installation. All fallible candidate preparation precedes the
final field replacement. Core revisions count logical operations; workbook state
advances once. Prepared IDs are not returned from a rejected worksheet candidate.

## Reads, calculation and diagnostics

capture_local pins metadata/index roots and a graph calculation root without
materializing complete arrays. Selected calculation validates the requested closure;
unrelated drafts remain allowed. Captures preserve original positions, bound target
IDs, definitions, raw decimal parts, source spans and transaction provenance.
pinned_content supplies an admitted point definition copy from that generation.

capture retains its complete global-validation contract. validate is explicitly a
whole-model diagnostic operation and uses the accepted core snapshot validator.
Neither is required by the ordinary local edit/capture path. Local success is not
proof of global validity. Cycles/parse/missing-reference drafts remain editable;
malformed low-level token structure is a hard transaction failure.

Rectangle reads validate scope/bounds/area/output limits, seek each requested row
and stop at its final column. They do not enumerate unrelated sheet content to
produce a viewport. Header/payload copy reservations precede copies. Returned
CellViews are caller-owned after return; borrowed/pinned metadata stays accounted
in the shared domain. Formula display continues to expose original source rather
than pretending moved references have been rendered into new coordinates.

## Scheduling and reclamation

Each core batch submits once. Over 256 operations, over 64 KiB source, or more than
256 typed tokens in one definition uses the existing heavy lane. Small edits use
the existing mutation lane. Runtime workers/counts/budgets are unchanged. Queue
statistics expose accepted/rejected tasks and pending-not-started high-water
(including at most two receiver-assigned tasks, not literal OS queue lengths).

Worksheet preparation parses/lower-binds bounded submissions on the caller, under
request admission; a future UI should invoke large authoring operations off its UI
thread. Core mutation/validation executes in the runtime. A bulk private-root guard
retires partial candidates on errors/cancellation. Pre-admitted resource wrappers
also retire metadata roots and journals on success, failure and last-owner release.
Large-generation destruction never falls back to the caller. Shared charges end
only after their last owner; copy-out reservations end when ownership transfers.

Tests hold both heavy workers, queue/cancel a large batch, force backpressure and
prove an independent small edit still runs. Partial-candidate cancellation checks
off-caller destruction and zero final charges. Ordinary queue/admission/disconnection
and cleanup recovery tests from accepted Stage 2 remain in regression.

See the [matched evidence](../performance/persistent-workbook-bulk.md) and
[review handoff](../handoffs/2026-10-19-persistent-workbook-bulk.md).


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
