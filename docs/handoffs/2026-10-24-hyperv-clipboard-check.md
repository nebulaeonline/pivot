# Owner-run clipboard separation check — pending consent

Prepared 2026-10-24. **Owner has explicitly consented to the current manual test
window; both guests subsequently passed by owner report.**
[Results and evidence limits](2026-10-24-hyperv-gui-isolation-results.md).
Consent was “Yes, I consent to replacing both
clipboards now; provide manual instructions.” These instructions do not
grant GUI-isolation approval or permission to launch automated desktop tests.
[Parent boundaries](2026-10-24-hyperv-gui-session-acceptance.md).

Use this only after explicitly agreeing a quiet window and consenting to replace
both clipboards. Existing copied content will be lost; restoration of formats or
history is not promised. Do not inspect clipboard history or use mail/personal
documents. Use blank scratch editors on host and guest. Keep Basic Session and
the reviewed resource settings unchanged. Test one VM at a time.

## Procedure

Record the VM name, current connection mode, time with timezone, guest boot/session
reference and resource-exposure observations. Existing evidence references are
Windows boot 2026-10-24 15:00:16 -04:00 / Console session 3 and Ubuntu boot
616f92de-4c4d-4a06-8937-3f64765a039d / logind session/seat Unknown. If the guest
has rebooted or the session/configuration changed, report that before testing;
do not label old identity evidence current.

1. In a blank **host** scratch editor, type a fresh marker such as
   `PIVOT-HOST-<VM-name>-<current-time>-A`. Select and copy only that marker.
2. In a blank **guest** scratch editor, type a different marker
   `PIVOT-GUEST-<VM-name>-<current-time>-B`. Select and copy only that marker.
   Paste into a new blank line in the guest: it must yield the guest marker.
3. On the host, paste into a new blank line: it must still yield the host A
   marker. This checks guest-to-host propagation.
4. On the host, type, select and copy a third marker
   `PIVOT-HOST-<VM-name>-<current-time>-C`. Paste locally to confirm C.
5. In the guest, paste into a new blank line: it must still yield guest B.
   This checks host-to-guest propagation.

If any paste produces unexpected text, stop. Record only whether it was the known
host/guest marker or other unexpected content; do not export unexpected personal
text. Preserve the first failed result. Do not change settings and replace it
with a passing rerun. Clipboard-copy commands failing locally are inconclusive,
not isolation passes. VMConnect's Type Clipboard Text command is input injection;
do not use it as the copy/paste path for this test.

Repeat for the other VM only within the agreed scope and quiet window, with fresh
markers. Use ordinary editor Copy/Paste inside each desktop, not synthetic input
or unattended scripts. Clear test-only scratch buffers when done; do not export
clipboard history. No personal clipboard backup is requested.

## Result record

Report one result per VM; no manual command-output transcription is needed:

```text
VM / time with timezone:
Basic Session unchanged; guest boot/session unchanged or describe changes:
Resource-exposure observations (including Unknowns):
Guest local copy/paste of B: correct / failed
Host paste after guest copy: retained A / changed / inconclusive
Host local copy/paste of C: correct / failed
Guest paste after host copy: retained B / changed / inconclusive
Unexpected propagation or other issue:
```

The owner can report these observations in chat for recording, or save the record
locally and return it under `artifacts/guest-gui-isolation/<VM-name>/<timestamp>/`.
Successful observations are evidence for separate parent review, not final
approval or a perpetual permit. Connection, integration, session or boot changes
require re-evaluation. No positive managed/AOT suite or host fallback follows
automatically. G3 stays held.
