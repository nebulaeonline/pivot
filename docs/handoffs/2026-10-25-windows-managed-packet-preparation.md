# Windows managed GUI packet — preparation for review

Subsequent [parent acceptance closes this preparation gate](2026-10-25-windows-managed-preparation-acceptance.md).
It conditionally authorizes a new enabled packet and one attended managed run after
verification. The disabled packet and submission statements below remain original.

2026-10-25. [Authorized preparation](2026-10-25-windows-guest-preflight-acceptance.md).
**Positive managed execution remains disabled. Nothing in this packet is permission
to launch it.** No new broker, unattended initiation, arbitrary command selector,
remote dispatch or product semantic changes. G3 held.

## Changes prepared

The fixed managed branch in `windows_attended_gui.py` now has complete invocation
and evidence handling behind the unchanged false approval constant. It runs only
the unchanged `tests/application-session/ui_smoke.py` via runpy, in the same Python
process with native source cwd, private environment and original terminal streams.
This lets the existing runner obtain its own affirmative consent and keeps Python
assertion tracebacks in the capture log. It does not supply an answer or permit.
Environment/cwd/argv/path are restored afterward. Build subprocess output retains
the console; it is not claimed to be fully transcribed by the Python tee.

Preflight CONFIRM remains a separate current-configuration acknowledgement. The
managed branch refuses a boot/session different from the reviewed Windows evidence
(boot 2026-10-25T00:37:42.0920420-04:00 / session 1). A future change requires review
of the binding, not automatic permission renewal or an automatic demand to repeat
clipboard testing. Host resource settings remain owner-attested.

The existing UI assertions are unchanged. Additive self-check report diagnostics
record actual window `PlatformImpl` type as `graphical_backend` and Windows process
session ID for main and peers. Existing `backend="real Rust"` means calculation
backend, not graphical backend. No inference from environment names is substituted.
The wrapper additionally requires four passing main/peer reports, zero charges,
backend/process evidence and matching current session. It records duration, source
integrity after execution, scoped survivors and post-run guest identity. An executed
runner failure remains Failed even if later evidence collection also has errors;
policy refusal remains Blocked. Successful runner output with incomplete evidence
is Blocked, not a claimed pass. No automatic retries or unrelated process kills.

`launch_windows_managed.py` is a native-local fixed bootstrap, not a dispatcher.
It verifies the pinned packet before importing helpers, captures output while
preserving real TTY status, handles prompt/path on one line and collects exact
native reports even after a failed runner. Hash verification precedes copying;
origin hashes and transfer receipts preserve native evidence. Original exporter
failure, recovered first preflight report and all older packets remain unchanged.

## Current clipboard observation

After fresh explicit consent for host/Windows clipboard overwrite, the owner was
given the bidirectional harmless-marker procedure and reported: “the clipboards
are still isolated,” with VM settings unchanged. This is the requested fresh
Windows-only owner attestation, not a new machine-generated audit. Exact sentinel
time and independent boot/session tracing were not supplied. No new Ubuntu check
or clipboard inspection was performed by the agent.

The owner explicitly rejects repetitive clipboard checks. Record that preference:
do not request routine repetition without a specific changed condition or observed
failure requiring reassessment. This does not silently grant perpetual authority
or remove current configuration review before the first positive run.

## Validation and frozen artifacts

Thirteen Windows non-GUI tests pass. They cover the corrected same-line exporter
with actual temporary report copying/hash rejection, source/receipt/helper failures,
scope/TTY refusal, stale authority, native roots, prepared boot/session mismatch,
outcome preservation, backend/session/zero-charge requirements and mocked fixed
runner execution without consent injection. Five console-stub helper regressions
pass. The real managed desktop builds warning-free with the new diagnostic fields;
no window or acceptance suite is launched by a build.

The initial sandbox build exited unsuccessfully without diagnostics; the outside-
sandbox compile passed. Both logs remain under `artifacts/windows-managed-preparation/`.
No accepted full headless CI was repeated. No Native AOT build/positive execution.

New immutable review packet:
`artifacts/windows-managed-preparation/windows-managed-review-20261025/`.
Its receipt covers exact source/build-input manifests, source.zip, wrapper/helpers
and corrected launcher. Preparation/verification receipts alongside it supply exact
counts and hashes. `enable-fixed-managed.patch` contains the proposed one-line
false-to-true approval change; it is **not applied** to working or frozen source.

The review packet is intentionally disabled. If approved, enabling and refreezing
creates a different execution packet/digest; never apply the patch in place to an
already frozen packet or claim the old digest covers changed bytes. The launcher
may accept only the fixed managed request, never a generic command or environment
authority. No owner execution command is issued during this preparation gate.

## Parent review request

Review the fixed-path implementation, corrected exporter, proposed enabling diff,
frozen disabled packet and current Windows owner observation together. Positive
execution still needs explicit approval and a newly pinned enabled execution packet.
The eventual run must retain unchanged consent, real Avalonia and nested peers,
native source/build/storage, existing assertions, failure evidence, scoped survivors
and zero-charge cleanup. Owner must stop if reviewed connection/resources change.
Native AOT and Linux GUI remain separately gated. No host fallback. G1/G2 and
headless gates closed; G3 held; G4–G7/production Undo/Redo deferred.
