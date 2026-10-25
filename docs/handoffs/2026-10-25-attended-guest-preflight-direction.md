# Windows attended guest GUI — revised direction

2026-10-25 parent review, attachment
`9fb09dd6-7f5c-43d1-8067-1fc3ebabc53b/Pasted text.txt`.
**Trust model accepted with scope reduction. No positive GUI execution authorized.**

The first useful gate is an owner-attended Windows guest run using the existing
interactive consent broker, not a new authentication subsystem. Separate:

| Gate | Scope | Current authority |
| --- | --- | --- |
| A | Windows x64 native-local preflight, existing interactive consent, existing managed UI suite | Prepare bounded implementation and non-GUI synthetic/negative tests; return for review before positive execution |
| B | Unattended initiation, distinct capability protocol, authenticated IPC, role/PID identity, ongoing revocation/supervision | Deferred; do not implement |

## Gate A requirements

Prepare a small fixed-entry guest-local wrapper, not a general command launcher.
Establish intended Windows x64 identity and current boot/user/session, accessible
interactive desktop (not service/session zero), native NTFS source/build/cache/temp/
persistence/original report roots, source/build-input integrity and the audited
`tests/application-session/ui_smoke.py` entry point. Remove unintended self-check
activation and inherited stale authorization from the child environment.

Require explicit owner acknowledgement of current Basic Session, host/VM/resource
configuration and the accepted clipboard-isolation conditions. Guest-local files
cannot independently prove those host conditions. The wrapper must not manufacture,
inject, pre-answer or emulate GUI consent: after successful preflight, the existing
runner still requires the owner's affirmative terminal response through the
unchanged interactive broker. Its live authority remains available to nested peers.

Use J: only for deliberate source/evidence exchange. Execute, build and acquire
original evidence on native guest NTFS. Contributor headless CI remains independent
of VMs. No remote dispatcher, shell launcher, VM lifecycle or host fallback.

Return implementation and non-GUI refusal/synthetic evidence for parent review
before any positive run. A later explicitly authorized managed run must retain
actual Avalonia windows/nested peers, existing assertions, source-matched native
builds, zero-charge cleanup and original first failures. Native AOT is a separate
subsequent positive gate. Linux desktop/backend checks require their own review.

## Deferred proposal

[The earlier capability proposal](../architecture/guest-gui-activation.md) remains
design material only. Do not implement its second protocol, authenticated process
identity, role-scoped credentials, renewal/revocation or supervision now. If that
work resumes, separate build/publish, active GUI and cleanup budgets rather than
adopting the proposed single 30-minute limit. Review expiry/failure semantics first.

Both x64 headless gates and observed isolation evidence remain accepted, with no
repeat runs requested and no perpetual isolation claim. UTM/Rosetta remain separate.
G1/G2 closed; G3 held; G4–G7/production Undo/Redo deferred. No product changes.
