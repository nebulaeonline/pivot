# Controlled Production Core Integration — Stage 1

Date: 2026-10-19. Recommendation: **Advance controlled integration** for another
reviewed bounded slice; do not switch the default backend or raise capacity limits.
Stop here for parent review.

## What changed

The parent accepted the compatibility foundation and requested migration through
the accepted core facade. WorkingModel now owns either its reference storage or,
after explicit activation, the existing persistent graph. Supported named decimal
input/formula edits, private staging, input updates and calculation captures run
through that facade. Activation relinquishes all original node/name/interface
containers, rather than maintaining a shadow model. The integration-only feature
keeps the research Graph API private. It does not introduce another parser, binder,
arithmetic executor or graph representation.

Core revisions are the actual WorkingModel Revision, including per-operation
increments, equal assignments, invalid drafts and failed-operation behavior.
Internal generations never stand in for service/publication revisions. Immutable
calculation captures/results retain their exact core revision and stable bindings.
The calculation-only capture is additive; full ModelSnapshot inspection is
explicitly not integrated, rather than silently performing a full conversion.

All private edit preparation/validation and its failure cleanup run on the bounded
runtime worker. Caller-executed selected calculation queues its traversal/partial
result scratch for disposal, including admission failures. Writer roots, branches,
captures and results automatically queue last-owner release. Activation uses an
admitted scoped import worker while preserving the original model on failure.
There is one canonical persistent edit implementation, reached by WorkingModel;
this stage replaces the reference storage for actual activated model operations.
It is not just another direct-Graph benchmark or a dual-written facade.

## Review map

- [API/revision/diagnostic matrix and downstream impact](../architecture/controlled-core-integration.md)
- [Memory ownership, reclamation and scheduling policy](../architecture/controlled-core-integration.md#ownership-and-reclamation-contract)
- [Complete benchmark method and reproduction](../performance/controlled-core-integration.md)
- [Integrated facade/runtime](../../crates/pivot-core/src/model/integration.rs)
- [WorkingModel dispatch and explicit unsupported-operation guards](../../crates/pivot-core/src/model.rs)
- [Differential/independent-oracle and ownership tests](../../crates/pivot-core/tests/core_integration.rs)
- [Canonical persistent state/execution](../../crates/pivot-core/src/model/structural.rs)
- [Facade-only Release example](../../crates/pivot-core/examples/integration.rs)
- [Guarded process runner](../../scripts/integration.py)
- [Validation and source provenance](../../artifacts/controlled-core-integration-validation.json)

## Controlled Release results

Same 32-logical-CPU x64 Windows/WSL2 machine; default optimized Release, no custom
LTO/target CPU, pinning or frequency controls. Toolchain/platform details are in
each environment.json. Always-on requested-allocation counters cover both caller
and worker. Runs are sequential separate processes, with regression builds outside
the final measured sweeps. This is controlled workload evidence, not product latency.

Each row has 3 warmups and 31 measured samples. Pipelines include expected-revision
check, private stage, edit, capture, selected evaluation where applicable, result
verification, writer installation and disposal. At 100K the selected chain has
1,000 formulas; at 1K it has 999. Formula positions change at the front/middle/tail.

| Platform | Base nodes | Input/capture ref / integrated, ms | Selected chain ref / integrated, ms | Worst formula position ref / integrated, ms |
| --- | ---: | ---: | ---: | ---: |
| Windows | 1,000 | 1.2714 / 0.0167 | 1.7174 / 0.6589 | 1.6863 / 0.7994 |
| Windows | 10,000 | 9.4457 / 0.0263 | 8.8519 / 0.4699 | 9.5798 / 0.9272 |
| Windows | 100,000 | 130.3696 / 0.0176 | 131.5758 / 0.4334 | 118.9778 / 0.6618 |
| Linux | 1,000 | 1.5941 / 0.0874 | 1.1779 / 0.4911 | 1.1924 / 0.7337 |
| Linux | 10,000 | 8.3615 / 0.0734 | 7.3945 / 0.4966 | 7.9244 / 0.6984 |
| Linux | 100,000 | 89.4042 / 0.0818 | 89.1173 / 0.4414 | 91.5645 / 0.8241 |

| 100K operation, p95 ms | Windows reference | Windows integrated | Linux reference | Linux integrated |
| --- | ---: | ---: | ---: | ---: |
| unchanged_capture | 77.4919 | 0.0008 | 64.7453 | 0.0003 |
| input_capture | 130.3696 | 0.0176 | 89.4042 | 0.0818 |
| input_chain | 131.5758 | 0.4334 | 89.1173 | 0.4414 |
| formula_front | 117.5144 | 0.5047 | 87.8890 | 0.4280 |
| formula_middle | 112.4670 | 0.6587 | 91.5645 | 0.5570 |
| formula_tail | 118.9778 | 0.6618 | 89.4826 | 0.8241 |
| invalid_draft_repair | 147.5870 | 112.6689 | 117.0480 | 94.5204 |
| named_add_formula | 125.6035 | 0.0947 | 101.2336 | 0.1821 |

All integrated 100K provisional gates pass: input/unchanged capture <5 ms p95,
local valid formula pipeline <10 ms, and input plus selected chain <25 ms.
The invalid-draft row deliberately invokes the full exact diagnostic bridge;
it remains O(N + E), outside the local valid-edit gate. It must not be advertised
as a cheap diagnostic or an incremental validation solution.

| 100K process | 16 captures, extra requested heap | 32 captures, extra requested heap | Peak requested heap | Peak sampled RSS | Complete final writer release, ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| Windows reference | 551.53 MiB | 1103.06 MiB | 1166.32 MiB | 1256.70 MiB | 26.8956 |
| Windows integrated | 61.17 KiB | 126.14 KiB | 93.20 MiB | 111.56 MiB | 43.1344 |
| Linux reference | 551.53 MiB | 1103.06 MiB | 1166.32 MiB | 1341.18 MiB | 10.5329 |
| Linux integrated | 61.17 KiB | 126.14 KiB | 93.20 MiB | 149.06 MiB | 63.3142 |

At the final settled writer state, live requested heap is about 31.15 MiB for
the reference versus 41.64 MiB for the integrated backend on both platforms.
Persistent state therefore has a larger base footprint in this fixture; the large
savings concern retained generations and avoided temporary full copies.

Retention rows hold changing structural generations, with the worker drained before
reading requested live heap. Cleanup enqueue and drain are reported separately in
the raw data. The integrated final writer's complete release is slower overall
than the reference in this fixture. Its low caller enqueue time is an ownership
benefit, not proof of faster total destruction. No last-owner charge remains after
finish in any case. Queue saturation can block the caller; there is no nonblocking
or hard-RSS guarantee.

Requested heap, sampled process RSS, allocation-domain charges and reservations are
distinct. The import/diagnostic reservation peak is conservative headroom, not a
measurement of allocated physical memory. The 2 GiB core-domain policy and 8 GiB
process stop did not change production limits. No million-cell support, document
loading, ranges, GUI or hosted service performance is claimed by this stage.

## Evidence and verification

- Windows: [artifacts/integration-final-windows/20261019T155445Z](../../artifacts/integration-final-windows/20261019T155445Z/summary.json), six passed processes, matching final runtime/test/runner SHA-256 manifest; [normalized metrics](../../artifacts/integration-final-windows/20261019T155445Z/metrics.json).
- Linux: [artifacts/integration-final-linux/20261019T155902Z](../../artifacts/integration-final-linux/20261019T155902Z/summary.json), six passed processes, matching final runtime/test/runner SHA-256 manifest; [normalized metrics](../../artifacts/integration-final-linux/20261019T155902Z/metrics.json).

Final combined feature regression: **246 Windows / 247 Linux tests**, including
11 new integration tests plus two queue-ownership unit tests. Cases cover the same
facade on both backends, generated independent arithmetic expectations, revisions,
private multi-edit branches, stale checks, retained captures, exact cycle/blocked
node diagnostics, malformed/unresolved imports and repair, original source and
stable bindings after rename/retirement/name reuse, exact decimal parts, imported
ports, explicit not-ready operations, admission failure before activation/edit/
capture, partial-calculation failure, shared domains and disposal after errors.
Full/disconnected queue tests prove preserved ownership and safe cleanup.

- [Windows combined tests](../../artifacts/integration-windows-tests-reviewed.log)
- [Windows combined strict Clippy](../../artifacts/integration-windows-lint-submitted.log)
- [Windows integration-only strict Clippy](../../artifacts/integration-windows-core-lint-submitted.log)
- [Linux combined tests](../../artifacts/integration-linux-tests-reviewed.log)
- [Linux combined strict Clippy](../../artifacts/integration-linux-lint-reviewed.log)
- [Final Windows full CI log](../../artifacts/integration-ci-final-windows.log)
- [Final Linux full CI log](../../artifacts/integration-ci-final-linux.log)

Final default CI reports: [Windows: 209 Rust tests](../../artifacts/integration-ci-final-windows/20261019T161056890529Z/summary.json), [Linux: 210 Rust tests](../../artifacts/integration-ci-final-linux/20261019T161152762588Z/summary.json). Both complete pipelines passed on the final source.

Full default CI verifies Rust formatting/strict Clippy, existing default tests,
.NET Release, 10,099 decimal vectors, frozen storage/protocol fixtures, real IPC,
daemon and REPL behavior. Windows named-pipe tests were rerun with the established
sandbox exception; no tests were weakened. An admission-fixture synchronization
race was corrected by draining pending root reclamation before reserving the final
headroom. Intermediate failures/logs remain in artifacts; submitted tests pass.
Linux file tests use disk-backed /var/tmp, not tmpfs. No dependency, codec, protocol,
application, host, workbook or storage source changes are required.

Native macOS status: **not run for this feature**. The prior native ARM64 Slice B
pass remains historical. A fresh run must copy SMB source to local APFS, especially
for .NET, and copy reports back into the shared artifacts directory. WSL results
are not native Mac evidence.

## Remaining compatibility and ownership gaps

- Full inspection ModelSnapshot, transfer authoring/export, port edits/invocation,
  rename, retirement and replace_input are explicitly not ready after activation.
  Default models retain all accepted behavior. Imported source/bindings/ports are
  inspectable and participate in validation, but unsupported no-op calls also
  reject. There is no hidden fallback conversion for ordinary supported edits.
- Exact invalid capture diagnoses use O(N + E) materialization on the worker.
  Initially invalid imports retain the full-validation repair fallback until valid;
  its large-scale latency has not had a separate sweep. Activation itself is an
  explicitly timed O(N) bridge, not a fast-file-load claim.
- Edits/diagnostics/reclamation share a worker per runtime. Long jobs/backpressure
  can delay another model's edit. Independent calculations remain caller-executed;
  several runtimes may share one allocation domain with separate workers. This
  stage does not establish multi-model throughput or cancellation-aware scheduling.
- Full queues apply blocking backpressure. A disconnected worker safely cleans up
  synchronously as an emergency exception. finish refuses live owners and must run
  off the interactive path; dropping the runtime handle instead detaches the join.
- Domain accounting excludes caller/request/error containers, iterator staging,
  handle/channel metadata and allocator/native/application reserves. Aggregate
  known-allocation sharing is implemented, not a complete process memory policy.
- Workbook demanded anchors/moves/retirement/paste and hosted final-directory
  binding, receipts, publication and portable revisions are not migrated. Existing
  host/storage/workbook paths remain on the reference backend unchanged.

The Stage 1 result justifies advancing a reviewed integration boundary, with
particular attention to full snapshot inspection and the runtime scheduling policy.
It does not justify a global switch, new capacity claims, C#/Avalonia work, grammar
or range additions, file changes or hosted persistence. Stop for review now.
