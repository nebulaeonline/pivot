# G1 ordinary Save promotion — macOS reconciliation and final review

Date: 2026-10-24. **Promoted ordinary Save has passed all applicable three-platform
Rust, managed/session/IPC, desktop and actual Native AOT gates. Parent accepts and
closes G1; G2 is separately authorized feature-only work and G3–G7 remain deferred.**
[Parent G1 closure and G2 authorization](2026-10-24-g1-closure-g2-authorization.md).
[Promotion implementation](2026-10-24-historical-save-g1-promotion.md),
[parent authorization](2026-10-24-historical-save-g1-internal-review.md),
[original Mac report](../../artifacts/macos-validation-20261024T133957Z/summary.md),
[independent verification](../../artifacts/history-save-g1-promotion-macos-review/verification.json).

## Native macOS execution

The Mac copied source off SMB to internal APFS under
`/private/tmp/pivot-g1-promotion-20261024T133957Z.r2wjj1jw`. Source, Cargo target,
temporary files, original journals, .NET bin/obj/publish and report output were local.
`diskutil` identifies APFS `disk3s1`, matched by the working directory's `df` mount.
The host and worker images are ARM64, `sysctl.proc_translated=0`, and .NET SDK
10.0.402 reports `osx-arm64`. Native AOT was published **and executed**, including
its three peer processes and adjacent Rust child; this was not publish-only proof.
Original source, target, binaries, journals and reports remain preserved on APFS.

| Mac gate | Verified result |
| --- | --- |
| Default / all-feature Rust | 348 / 391 passed; zero failed or ignored |
| Format, default/all-feature strict lint | Passed |
| .NET build, protocol, IPC, managed session | Passed |
| Managed desktop native Save/Open/Save As | Passed |
| Native AOT publish and actual execution | Passed, no publish warnings |
| Managed/AOT main + three peers | 8 reports, correct modes, zero charge |
| Desktop acceptance fields | All twelve passed in both main reports |
| Main managed/AOT WB5 fixtures | Both length and digest verified |
| Historical native matrix | 69/69 cases, 884 retained native hashes |
| Completed logs / deliberate fault kills | 150 zero-charge / 24, plus held owner -9 |

The 150 completed logs include seeds/decline probes; 93 are result-bearing
completed workers/probes. These are log counts, not additional cases.
`historical-native-save` confirms real WB5 install/Open, preserved Redo and exact
saved-content clean B → dirty D → clean B. The strict generic-transfer rejection
remains covered. Rich sparse/cold tests and the association use the promoted
ordinary exporter; history activation stays inside the accepted research boundary.
Desktop tests cover ordinary nonhistorical Save workflows, not a historical UI
workflow or production Undo/Redo.

No setup corrections, test-gate failures, reruns, source/test/limit changes or
upgrades were reported in this Mac run. Cleanup reports no owned survivors and
no remaining desktop clipboard lock. Timings and published image sizes are
observations, not benchmarks or a performance acceptance claim.

## Independent Windows reconciliation

Before changing docs, Windows independently checked:

- All **1,522 copied report hashes and byte lengths**, APFS-origin manifest checksum
  and separate transfer-receipt checksum.
- All **236 Rust/managed/test/script inputs** against the frozen promotion manifest
  and current tree, plus the Mac's clean before/after input receipts.
- All **474 frozen source hashes**, and **475 copied source files** including the
  ignored GPT baseline, against the current tree before reconciliation. That one
  extra file is not an implementation change. Mac source inventories are unchanged
  before/after locally and on the share.
- Raw gate exits, Rust test totals, all twelve managed/native main acceptance fields,
  main/peer execution modes and zero-charge reports, native publish inventory and
  adjacent-child fingerprint consistency, managed session cleanup, and both actual
  main WB5 fixture checksums.
- All **884 retained native-file hashes**, raw 69-case receipts, 150 completed
  zero-charge logs, 24 intended fault kills and the separate held-owner result.
- Original promotion source/input/evidence/verification/binary receipt checksums
  remain unchanged. The prior Windows/Linux-only frozen verification's pending-Mac
  status is historical; this review adds evidence without regenerating that receipt.

Mac independently verified original APFS journals and copied evidence before SMB
transfer. The original APFS journals, published executables and symbols were not
returned; Windows verified their reports/inventories, not those inaccessible bytes.
The Rust session target/adjacent-copy SHA-256s match in the verified binary receipt.
No attachment or recovery probes were run against SMB copies of private history.

Generated document inventory includes deliberate obsolete/corrupt negative fixtures;
it is not a claim that every returned `.pivot` file is valid. The two main saved
managed/AOT fixtures were independently verified as valid WB5 envelopes.

## Three-platform G1 closure recommendation

| Promotion gate | Windows NTFS | Native Linux ext4/WSLg | Native Mac ARM64/APFS |
| --- | --- | --- | --- |
| Default / all-feature Rust | 347 / 390 | 348 / 391 | 348 / 391 |
| Strict lint, managed/session/IPC | Passed | Passed | Passed |
| Managed desktop Save/Open/Save As | Passed | Passed | Passed |
| Actual Native AOT desktop/peers | Passed on unchanged rerun | Passed | Passed |
| Historical native cases / original hashes | 69 / 884 | 69 / 884 | 69 / 884 |

The promoted source now has **207 native cases and 2,652 original-file hashes**,
24 verified main/peer managed/AOT zero-charge reports and six independently checked
main saved WB5 fixtures across the three platforms. The parent accepts ordinary
Save promotion and **closes G1**, within the linked review scope.

Preserve the initial Clippy failures, Linux managed Fill selection/scroll failure
and Windows AOT unsupported-copy clipboard failure, together with their final
unchanged-source passes. The UI causes remain unresolved and separate from the
older null-destination issue. A clean Mac run does not establish their causes or
resolve them. No UI implementation or assertion was changed for this milestone.

Preserve native installation-uncertainty fences, mixed-invalid first-core diagnostic
ordering qualifications, process-kill versus power-loss limits, filesystem/network
exclusions and retirement versus physical reclamation distinctions. There is no
format/protocol change, capability expansion, limit increase, history coordinator,
new backend, mutation-local producer or desktop Undo/Redo integration.
Only documentation and independent review evidence were added in reconciliation;
all 236 implementation/gate inputs remained unchanged during reconciliation. G1 is
now closed; the separate G2 authorization does not authorize G3–G7.
