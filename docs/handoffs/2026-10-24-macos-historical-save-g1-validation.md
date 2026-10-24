# G1 native macOS/APFS validation — evidence review

Date: 2026-10-24. **Three-platform validation of the internal G1 correction is
complete and accepted by the parent. The later ordinary Save promotion is implemented separately;
this Mac run does not validate that new source. G1 closure awaits its final gates. G2–G7 remain deferred.**
[Promotion source and fresh full Mac gate requirement](2026-10-24-historical-save-g1-promotion.md).
[Internal acceptance and promotion authorization](2026-10-24-historical-save-g1-internal-review.md).
[Implementation packet](2026-10-24-historical-native-save-g1.md),
[original Mac report](../../artifacts/macos-validation-20261024T100245Z/summary.md),
[independent verification](../../artifacts/history-save-g1-macos-review/verification.json).

## Scope and results

Source, target, temporary files and original native journals were on internal APFS
under `/private/tmp/pivot-g1-20261024T100245Z.c91sqcjc`. The filesystem probe queried
`/System/Volumes/Data` directly; `df` identifies the same `disk3s1` APFS volume.
The worker is Mach-O arm64, rustc host is `aarch64-apple-darwin`, and
`sysctl.proc_translated=0`. This was native execution, not testing from SMB.
The original source, worker, journals and report remain on the Mac.

| Gate | Result |
| --- | --- |
| Default Rust tests | 347 passed; zero failed or ignored |
| All-feature Rust tests | 390 passed; zero failed or ignored |
| Format / strict all-target all-feature lint | Passed |
| Release worker build | Passed |
| Native Save/restart matrix | 69/69 cases |
| Retained native-file hashes | 884 verified |
| Completed worker/probe logs | 150 with `FINISH 0` |
| Deliberate fault kills / separate held owner | 24 / exit -9 |

The matrix summary has 93 result-bearing completed workers/probes; 150 additionally
counts seed and decline logs. These are log counts, not additional test cases.
No test-gate failures, reruns, source changes, limit changes or tool upgrades were
reported. Cleanup reports no surviving owned workers. Mac observed live-directory
rename succeeding and passed the namespace-unavailable/restore checks; this does
not remove the documented Windows locked-directory behavior.

`historical-native-save` explicitly passed actual WB5 installation/reopen after
Undo, Redo preservation and saved B → dirty D → clean B. The strict generic-transfer
historical rejection remains a negative regression. The all-feature suite includes
the rich sparse/cold historical content round-trip, actual child-process native
Save/Open, source-authority/admission/cancellation/codec checks and isolated-cycle
checks. Exact typed content, durable identities and provenance remain the comparison
contract; mixed-invalid first-core diagnostic traversal order remains qualified in
the implementation packet.

## Independent verification

Before editing documentation, Windows independently checked:

- All **1,097 copied evidence hashes and byte lengths**, the APFS-origin manifest
  checksum and the separate transfer-receipt checksum.
- All **168 frozen build inputs** against both the G1 manifest and current tree.
- All **470 frozen source hashes** against Mac's copied source inventory, plus the
  ignored GPT baseline: **471 matching source files** before reconciliation. The
  extra baseline file is not an implementation difference. Mac before/after
  receipts report no local or shared source/input changes.
- All **884 retained native evidence hashes** and the raw logs supporting Rust
  counts, 69 cases, 24 fault kills, 150 zero-charge logs and the held-owner outcome.
- Original G1 source/input/evidence/verification/binary manifest checksums remain
  unchanged against its frozen receipt.

The Mac separately verified the original APFS journals and retained evidence before
SMB copying. Windows cannot independently rehash those APFS originals or the retained
Mac binary; its verified receipt records binary SHA-256
`a33719dff253c946923c9e864fb986a23ed44a1662db87978fc95d8d1f51fbfc`.
No attachment probes were run against SMB evidence copies.

The original G1 frozen receipt still describes its Windows/Linux-only preparation
point, including then-pending Mac validation. It was not regenerated. The separate
review receipt and this document add the source-matched Mac evidence. The original
Mac report's phrase “complete feasibility parent review” is inherited wording:
the four-stage feasibility program is already accepted/closed; this submission is
**G1 implementation review**.

## Parent review boundary

G1 now has **207 passing native cases** and **2,652 original-file hashes** across
Windows NTFS, native Linux ext4 and native macOS ARM64/APFS. Rust totals are
346/389, 347/390 and 347/390 (default/all-feature), with strict lint on all three.
The parent accepts the **internal experimental correction** within that scope under
the linked review. Ordinary Save promotion and final G1 closure remain separate.

This run deliberately did not include managed/session, desktop, Native AOT or
manual UI checks. Ordinary application Save still uses strict full-ledger export;
only the feature-only research association uses the explicit historical exporter.
Before promoting that exporter into ordinary Save, run the full applicable
source-matched managed/session, desktop Save/Open and actual Native AOT gates
against the corrected path on all three platforms, including zero-charge cleanup.
These requirements remain pending; no promotion is claimed.

Retain installation-uncertainty fencing, process-kill versus physical power-loss
limitations, private-file identity and network-share exclusions, retention versus
physical reclamation/secure erasure, and the existing unrelated reliability issues.
No backend, mutation-local producer, coordinator, protocol change or desktop
Undo/Redo integration is authorized by this reconciliation. Only documentation and
an independent review receipt were added; no implementation tests needed rerunning.
