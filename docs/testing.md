# Testing Pivot

Contributor builds and tests do not require a VM, Hyper-V, UTM, SSH or remote
execution software. Use Python 3.8+, Rust with rustfmt/clippy, stable .NET 10 SDK
matching `global.json`, and the native compiler/linker required by your platform.
First-time dependency restore requires package access.

From the repository root:

```sh
python scripts/ci.py
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
```

Ordinary CI runs headless, including .NET builds, decimal/protocol parity, real
IPC/session tests and fixture-contained failure tests. It includes the execution-
safety tests below. Building the desktop project does not launch it. Each CI run
has a unique report directory under `artifacts/ci`; inspect its `summary.json`.
Missing build tools remain genuine prerequisites, independent of VM availability.

## Deliberate local desktop tests

Choose platform coverage from the changed behavior and architecture exposure;
do not indefinitely substitute macOS ARM64 for Windows/Linux ARM64 GUI coverage.
Review each target's most recent managed/AOT GUI receipts when choosing the matrix.
Owner direction for ADR-0031 and the accepted ADR-0032 validation plan require managed **and Native AOT** GUI execution on
Windows x64, Linux x64, Windows ARM64, Linux ARM64, macOS ARM64 and macOS x64.
This milestone covers all six configured targets; future runs choose applicable
coverage. macOS x64 under Rosetta remains separately identified emulated evidence.
Keep short target-local launchers and the
normal interactive consent policy on every selected target.

```sh
python scripts/ci.py --desktop
python tests/application-session/ui_smoke.py --output-dir artifacts/application-ui
python scripts/verify_native_aot.py
```

These commands open real application windows, change focus and use the system
clipboard. The AOT command publishes **and executes** graphical acceptance; it is
not a headless build gate. A local interactive terminal must show the warning and
receive `y` or `yes` before execution starts. Leave other applications alone for
the duration of the test. The clipboard lock only serializes cooperating suites;
it does not protect unrelated applications or preserve every clipboard format.

No automatic `--yes`, CI-variable, hostname or isolation-flag bypass exists.
Redirected/noninteractive input or output, declined consent and invalid/expired
permits return exit **2** with a machine-readable **Blocked** result. This is an
execution-policy result, not evidence of a product test failure or pass. Headless
tests continue to work without consent or VM configuration.

The wrapper keeps a short-lived, loopback-only consent broker alive. Nested GUI
runners and native/managed clipboard peers validate the live capability; they do
not prompt repeatedly. Direct self-check activation through `PIVOT_UI_*` variables
without a valid broker refuses **before Avalonia initialization**. Use the supported
wrapper rather than hand-setting self-check environment variables. Once a suite
ends its permit is invalid; don't copy it into shell configuration or scripts.

For attended validation, provide short target-local launchers and a text file of
commands under `artifacts/`. The owner cannot paste commands into the ARM64 targets
or Ubuntu x64; do not require transcription of long frozen-source paths or run IDs
from another screen. The current capacity launcher is `~/PivotValidation/cap.py`:
after `cd ~/PivotValidation`, run `python cap.py` on Windows or `./cap.py` on Linux
and macOS. Keep the ordinary runner consent prompt and prevent duplicate attempts.
Update the generated launcher for a new prepared run; retain the previous evidence.
ADR-0030 uses `~/PivotValidation/fmt.py`: `python fmt.py` on Windows, `./fmt.py`
on Linux/macOS. It runs the prepared managed suite then Native AOT, keeping the
ordinary consent prompt for each and refusing duplicate/failed attempts. The short
commands are also in `artifacts/formatting-commands.txt`.

ADR-0031 uses `~/PivotValidation/rows.py`: `python rows.py` on Windows and
`./rows.py` on Linux/macOS. Windows/Linux run managed then Native AOT. The Mac
launcher runs ARM64 managed/AOT, then x64 managed/AOT under Rosetta, with a normal
consent prompt for every suite. Run one target at a time; shared-host contention
can distort capacity measurements. Commands are in `artifacts/structure-commands.txt`.

ADR-0032 uses `~/PivotValidation/sort.py`: `python sort.py` on Windows and
`./sort.py` on Linux/macOS, with commands in `artifacts/sorting-commands.txt`.
It follows the same managed/AOT ordering and normal local consent. Run only the
targets currently requested; a failed pilot stops the rollout until inspected.

`python scripts/run_desktop.py` remains the ordinary manual application launcher.
It strips inherited automation activation/permits and does not run acceptance.
No normal workbook editing, file picker or clipboard behavior was changed.

## Test the policy without opening windows

```sh
python tests/execution-safety/verify.py
```

This builds a small console probe linking the actual C# pre-start guard, then runs
Python routing/refusal tests and managed probe subprocesses. It has no Avalonia
dependency and never accesses the clipboard or injects input. Interactive answers
are mocked only inside these synthetic tests; they grant no real desktop run.

## Optional isolated validation

VM validation is an optional future orchestration capability. No VM-selection
flags or remote services have been added. All unattended graphical execution is
currently blocked until an isolated guest profile has been independently verified
and its activation reviewed. Installing or naming a VM is insufficient.

Owner acceptance can require more platforms than an individual contributor can
run. A missing optional guest is NotConfigured, not a platform pass or a failed
product test; failures on a configured/executed target must remain failures.
There is never an automatic fallback to the host desktop.

See the [execution-policy design and audit](architecture/validation-execution-safety.md)
for the owner matrix, verification requirements and deferred work. Prior milestone
receipts remain historical evidence, not authorization to run old GUI commands
unattended on an active desktop.


## Artifact retention

Follow [build output and validation artifact retention](artifact-retention.md):
keep at most two completed run product sets per RID/suite, discard historical
compiler caches, and archive older closed evidence with verified manifests.
Collect results and verify retained archives before pruning; keep active runs and live configuration. Perform this cleanup at the end of each validation batch, including failed batches once their diagnostics are collected. Record native free-space recovery and archive locations with the batch evidence. Check free
space before full batches and perform maintenance at milestone close and weekly.
Historical archives do not create a requirement to rerun accepted tests.
