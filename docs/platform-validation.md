# Linux and macOS validation

## Current candidate: spreadsheet aggregate functions

ADR-0021 now uses WB5-only read/write; obsolete WB4 decoding and migration are
removed. Windows final managed CI, strict all-feature checks and actual Native AOT
pass (339/355 Rust tests), eleven desktop fields and zero-charge peers. Final Linux
managed/strict all-feature/Native AOT gates pass (340/356 Rust tests), with eleven
fields and zero-charge peers. Fresh native Mac/APFS gates also pass (340/356),
all eleven fields, actual ARM64 Native AOT/adjacent Rust child and zero-charge peers.
Mac run `macos-validation-20261023T090419Z` matched 407 source and 206 build-input
hashes; its receipt reports 458 returned files verified against APFS originals.
Windows independently matched source/input hashes and inventoried returned evidence;
the Mac receipt omits per-file original hashes for independent transfer comparison.
[Corrected candidate](handoffs/2026-10-23-wb5-only-aggregate-correction.md),
[completed Mac packet](handoffs/2026-10-23-macos-wb5-only-instructions.md).
The previous Mac run `macos-validation-20261022T233650Z` passed against the old
WB4-reading source, with 405 source, 207 build-input and 453 returned hashes checked.
That evidence is preserved, not claimed for this correction. Final acceptance is
pending. CB2/private v4/hosted v1 remain unchanged. Earlier milestones below retain
their historical formats and counts.

## Accepted preceding: document-owned worksheet text

ADR-0018 design and implementation are accepted; the worksheet-text milestone is closed. The new text candidate
passes final Windows/Linux managed CI (314/315 default Rust tests), strict
all-feature lint/tests (330/331) and Native AOT publish/execute without warnings.
All eight desktop fields and all native peers pass with zero charge; 183 build/test
input hashes match. Fresh native Mac/APFS managed CI, strict all-feature lint,
315/331 default/all-feature tests and Native AOT also pass without warnings or
product fixes/reruns. All 359 copied source hashes and 183 build/test inputs match;
438 returned files are present. All native main/peer workflows pass with zero charge.
The parent review received 2026-10-21 accepts this implementation and the
one-byte escape-overhead clarification. WB4, private session
version 2 and native clipboard v2 are deliberate cuts. The original Mac report has
one stale clipboard-v1 sentence; source/version evidence confirms v2, as documented
in the handoff. Prior WB3/AOT evidence below remains historical. [Implementation/current evidence](handoffs/2026-10-21-worksheet-text-implementation.md),
[Mac APFS packet](handoffs/2026-10-21-macos-worksheet-text-instructions.md).

## Accepted baseline: Native AOT

Windows x64 and Ubuntu/WSLg x64 Native AOT publish-and-execute gates pass without
warnings. The Linux final image executes from ext4. Native main/helper windows
cover clipboard, WB3 save/reopen and close workflows with zero-charge cleanup.
Ordinary Windows/Linux full CI also passes. Fresh native macOS ARM64/APFS managed
CI, strict all-feature lint, 310/326 default/all-feature tests and Native AOT
publish/run pass without warnings. Native main and all peers pass with zero charge;
349 copied source hashes and 178 build/test inputs match. All 426 returned files
are present. No product fixes or gate reruns were required. This baseline is ready
and accepted by the parent review received 2026-10-20; the milestone is closed.
[Evidence and limitations](handoffs/2026-10-20-native-aot-baseline.md),
[fresh Mac APFS packet](handoffs/2026-10-20-macos-native-aot-instructions.md).

Both managed and native desktop runners now enforce shared Windows/WSLg clipboard
serialization. Cross-OS exclusion checks pass in both directions.

## Accepted: rectangular clipboard, WB3 and desktop acceptance

[Implementation](handoffs/2026-10-20-rectangular-clipboard.md),
[automation and integrated Mac evidence](handoffs/2026-10-20-desktop-acceptance-automation.md),
[parent review packet](handoffs/2026-10-20-rectangular-clipboard-review-packet.md).
Windows/Linux/native Mac full CI passes (309/310/310 default Rust tests), with strict
all-feature lint and 325/326/326 all-feature tests. Final desktop/peer workflows pass
on all three platforms, including cross-process private-format clipboard, directional
commits, real close choices and zero-charge cleanup. Windows/WSLg final UI follow-ups
cover changes after their integrated CI runs; Mac integrated CI covers the final tree.

The [fresh Mac APFS run](../artifacts/macos-validation-20261020T235001Z/REPORT.md)
needed no product fixes, setup failures or gate reruns. All 339 copied source hashes
independently matched before doc integration and all 173 final build/test hashes match.
The transfer record verifies 391 returned files; all are present here. Source, builds,
temporary/document files and original reports used APFS. No surviving owned processes.
WB3/rejected-draft reopen and explicit WB1/WB2 rejection pass with frozen hosted v1 intact.

The owner replaced recurring manual five-group replay with deterministic acceptance
and optional inspection. No physical keys, IME/accessibility, native-picker or clipboard-
manager certification is inferred; no optional Mac manual run is claimed. Known WSLg
native abort, Windows-mounted-path uncertain installation and historical Linux viewport
p95 exception remain unresolved. The parent implementation review accepts this milestone and closes the workstream.
[Acceptance and operational follow-up](handoffs/2026-10-20-rectangular-clipboard-implementation-review.md).
Historical results below retain their original milestone scope.

## Accepted: rectangular worksheet selection

[Interaction specification and evidence](handoffs/2026-10-20-rectangular-selection.md),
[fresh native APFS report](../artifacts/macos-validation-20261020T222305Z/REPORT.md).
Windows/Linux/native Mac full CI passes (290/291/291 default Rust tests), with
actual-window capture/drag, navigation, auto-scroll, cancellation and editor checks
and zero-charge close. Mac strict all-feature lint and 307 all-feature tests pass.
Pure UI selection leaves Rust revision/dirty state unchanged and creates no graph
selections or content. All 327 copied Mac source hashes and 168 reference build
inputs match; 368 returned reports are transfer-verified, without product fixes,
setup failures or gate reruns. Native source/builds/tests used internal APFS.

Windows/WSLg manual checks passed per user confirmation. The Mac report's final
user-reported manual acceptance supersedes its earlier pending notes. Rectangular
cut/copy/paste is absent as expected and remains future scope; clipboard commands
still target the active cell. No additional Fn/IME/clipboard-manager coverage is
inferred. Static PNGs show active selection, not an independent visual certification
of gray inactive state. Prior platform/filesystem defects remain unresolved.

## Accepted: direct single-cell paste

Windows/Linux/native Mac full CI passes (290/291/291 default Rust tests), with final
actual-window paste checks and zero-charge close. Mac also passes strict all-feature
lint and 307 all-feature tests. See the [focused handoff](handoffs/2026-10-20-direct-single-cell-paste.md)
and [fresh native APFS report](../artifacts/macos-validation-20261020T173814Z/REPORT.md).
All 324 source hashes and 168 reference build inputs match; the transfer record
verifies 367 returned files. No product fixes, setup failures or gate reruns were
needed. Source/builds/tests and original reports were local to APFS, not SMB.

The Mac report's final user-confirmed manual acceptance supersedes its earlier
pending notes. The user confirms manual functionality across Windows, WSLg and
macOS works as intended. Specific IME/clipboard-manager coverage is not inferred.
Only direct single-cell paste is delivered; rectangular paste/copy-fill remain
future work. Prior platform/filesystem defects remain separate and unresolved.

## Accepted: in-place worksheet editing

See the [review packet](handoffs/2026-10-20-in-place-editing-review-packet.md) and
[implementation handoff](handoffs/2026-10-20-in-place-editing.md).
Windows/WSLg final clipboard/focus window checks pass. The
[fresh native APFS follow-up](../artifacts/macos-validation-20261020T170901Z/REPORT.md)
passes full CI, strict lint, 291 default / 307 all-feature Rust tests, .NET/parity/
IPC/application and actual-window editor/clipboard/focus checks. All 320 source
hashes and 168 build inputs match; 366 returned files are transfer-verified. No
product fixes or validation-gate reruns occurred. The report originally marked physical-key/manual acceptance pending. The user
subsequently confirmed that copy/paste key presses work and authorized manual
acceptance of those checks. The prior failure is retained as history; deferred
paste UX remains open.
The user deferred immediate grid-cell paste commit to the next round; current
paste-as-draft behavior is documented, not changed before parent review.

## Accepted: ranges and SUM Slice C

Full Windows/Linux/native macOS CI passes (290/291/291 default Rust tests,
306/307/307 all-feature), with strict lint, .NET/parity/IPC, real session and
actual-window SUM/Save/Open checks and zero-charge normal close.
[Implementation/quotas](handoffs/2026-10-20-ranges-and-sum-slice-c.md),
[validation/performance](handoffs/2026-10-20-ranges-validation.md).

The [fresh Mac report](../artifacts/macos-validation-20261020T130207Z/REPORT.md)
records local APFS source/builds/tests, 318 matching copied source hashes and 168
matching build inputs, no product fixes or validation reruns, and 363 verified
returned report files. Mac native pickers, all dirty-close choices and untouched
close passed per user confirmation. Windows manual native-dialog checks also passed.
The user confirmed WSLg Save-on-close succeeds on local WSL2 storage and untouched
close no longer prompts. Windows-volume Save-on-close reported installation
uncertainty; mounted-path replacement and the historical WSLg native abort remain
unresolved. Native filesystem passes do not establish mounted-path certainty.

## Accepted historical: native workbook persistence Slice B

The [Slice B handoff](handoffs/2026-10-20-workbook-persistence-slice-b.md) records
new document, native file-failure, cross-process Save/Open and actual-window evidence.
Windows/Linux/native macOS CI and actual Save/Open window checks pass (280/281/281
default and 296/297/297 all-feature Rust tests, six .NET projects, zero-charge normal
close). The fresh [native report](../artifacts/macos-validation-20261020T111827Z/REPORT.md)
validates this source on internal APFS: all 305 copied-file hashes match, no fixes
or reruns, 356 returned report files transfer-verified. APFS file-failure and
independent-process restoration tests pass; the rendered window was inspected.
OS pickers and dirty-close dialog choices were not manually exercised. The
[APFS packet](handoffs/2026-10-20-macos-workbook-persistence-instructions.md) is complete.
The unresolved initial WSLg startup defect remains on record.

## Accepted historical: runnable application/session and worksheet Slice A

The desktop now uses the real canonical Rust child-session boundary. Full CI adds
Rust/C# application semantic/lifecycle checks. Actual GUI execution is checked
separately with `python tests/application-session/ui_smoke.py`; it needs a display,
uses real Rust calculations and writes a rendered image and timing observations.
Windows and isolated Linux full CI pass (275/276 default tests, 291/292 all-feature),
along with final real Rust/C# boundary checks and zero-charge normal shutdown.
Windows and the final WSLg window/image checks pass. The initial WSLg attempt aborted
natively with free(): invalid pointer; later debugger/normal runs passed, but the
startup failure remains unresolved. This is not unconditional Linux GUI reliability
or a native Linux desktop deployment claim. Native macOS application full CI and
actual GUI execution now pass from a fresh internal APFS copy: 276 default / 292
all-feature Rust tests, strict lint, six .NET projects, decimal/protocol/document/IPC
and boundary gates, and zero-charge boundary/UI close. No fixes or reruns occurred.
All 294 source hashes match the pre-update workspace; 351 returned reports were
transfer-verified. The rendered 18x10 image was inspected without clipping/overlap.
[Native report](../artifacts/macos-validation-20261020T101213Z/REPORT.md). The
[application handoff](handoffs/2026-10-19-application-worksheet-slice-a.md) records
current evidence; use its [APFS packet](handoffs/2026-10-19-macos-application-slice-a-instructions.md)
for macOS. Engine-only reports below do not establish application execution coverage.

## Accepted historical: canonical engine consolidation

Windows, isolated WSL2 Linux and native macOS full CI pass (275/276/276 default
tests), along with 291/292/292 all-feature tests, strict feature lint, .NET Release, 10,099 decimal vectors,
frozen portable/protocol fixtures, document Save/Open and real daemon/CLI/REPL.
The Linux source copy isolates .NET bin/obj outputs; 113 product/test/harness hashes
match the workspace. Thirty-three sequential Release cases per platform (99 total) pass
correctness/resource assertions; all final charges are zero. One Linux viewport
p95 misses 5 ms at 5.343 ms; the single matched repeat is 3.381 ms. Both remain
reported, with no threshold relaxation. Fresh native macOS passes all targets.
The [native report](../artifacts/macos-validation-20261020T015824Z/REPORT.md) uses
internal APFS, matches 280 copied-file hashes before documentation edits, and records
no fixes/reruns. Native core no-default tests pass 62, using the canonical engine.
All eight native pressure samples were normal; maximum wait4 child high-water is
312.34 MiB, distinct from sampled RSS and unavailable native live RSS guarding.
All 100 children including the Linux diagnostic repeat finish with zero charges.

See [current handoff](handoffs/2026-10-19-canonical-engine-consolidation.md),
[performance evidence](performance/canonical-engine-consolidation.md) and
[native APFS instructions](handoffs/2026-10-19-macos-canonical-engine-instructions.md).
All earlier sections describe historical milestones and constructor policies.

## Historical: persistent workbook readiness (2026-10-19)

The [readiness follow-up](handoffs/2026-10-19-persistent-workbook-readiness.md)
profiles construction and tests private-index reuse. Source-matched Windows/Linux/macOS
validation passes: full default CI (251/252/252), all-feature regression (300/301/301),
core fallback (21 each), workbook-only (49 each), strict lints and 234 matched children.
All final domain charges are zero and all 68 benchmark source hashes match.
The [fresh native report](../artifacts/macos-validation-20261020T010244Z/REPORT.md)
used internal APFS for source, builds, temporary files and reports; all 296 copied
source hashes match the reviewed tree before these documentation edits, and transfer
verification covers 464 returned report files. No native source fixes or reruns occurred.
Native 100K workbook construction improves 1.44–1.68x to 0.742–0.999 s; all ordinary
review targets pass. All 52 pressure samples were normal. wait4 reports completed-child
CPU/high-water (maximum 1,418.8 MiB), while native live RSS guarding remains unavailable.
Earlier warning-pressure findings remain relevant; this is not an OS-capacity increase.
Production constructor selection stays reference pending review. The accepted
foundation below is separate historical evidence.

## Accepted persistent workbook and bulk foundation (2026-10-19)

Full default CI passes on Windows/Linux/native macOS (248/249/249 Rust tests),
including .NET, decimal/protocol parity and real IPC. All-feature regression passes
296/297/297, true core fallback 21 each and workbook-only 48 each. Required strict
lint configurations pass. All 335 Release/diagnostic children pass with zero final
domain charges. Production workbook selection stays reference.

The [native report](../artifacts/macos-validation-20261019T220845Z/REPORT.md) validates
this slice on Apple M1 ARM64 from a fresh internal APFS copy; all 290 source hashes
matched before the documentation update. All 109 native children pass, without
source changes or reruns. Native 100K numeric core batch is 239.8 ms versus 1.205 s
individual, and workbook population is 1.14–1.40 s. Existing interactive and proposed
bulk targets pass. Warning-level memory pressure occurred during large cases;
separate sparse RSS samples do not establish child high-water and the runner RSS
guard remains unavailable on macOS.

See the [current evidence](performance/persistent-workbook-bulk.md),
[handoff](handoffs/2026-10-19-persistent-workbook-bulk.md) and
[APFS reproduction packet](handoffs/2026-10-19-macos-persistent-workbook-bulk-instructions.md).
Earlier native reports below apply to preceding accepted milestones.

## Accepted standalone default transition (2026-10-19)

The standalone selection/lifecycle transition now passes full CI on Windows,
Linux WSL2 and native macOS ARM64: default counts 241/242/242, combined 264/265/265,
fallback core 21 on every platform, and ninety successful Release children.
See the [transition report](performance/standalone-default-transition.md) and
[review handoff](handoffs/2026-10-19-standalone-default-transition.md).

The native run used a fresh internal APFS source/target/temp/report copy; all 274
source hashes matched before the documentation update. Evidence:
[macOS transition report](../artifacts/macos-validation-20261019T191136Z/REPORT.md).
Individual-edit bulk construction and full snapshots retain their measured costs;
native sparse RSS samples remain distinct from child high-water measurements.
Earlier native Stage 2 results below remain historical.

## Accepted controlled core integration Stage 2 (2026-10-19)

Full default CI and combined feature regression now pass on Windows (259 tests),
Linux WSL2 (260) and native macOS ARM64 (260). Default Rust counts are 209/210/210.
All ninety optimized 1K/10K/100K facade benchmark processes passed their correctness
assertions and review targets. Default backend, capacities and application paths
remain unchanged. See the [Stage 2 handoff](handoffs/2026-10-19-controlled-core-integration-stage2.md)
and [performance evidence](performance/controlled-core-integration-stage2.md).

Native macOS used a verified internal APFS copy for source, targets, temporary test
files and initial reports. Returned evidence is under
`artifacts/macos-validation-20261019T182116Z`. Native RSS high-water remains
unavailable from the runner; sparse separate samples are disclosed in the report.
Earlier milestone sections below remain historical and should not be read as the
latest Stage 2 platform status.

## Compatibility foundation - 2026-10-19

Windows default full CI passes 209 Rust tests, .NET Release, decimal/protocol parity
and actual IPC/REPL. Final feature tests pass 233 on Windows and 234 on Linux, with
all-target feature Clippy. Linux default passes 210 tests and actual IPC/REPL with
file tests on disk-backed /var/tmp. No new Linux .NET or native Mac run is claimed.
See the [foundation handoff](handoffs/2026-10-19-compatibility-foundation.md) for exact
logs, standalone-core Release distributions, final source manifests, 32-generation
retention and guarded 1M prototype repeats on both platforms. Prior research results
below remain historical. Native macOS corroboration must build/test on local APFS.

## Structural mutation research - 2026-10-19

Windows full default CI passes 209 Rust tests plus .NET Release, decimal/protocol
parity and real IPC/REPL. The opt-in workspace passes 222 tests and feature Clippy.
Linux passes 210 default tests, 223 feature tests, feature Clippy and actual IPC/REPL.
The [structural handoff](handoffs/2026-10-19-structural-mutation-feasibility.md)
records matching Release comparisons, separate process memory, million-cell guards
and Linux results. These experiments do not replace the older Mac report: native
macOS corroboration of the paged prototype remains pending. Use local APFS for Mac
source/build/.NET work and return artifacts to SMB.

## Scaling feasibility research — 2026-10-19

Windows default full CI passes 209 Rust tests plus .NET/parity/IPC; opt-in feature
checks pass 215 tests and all-targets Clippy. Available WSL2 checks pass default
210 / opt-in 216 Rust tests, formatting/Clippy and actual IPC. Optimized baseline/
prototype sweeps at 1K/10K/100K, changing-key controls and allocation profiling ran
sequentially on the same Ryzen 9 3950X host. Native sampling required recording on
Linux /var/tmp and offline address resolution; see the
[research handoff](handoffs/2026-10-19-scaling-feasibility.md) for exact evidence,
profiler limitations and the incomplete RSS-bounded 1M diagnostic. Native macOS
corroboration of this experiment remains pending; prior Slice B results below
are historical. No new Linux .NET, GUI, AOT or durability guarantee is claimed.

## Spreadsheet scalar formulas Slice B — 2026-10-19

Full Windows CI passes 209 Rust tests, .NET Release, decimal/protocol parity and
real named-pipe IPC/REPL. Ubuntu WSL2 formatting, Clippy, 210 Rust tests and real
Unix-socket IPC/REPL also pass, with the existing disk-backed Linux file-test root.
See the [Slice B handoff](handoffs/2026-10-19-spreadsheet-formulas-slice-b.md) for
exact reports and separated parser/resolver/edit/capture measurements. Native
macOS ARM64 full CI now passes too: 210 Rust tests, .NET Release, decimal/protocol
parity and actual IPC/REPL. No new Linux .NET, GUI or Native AOT run is claimed.

### Native macOS follow-up — 2026-10-19

macOS 27.0.1 (26A434), arm64, Rust 1.101.0-nightly (b6eeb93c9, 2026-10-18),
Cargo 1.101.0-nightly (3960fa69b), Python 3.9.6, .NET SDK 10.0.402.
The current SMB working tree, including uncommitted/untracked sources, was copied
to `/private/tmp/pivot-macos-validation.cronqr_4/source` on confirmed local APFS.
Cargo targets and all .NET bin/obj outputs stayed local; only reports returned to
`artifacts/macos-validation-20261019T093243Z` on the share. The 215-file SHA-256
manifest matches this working tree before these documentation-only updates.

Full CI and the separate formula cost run both exited zero; no product changes,
test weakening or CI reruns were needed. Two filesystem preflight attempts failed
before builds, then APFS verification succeeded against the Data mount point.
The 210-test count already includes the cost test; its separate run adds no test.
See the [tracked review](handoffs/2026-10-19-macos-validation.md),
[original Mac report](../artifacts/macos-validation-20261019T093243Z/macos-validation.md)
and [CI summary](../artifacts/macos-validation-20261019T093243Z/ci/20261019T093324130572Z/summary.json).
Native GUI/AOT, actual cross-user denial and power-loss durability remain untested.
Older sections below describe evidence available at their original milestone;
this current suite also reruns their retained regression tests.

## Sparse worksheet projection Slice A — 2026-10-19

Full Windows CI passes 200 Rust tests, .NET Release, decimal/protocol parity and
actual named-pipe IPC/REPL checks. Available Ubuntu WSL2 formatting, Clippy,
201 Rust tests, small-workload measurements and Unix-socket IPC/REPL also pass.
See [worksheet Slice A handoff](handoffs/2026-10-19-workbook-projection-slice-a.md)
for exact reports and the deliberately small, full-copy proof limitations.
Existing file regressions retain their disk-backed Linux test root. macOS was
pending at submission; the follow-up above passes the retained tests on APFS.
No new Linux .NET, GUI or Native AOT run is claimed.

## Experimental model files — 2026-10-19

Slice B passes full Windows CI with 185 Rust tests, .NET Release, cross-language
parity and actual named-pipe IPC/REPL. Available WSL Linux formatting, Clippy,
186 Rust tests and actual Unix-socket IPC/REPL also pass. The extra test covers
Unix permissions and aliases. See [Slice B evidence](handoffs/2026-10-19-document-foundation-slice-b.md).

Windows files were tested on the local fixed NTFS J: volume. Final Linux file
checks used `/var/tmp/pivot-experimental-file-tests` on the native disk filesystem
(`stat` reports ext2/ext3), selected via PIVOT_FILE_TEST_ROOT because `/tmp` is tmpfs.
This proves the bounded experimental workflow across processes, not power-loss
or OS-crash durability. macOS was pending then; the follow-up above now covers
the retained tests on APFS. Linux .NET was
not rerun for this Rust-only change; no production document format is selected.

## Document Foundation Slice A — 2026-10-18

Full Windows CI passes with 167 Rust tests, .NET Release, decimal/protocol parity
and executable IPC/REPL checks. WSL Linux formatting, Clippy, the 167-test Rust suite
and real IPC harness also pass. See the
[Slice A handoff](handoffs/2026-10-18-document-foundation-slice-a.md) for exact reports.
This validates hosting-independent in-memory documents and hosted compatibility,
not native file I/O, a document container or database durability. No Linux .NET
rerun was needed for this Rust-only change; macOS was pending at submission and
is now covered by the follow-up above.

## Portable storage foundation — 2026-10-18

Windows full CI and WSL Linux Rust/IPC checks cover the portable record codec and
validated restoration foundation, including its frozen compatibility corpus.
See the [storage foundation handoff](handoffs/2026-10-18-portable-storage-foundation.md)
for exact reports. This validates portable in-memory restoration, **not SQLite or
database durability**. macOS was pending at submission; its retained tests now
pass in the follow-up above.

## Minimal REPL authoring — 2026-10-18

Full Windows CI and available WSL Linux Rust/IPC checks cover the new authoring
commands, mutation recovery, conflicts and executable client tests. The Rust suite
has 135 tests. The real-process harness exercises both ordinary invocation and
authoring REPL workflows. See the [REPL authoring handoff](handoffs/2026-10-18-minimal-repl-authoring.md)
for exact evidence. macOS ARM64/APFS was pending at submission; the follow-up
above now covers Stage B and the REPL.

## Stage B authoring — 2026-10-18

Full local CI passed on Windows and Ubuntu WSL2, including the new daemon
authoring tests over named pipes and Unix sockets. The final Rust suite has
130 tests. Linux formatting, Clippy and all Rust tests were rerun after the final
embedded message-limit correction. See the [Stage B handoff](handoffs/2026-10-18-daemon-authoring.md)
for exact reports and coverage. .NET SDK versions were 10.0.402 on Windows and
10.0.112 in WSL. The original macOS validation below predates authoring; the
2026-10-19 follow-up above now covers it.

## Verified coverage

Windows named pipes and Unix-domain sockets on Ubuntu WSL2 Linux and macOS ARM64 have passed
the full local CI pipeline: Rust tests/lint/formatting, .NET Release solution build,
10,099 decimal compatibility vectors, generated Rust/C# protocol fixtures, and real
daemon/CLI integration. Linux uses .NET SDK 10.0.112; macOS uses 10.0.402.

This verifies Linux code running on the WSL2 Linux kernel, not merely a Windows
binary invoked from a Linux shell. Socket endpoints are created in Linux `/tmp`.
Cross-user access denial, Native AOT, and native Linux deployment remain separate
coverage items. GUI execution is not part of the pipeline.

The Linux .NET run used an isolated source copy under
`artifacts/linux-dotnet-20261018T164648Z/source` so Windows bin/obj outputs were not
overwritten. Its CI report is
`artifacts/linux-dotnet-20261018T164648Z/ci/20261018T164652188086Z/summary.json`.

## macOS

In the Mac's VS Code terminal, change into the Pivot repository root. Prerequisites
are Rust/Cargo with rustfmt and Clippy, a working native linker/C toolchain, and
Python 3.8+. No .NET SDK is needed for these Rust/IPC checks.

```sh
export CARGO_TARGET_DIR="$PWD/target/macos"
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
python3 tests/ipc/verify.py --output-dir artifacts/ipc-macos
```

The separate target directory prevents shared-folder build collisions with Windows
or Linux. The IPC script discovers Cargo's actual output directory automatically.
It starts a unique local daemon, runs CLI/protocol checks, and cleans up the process
and endpoint. Inspect `artifacts/ipc-macos/results.txt` and `daemon.log` afterward.
The Unix permission test also runs as part of `cargo test` on macOS.

With a native .NET 10 SDK, full CI can also run as `python3 scripts/ci.py`. Use a
separate local checkout for full multi-OS CI, since .NET bin/obj outputs are currently
shared by path and should not be built concurrently across operating systems.
Use local APFS storage for source and build outputs on macOS. The SMB-mounted
workspace failed in `GenerateDepsFile` with a file-sharing error on
`Pivot.Client.dll`; a fresh local APFS copy passed the unchanged pipeline.
A sandbox that forbids Unix socket binding must allow that capability for the
Rust socket test and IPC harness; do not weaken endpoint checks to bypass it.

### Actual macOS run — 2026-10-18

macOS 27.0.1 (26A434), native ARM64; Apple Clang 21.0.0; Python 3.9.6;
Rust 1.101.0-nightly (8006091eb, 2026-10-17); Cargo 1.101.0-nightly
(3960fa69b, 2026-10-12); .NET SDK 10.0.402.

Full `python3 scripts/ci.py` passed from the isolated APFS source copy
`/private/tmp/pivot-macos-validation-20261018T171109Z/source`, with
`CARGO_TARGET_DIR=/private/tmp/pivot-macos-validation-20261018T171109Z/target/macos`.
The copy excluded `.git`, `target`, `artifacts`, `bin`, and `obj`.
All 41 Rust tests passed, including Unix endpoint permissions, same-user peer
credentials, endpoint exclusivity, and replacement-safe cleanup. The five-project
.NET Release solution build had zero warnings/errors; all 10,099 decimal vectors
and 58 protocol decimal layouts passed. Real daemon/CLI integration passed,
including malformed traffic survival, concurrent clients, and graceful cleanup.
No product code changes were required. ADR-0005 remains Proposed.

Report: `artifacts/macos-validation-20261018T171109Z/ci/20261018T171247760372Z/summary.json`.
The run directory contains detailed gate logs, parity reports, and
`ipc/results.txt` plus `ipc/daemon.log`; the latter records READY, expected
malformed-client rejections, and STOPPED. Earlier failed reports are retained:
`20261018T171120582806Z` (sandbox denied socket binding) and
`20261018T171208467020Z` (SMB .NET file-sharing failure).
Native AOT, GUI execution, and actual cross-user denial were not run.

## WSL2 rerun

From a PowerShell terminal on this development machine:

```powershell
wsl.exe -d Ubuntu -- bash -lc 'cd /mnt/j/coding/pivot && export CARGO_TARGET_DIR=/mnt/j/coding/pivot/target/linux-wsl && cargo fmt --all -- --check && cargo clippy --locked --workspace --all-targets -- -D warnings && cargo test --locked --workspace && python3 tests/ipc/verify.py --output-dir artifacts/ipc-linux-wsl'
```

Paths and distro name are specific to this machine. Dependencies and Linux compiler
tools live inside WSL, while build products remain in the ignored Linux target
subdirectory. Windows builds continue using their existing target directory.


## Controlled core integration — Stage 2 (2026-10-19)

The [Stage 2 report](performance/controlled-core-integration-stage2.md) records
Windows/Linux feature regression, complete default CI and isolated Release scaling.
Native macOS ARM64 Stage 2 validation now passed; earlier Mac results remain historical. Copy the source from SMB to local APFS before
building or testing, especially .NET, then return logs, source hashes and summary
to the SMB artifacts directory. No native Mac performance claim is made here.

Final Stage 2 Windows/Linux results: full default CI passed, including .NET and real IPC; combined feature regression passed 259/260 tests. All thirty Release processes on each platform passed with matching source manifests. Native macOS ARM64 Stage 2 subsequently passed; see below.


Native macOS Stage 2 validation (20261019T182116Z) passed on Apple M1 ARM64 from
an isolated internal APFS copy: 210 default, 237 integration-only, 234 research-only
and 260 combined tests, three strict feature lint configurations, complete .NET/
parity/real IPC CI and all thirty Release benchmark processes. All copied source
hashes matched before this documentation update; no product changes were needed.
All review targets passed and final domain charges were zero. The runner does not
support macOS RSS; separate sparse native samples do not establish child high-water
marks. See the [native report](../artifacts/macos-validation-20261019T182116Z/REPORT.md)
and [Stage 2 performance addendum](performance/controlled-core-integration-stage2.md).

Standalone transition final Windows/Linux results: full default CI passed (241/242 tests), combined regression passed 264/265, reference-only core passed 21 on both, all strict lint configurations passed and all sixty verified Release children passed the interactive targets. Native macOS transition subsequently passed (242 default, 265 combined and 21 fallback tests, full CI and thirty more Release children); see the transition handoff for current three-platform results and the retained bulk-construction tradeoff.


Native standalone transition follow-up (20261019T191136Z): all requested gates
passed on internal APFS, with no product changes or reruns. Full CI included .NET,
10,099 decimal vectors, protocol compatibility and actual Unix-socket IPC; all
thirty native benchmark children passed with zero convenience/session final domain
charges. Native 100K p95 input/capture was 0.013 ms, input/chain 0.331 ms, tail
formula 0.491 ms and bounded inspection 0.007 ms. Bulk fixture construction still
cost 1.300 seconds versus 0.0467 reference; full snapshot 48.748 ms versus 30.313
reference. Native RSS guard/high-water support remains unavailable; separate
monitoring is sampled evidence only. All three-platform validation gates are now
satisfied for parent review; no downstream backend migration is included.
