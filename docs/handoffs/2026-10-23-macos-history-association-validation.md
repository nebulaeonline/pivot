# Stage 4 native macOS/APFS validation — evidence review

Date: 2026-10-23. **Native ARM64/APFS validation passed on the frozen Stage 4 Rust
implementation. [Complete feasibility acceptance](2026-10-23-disk-history-final-feasibility-review.md)
is now recorded for the tested research scope;
production history integration and physical power-loss guarantees remain deferred.**
[Stage 4 review packet](2026-10-23-history-association-stage4.md),
[original Mac report](../../artifacts/history-stage4/macos-validation-20261023T222012Z/summary.md),
[independent verification](../../artifacts/history-stage4/macos-review-20261023/verification.json).

## Scope and results

Source, build target, temporary test files and original journals were on internal
APFS under `/private/tmp/pivot-stage4-20261023T222012Z.cxuikrci`. Device IDs and
`/System/Volumes/Data` information confirm APFS. The worker is reported as Mach-O
arm64, with `sysctl.proc_translated=0`; this was native execution. Results were hashed
on APFS before copying to the SMB share. Original source/target/journals remain on
the Mac; attachment probes were not run against the copied SMB evidence.

| Gate | Result |
| --- | --- |
| Workspace default tests | 347 passed, no failures or ignored tests |
| Workspace all-feature tests | 383 passed, no failures or ignored tests |
| Formatting and strict all-target/all-feature Clippy | Passed |
| Release research worker and matrix | 69/69 cases passed |
| Completed worker logs, including seed/decline workers | 150, all FINISH 0 |
| Intentional fault workers | 24 readiness logs without FINISH |
| Separate held owner | Terminated with -9; contender exclusion passed |
| Maximum observed whole-domain charge | 27,214,911 bytes; not RSS |

Mac directory rename succeeded with the owner open. The namespace-unavailable
barriers and later restoration passed, as on Linux. Windows instead prevented the
live rename through its locking behavior, then passed the unavailable-after-release
case. Both outcomes remain explicitly reported; no assertion was removed.

The existing historical-export rejection passed as a negative test. Capturing a
new export from older B after later allocation floors remains unsupported; earlier
prepared B exports survive Undo/Clear and finish correctly. No codec, test, capacity,
dependency or production code adjustment was made on the Mac. No desktop, .NET,
Native AOT or manual keyboard tests were needed for this Rust-only research change.

## Independent evidence checks

The Windows review verified all **1,092 APFS-origin evidence-file hashes** and the
separate transfer receipt checksum. The evidence manifest checksum matches both
APFS-before-transfer and SMB values. All **166 Rust build inputs** match the frozen
Windows/Linux manifest and current source. All **462 copied source hashes** matched
before documentation reconciliation: these cover the 461-file frozen source plus
the ignored `gpt_PIVOT_CODEX_BASELINE.md`; the extra file is not an implementation
change. Mac before/after receipts report no source or input changes.

All **884 retained native journal/oracle hashes** were independently checked in the
copied evidence. The Mac separately verified these against original APFS journals
before copying. The original binary remains on APFS; its hash/architecture are
recorded in the verified native binary receipt, not independently rehashed from
Windows. Raw logs independently reproduce the test counts, 69 cases, 24 fault kills,
150 completed zero-charge logs and held-owner -9 outcome. The matrix summary's 93
completed result-bearing workers/probes excludes seed/decline logs; 150 is the full
log count, not an additional case count.

There was one pre-test setup failure: `diskutil info` rejected the temporary-directory
argument. The agent identified the actual mounted data volume and matched device
IDs before running gates. Its original probe and correction are preserved. No test
gate was rerun or weakened, and no tooling was installed or upgraded.

## Review boundary

Stage 4 now has **207 passing cases** across local Windows NTFS, native Linux and
native macOS/APFS, with **2,652 original journal/oracle hashes**. This is matching
research evidence for the tested filesystem/operation cases, not parent acceptance
of the complete architecture or a universal filesystem guarantee.

Keep the Stage 3 trusted-anchor/suffix-loss qualification, Windows namespace limits,
physical power-loss and device-cache limitations, network-share exclusions and
untested external-operation combinations. Full export, expensive validation, linear
history scans, historical native export, retention/secure deletion and production
packaging remain separate unresolved work. No physical reclamation or compaction
was added. Stop for parent review before proposing production integration.

Only documentation and a separate review verification receipt were added during
this reconciliation. Original Windows/Linux/Mac source/evidence manifests, failures
and transfer receipts remain unchanged; no implementation tests needed rerunning.
