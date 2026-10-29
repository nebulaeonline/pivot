# ADR-0029: Production workbook capacity and identity scalability

Date: 2026-10-29.
Status: **Accepted and closed by parent implementation review (2026-10-29).**
Parent accepts the capacity, performance and compact-corpus memory results, the
separately reported approximately 1.08 GiB identity-exhaustion peak, and the final
persistence corrections. Supplied source-qualified native headless and attended
desktop evidence is sufficient; the desktop runs predate the final Rust persistence
corrections, as recorded. No additional GUI/AOT qualification or reconciliation
packet is required. Preserve original failures and source qualifications. Short
target-local attended validation launchers are approved as the standing workflow.
The independent 4,096-position formatting limit remains unchanged. ADR-0023 stays
parked at G3; no production history work is authorized.

The [implementation handoff](../handoffs/2026-10-29-production-workbook-capacity.md)
records measurements, source-qualified validation, retained failures and limitations.

Parent clarifications: both prescribed table shapes must pass end to end; validate
the 262,144 lifetime-identity budget independently with few positioned cells and
without recycling/reset; diagnose material scaling failures at the first failing
checkpoint rather than weakening targets. A smaller accepted capacity requires
explicit parent review. The independent 4,096-style-position limit remains and
must be visible: computational capacity is not full-sheet formatting capacity.

## Context

CSV makes the current production workbook ceiling visible in ordinary datasets.
The next priority is useful spreadsheet capacity with predictable admission and
latency. Preserve the canonical persistent graph, immutable captures, exact typed
values, stable binding and atomic mutations. ADR-0023 remains parked at G3; none
of its experiments, observer fixes or history enablement are assigned here.

This design is based on source inspection, not a new benchmark. Existing bulk and
readiness harnesses are reusable references, but their experiment feature bypasses
some production checks and does not establish production readiness at a larger size.

### Current limits and coupling

| Boundary | Current production behavior | Consequence |
| --- | --- | --- |
| Workbook defaults | 1,024 live-cell positions, 8,192 allocated cell records, 1 MiB estimated workbook-owned data | Raising only the visible cell count will hit other admission boundaries |
| Workbook constructor | Non-experiment guard rejects live limits above 4,096, allocated limits above 8,192 or data budgets above 1 MiB | A changed default alone cannot enable the target |
| Position admission | `Transaction::anchor_on` counts the positions index; references allocate Blank anchors | Positioned blanks and reference targets consume capacity even without user-authored content |
| Clear | Changes content to Blank; retains position and stable identity | Clear does not promise quota reclamation; repeatedly editing an existing position should reuse its identity |
| Identity allocation | Batch add uses a checked monotonic node counter; workbook adds CellId and portable GraphNodeId UUIDs | Do not infer an allocation bottleneck solely from the 8,192 policy limit; measure it |
| Core graph defaults | 1,002,048 allocated node slots and 512 MiB definition bytes | These larger ceilings do not certify workbook, selection or save/reopen capacity |
| Selection subsystem | 128 selections, 8,192 selection identities, 4,096 points and consumers, 65,536 member edges/formula and invalidation nodes | These are different resources from cell/node identities; range-heavy formulas need separate workloads |
| Native storage defaults | 1,024 nodes, 8,192 identities, 1 MiB codec budget; workbook envelope 4 MiB | A workbook must remain savable at the advertised capacity |
| Workbook decoder | Includes explicit 8,192 record/count clamps | All admission and validation paths need an inventory, including reopen |
| CSV policy | 4 MiB input/output, 65,536 logical fields | Data interchange has independent quotas; logical fields include blanks |
| Worksheet/client geometry | 1,024 rows × 1,024 columns, 16 sheets | A 2,000-row CSV fails coordinates even if it contains few populated cells |
| Interactive/session bounds | 256 edits or calculation outputs per request; 1 MiB private frames; bounded viewport reads | Keep interaction bounded; a larger workbook does not justify larger unbounded messages |

Sources: workbook `Limits`, `persistent_checked`, `anchor_on` and Clear;
core structural `Limits`, integration batch allocation and selection constants;
storage `Limits`; workbook-document decoder; CSV `Policy`; session and desktop
coordinate validation. Count each actual domain separately in diagnostics and
measurements. `Stats.live_cells` currently reflects positioned records, not a count
of nonempty CSV fields or all memory retained by old captures.

## Decision

Deliver one production-capacity milestone, measured in steps and reviewed once at
completion. First qualify the existing production behavior at 1,024, then assess
10,000 and 65,536 positioned graph cells using the actual production path. Stop at
the first failed correctness/resource gate and diagnose that failure; do not run a
larger case or simply increase admission again. Intermediate sizes are checkpoints,
not separately accepted features or reasons for additional review packets.

### Proposed product target

| Resource | Proposed initial target | Scope |
| --- | --- | --- |
| Positioned graph cells | 65,536 total per workbook across all sheets | Includes Blank anchors and cleared positions; formatting-only blanks remain graph-free |
| Allocated cell/node identities | 262,144 per workbook | Explicit lifetime/retained-record headroom; finite and checked, with no identity reuse |
| Worksheet coordinates | 65,536 rows × 1,024 columns per sheet | Sparse coordinates, existing 16-sheet ceiling; no row/column structural editing |
| CSV logical fields | 1,048,576 per import/export | Independent of positioned-cell admission; blanks count as fields |
| CSV encoded bytes | 16 MiB per input/output | Independent of cell String limits and runtime admission |
| Native workbook envelope | At most 64 MiB | Proposed upper bound, subject to actual codec sizing and checked-work accounting |
| Single String/formula source | Existing 4,096-byte limits | Capacity is not permission for unbounded cell payloads |
| Interactive command/frame | Existing bounded commands and 1 MiB frames | Use bounded batches for bulk work; retain small viewport responses |

These numbers are proposed admission targets, not measured capabilities or a
promise that every possible 65,536-cell graph fits. Long strings, formula tokens,
range/consumer counts, dependency edges and retained captures remain independently
bounded. The implementation must publish those limits, provide concrete rejection
reasons and support the workload corpus below without silently relaxing other
budgets. Workbook-owned data, codec bytes/items/work, evaluation and selection
budgets must be derived from measured representations before defaults change.
Do not turn the 64 MiB envelope ceiling into a preallocation on every save.

At minimum, a 2,000 × 10 numeric table and a 4,096 × 16 numeric table must import,
calculate as applicable, export and save/reopen. The latter reaches 65,536 cells.
A sparse 65,536-row single-column file must pass coordinate checks. An otherwise
identical input exceeding any admitted limit must fail atomically and name that
limit. Leading zeros, quotation policies and empty String/Blank behavior stay as
accepted in ADR-0028.

### Identity and mutation contract

Retain stable cell, graph and reference identities across edits, moves, copies,
captures and native reopen. Repeated replacement at an existing cell must not
consume a fresh cell/node identity. Test Clear/repopulate separately from retire
and new-position churn. Tombstones and old captures remain distinguishable from
live positioned cells in diagnostics; no silent identity recycling, rebinding,
counter reset or compaction is introduced to meet a benchmark.

Keep allocation checks and candidate state isolated until commit. Failed imports,
edits, quota admission and cancellation must leave the live workbook and its
revision unchanged. New-workbook CSV imports may build across bounded transactions
in a candidate, as accepted in ADR-0028. Do not generalize that mechanism into
partially visible bulk edits of a live workbook.

Profile identity assignment, naming/index insertion, dependency binding,
validation/invalidation, workbook indexes and cleanup separately. Monotonic node
assignment is already constant-sized checked arithmetic; UUID generation and
index maintenance must be measured before selecting an optimization. Range-bearing
graphs can trigger full canonical validation on topology changes; include this
path rather than declaring scalability from independent literal insertion alone.
Preserve graph validation, lazy branch semantics, error provenance and cycles.

### Proposed performance and memory gates

Measure native release builds on Windows x64, Linux x64 and macOS ARM64, recording
CPU, memory, filesystem and toolchain. Targets below apply to the specified compact
workload corpus, using wall time and separately reported component timings. They
are design budgets for review, not current measurements.

- A 65,536-cell numeric CSV imports to a ready workbook within 10 seconds,
  including parsing, conversion, candidate admission and replacement. Report
  picker time separately and exclude human dialog time.
- Native Save/reopen and calculated CSV export each complete within 10 seconds
  for the compact corpus. Verify exact resulting data and identities, not just time.
- Capture p95 is at most 5 ms; a 200-cell viewport and an ordinary local edit with
  bounded dependency closure each have p95 at most 100 ms. Report at least 31
  steady samples. Capture must keep sharing state rather than copying all cells.
- Report full fan-out/aggregate recalculation separately, targeting at most
  2 seconds for the defined 65,536-cell corpus. Dependency-proportional work is
  expected; a bounded local edit must not pay for unrelated cells.
- For the compact 65,536-cell workbook, target at most 256 MiB of steady charged
  domain memory and 512 MiB peak during import/save/reopen with four retained
  captures. Account for two coexisting workbooks during atomic replacement.
  Retain the existing process-wide runtime policy; these are workload gates,
  not a claim that domain accounting bounds OS RSS or all allocations.
- Report allocation count/bytes, domain high-water, native RSS/working set and
  cleanup drain separately. State sampling and platform limitations. All owned
  runtime charges must return to zero after disposal; RSS need not return exactly
  to its initial value because system allocators can retain pages.

Compare 1,024/10,000/65,536 build time and allocation slopes. Investigate apparent
quadratic growth before raising limits. Do not impose timing thresholds on shared
CI machines without recording the host and noise; retain raw samples and explain
an exceeded budget instead of weakening it after the fact.

### Persistence and client boundaries

Inventory constructor, bulk mutation, capture/evaluation, selection, native
encoder/decoder, file broker, private session, clipboard/fill, address parser,
scrolling and CSV limits before changing policy. One declared workbook profile
should drive shared bounds where practical; prevent Rust/C# coordinate drift.
Do not broaden hosted-v1 model quotas or external protocol commitments implicitly.

Preserve existing WB9 representation when larger admitted counts and coordinates
fit its existing encodings; confirm this by reading the codec. Increasing quotas
alone does not require a format cut. The new reader must continue opening valid
current small WB9 workbooks. Older applications may reject larger files by their
old admission policy; explain this without promising backward readability or
reopening the general compatibility/migration policy. Change private framing or
versioning only if command shapes actually require it; retain bounded frames.

Large workbooks must not make clipboard/fill or styles accidentally unbounded.
Keep their existing per-operation budgets unless a concrete required workload
demonstrates a separately justified adjustment. Cross-sheet references must use
stable bindings with the same total workbook quota, not one enlarged quota/sheet.

## Alternatives considered

Raising only the cell default leaves constructor, storage, coordinates, selection
and allocation constraints in place. Advertising a million cells immediately
would skip an evidence-based useful milestone. Enabling experiment profiles in
the desktop would bypass production guardrails without proving save/reopen or
interaction behavior. Identity recycling or graph compaction adds reference and
capture semantics that are unnecessary for the first bounded increase. Replacing
the persistent backend, adding SQLite/history or building an observer framework
would expand the assignment beyond the production capacity problem.

## Consequences

This milestone crosses Rust workbook/core admission, native storage and C# address
handling, so a constant-only change is insufficient. Reuse existing measurement
and validation tools, correcting only concrete shortcomings relevant to this
scope. No new infrastructure framework, dependency or persistent store is proposed.
The target intentionally raises row coordinates as well as graph capacity so CSV
datasets with more than 1,024 rows become usable.

Quotas remain explicit finite product policy. In particular, Clear retaining a
position is existing semantics; the UI must not claim it frees all capacity.
Any proposed reclamation mechanism or automatic compaction requires a separately
reviewed identity contract. This first milestone may instead expose meaningful
capacity/allocated-identity diagnostics and actionable exhaustion errors.

## Verification / reconsideration trigger

After design review, implement the smallest complete production slice. Use numeric
tables, leading-zero/text/Boolean and multiline CSV, a mixed formula workbook,
bounded local chains, full fan-out, ranges/aggregates, cross-sheet references and
blank reference anchors. Include repeated edit/Clear/repopulate cycles, moved
identities, retire/new-position churn, four retained old captures and save/reopen.
Use independent closed-form/typed oracles; do not obtain expected values from the
same evaluator under test. Record actual counts of positioned cells, content,
allocated identities, selections, points, consumers and edges for each case.

Test every quota at its boundary and immediately beyond it, including aggregate
memory during candidate replacement, malformed/cancelled imports, exhaustion,
stale revisions and failed save installation. Retain native Save/CSV uncertain
installation semantics. Cover maximum row addressing, per-sheet scroll/selection,
formula copy/fill and bounded viewport behavior in actual windows on all three
native targets. Run focused headless checks and applicable AOT validation if the
changed paths warrant it. Keep original failures and follow artifact retention.

Deliver one concise implementation handoff with measurements and limitations, then
stop for parent implementation review. Reconsider target sizes if the corpus or
resource gates cannot be met without changing settled semantics; report the
specific bottleneck before proposing a smaller accepted scope or broader work.
This proposal does not authorize an experiment beyond 65,536 positioned cells,
production history integration or resumption of ADR-0023.
