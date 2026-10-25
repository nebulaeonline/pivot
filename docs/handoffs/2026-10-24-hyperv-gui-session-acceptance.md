# Hyper-V GUI session evidence — partial acceptance

2026-10-24 parent decision, owner-supplied attachment
`6a970cc2-ec3d-4c11-9a76-8301167c2582/Pasted text.txt`.

Accept the [initial session evidence](2026-10-24-hyperv-gui-session-review.md):
owner-confirmed VMConnect Basic Session for both x64 guests, Windows Console/session
3 and Ubuntu GNOME/Wayland observations. These are preliminary session findings,
not approval of graphical isolation or evidence of Avalonia's eventual backend.
Neither guest is GUI-isolation-approved.

Preserve the returned reports and their disclosed origin-hash limitations. Do not
repeat these observations simply to obtain stronger hashes. The headless capability
and full Pivot CI gates remain accepted/closed; no rerun is requested.

## Next bounded evidence

Complete an owner record for each guest covering clipboard/history synchronization,
other active VMConnect/Enhanced Session/RDP or graphical clients, drive/folder/
printer/smart-card/USB/device redirection and host filesystem access beyond the
explicit J: or /mnt/j SMB/CIFS exchange. Record Unknown where not established.
Inspect only the test connections; do not change integration settings, disconnect
working sessions or install software to complete this inspection.

Attempt narrowly scoped Ubuntu desktop-process/logind session and seat correlation.
Do not enumerate unrelated users or dump full environments. If confidence is
insufficient, retain Unknown rather than inventing a session identity.

Return those metadata findings for the next review. The harmless bidirectional
clipboard sentinel remains pending separately scheduled, explicit owner consent.
Do not inspect personal clipboard history, access mail or assume restoration of
prior clipboard formats. When authorized, bind results to the actual VM, Basic
Session connection, boot/session, resource settings and timestamps. Preserve any
unexpected propagation as the first failed result; changed configuration needs
separate evidence and review.

No unattended permit, positive managed/AOT graphical execution, remote orchestration
or host fallback is authorized. UTM ARM64 remains separate. G1/G2 remain closed;
G3 remains held; G4–G7 and production Undo/Redo remain deferred.
