# Runnable application: C# session and worksheet Slice A

Status: Slice A and ADR-0014 accepted by parent review received 2026-10-20; Windows/WSL2 Linux CI and final application
checks pass. Native macOS full CI, boundary and actual GUI checks also pass. An initial WSLg native startup abort remains
an explicit reliability limitation despite later successful runs.
The parent accepts ADR-0013/consolidation and closes the engine migration workstream.
This slice delivers a real local application, not a mocked worksheet or Rust-only demo.
[ADR-0014](../adr/0014-application-session-and-worksheet.md) describes the narrow boundary.

## Run

From the repository root with Rust, .NET 10 and Python installed:

```sh
python scripts/run_desktop.py
```

The launcher builds the Rust release session, discovers Cargo's actual target path,
sets PIVOT_SESSION_PATH and starts the desktop. Alternatively build/copy pivot-session
beside the desktop binaries or set that environment variable to its absolute path.
The default `dotnet run` desktop explains a missing child path instead of substituting
fake values. This is a development launcher, not a packaged/AOT distribution.

Select a cell, enter an invariant decimal or =A1+B1 in the formula field and commit
with Enter/button. Clear and Move selected preserve the canonical worksheet rules.
Wheel/page buttons scroll rows; Shift+wheel/column buttons scroll columns. Up to
20x10 cells are painted, with rows adapting to available window height. The UI reads
a whole bounded rectangle per request, displays Rust values and selected-cell errors,
and does not create a worksheet-sized control tree. A fresh unsaved workbook opens
on startup; closing Pivot discards it. File Save/Open is not added to this UI slice.
The formula field shows original submitted source; moving a cell preserves existing
target identities. Recommitting a formula explicitly resolves its addresses again.

## Implementation and state

`apps/pivot-session` owns one Workbook with production Limits::default and an explicit
IntegrationRuntime. A separately versioned additive application Protobuf schema reuses
Decimal128 and the existing frame shape over inherited local pipes. Frozen hosted
schemas/descriptors/fixtures remain unchanged. There is no network listener or duplicate
hosted authoring/receipt system. One child is owned by one application session.

Pivot.Client.WorkbookSession provides async atomic edits, reads, captures, selected
calculations, release and close. UI operations await pipe I/O; Rust performs authoritative
parsing/binding/arithmetic and worksheet work. Opaque exact-Expected tokens, retained
capture revisions, Rust result generations and UI request sequence checks prevent old
responses from replacing newer state. No C# evaluator or research constructor is used.

One in-flight request, eight accepted callers plus reserved close, four retained
captures, 256 edits/outputs/read-area cells and 1 MiB frames bound this slice. Core and
workbook product caps remain unchanged. Incoming/returned buffers are outside the Rust
retained ledger; the application bounds its own accepted requests and latest viewport.

Queued cancellation prevents dispatch. After dispatch, the client drains the reply,
tracks known committed state and discards the cancelled result; accepted edits are
not rolled back. Cancelled capture creation releases its handle. A 15-second transport
deadline is terminal; an unacknowledged edit may have committed and is never replayed.
CLOSE releases captures/book, joins workers and reports remaining charge. Idempotent
async disposal can terminate a stalled child after bounded waiting; abrupt failure
cannot claim a graceful zero-charge receipt or recover the volatile workbook.

## Verification and observations

Full local CI includes the real C# to Rust application-session executable in addition
to existing .NET/parity/frozen protocol/portable/document/daemon regressions. Boundary
assertions cover Decimal96 coefficient/scale/signed zero and overflow, scalar formulas,
dependent edits, stable moves/reuse and explicit rebinding, invalid drafts/repair,
atomic rollback/stale revisions, immutable S/T, bounded reads, capture limits,
queued/late cancellation, abandoned captures, transport failure and orderly cleanup.

An opt-in real-window self-check runs:

```sh
python tests/application-session/ui_smoke.py --output-dir artifacts/application-ui
```

It executes the real entry/edit/scroll paths against Rust, checks displayed revisions
and generations, writes a rendered PNG and timings, and closes the session with zero
remaining charge. It needs a graphical desktop; a headless build is not a GUI pass.
Timing diagnostics in normal interactive runs are opt-in with PIVOT_DIAGNOSTICS=1.

Boundary metrics use 31 samples after three warmups. Rust execution, C# encode/decode,
Rust first-pass encode probe and full C# request/response latency are separated from
an inferred framing/transport/scheduling residual; this is not pure kernel transport
or an exact additive decomposition of separately reported medians. UI observations
include 35 ms scroll coalescing and renderer scheduling, not physical display scanout
or an eight-sample p95/SLA. No claim is made for 100K application defaults or production deployment.
See the [full component observations](../performance/application-worksheet-slice-a.md).

## Results and limitations

| Check | Windows | Isolated WSL2 Linux | Native macOS |
|---|---:|---:|---|
| Full CI, including Rust/C# application boundary | Passed | Passed | Passed |
| Default Rust tests | 275 | 276 | 276 |
| All-feature Rust tests / strict lint | 291 / Passed | 292 / Passed | 292 / Passed |
| .NET 10 Release solution | Passed (six projects) | Passed (six projects) | Passed (six projects) |
| Final real boundary semantics/cancellation/disposal | Passed | Passed | Passed |
| Actual Avalonia self-check and image inspection | Passed | Final run passed; initial native abort unresolved | Passed |
| Normal CLOSE remaining Rust charge | 0 | 0 | 0 |

[Windows full CI](../../artifacts/application-ci-final-windows/20261020T032047372785Z/summary.json),
[Linux full CI](../../artifacts/application-ci-final-linux/20261020T032048622620Z/summary.json),
[Windows feature tests](../../artifacts/application-all-features-windows.log),
[Linux feature tests](../../artifacts/application-linux.log),
[evidence audit](../../artifacts/application-evidence-audit.json).
Full CI is followed by final targeted C# boundary and real-window checks for the
last queue-byte, displayed-revision and viewport-bound guards. Rust and frozen wire
behavior did not change between those checks. All 136 product/test/harness and root
build-file hashes match the isolated Linux copy. The final timing loops were
sequential with no concurrent build/benchmark work.

31-sample complete C# request p95, milliseconds (not UI latency):

| Operation | Windows | WSL2 Linux | Native macOS |
|---|---:|---:|---:|
| edit | 0.130 | 0.387 | 0.117 |
| viewport | 0.307 | 0.906 | 0.220 |
| calculate | 0.281 | 0.326 | 0.119 |

Session startup/shutdown are single observations; see the performance record for
numbers, component timings and interpretation. Eight UI edit-to-render observations
are 40.64–55.18 ms Windows / 41.08–129.74 ms WSLg, including 35 ms coalescing and a
cold first Linux action. Normal UI updates are asynchronous; this is not a p95/SLA
or physical scanout measurement. Both final rendered images were inspected.

The first WSLg GUI attempt aborted natively (`free(): invalid pointer`, SIGABRT)
before producing a report. A debugger attempt did not reproduce the failure and
captured no failure backtrace; one normal diagnostic repeat and the final-source
normal run pass. No renderer workaround or dependency change was applied. Keep
that first failure as unresolved startup-reliability evidence, not an unconditional
Linux GUI pass. [Original log](../../artifacts/application-execution-linux.log),
[debugger log](../../artifacts/application-native-backtrace.log),
[normal repeat](../../artifacts/application-ui-linux-repeat/results.json),
[final Linux window](../../artifacts/application-ui-final-linux/results.json).
WSLg execution does not establish native Linux desktop deployment coverage.

Transient NuGet metadata access required a fresh restore; no audit suppression was
committed. Compiler checks corrected deprecated Avalonia APIs and a Rust Copy clone.
Image inspection exposed fixed-height overlap; the viewport now fits available
space. Final checks include exact decimal-input rejection (no silent .NET rounding),
explicit expected tokens, stale-display conflict and worksheet-edge scrolling.
No assertions or resource limits were weakened.

## Scope and review

The original Linux backend viewport p95 miss (5.343 ms) and its 3.381 ms repeat remain
on record; this application work does not resolve their cause or relax the target.
Raw ignored benchmark CSV/logs need retention separately from Git; existing artifacts
are retained in-tree and tracked Markdown memorializes the historical measurements.
No archival duplicate source tree or new migration milestone is introduced.

No new engine, persistence provider/container, ranges/SUM, copy/fill, advanced formatting,
hosted persistence or capacity increase is included. Stop for review after Slice A.
The [native APFS packet](2026-10-19-macos-application-slice-a-instructions.md) was completed;
Slice A and ADR-0014 are accepted. The
[next assignment](2026-10-20-application-slice-a-review.md) is native workbook
Save/Open and application lifecycle; its implementation is not part of this slice.

## Native macOS follow-up — 2026-10-20

[Native report](../../artifacts/macos-validation-20261020T101213Z/REPORT.md): Apple M1,
macOS 27.0.1 ARM64, .NET 10.0.402 and Rust nightly 1.101.0. Source, targets,
.NET bin/obj and temporary files ran from verified internal APFS, copied from SMB.
All 294 copied-file hashes match this workspace before documentation integration;
local/share final comparisons show no edits. HEAD was
`1f548ffb67bd8ad56c15fc2ad769f83470d957cb`; the manifest covers the working tree.
All 351 returned report files passed transfer verification, with no missing report links.

Full CI passed 276 default Rust tests, .NET Release (six projects, zero warnings/errors),
10,099 decimal vectors with zero differences, frozen protocol/portable codec checks,
real daemon/CLI/REPL/document tests and the application boundary suite. Strict
all-feature lint and 292 all-feature tests passed. No source fixes, dependency or
fixture changes, assertion weakening or validation reruns were needed.

The actual native Avalonia window passed against its owned Rust child. Its rendered
18x10 image was inspected: controls, grid and footer fit and remain readable, with
no overlap. Both boundary and GUI normal close reported zero remaining charge;
no owned processes remained. Eight edit-to-render observations were 39.45–108.56 ms
(first action 108.56 ms, subsequent actions 39.45–41.42 ms), including 35 ms coalescing.
This is rendered-frame evidence, not OS input automation, physical scanout or a UI SLA.
The WSLg native abort and historical backend viewport variability remain unresolved.
