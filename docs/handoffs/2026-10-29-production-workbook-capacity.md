# ADR-0029 — production workbook capacity implementation

2026-10-29. Accepted and closed by parent implementation review.

Production workbooks now admit 65,536 positioned graph cells across all sheets,
262,144 lifetime cell identities and 65,536 rows × 1,024 columns. CSV admission is
1,048,576 fields / 16 MiB; the native envelope is 64 MiB. Desktop navigation,
scrolling, Fill and clipboard use the larger row extent. Session/document labels
expose positioned and allocated counts separately. Clear does not recycle identities.
Formatting explicitly retains its independent 4,096-position limit.

WB9/CB4/private v7 remain in use; capacity counters are additive protocol fields.
Existing small WB9 documents expand their coordinate extent while preserving
CellIds and bound references. Previously rejected out-of-range formulas stay
rejected through load/save/reload until explicitly resubmitted. Interactive command,
frame, selection, actual range-consumer and per-formula limits remain unchanged.
Larger document admission is attached to the workbook model; generic hosted defaults
remain bounded as before. No history, identity compaction or new dependency is included.

## Measurements and validation

The 1,024 baseline and 10,000/65,536 production checkpoints passed. Both required
numeric shapes (2,000 × 10 and 4,096 × 16) pass complete real .NET/Rust
import/calculation/CSV export/native Save/reopen with independent full-table oracles.
The original frozen-source 65,536-cell measurements are milliseconds:

| Native target | Import | Export | Save | Reopen | Capture p95 | Edit p95 | Viewport p95 | Fan-out recalc |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Windows x64 | 1,925.91 | 232.46 | 469.89 | 1,718.51 | 0.307 | 0.511 | 3.173 | 317.26 |
| Linux x64 | 1,778.53 | 201.18 | 434.20 | 1,294.23 | 0.240 | 0.447 | 3.835 | 248.76 |
| macOS ARM64 | 705.07 | 89.88 | 302.32 | 694.62 | 0.084 | 0.076 | 1.220 | 104.42 |

Numeric-table charged peak was 483,969,708 bytes (461.55 MiB); formula-corpus
steady charge was approximately 249.8 MiB. The independent 262,144-identity case
retains one positioned cell and four old captures, verifies exhaustion rollback,
Clear/repopulate and native identity continuity. Its approximately 1.08 GiB charged
peak is separate from the compact-table 512 MiB objective and remains inside the
existing 2 GiB runtime policy. This is a material cost of lifetime identity retention.
Domain accounting is not process RSS; sampled process measurements are retained in
the raw results. macOS's peak-working-set API returned zero, which is unavailable
evidence rather than a zero-memory claim.

Final persistence review also caught two compatibility/admission problems: rejected
legacy formulas could change interpretation on a second reopen, and text/Boolean
overlays incorrectly inherited the 4,096 range-consumer bound. The corrections
preserve rejected source intent, distinguish scalar overlays from actual range
consumers, index native definitions and restore a private candidate in bounded
batches. Text-only tables of `001234` and Boolean tables now have independent exact
10,000/65,536-cell native round-trip checks; these are also part of the reusable
application-session verifier. Quoted text retains leading zeros exactly.
At 65,536 text cells, Save/reopen measured 603/3,090 ms on Windows,
660/3,218 ms on Linux and 340/1,706 ms on macOS, with a 499,961,852-byte
charged peak (476.80 MiB). Boolean peak was 494,456,828 bytes (471.55 MiB);
the slowest Boolean reopen was 3,482 ms. Each target passed 272 focused Rust
test executions and all five standalone workloads reported zero remaining charge.
[Final typed measurements](../../artifacts/capacity-final-measurements.json).

Evidence and source scope:

- Original native headless CI: Windows/macOS 13/13 in
  [20261029T164646Z-3bd4393f](../../artifacts/validation-matrix/20261029T164646Z-3bd4393f/matrix.md);
  Linux recovery 13/13 in
  [20261029T165617Z-3874e2bf](../../artifacts/validation-matrix/20261029T165617Z-3874e2bf/matrix.md).
  All 381 implementation inputs match the original
  [source audit](../../artifacts/capacity-source-audit.json).
- Attended actual-window suites: all three targets passed, four successful reports
  apiece, zero final charges and no surviving owned processes. Coverage includes
  high-row address/select/scroll/edit, Fill, native clipboard, Save/reopen and counters.
  [Collected reports](../../artifacts/validation-matrix/20261029T171103Z-6b961dee/matrix.md)
  and [frozen desktop source audit](../../artifacts/capacity-desktop-source-audit.json).
- Final persistence corrections have separate source-qualified native receipts:
  [summary](../../artifacts/capacity-final-summary.json),
  [Windows](../../artifacts/capacity-final-win-x64.json),
  [Linux](../../artifacts/capacity-final-linux-x64.json),
  [macOS](../../artifacts/capacity-final-osx-arm64.json).
  These check core/workbook/document regressions, both typed table sizes,
  independent identity exhaustion and complete real .NET/Rust session regression.
  The seven changed implementation paths and all 381 hashes are recorded. Desktop
  application sources are unchanged; the final Rust persistence corrections were
  checked headlessly and are not relabeled as a fresh graphical run.
- Final local workspace all-feature regression: 445 test executions passed;
  [log](../../artifacts/capacity-final-allfeatures-corrected.log).
  [Strict all-target/all-feature Clippy](../../artifacts/capacity-final-clippy.log) passed.
  [Original native measurements](../../artifacts/capacity-native-measurements.json)
  retain memory, allocation traffic and latency observations.

## Retained failures and limits

The initial 65,536-cell attempt exceeded the compact peak objective at 619,810,172
bytes. Removing a duplicate document-projection reservation and indexing native
restoration brought it below 512 MiB without raising the gate. Original
[failure](../../artifacts/capacity-65536.log) and
[correction](../../artifacts/capacity-65536-focused-correction.log) remain retained.
The initial 10,000-text reopen failure is likewise retained separately from its
[corrected result](../../artifacts/capacity-text-10000-corrected.log).

Ubuntu's original headless attempt was interrupted by a VM reboot; its original
setup failure remains unchanged. Recovery used the same frozen implementation.
[Continuity evidence](../../artifacts/capacity-linux-continuity.json) and
[approved-policy cache cleanup](../../artifacts/capacity-linux-cache-cleanup.json)
are retained. A local final-regression invocation used an incompatible old default
target directory and failed at link time; the complete successful rerun used the
established nightly target directory, with both logs retained.

Runtime limits remain explicit; capacity does not imply unlimited formatting,
formula sources, ranges or retained identity history. ADR-0023 remains parked at G3.
No additional AOT qualification was assigned to these existing controls and native
persistence paths. Short target-local `cap.py` launchers and
`artifacts/capacity-commands.txt` avoid transcribing long attended commands; future
attended runs should keep this convention and the runner's interactive consent.

Parent accepts the implementation, measurements, separately qualified identity
memory cost and supplied validation. The desktop/final-persistence source distinction
is accepted without another attended run. No additional GUI/AOT qualification or
reconciliation packet is required. Short target-local attended launchers are approved
as the standing workflow. Retain original failures under the accepted retention policy.
