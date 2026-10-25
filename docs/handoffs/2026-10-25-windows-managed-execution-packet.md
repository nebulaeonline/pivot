# Windows managed UI — verified enabled packet and one attended run

Subsequent result: [the single run executed; UI assertions pass but evidence is Blocked](2026-10-25-windows-managed-first-run.md).
Do not replay the command below. Its authorization is consumed; these instructions
retain the original execution record.

2026-10-25. [Conditional execution authorization](2026-10-25-windows-managed-preparation-acceptance.md).
**Final integrity/negative checks passed. One owner-attended managed run is ready;
no positive GUI execution has occurred yet.** Native AOT remains separately gated.

## Packet reconciliation

New immutable packet:
`artifacts/windows-managed-preparation/windows-managed-execution-20261025/`.
Receipt SHA256:
`2055df34ffc0cfdff9a3951ce3e8798976d49710fb50b0f696619e519c345b7f`.

Derived directly from the accepted disabled review packet, not the mutable checkout.
Independent reconciliation verifies eight packet files, all 529 source files and
258 build inputs. Exactly one source path differs: the wrapper's reviewed
`POSITIVE_RUN_APPROVED = False` becomes `True`. The top-level helper and archived
source agree; manifests are updated to those exact bytes. Other helper/launcher,
runner, assertion, consent and application bytes are unchanged. No extra patch is
applied to code. The included enabling patch is retained as provenance only.

The original disabled packet, earlier preflight packet, original Blocked exporter
capture and recovered successful report remain unchanged. The working-tree wrapper
also remains disabled. Frozen documentation reflects the review snapshot rather
than newer acceptance records; that is not source drift in this derived packet.

Twelve applicable frozen non-GUI tests pass. The disabled-only test is inapplicable
to the enabled snapshot and was explicitly omitted, not rewritten into a pass.
Two additional real subprocess checks verify that the enabled wrapper and unchanged
UI runner each return Blocked/2 with noninteractive input/output before any build
or GUI activation. No affirmative response, clipboard access or window is used.

[Independent verification receipt](../../artifacts/windows-managed-preparation/enabled-packet-verification.json),
[negative-test log](../../artifacts/windows-managed-preparation/enabled-snapshot-negative-tests.log).
The reviewed warning-free build remains applicable because the enabling delta is
Python-only. No accepted full headless CI was rerun.

## Owner command — ordinary CMD inside Pivot-Win11-x64

```bat
J:\coding\pivot\artifacts\windows-managed-preparation\run-windows-managed.cmd
```

This opens the existing native x64 VS command-line environment, imports the packet
to a fresh `C:\PivotWork\managed-input\windows-managed-execution-20261025`, then
executes only its native-local fixed bootstrap. It never starts/stops/connects a VM
or installs anything. An existing destination refuses; do not delete and replay.

The preflight will require **CONFIRM** for the current ROBPC/Hyper-V guest binding,
Basic Session/resource/isolation conditions. The prepared managed branch is bound
to boot `2026-10-25T00:37:42.0920420-04:00`, session 1. A changed binding stops before
GUI activation and must return for review. Do not bypass it or automatically repeat
clipboard testing. The fresh owner-observed clipboard pass is accepted for this run.

After preflight, the unchanged GUI runner independently asks whether to proceed
with windows/focus/clipboard activity. Answer **y** only when ready for this one
attended run; no other input is supplied by the wrapper. If current configuration
changed, decline/stop and report it. Do not use other applications in the guest
during the test. Owner must stop if connection or resource settings change.

Cold Cargo/NuGet restore and managed compilation may take time. All source, output,
cache, temporary fixture, persistence and original evidence paths stay on guest
NTFS. No Native AOT run or full headless CI is invoked. J: is import/export only.

## Results and stop

Original reports remain under the unique native `C:\PivotWork\attended-*` work
directory. The bootstrap captures Python-level output, exact native reports and
manifests, then hashes locally and exports evidence to the existing shared root:

`artifacts/attended-guest-preflight/results/Pivot-Win11-x64/preflight-capture-<id>/`.

Transfer verifies native-origin hashes and both manifest digests. Preserve the
console if it shows useful build diagnostics: native build subprocess output is
visible but not guaranteed to be fully transcribed by the Python tee. Use actual
application/peer reports to establish windows, backend/session, assertions and
zero charges; invocation alone is not a GUI pass.

Tell the repository-side agent when complete, including any visible error. Do not
rerun after a failure, refusal, interrupted run or transfer problem. First outcomes
are preserved: setup/policy Blocked, executed runner failures Failed. Copy-only
recovery can repair a transfer without replaying tests. Do not kill unrelated
processes or silently relax assertions. Return results for parent acceptance before
Native AOT; no Linux GUI, unattended authority, dispatcher or host fallback. G3 held.
