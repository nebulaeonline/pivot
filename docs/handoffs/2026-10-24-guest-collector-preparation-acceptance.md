# Collector preparation accepted; guest qualification remains open

2026-10-24. Parent review accepts the bounded collector preparation, synthetic
tests and documented permission blockers. **No VM is headless-qualified or approved
for unattended GUI execution. Guest qualification remains incomplete and open.**
G1/G2 and execution safety remain accepted/closed; G3 remains held and unstarted.

Source: owner-supplied attachment
`f2fcca56-236d-4351-8638-4ddcdeeecb12/Pasted text.txt`.
[Capture workflow](2026-10-24-guest-qualification.md).

The owner is to run the read-only Hyper-V host collector from Administrator
PowerShell and return its original reports. Guest metadata and explicit native
Rust/.NET probe receipts must be collected inside the intended guests. Do not
automatically elevate, change Hyper-V group membership/UAC/security settings,
enable remote services or reconfigure VMs to overcome the access blocker.
Preserve the original Blocked results; they are environmental access failures,
not failed VM validation.

Interpret collector success, native capability evidence, host integration settings
and independently reviewed GUI isolation separately. A successful query of an
unsafe setting does not make it safe. A successful tiny probe is not full Pivot
headless CI acceptance. The existing report fields are explained in the workflow;
no new authorization abstraction or schema change is introduced by this receipt.

The bidirectional clipboard sentinel procedure is approved only as a later
owner-scheduled procedure, not for execution now. Bind any future approval to
the actual VM configuration and connection/session mode; relevant changes
invalidate it. No unattended permits, orchestration, lifecycle automation,
provisioning or VM-selection flags are approved.

## Owner-confirmed shared-drive layout

| Guests | Source/evidence share | Pivot path |
| --- | --- | --- |
| Windows Hyper-V and UTM | `J:\`, SMB mapping to the host's `J:\` root | `J:\coding\pivot` |
| Linux Hyper-V and UTM | `/mnt/j`, CIFS mount of the host's `J:\` root | `/mnt/j/coding/pivot` |

These are deliberate source/evidence exchange shares, not native guest test roots.
Copy source to guest-local storage before builds, probes, persistence tests and
original evidence acquisition. Suggested roots are `C:\PivotWork` on local NTFS
and `$HOME/pivot-work` on local Linux disk. Hash evidence there before copying
it back to a fresh directory on the share; retain native originals.

The explicit SMB/CIFS mapping is an intentional exception to otherwise restricted
host-folder integration. Record its scope in the isolation packet; it does not
prove clipboard separation or permit unrelated host drives/folders/devices. Whether
it should be disconnected during later GUI acceptance belongs to the per-guest
activation review, not an automatic change by the collectors. The host's physical
J: and a guest's SMB-mapped J: must never be treated as the same filesystem evidence.

Next: obtain and reconcile real host/guest receipts and missing prerequisites,
then return for qualification review. Stop before GUI activation and keep G3 held.
