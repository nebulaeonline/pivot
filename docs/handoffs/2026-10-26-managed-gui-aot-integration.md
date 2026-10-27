# Managed GUI and Native AOT integration

All attended runs have been collected: managed GUI passed 6/6; Native AOT passed
5/6, with Windows ARM64 failing the clipboard UI self-check after publication.
See [final collected results and failure evidence](2026-10-26-gui-aot-results.md).
The accepted six-RID headless gate was
not reopened or rerun. No workbook, graph, formula, persistence, protocol or
product-limit changes. G1/G2 remain closed; G3 remains held.

## Implemented

The unified runner prepares managed/AOT work through the existing profiles and
records session/compiler prerequisites without starting GUI. Target-specific
activation files and short `scripts/attend.cmd` / `scripts/attend.sh` launchers
run the original suites in the owner's graphical terminal. `both` selects managed
and AOT sequentially, reusing the established live broker; it does not preanswer
consent or create a replacement broker. `mac` serializes native ARM64 and Rosetta.

`--collect` retrieves attended outcomes into a new matrix, preserving previous
plans and results. It checks expected RID/suite/run identity and all four copied
original GUI reports, not merely an exit code or top-level Passed marker. Pending
activation, setup failure and actual functional failure remain distinct.

Read-only prerequisites check the Windows input desktop/session, Mac console
user/Aqua context, or active unlocked Linux graphical login. Initial Ubuntu x64
activation was blocked: its graphical session was active/unlocked Wayland, while
GNOME also used a separate user-manager scope. Local terminals now resolve the
same user's logind primary display; SSH and display-variable-only cases remain
refused. The retried real Ubuntu GUI/AOT suites passed. Blocking diagnostics are
automatically returned to shared storage for future failures.

Both original native-Mac GUI runners now take one fixed machine-wide kernel file
lock across checkouts and independent invocations. Clipboard overrides do not
bypass it. It is held during the four-application window/clipboard suite and is
released on descriptor close/process exit without unlinking its inode. It grants
no GUI authority; the existing live consent is still required.

Automation reports now record actual .NET process architecture for the main app
and every peer. Worker and Native AOT checks require the selected architecture;
Rosetta stays explicitly labeled translated x64. AOT prerequisites select ARM64
VS tools for Windows ARM64 and matching SDK libraries, or clang/native SDK/zlib
requirements on Unix. All six prerequisite probes are ready; probes alone do not
claim publication or execution.

## Collected execution results

| Target | Managed GUI | Native AOT |
| --- | --- | --- |
| Hyper-V Windows x64 | Passed | Passed |
| Hyper-V Ubuntu x64 | Passed | Passed |
| UTM Windows ARM64 | Passed | Failed — clipboard UI self-check |
| UTM Ubuntu ARM64 | Passed | Passed |
| Native Mac ARM64 | Passed | Passed |
| Mac Rosetta x64 | Passed | Passed |

Windows x64 reports prove real Win32 backends, x64 main/peers, four zero final
charges and no reported owned survivors. AOT published the native image without
warnings and executed that same published image for all peers.

Ubuntu x64 reports prove real X11 backends, x64 main/peers and zero-charge cleanup
in both modes. Recorded startup-to-ready was 1103.4215 ms managed and 177.4644 ms
AOT in these runs. These are single-run observations, not a new benchmark study.

Original prepared matrices under `artifacts/validation-matrix/`:

- Managed: `20261026T034147Z-35def040`
- Native AOT: `20261026T034443Z-eb878098`

Collected original execution evidence:

- Windows managed: `20261026T035017Z-ca8f9ad8/win-x64/`
- Windows AOT: `20261026T100243Z-fc4bb899/win-x64/`
- Windows + Ubuntu managed: `20261027T022913Z-188132a4/`
- Windows + Ubuntu AOT: `20261027T022911Z-c07ac5df/`

The earlier collections above remain historical partial results. Final managed
collection `20261027T025246Z-deb1324f/` is Passed on all six; final AOT collection
`20261027T025246Z-b82c5b95/` is Failed because Windows ARM64 emitted a
NullReferenceException in ClipboardChecklistAsync. Publication/launch succeeded,
but actual functional acceptance did not. No six-RID AOT pass is claimed.

## Implementation verification

Windows headless integration tests currently run 14 cases: 12 pass and the two
POSIX lock cases are skipped. They cover pending/mismatched/missing collection, original
report architecture, completed-suite skipping, source availability, local GNOME
manager-session resolution and refusal of SSH/display-only contexts.

Native APFS checks passed the earlier 12-case integration suite including both
kernel lock cases, 14 existing orchestrator checks, three existing clipboard-lock
checks and the nine-case real .NET policy suite. Its required console guard probe
was built through `tests/execution-safety/verify.py`; no GUI was loaded. Reports:
`artifacts/gui-integration-validation/gui-integration-check-20261026T095929Z/`.
The later two Linux-session cases passed on Windows and the real Ubuntu x64 run
verified the updated path. Earlier native desktop compilation succeeded with no
compiler warnings/errors at
`artifacts/gui-integration-validation/gui-integration-check-20261026T014715Z/`.

The policy test initially invoked directly on ROBPC lacked its required probe;
the native verification used the documented build-and-test wrapper. No failed
product assertion was removed or weakened.

[Activation and collection instructions](../validation-gui-aot.md) contain the
remaining short target commands. No VM lifecycle automation, global security
changes, Mac Remote Login Full Disk Access changes, unrelated-process cleanup,
forensic process inventories or immutable source packets were introduced.
