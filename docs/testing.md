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
Collect results before pruning; keep active runs and live configuration. Check free
space before full batches and perform maintenance at milestone close and weekly.
Historical archives do not create a requirement to rerun accepted tests.
