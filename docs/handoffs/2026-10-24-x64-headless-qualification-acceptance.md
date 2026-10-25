# Hyper-V x64 guests — headless qualification accepted

2026-10-24. **Pivot-Win11-x64 and Pivot-Ubuntu-x64 are HeadlessQualified.**
Full Pivot guest CI has not been executed or accepted. GUI isolation is not
approved. Active workstream remains infrastructure; **G3 stays held and unstarted**.

Execution follow-through: [actual source-matched guest CI evidence](2026-10-24-hyperv-headless-ci-validation.md)
now records both x64 guests passing all 13 headless steps. Its parent acceptance
remains pending. The unexecuted status above describes this original qualification
decision, not the later execution result; no historical receipts were regenerated.

Source: owner-supplied parent review attachment
`119a73fb-43da-4570-8aac-97b82b8a1817/Pasted text.txt`.
[Guest evidence review](2026-10-24-x64-guest-evidence-review.md),
[capture workflow and host identities](2026-10-24-guest-qualification.md).

## Accepted capability

| Guest | Accepted evidence | Separate unpassed gates |
| --- | --- | --- |
| Hyper-V `Pivot-Win11-x64` | Native x64 Windows/Python/Rust/.NET, NTFS, native console execution, MSVC Hostx64/x64 selection | Full Pivot CI; GUI isolation; positive managed desktop/AOT |
| Hyper-V `Pivot-Ubuntu-x64` | Ubuntu 26.04.1 LTS, native x64 Rust/.NET/ELF execution, C/C++ tools, ext4 | Full Pivot CI; GUI isolation; positive managed desktop/AOT |
| UTM `Pivot-Ubuntu-arm64` | No supplied guest receipts | Qualification and all target acceptance |
| UTM `Pivot-Win11-arm64` | Optional; no supplied guest receipts | Qualification and all target acceptance |

The parent accepts corrected host identity queries and the seven guest reports
with 73 verified file hashes. Preserve earlier permission failures, the collector
name-shadowing fix, incomplete metadata candidates and original native probes.
The Windows guest hostname `WinDev2407Eval` remains explicitly associated by the
owner with its Hyper-V VM identity; hostname alone is not authorization evidence.

Keep the linker help exit 1100 as a metadata anomaly, not a failed native build.
The parent requires no extra successful probe rerun to remove it. Retain the .NET
first-run certificate side effect and future generation-disable setting; do not
perform blanket certificate cleanup or replay completed tests.

## Next bounded assignment — existing headless CI

Run the existing `python scripts/ci.py` pipeline inside each qualified x64 guest,
with source matched before/after and no `--desktop`. This is authorized but has
not started under this acceptance receipt. The host/WSL results are not substitutes.

Use Windows `J:\coding\pivot` / Linux `/mnt/j/coding/pivot` only for deliberate
source/evidence transfer. Source, Cargo target, .NET bin/obj, temp fixtures and
reports must be on guest-local NTFS/ext4. Freeze input/source manifests and preserve
exact commands/exits, native-origin logs/hashes, first failures and owned-child/
charge evidence before copying reports back to fresh shared artifact directories.

Reuse the existing Rust, .NET, parity, IPC, application-session and execution-safety
assertions. Report missing tools/dependencies as setup blockers and actual executed
test failures as failures. Do not auto-install, weaken tests or rerun until green.
Preserve contributor CI without any VM requirement.

## GUI and other stop boundaries

Actual host connection/redirection settings, guest graphical-session ownership and
the deliberately scheduled bidirectional clipboard sentinel are still separate
evidence requirements. Enhanced Session availability, enabled services and session
hints establish no isolation approval. A headless-qualified VM receives no GUI permit.

No unattended desktop/AOT graphical acceptance, host fallback, SSH/PowerShell
dispatcher, VM lifecycle automation, provisioning, final VM CLI or production change.
Keep ARM64 qualification separate and the Ubuntu 18.04 compatibility environment
untouched. G1/G2 remain closed; G3 is held and G4–G7 deferred.

Return source-matched full Windows/Linux Hyper-V headless CI and remaining GUI
configuration findings for parent review. Do not automatically grant GUI authority.
