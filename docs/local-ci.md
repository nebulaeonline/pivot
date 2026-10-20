# Local CI

Run from the repository root:

```text
python scripts/ci.py
```

This is an on-demand local pipeline. It requires no CI server, container runtime,
cloud account, or Python packages. It can later be called unchanged by a hosted
runner or local build service.

## Prerequisites

- Python 3.8 or newer, available as `python`.
- Rust/Cargo with the `rustfmt` and `clippy` components.
- .NET 10 SDK. `global.json` selects an installed stable .NET 10 feature band.
- A C# desktop build environment appropriate for the host OS.

Package access is needed on the first run. Builds use normal Cargo and NuGet
caches. Rust tooling is not yet pinned; the actual versions are recorded in logs.
Cargo gates use the committed lockfile. NuGet dependencies are not yet lockfile
enforced, so the pipeline is not a hermetic build.

## Gates

1. Record Rust, Cargo, .NET, and Python versions.
2. Check Rust formatting without modifying source files.
3. Run Clippy across workspace targets with warnings treated as errors.
4. Run all Rust workspace tests, including documentation tests.
5. Build the full .NET solution in Release with warnings treated as errors.
6. Run all 10,099 Rust/C# decimal compatibility cases using the Release C# harness.
7. Generate and exchange Rust/C# invocation and authoring fixtures, including
   frozen-v1.0 codec/current-host and descriptor compatibility checks.
8. Start a temporary demo daemon and test IPC/CLI/REPL round trips, extended idleness,
   malformed-client survival, and explicit reconnect without ambiguous invocation replay.

The runner stops at the first failed command and returns a nonzero exit status.
It streams output to the console and preserves each command's output in a log.
No commits, hooks, deployments, or background services are installed.

## Reports

Each invocation creates a unique timestamped directory under `artifacts/ci/`:

```text
artifacts/ci/<run>/
  summary.json
  rust-format.log
  rust-lint.log
  rust-tests.log
  dotnet-build.log
  decimal-parity.log
  decimal-parity/
    results.txt
    backend-baseline.txt
```

The summary records command arguments, exit codes, durations, and log paths for
executed steps. An early failure leaves later gates unexecuted. Python/platform
information is included. Missing required tools produce a failed summary.
Reports are ignored by Git and accumulate until manually removed.

To choose another report root:

```text
python scripts/ci.py --output-dir artifacts/my-ci
```

The compatibility checker now accepts `--output-dir` and `--configuration`.
CI writes its generated reports into the run directory, preserving the reference
reports in `tests/decimal-parity/`.

## Current coverage limits

Workspace Rust tests now include the storage-codec compatibility corpus and
portable restoration and in-memory document-session tests. They do not exercise
a production document format, create a database or prove power-loss durability.
The experimental file tests do exercise headless disk round trips, independent
processes, pre/post-install failures and conservative uncertain-save recovery.
Run `cargo test --locked -p pivot-storage` for the focused foundation checks.
The workspace also tests pivot-workbook's bounded single-sheet proof and scalar
text formula parser/binder. Run
`cargo test --locked -p pivot-workbook` for its focused semantic checks and
`cargo test --locked -p pivot-workbook --test costs -- --nocapture` for small
unoptimized measurements. These are not scale benchmarks or workbook-file tests.

The full pipeline is verified on Windows with .NET 10.0.402 and Ubuntu WSL2 Linux
with .NET 10.0.112, and macOS 27.0.1 ARM64 with .NET 10.0.402, using installed
Rust nightly toolchains. Linux and macOS used isolated source/build outputs.
For the current formula Slice B, Windows and native macOS full CI passed 209/210
Rust tests respectively; WSL2 Rust/IPC checks passed 210. Linux .NET's full-pipeline
evidence predates this slice. The [latest Mac review](handoffs/2026-10-19-macos-validation.md)
links its CI and separate cost reports copied back to the shared artifacts directory.
On macOS, use local APFS storage: the SMB-mounted source copy failed with a
.NET file-sharing error; the local APFS copy passed all gates. See
[platform validation](platform-validation.md) for reproducible commands.
GUI execution and Native AOT publishing are not gates yet. There is no automatic
trigger: run the command before handoff or wire it into future CI hosting.

Formula costs: `cargo test --locked -p pivot-workbook --lib report_text_formula_costs -- --nocapture`
separates parser and existing-anchor resolution costs from full formula/input edits,
snapshot capture and evaluation. See the [Slice B note](architecture/spreadsheet-scalar-formulas.md).

## Opt-in scaling research

The [compatibility foundation](performance/compatibility-foundation.md) adds
`python scripts/foundation.py` for process-isolated general-core name, edge, port,
repair, rejection, churn and reclamation benchmarks. It shares the existing
scaling-experiment feature and analysis script. It is not a default CI gate or a
production backend switch. The feature suite currently passes 233 Windows / 234
Linux tests; default Windows full CI and Linux default/IPC checks are retained.

The newer [structural experiment](performance/structural-experiment.md) runs with
`python scripts/structural.py`. It isolates baseline and paged implementations in
separate guarded processes and normalizes the complete formula-edit/validation/
capture/evaluation boundary. `python scripts/structural_analyze.py <run-directory>`
produces full percentile tables. These remain manual research checks, not additions
to default CI or production-capacity guarantees. The same scaling-experiment feature
enables both generations of research and their correctness tests.

Default CI retains production behavior and admission limits. Run
`cargo test --locked --workspace --features pivot-workbook/scaling-experiment`
and matching workspace Clippy/all-targets for the isolated prototype and six extra
correctness tests. `python scripts/scaling.py` builds an optimized Release harness
and writes latency/allocation/phase/environment evidence separately from CI.
See the [specification](performance/scaling-experiment.md) and
[assessment](architecture/graph-workbook-scaling-feasibility.md). These are manual
research runs, not mandatory latency assertions in ordinary CI.

On this Windows machine the pre-existing debug target's generated proc-macro DLLs
could not be overwritten (LNK1104), even outside the sandbox. A fresh target directory
`target/scaling-windows-checks` passed the checks; do not weaken tests or delete
loaded binaries. Set CARGO_TARGET_DIR for CI and its subprocesses when reproducing.

## Controlled core integration — Stage 1

The `core-integration` feature opts into the WorkingModel activation/capture seam;
it does not enable the public research Graph API or change the default constructor.
Run both the core-only configuration and the combined regression configuration:

```text
cargo clippy --locked --workspace --all-targets --features pivot-core/core-integration -- -D warnings
cargo clippy --locked --workspace --all-targets --features pivot-core/core-integration,pivot-workbook/scaling-experiment -- -D warnings
cargo test --locked --workspace --features pivot-core/core-integration,pivot-workbook/scaling-experiment
```

The combined suite passes 246 tests on Windows and 247 on Linux WSL2. Full default
CI also passes on both platforms, including .NET, numeric/protocol parity and real
IPC. Windows named-pipe tests require execution outside the restricted sandbox.
Use a distinct Windows target directory if generated DLLs are held open; Linux
file tests use the disk-backed PIVOT_FILE_TEST_ROOT documented above.

`python scripts/integration.py` compares optimized full facade operations at
1K/10K/100K, with 16/32 retained structural captures, allocation counters, RSS
sampling and complete reclamation. Run benchmark processes sequentially and keep
regression builds separate from measured latency. See the
[specification](performance/controlled-core-integration.md) and
[Stage 1 handoff](handoffs/2026-10-19-controlled-core-integration.md).


## Controlled core integration — Stage 2

Stage 2 keeps the core-only and combined feature commands above. Also lint the
research-only configuration to catch accidental feature coupling:

```text
cargo clippy --locked --workspace --all-targets --features pivot-workbook/scaling-experiment -- -D warnings
python scripts/integration_stage2.py --output-dir artifacts/stage2
```

The Stage 2 harness covers standalone editing, ports, initially-invalid repair,
local capture, bounded inspection, complete diagnosis/snapshots, admitted transfer,
contention and retained generations. Run all benchmark processes sequentially;
keep regression builds outside the latency runs. See the
[Stage 2 report](performance/controlled-core-integration-stage2.md) for final
platform results, source manifests and proposed budgets. Default CI still checks
reference behavior and all .NET/parity/real IPC gates. Native macOS requires a
fresh local APFS source copy; do not run .NET build/tests on the SMB share. Write
returned evidence to the shared artifacts directory.

Stage 2 final results: 259/260 combined feature tests (Windows/Linux), 209/210 default tests, three strict feature lint configurations, full default CI and all thirty Release processes per platform passed. Native macOS ARM64 Stage 2 passed; see the validation update below.


Native macOS Stage 2 validation (20261019T182116Z) passed on Apple M1 ARM64 from
an isolated internal APFS copy: 210 default, 237 integration-only, 234 research-only
and 260 combined tests, three strict feature lint configurations, complete .NET/
parity/real IPC CI and all thirty Release benchmark processes. All copied source
hashes matched before this documentation update; no product changes were needed.
All review targets passed and final domain charges were zero. The runner does not
support macOS RSS; separate sparse native samples do not establish child high-water
marks. See the [native report](../artifacts/macos-validation-20261019T182116Z/REPORT.md)
and [Stage 2 performance addendum](performance/controlled-core-integration-stage2.md).


## Standalone default transition

Core integration is now a default pivot-core feature: default CI exercises persistent
standalone construction as well as downstream explicit reference boundaries.
Run the transition [validation commands and benchmark harness](performance/standalone-default-transition.md),
including a true core fallback test/lint with `-p pivot-core --no-default-features`.
Workspace feature unification can enable integration through other dependencies;
explicit new_reference is the downstream compatibility policy, not reliance on
feature absence. The research-only lint uses `-p pivot-core --no-default-features
--features scaling-experiment`. Full default CI and combined tests remain required.
Native macOS must validate a fresh local APFS copy of this transition source;
[copy-ready instructions](handoffs/2026-10-19-macos-standalone-transition-instructions.md)
include all feature gates, fallback, benchmarks and SMB report return.

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


## Opt-in persistent workbook and bulk foundation

Default CI retains reference Workbook coverage. Additionally run:

```sh
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo test --locked -p pivot-workbook --features persistent-core
cargo test --locked -p pivot-core --no-default-features
```

Current all-feature counts are 296 Windows / 297 Linux / 297 native macOS, fallback 21 each. Keep true
fallback/research-only core lints as above. Run benchmarks only after all builds and
correctness processes finish; see [method/evidence](performance/persistent-workbook-bulk.md).
Native validation requires a fresh APFS working-tree copy; the previous transition
report is not evidence for this implementation. Use the
[new packet](handoffs/2026-10-19-macos-persistent-workbook-bulk-instructions.md).

Final bulk-foundation full CI passes on Windows/Linux (248/249 default Rust tests),
including .NET, 10,099 decimal vectors, frozen C# protocol and actual local IPC.
Combined feature counts are 296/297, fallback 21 each and persistent-workbook-only
48 each; all listed strict lint configurations pass. The 226 sequential Release
children also pass; see the linked performance record for raw reports and caveats.
Native macOS now passes full CI (249 default / 297 all-feature / 21 fallback /
48 workbook-only), all required strict lint gates and 109 Release/diagnostic
children, bringing the three-platform total to 335. The fresh internal APFS copy
matched all 290 source hashes before this documentation update; no native source
fixes or reruns were needed. See the
[native report](../artifacts/macos-validation-20261019T220845Z/REPORT.md). Warning-level
memory pressure was observed; native RSS samples are not child high-water and the
runner RSS guard remains unavailable. The APFS/SMB packet above remains the
reproduction procedure.


## Persistent workbook readiness follow-up

ADR-0012 is accepted. For the subsequent ownership-aware private-index change,
repeat full default CI, all-feature workspace strict lint/tests, true core fallback
and research-only lint, and persistent-workbook-only coverage. The
[readiness harness commands](performance/persistent-workbook-readiness.md) run 78
sequential children per platform, separating ordinary timing from profile probes
and retaining the immutable insertion control. Run all correctness/build processes
before latency measurements. The new Unix wait4 fields record completed-child CPU
and RSS high-water; they do not supply macOS live RSS guarding. Native validation
requires the [new APFS packet](handoffs/2026-10-19-macos-persistent-workbook-readiness-instructions.md),
not reuse of the accepted 335-child foundation evidence.

Readiness final Windows/Linux/macOS validation passes: default CI 251/252/252 Rust tests,
all-feature regression 300/301/301, core fallback 21 each and workbook-only 49 each.
All strict configurations above pass, along with .NET/parity/real IPC and 234
source-matched ordinary/profile/core children. All final domain charges are zero.
The [fresh native readiness report](../artifacts/macos-validation-20261020T010244Z/REPORT.md)
passes all 78 children and native CI on internal APFS, with 296 matching source hashes
before documentation updates and no source fixes/reruns. All 52 native pressure samples
were normal; wait4 high-water remains post-exit evidence, not a live RSS guard.
The accepted foundation's 335 children remain separate historical evidence.
