# Linux and macOS validation

## Current: persistent workbook readiness (2026-10-19)

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
