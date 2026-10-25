# Peer backend lifecycle correction — ready for review

Subsequent [parent acceptance closes the correction gate and authorizes one attended confirmation after verification](2026-10-25-peer-backend-correction-acceptance.md).
The pending-authorization statements below retain the original submission context.

2026-10-25. Implements the [authorized narrow correction](2026-10-25-windows-managed-functional-acceptance.md).
**No positive diagnostic-confirmation run has been authorized or executed.** The
first overall Blocked/2 and accepted functional execution evidence stay unchanged.

## Production delta

In ClipboardPeerAsync, one local nullable string captures this peer window's
`PlatformImpl?.GetType().FullName` immediately before registering the existing
Closed handler. The handler serializes that captured value instead of querying
the already-closed window. This is one added statement and one field-expression
replacement. No backend literal or main-window value is substituted.

Schema, process/session diagnostics, close event/report timing, clipboard checks,
functional assertions and final-charge accounting are unchanged. Missing live
backend still serializes no value and fails the existing strict wrapper evidence
check. No runner/consent/authorization change is part of this correction.

## Non-GUI validation

The new lifecycle regression extracts the **actual capture and Closed-handler
fragment** from production source and compiles it with the actual PeerReport and
generated JSON metadata in a standalone .NET console project. A window double
clears PlatformImpl before raising Closed. It verifies the previously captured
live value survives serialization and passes the wrapper's evidence check. With
no live backend, the field remains absent and that same wrapper refuses it.
The test has no Avalonia reference, windows, clipboard API or GUI consent.

The lifecycle regression passes against both the worktree and independently
extracted frozen candidate. Thirteen existing Windows non-GUI preflight tests pass.
Managed desktop build succeeds with zero warnings/errors. Logs are retained under
`artifacts/windows-peer-backend-correction/`. No full CI, Native AOT or GUI suite ran.

The first packet-builder attempt correctly stopped before creating a candidate:
its delta assertion mixed normalized worktree newlines and CRLF archive text.
The builder now normalizes only for comparison and preserves the original archive
line endings in the candidate. That preparation failure is retained in
`first-packet-preparation-attempt.json`; no original packet was modified.

## Immutable confirmation candidate

`artifacts/windows-peer-backend-correction/windows-managed-confirmation-review-20261025/`

Receipt SHA256:
`04aa8cd19f0f92a4064bda4464549b185dead13a54219fd50f16f7c56905bbbd`.

Derived directly from the original consumed enabled packet, not a mutable checkout:
**530 source files / 259 inputs / eight receipt-covered packet files**. Independent
verification establishes exactly one modified production source path
(`ClipboardAutomation.cs`) with the two-line correction, and exactly one added
non-GUI lifecycle test. Original line endings are preserved. All other source bytes,
fixed wrapper/launcher, strict evidence requirements, interactive consent broker
and managed UI runner/assertions match the original execution packet.

The candidate inherits the original fixed path's enabled constant; that is code
state, **not renewed execution permission**. There is no owner launch command for
this candidate. The working-tree wrapper remains disabled. Original disabled and
enabled packets, first managed Blocked capture and recovered preflight stay intact.

- [Minimal production diff](../../artifacts/windows-peer-backend-correction/peer-backend-correction.diff)
- [Independent candidate verification](../../artifacts/windows-peer-backend-correction/packet-verification.json)
- [Frozen lifecycle regression](../../artifacts/windows-peer-backend-correction/frozen-lifecycle-test.log)
- [Warning-free managed build](../../artifacts/windows-peer-backend-correction/managed-build.log)

## Review and stop

Return correction, regression/build evidence, exact source delta and candidate for
parent review. Separate explicit approval is needed for one attended Windows
diagnostic-confirmation run. That run must retain current VM/session checks,
native NTFS roots, separate owner CONFIRM and GUI consent, real windows/nested
peers, all functional assertions, zero charges, source integrity, scoped survivors
and native-origin/transfer hashes. No reuse of the consumed run authorization.

No repeated clipboard sentinel is requested; the parent accepts the fresh owner
observation absent a specific changed condition or contrary evidence. Native AOT
and Linux GUI remain separate gates. No unattended initiation, new protocol,
dispatcher, lifecycle automation, host fallback or product behavior change.
Headless gates/G1/G2 remain closed; G3 held; G4–G7/production Undo/Redo deferred.
