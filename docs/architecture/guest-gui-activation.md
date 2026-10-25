# Per-guest GUI activation — proposal

**Superseded scope, 2026-10-25:** the parent accepts the attended trust model but
defers the separate capability protocol and supervision proposed below. Current
work is [Gate A: Windows preflight using the existing interactive broker](../handoffs/2026-10-25-attended-guest-preflight-direction.md).
Do not implement the second protocol or treat the proposed 30-minute lease as an
approved requirement. This document retains the earlier proposal for future review.

2026-10-25. **Proposed; not approved or implemented.**
[Authorization](../handoffs/2026-10-25-hyperv-isolation-acceptance.md).
Windows x64 first; Ubuntu follows only after review of the same mechanism and its
platform-specific checks. Operational unattended authority stays disabled.

## Scope and authority

Start with **owner-attended activation of one guest-local run**, followed by
unattended execution of that run's approved test tree. This is not unattended
creation or renewal of authority. The owner launches the activation in a terminal
inside the intended VMConnect Basic desktop, confirms the current host/VM/resource
conditions and explicitly approves the exact category/source packet. No remote
dispatcher, VM lifecycle operations, host input injection or automatic activation.

This deliberately requires one approval per suite, not per test/peer. It cannot
establish isolation against a malicious owner or same-user process. Like the
existing consent guard, it prevents cooperative accidental execution; the VM and
owner-reviewed host configuration supply the isolation boundary.

A reviewed profile is necessary but never sufficient authority. It records host
ROBPC and exact Hyper-V VM UUID, expected native guest identity, Basic Session,
resource-exposure observations, deliberate exchange mapping, accepted sentinel
evidence and its limitations, allowed suite/entry point, source/input hashes,
native roots, required runtime checks and parent decision reference. Historical
session IDs are context, not mandatory future values or perpetual permission.

At activation the owner binds the current guest to the exact host/VM connection
and attests that reviewed settings remain unchanged. Guest hostname/SMBIOS metadata
alone cannot prove that binding. Current boot, user and graphical identity are
measured separately inside the guest. If the owner cannot establish the host-side
conditions, activation is Blocked. No saved JSON, environment variable, historical
sentinel or arbitrary certificate can replace this live owner step.

This proposal does not automatically inspect current host VMConnect settings from
the guest. It records that limit rather than claiming a technically enforced
host-configuration witness. Fully unattended initiation or automated independent
host attestation would require a separate reviewed mechanism and is out of scope.

## Run lifecycle and proposed limits

1. Verify the approved profile, explicit source packet and fixed entry point.
   Refuse missing/expired/unapproved profiles before builds or graphical children.
2. Verify guest-native execution and local NTFS/ext4 roots for source, outputs,
   cache, temp, persistence and original reports. Reject redirected/network roots,
   path escapes and link/reparse-point redirects in managed writable trees.
   The share is used only for explicit packet import and report export, outside
   the active GUI run. Do not change or disconnect the share.
3. Measure current boot, user and session. On Windows require the caller and child
   in the same interactive session and accessible input desktop/window station;
   record their IDs/names and revalidate before GUI startup. Refuse service/session
   zero, locked/unavailable desktop or session changes rather than guessing.
4. Present the exact profile, current identity, roots, suite and expiry. Obtain a
   fresh owner acknowledgement covering host/VM/configuration continuity and the
   usual keyboard/focus/clipboard effects. No renewal from redirected stdin.
5. Start a separate guest-local live broker, maximum **30 minutes**, one suite,
   no automatic extension. Reserve a specific launch identity; mint scoped child
   capabilities only for audited runner/application/peer roles. Keep secrets out
   of evidence. Proposed polling interval: one second, plus every child admission.
6. Recheck measured boot/session/desktop, source integrity and broker lifetime.
   Owner must stop the run if host connection/resource conditions change; these
   owner-attested conditions are not silently presented as machine-monitored.
   Broker loss, expiry or measured change revokes further admissions and stops
   owned GUI descendants. Do not kill unrelated processes or retry to obtain green.
7. Record first outcomes, runtime/backend and process identities, source hashes,
   charge cleanup and scoped survivors. Hash native reports before explicit export.
   Broker exit revokes authority; no reusable token is written to disk.

Thirty minutes is a proposed upper bound, not a new product limit. Expiry during
build/run produces a retained timeout/revocation outcome, not a passing test. Any
need for a longer category budget requires review rather than silent renewal.

## Enforcement changes required before activation

The current Python and C# guards implement an interactive nonce/HMAC broker with
no executable/category/session claims and only startup validation. Do **not** feed
a new guest broker into that existing generic permit as an isolation bypass.

Add a distinct versioned guest capability protocol, explicit mode selection and
strict bounded messages. Validate PID/start identity, boot/session, suite, role,
canonical image/source provenance and lease on the broker side using platform
process APIs; caller-supplied claims alone are insufficient. A stale environment
value can locate a broker but cannot create authority. Protocol mismatch refuses.

Python runners and the C# pre-Avalonia guard must both validate that capability;
nested managed/AOT peers inherit only their permitted role. Windows should use
an authenticated current-user local IPC endpoint with kernel-provided client
identity; exact API/protocol details and AOT support are an implementation-review
gate. Add an in-process revocation watcher and owned-process supervisor so expiry
does not merely block future startups while existing automation continues.

Keep the existing interactive consent broker and manual app launch behavior intact.
Headless environments strip both capability modes. Conflicting/invalid guest
authority must fail closed rather than falling back to interactive or host mode.
No environment-only switch marks a machine isolated. No contributor VM requirement.

## Fixed categories and platform order

First category: `tests/application-session/ui_smoke.py` and its existing managed
application/nested peers. Do not rerun full accepted headless CI via `ci.py
--desktop` merely to reach this category. Second, separately approved category:
`scripts/verify_native_aot.py`, including native publish and real executable/peers.
No arbitrary command argument, shell or general program launcher is exposed.

For Ubuntu, preserve logind session/seat Unknown when unavailable. Define an
equivalent authenticated local IPC and actual desktop ownership/liveness check;
DISPLAY/WAYLAND_DISPLAY alone cannot grant authority. Record the application's
actual Avalonia backend at runtime. Windows approval does not approve Ubuntu's
adapter. Neither adapter is implemented by this document.

## Controlled implementation and review gates

1. Parent reviews this trust model, attended activation, proposed lease, Windows
   checks, IPC identity design and per-guest profile requirements. Profiles remain
   templates without approval authority until separately reviewed.
2. Implement Windows broker/guard/runner changes with headless synthetic and real
   non-Avalonia policy probes: absent/stale/forged/replayed capability, wrong VM
   attestation/profile, boot/session, executable/role/source, roots, expiry,
   revocation, nested peers and broker death. Test actual OS identity adapters;
   simulations alone do not qualify them. Keep initial failures.
3. Return implementation and Windows profile for review. No graphical positive
   execution occurs implicitly during implementation or policy testing.
4. After explicit authorization, run one source-matched Windows managed suite;
   later separately authorize Native AOT. Retain all assertions, actual nested
   execution, host clipboard separation and zero-charge cleanup. No quiet test
   window or successful run grants permanent authority.
5. Review Ubuntu adapter/profile and repeat that staged gate on guest-local ext4.

Stop after the proposal for parent review. No production/history changes; G3 held.
