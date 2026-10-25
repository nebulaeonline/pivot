# Actual Windows guest preflight — accepted and closed

2026-10-25 parent decision, attachment
`eb0e4905-e5be-4b81-9a2c-16679d370c2b/Pasted text.txt`.
**Actual guest non-GUI preflight accepted. Positive GUI execution not authorized.**
[Verified results](2026-10-25-windows-guest-preflight-results.md).

Accept the real guest identity, boot/user/session 1, WinSta0/Default checks, native
NTFS roots, 525 source files/257 inputs, fixed entry point, owner CONFIRM and
native-origin report verification. Preserve original bootstrap Blocked/2 and the
recovered original wrapper PreflightPassed as separate outcomes. Recovery was
copy-only; do not replay preflight or rewrite the original packet. The corrected
exporter belongs only in a newly frozen, reviewed packet.

## Next authorized preparation

Prepare a new immutable Windows managed-only packet with accepted preflight/helpers,
corrected exporter, exact manifests, fixed `tests/application-session/ui_smoke.py`
path, native-local source/build/cache/temp/fixture/original report roots, original
failure retention, hashes and verified transfer. Prepare the enabling diff and
non-GUI negative tests, **keeping positive execution disabled during preparation**.
Return the packet, diff, tests and current configuration evidence for parent review.
Any enabling code change needs separate review before execution.

No arbitrary command selector/shell, unattended flag, environment-only authority or
new consent protocol. Preflight CONFIRM and the unchanged runner's affirmative
terminal GUI consent remain separate decisions; neither may be injected or reused.

## Current-session clipboard condition

The accepted clipboard observation referenced October 24 boot/session 3, while
current preflight records October 25 00:37:42 -04:00 / session 1. Before the first
positive run, require one fresh **explicitly scheduled and authorized** owner-run
bidirectional harmless sentinel check in the current Windows connection. This is
narrow revalidation after an actual identity change, not a repeat full qualification.
Previous consent is not authorization for a new clipboard overwrite now.

Bind the actual outcome to current configuration/boot/session and retain any first
failure. Reassess after further connection/resource/boot/relevant session changes.
Owner acknowledgement alone does not independently prove clipboard continuity.

Only explicit subsequent approval may authorize one managed acceptance run. Retain
actual Avalonia/nested clipboard peers, all assertions, source integrity, native
storage, first failures, scoped survivors and zero-charge cleanup. No full headless
CI rerun just to reach managed UI. Native AOT and Ubuntu GUI remain separate gates.

No second protocol, unattended initiation, remote dispatcher, VM lifecycle, lease,
revocation supervisor or host fallback. J: remains import/export only. Headless
gates/G1/G2 stay closed; G3 held; G4–G7/production Undo/Redo deferred. No product changes.
