# G2 — native macOS evidence reconciliation

2026-10-24. Three-platform feature-only Rust validation is complete; **G2 is accepted/closed**.
[G2 acceptance and owner scheduling hold](2026-10-24-g2-acceptance-g3-held.md).
G1 remains closed; G3 is authorized but held, not started, during infrastructure work.
G4–G7 remain gated.
[Implementation and coverage](2026-10-24-mutation-local-records-g2.md).

The [Mac report](../../artifacts/macos-validation-20261024T173245Z/summary.md)
records native ARM64 execution on macOS 27.0.1, Rust 1.101.0-nightly
(`19833af72`, aarch64-apple-darwin), Python 3.9.6 and no Rosetta. Source, Cargo
target, test/temp data and acquisition reports were on local APFS under
`/private/tmp/pivot-records-g2-20261024T173245Z.imb_urg3/`; originals remain there.
SMB was used only for source/evidence exchange. No source/tests/limits changes,
tool upgrades or reruns were reported.

| Gate | Windows | Native Linux | Native Mac |
| --- | --- | --- | --- |
| Rust default / all-feature | 347 / 401 | 348 / 402 | 348 / 402 |
| Format, strict default/all-feature lint | Passed | Passed | Passed |
| Release worker / fresh-process samples | Passed / 45 | Passed / 45 | Passed / 45 |
| Domain charge after completed sample | 0 | 0 | 0 |

The returned raw Mac logs independently confirm seven successful gates, zero
failed/ignored Rust tests and 45 logs terminating `FINISH 0`. Five cases at
32/256/768 nodes have three samples each; all 135 processes across three platforms
pass. Mac emitted-record/core-read counts match the existing packet:
input 3/12, formula 3/12, range-input 4/12, range-formula 10/16, allocation 9/14.
Canonical scans remain zero except range-formula 35/259/771. These are work
diagnostics, not constant-time complete-edit or cross-hardware speedup claims.
Existing roughly 16.8 MB range invalidation reservation and old/new ledger-charge
publication peaks remain explicit costs. Requested domain charge is not RSS.

Independent Windows reconciliation verified:

- All **85 APFS-origin evidence hashes**, manifest hash and transfer receipt checksum.
- All **173 build inputs** against the original G2 manifest and current source.
- The **483-entry frozen source manifest**, including current source before these
  documentation edits. The Mac copied 484 files; its only extra was the ignored
  GPT baseline. Before/after source inventories report no changes.
- Raw test totals, sample grid, record/read/scan expectations and all 45 finish logs.

[Verification receipt](../../artifacts/history-records-g2/macos-review-verification.json)
and [review script](../../artifacts/history-records-g2/review_macos.py) retain these
checks. The worker fingerprint agrees between the returned binary receipt and
sample runner receipt; the actual ARM64 executable remains on APFS and was not
independently rehashed on Windows. Cleanup reports no owned worker survivors.
Original Mac and pre-Mac G2 manifests/receipts are preserved unchanged. Any pending
status inside those frozen historical receipts describes their original capture.

No managed, GUI, clipboard, .NET or Native AOT validation was needed or claimed for
this feature-only Rust producer. This is not VM validation or safety-policy
acceptance. The separate infrastructure workstream resumed after this reconciliation
and remains the active detour; G3 is held. No product/default mutation or
persisted-journal promotion.

The original Linux Rust clipboard-test shutdown assertion failure and its passing
unchanged-source targeted rerun remain preserved. A clean Mac pass does not diagnose
that failure or the older Windows/Linux GUI reliability observations.

The parent accepts this reconciliation with the implementation handoff, coverage/
ownership matrix and measurements. G2 is closed. The owner explicitly holds G3
during the infrastructure detour; no implementation or backend selection begins.
