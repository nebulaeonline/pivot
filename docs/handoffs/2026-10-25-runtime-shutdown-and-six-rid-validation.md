# Runtime shutdown correction and six-RID headless validation

Date: 2026-10-25. Parent review accepted and closed the six-RID headless milestone,
runtime ordering correction and cancellation test corrections. GUI/Native AOT
integration is authorized next. See [closure and next scope](2026-10-25-six-rid-headless-closure.md).
G1/G2 remain closed; G3 remains held.

## Final result

One unified matrix passed all 13 existing gates on all six designated targets:
`artifacts/validation-matrix/20261026T002242Z-f76230c2/`.
Original per-target CI summaries/logs and machine-readable results are retained
there. All 78 steps passed. ROBPC dispatched and collected; no local ROBPC/WSL
result was substituted. Windows work roots are NTFS, Ubuntu ext4 and macOS APFS.
Native target execution is verified; macOS x64 is explicitly Rosetta execution
with an ARM64 Python controller and a verified x86_64 Rust release binary.

| RID | Environment | .NET SDK | Headless CI |
| --- | --- | --- | --- |
| win-x64 | Hyper-V Windows x64 | 10.0.402 | Passed 13/13 |
| linux-x64 | Hyper-V Ubuntu x64 | 10.0.403 | Passed 13/13 |
| win-arm64 | UTM Windows ARM64 | 10.0.403 | Passed 13/13 |
| linux-arm64 | UTM Ubuntu ARM64 | 10.0.113 | Passed 13/13 |
| osx-arm64 | Mac mini native ARM64 | 10.0.402 | Passed 13/13 |
| osx-x64 | Mac mini Rosetta x64 | 10.0.403 | Passed 13/13 |

## Runtime correction

The original Linux ARM64 exit 101 at
`late_selection_walk_cancellation_preserves_all_roots` is preserved under
`artifacts/validation-matrix/20261025T201831Z-64511277/`.
Twenty-five isolated instrumented attempts passed and did not reproduce that
historical assertion. Inspection then identified a concrete completion race:
`Task::drop` notified lane-drain waiters before automatically dropping the task's
cleanup ticket. A waiter could observe a drained lane while that task still
counted as a live cleanup owner, causing `finish()` to refuse shutdown.

The minimal production fix explicitly drops the task ticket before decrementing
pending and notifying waiters. `finish()`, resource limits and the original
assertion are unchanged. Rejected payloads retain the existing reclamation path.

The controlled regression holds cleanup at a mutex boundary and checks that the
actual lane flush cannot complete before ticket release. The pre-fix ordering
failed the regression on native ARM64. A final counterexample run also restored
only the old ticket order in an isolated native source copy and failed the final
actual-flush regression (exit 101). The working source and passing matrix were
unchanged. All 27 library tests passed on ARM64 and
x64 after correction. The final regression calls the real flush method rather
than duplicating its wait loop. The original CI log cannot identify its exact
finish branch: the regression proves this defect and failure mechanism, without
retrospectively proving which branch caused the historical failure.

## Test-lifecycle corrections

Broader testing exposed two independent timing assumptions; their failures are
preserved and their behavioral assertions remain in place.

The Rust partial-traversal cancellation fixture created 2,047 scalar edits without
draining bounded asynchronous cleanup. It failed during setup with cleanup
capacity exhausted. The fixture now builds the same 1,024-node dependency chain
in one existing batch. Cancellation, unchanged revision, scratch reclamation,
immutable capture results, successful shutdown and zero charges remain asserted.
No production capacity increase or retry was added.

The application-session test cancelled immediately after starting a small request,
then required cancellation even if the request had already completed. On ARM64 it
failed at Expected cancellation; the unhandled .NET exception produced SIGABRT
and the owner-visible crash popup. The test now cancels at a deterministic point
after a real request is flushed and before its response is read. A nullable internal
hook, accessible to the ApplicationSession friend assembly, provides synchronization;
ordinary clients leave it null. Apply, capture and calculate still use the real
Rust child. Assertions still require draining the reply, known committed state,
accepted edits, abandoned capture release and next-frame alignment. Pre-dispatch
cancellation, public APIs and production cancellation semantics are unchanged.

## Independent Linux x64 CLR abort

The first unified matrix, `20261026T000710Z-91cb828d`, finished with five Passed
and Linux x64 Failed. Ubuntu SDK10.0.113/runtime10.0.13 aborted inside CoreCLR
during dotnet build with 0x80131506. Selected Apport fields confirm the native
abort without establishing its cause. An unchanged focused build later passed;
no OOM records were found, but later free-memory measurements cannot rule out
earlier pressure. This is separate from ARM64's managed test exception.
The original matrix remains Failed.

The owner approved a private Microsoft .NET SDK installation. SDK10.0.403 with
runtime10.0.14 was downloaded using Microsoft's release metadata, SHA512-verified,
and installed under `~/PivotValidation/toolchains/dotnet-linux-x64-10.0.403`.
Ubuntu's system installation was left unchanged. The Linux x64 profile explicitly
selects it through `dotnet_root`; the worker validates the executable and sets its
private PATH/DOTNET_ROOT variables before tool verification. The private SDK passed
focused build/application checks, all 13 Linux x64 gates, and the final matrix.
This verifies the chosen toolchain; it does not establish the old CLR abort's cause.
Private crash-dump directories were not exported as shared memory dumps.

## Scheduling and GUI boundary

Owner clarified that two headless workers is a Mac-only limit, shared by native
macOS ARM64, Rosetta x64 and both UTM guests. Profiles now assign those targets to
`mac-host` with capacity two. The default global ceiling is six, leaving Hyper-V
jobs independent. Fourteen focused runner tests pass, including controlled Mac
capacity and independent Hyper-V execution, invalid-limit rejection, private SDK
selection and unchanged global environment.

Discovery-only integration with the updated scheduler succeeded on all six at
`artifacts/validation-matrix/20261026T004147Z-5b250c62/`; its summary records global
ceiling six and mac-host capacity two. Discovery is not an additional CI pass.
The completed functional matrix began with the earlier global-two invocation;
both Hyper-V targets had finished when the owner clarified the policy. Remaining
work was already within the Mac cap, so it was not restarted. Application/runtime
and application-test inputs remained fixed; the scheduler change was verified
separately through focused tests and six-target discovery. No claim that the
whole shared tree stayed unchanged throughout validation.

Native macOS ARM64 and Rosetta x64 GUI tests must never overlap, including across
separate invocations: they share one desktop. GUI remains serial with explicit
live consent. The Mac headless capacity of two does not authorize GUI overlap.
No new GUI/AOT coverage is claimed; Native AOT still requires actual publish and
execution. No VM settings, system swap settings, workbook/formula semantics,
persistence formats, protocols or product limits were changed.

## Diagnostic evidence

Focused reports are under `artifacts/runtime-investigation/`:

| Evidence | Directory/file | Result |
| --- | --- | --- |
| Isolated original test | `arm64-runtime-20261025T235402Z` | 25 passed; no isolated reproduction |
| Pre-fix completion regression | `arm64-ticket-regression-before-20261025T235535Z` | Failed, exit 101 |
| Broader Rust fixture failure | `arm64-runtime-after-20261025T235617Z` | Regression passed; fixture setup failed |
| Corrected ARM64 library suite | `arm64-runtime-after-20261025T235718Z` | 27 passed |
| x64 library suite | `x64-runtime-after-20261025T235747Z` | 27 passed |
| Final regression with old order, isolated copy | `arm64-real-flush-before-20261026T004407Z` | Expected failure, exit 101 |
| Final real-flush regression/library suite | `arm64-runtime-after-20261026T001945Z` | 27 passed |
| Dispatch-boundary application checks | `arm64-session-after-20261026T000347Z` | Passed |
| Linux x64 crash diagnostics | `linux-x64-clr-crash.txt`, `linux-x64-clr-apport.txt` | Cause unresolved |
| Private SDK integrity/install | `linux-x64-private-sdk/installation.json` | SHA512 verified, SDK10.0.403 |
| Private SDK focused checks | `linux-x64-private-sdk-probe/` | Build and application checks passed |

The intermediate full ARM64 run at `20261025T235747Z-a4e36648` passed 12 gates
and failed application-session with the managed cancellation exception. Corrected
ARM64 full CI passed 13/13 at `20261026T000426Z-73a1b9dd`. Private-SDK Linux x64
full CI passed 13/13 at `20261026T001734Z-bc13162c`. These matrix directories are
under `artifacts/validation-matrix/`. The final six-target matrix above validates
the final application/runtime inputs together; earlier passes do not replace it.
