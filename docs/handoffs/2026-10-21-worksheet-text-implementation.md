# Bounded worksheet text — implementation and validation

Date: 2026-10-21. **Windows/Linux/native Mac managed and Native AOT validation
pass. Ready for parent implementation review; acceptance pending.**
[Accepted ADR-0018](../adr/0018-document-owned-worksheet-text.md),
[design review](2026-10-20-worksheet-text-design-review.md).

## Result and scope

Ordinary worksheet labels and descriptions are canonical Rust-owned content with
stable CellIds. They are entered, edited, rendered, moved, cleared, copied/pasted
and saved/reopened alongside exact decimal inputs and numeric formulas. There are
no computational string values/functions, structured records, dynamic projections,
connectors, formatting or capacity increases.

`Content::LiteralText` is distinct from source-bearing `TextFormula`. A payload-free
`NonNumeric` definition preserves the existing numeric node binding; no text enters
the graph. It is valid content, not an invalid formula. Scalar numeric consumers
and SUM return explicit nonnumeric-input errors, including through dependent
formulas. Text is neither parsed numerically, coerced to zero nor skipped in SUM.
Normal replacement/clear invalidation and immutable captures retain their meaning.
Viewport reads skip numeric evaluation for labels; labels have no numeric value or
formula error of their own. Rendering clips literal text to its cell.

The marker required enum cases, evaluation errors, batch replacement and transfer
handling, without a general value-system or evaluator redesign. It carries no
string payload and creates no new runtime value kind. Complete snapshot validity
and canonical restoration accept the marker. Frozen scalar exports, including
captured exports and hosted record codecs, reject it explicitly. Frozen hosted v1
wire/schema remains unchanged.

## Entry, editing and bounds

Rust classifies both ordinary entry and external TSV. Empty input clears; exactly
one leading apostrophe forces literal text and is removed. A lone apostrophe clears;
empty literal text is unsupported. Formula-shaped input remains a numeric formula
or preserved invalid draft. Fixed-point decimal-shaped input uses exact parsing,
scale admission and signed-zero metadata; overprecision/overflow is rejected,
never turned into text. Other input becomes exact literal UTF-8, including spaces,
scientific-looking labels and apostrophe-leading labels.

F2/formula-bar source reconstructs an apostrophe before text. Grid display omits
that escape. No-op edits preserve content/binding; explicit entry uses Rust
classification. Paste inside an editor remains uncommitted insertion; grid paste
is immediate atomic entry. Editor byte checks use UTF-8 rather than UTF-16 length.

**Implementation clarification for review:** the 4,096-byte ceiling applies to
stored text payload. One forced-text apostrophe is syntax overhead, so escaped
entry/TSV can use 4,097 bytes for a 4,096-byte payload. The full encoded source,
including this byte, still counts toward batch and clipboard budgets. This refines
the ADR's earlier statement that the escape counted within the per-entry ceiling:
that statement conflicts with lossless round trips for every supported maximum-size
payload. A boundary regression proves the maximum-size Unicode TSV round trip.
No stored-text, rectangle, frame, workbook or domain capacity was increased.

Native clipboard uses PIVOTCB2 and cell-block-v2 platform formats with a literal-text
slot. Native text is never reclassified; formula copy translation is unchanged.
TSV retains the unquoted rectangular grammar. Every exported text field gains one
apostrophe, including text beginning with an apostrophe, making Pivot TSV reversible.
The apostrophe can appear in other applications; universal interoperability is not
claimed. C0/C1 controls, tabs, CR/LF, NUL and Unicode line/paragraph separators are rejected
rather than stripped.
Existing 256-slot, 128-KiB clipboard source-work and 256-KiB combined output ceilings
remain. Invalid/stale/over-budget batches leave the published workbook unchanged.

## Compatibility cuts

PIVOTWB4 stores literal text and the graph marker, validating their consistency on
import. WB1/WB2/WB3 are explicitly unsupported; there is no migration reader. Failed
Open leaves the current document, revision, dirty state and content intact. Existing
stable bindings, decimal metadata, rejected drafts and checkpoint behavior remain.

The private application session now requires version 2, with separate typed text
and user-entry edit arms plus text read content. The Protobuf package/C# namespace
remains `application.v1`; the negotiated request/response version is the compatibility
boundary. Native clipboard v1-only payloads are rejected on grid paste; malformed
supported native data never falls back to TSV. These changes do not version hosted v1.

## Verification

Focused Rust tests cover Unicode/emoji/combining marks, whitespace, apostrophes,
numeric/formula-looking labels, exact signed zero, invalid numbers, controls and
UTF-8 bounds; scalar/SUM/downstream errors, number/text/number/blank replacement,
stable identities, immutable captures, mixed native/TSV rectangles, overlap,
atomic rollback, unsupported native data and zero-charge disposal. Document tests
cover WB4 text/marker restoration, retained identities and old-version rejection.
Core tests distinguish valid marker evaluation from rejected frozen scalar exports.

C#–Rust boundary automation adds text content and errors, immutable calculation
captures, native/TSV mixed blocks, dirty checkpoints, old-version failed Open,
WB4 reopen and zero-charge close. Desktop automation adds routed inline text entry,
F2/no-op editing, text/formula escapes, strict scalar/SUM diagnostics, mixed clipboard
and WB4 save/reopen. The existing separate-process clipboard peer now receives text
alongside numbers/formulas. Save/Discard peers close with active text drafts; an
independent Rust session reopens and checks the expected text and numeric generations.
The source-generated desktop report has a required `worksheet_text` acceptance field.
Managed and Native AOT gates both require it; native execution still proves exact
main/helper image paths, adjacent child discovery and zero charge.

Source/test reference: [183 input hashes](../../artifacts/worksheet-text-build-inputs.json).
Routine manual replay is not required. Physical keys, IME/accessibility and native
pickers retain the targeted spot-check boundary; automation does not certify them.

Windows final full CI passes with 314 default Rust tests; strict all-feature lint
and 330 all-feature tests pass. Linux final full CI passes with 315 default Rust
tests; strict all-feature lint and 331 all-feature tests pass. Both managed desktop
reports and all helper workflows pass with zero charge. The real C#–Rust text
boundary report passes on both hosts. Native Windows publish/run passes without
warnings, as does final native Linux publication/execution from ext4. All eight
desktop fields and all peers pass, with adjacent Rust discovery and zero charge.
All 183 build/test hashes match on Windows and the final isolated Linux source.

- [Windows full CI](../../artifacts/worksheet-text-ci-windows-current/20261021T092757261162Z/summary.json)
- [Linux full CI](../../artifacts/worksheet-text-ci-linux-current/20261021T092810938797Z/summary.json)
- [Windows strict all-feature lint](../../artifacts/worksheet-text-all-features-lint-windows-current.log)
- [Windows all-feature tests](../../artifacts/worksheet-text-all-features-windows-current.log)
- [Linux strict all-feature lint](../../artifacts/worksheet-text-all-features-lint-linux-current.log)
- [Linux all-feature tests](../../artifacts/worksheet-text-all-features-linux-current.log)
- [Windows native summary](../../artifacts/worksheet-text-native-aot-windows-current/20261021T092923055847Z/summary.json)
- [Linux native summary](../../artifacts/worksheet-text-native-aot-linux-current/20261021T093107155053Z/summary.json)
- [Independent final review audit](../../artifacts/worksheet-text-final-review-verification.json)
- [Linux native report transfer](../../artifacts/worksheet-text-linux-native-transfer-verification.json)
- [Windows text boundary](../../artifacts/worksheet-text-ci-windows-current/20261021T092757261162Z/application-session/text.json)
- [Linux text boundary](../../artifacts/worksheet-text-ci-linux-current/20261021T092810938797Z/application-session/text.json)
- [Windows input verification](../../artifacts/worksheet-text-windows-input-verification.json)
- [Linux input verification](../../artifacts/worksheet-text-linux-input-verification.json)
- [Windows private-version probe](../../artifacts/worksheet-text-application-version-probe-windows.json)
- [Linux private-version probe](../../artifacts/worksheet-text-application-version-probe-linux.json)

Linux source/.NET outputs, native publication/execution and document files are on
ext4 under `/var/tmp/pivot-worksheet-text-20261021` and
`/var/tmp/pivot-worksheet-text-files`; the existing Cargo cache under
`/mnt/j/coding/pivot/target/ranges-linux` was reused. Reports return to the shared
artifacts tree; native publish binaries/symbols remain on ext4 with retained size
and SHA-256 inventories. Windows native binaries remain alongside their reports.
Single Main-to-ready observations remain diagnostics, not comparative benchmarks.
Windows: 22,215,168-byte desktop, 42,780,712-byte runtime payload excluding symbols,
560.57 ms Main-to-ready. Linux: 24,980,336-byte desktop, 41,411,152-byte payload,
162.10 ms Main-to-ready. Exact symbol-inclusive outputs and hashes are in summaries.
Windows managed and final Linux native text screenshots were inspected: labels,
literal formula-looking text, clipped single-cell rendering and explicit formula
errors are visible. This does not certify physical input or typography across fonts.

Initial failures are retained: sandboxed named-pipe tests lacked access; ordinary
unsandboxed CI passes those paths. Initial desktop testing exposed missing formula
work reservation for the new entry arm and sequencing/focus assumptions in the
extended tests; these were corrected. Existing numeric-rejection tests now use a
real overflowing decimal because arbitrary words are valid text. A format-corruption
test now changes WB4 to an unsupported version instead of accidentally selecting WB4.
A first warning-free Windows AOT publish exposed an input-before-frame-readiness
race in the new fixture; the fixture now waits for the displayed revision before
sending input. Final verification also rejects Unicode line/paragraph separators
to enforce the single-line policy. No warnings or assertions were disabled to obtain a pass.

## Native Mac evidence integration

The [fresh native Mac report](../../artifacts/macos-validation-20261021T093751Z/REPORT.md)
passes full managed CI with desktop automation, strict all-feature lint, 315 default /
331 all-feature Rust tests, and actual osx-arm64 Native AOT publication/execution.
There were no product edits, setup failures or gate reruns. Source, Cargo target,
.NET bin/obj, temporary files, document fixtures, native publication and original
reports all used internal APFS. Preserved root:
`/private/tmp/pivot-worksheet-text-20261021T093751Z.zqpqkf8h`.

.NET SDK 10.0.402 and native toolchain inventories are retained. Both published
desktop and adjacent Rust child are Mach-O ARM64; Rosetta translation was off.
The [native summary](../../artifacts/macos-validation-20261021T093751Z/native-aot/20261021T094019813534Z/summary.json)
has no publish warnings. Main and all three helper processes report the exact
published native image, correct execution mode, passing workflows and zero charge.
Managed main/peers report managed execution. All eight desktop fields pass on both
paths, including literal text, mixed clipboard, strict scalar/SUM errors and WB4
save/reopen. The text boundary report passes; the private application-version probe
confirms v1 rejection/v2 acceptance and zero-charge cleanup.

Native desktop size is 19,010,832 bytes; runtime payload excluding symbols is
40,847,632 bytes; symbol-inclusive output is 97,608,016 bytes. Main-to-ready was
819.07 ms in one observation, excluding loader time. These are diagnostics on this
host, not comparative performance results. Native binaries/symbols remain on APFS;
the returned summary inventories their sizes/hashes.

The [independent review audit](../../artifacts/worksheet-text-macos-review-verification.json)
confirms all 359 copied source hashes and 183 build/test input hashes against this
workspace before doc integration, independently counts Rust results and verifies
main/peer reports, WB4 fixture integrity and report links. All 438 returned files
are present and locally hashed; the Mac transfer record reports SHA-256 equality
for every returned report. No owned application/child processes survived. The
native cache clipboard lock was released; no stale lock was removed.

The returned native text screenshot was inspected: labels, escaped numeric-looking
text, literal formula-looking text, clipping and explicit numeric-consumer errors
are visible. No optional manual launch or physical-input coverage is claimed.

**Report prose correction:** the original REPORT.md still names
`org.pivot.cell-block-v1` in its format paragraph. Tested source hashes and
[clipboard-version evidence](../../artifacts/macos-validation-20261021T093751Z/clipboard-version-evidence.json)
confirm PIVOTCB2 and `org.pivot.cell-block-v2`. Its later version-evidence section is
correct. The earlier format paragraph also mentions WB1/WB2 rejection without WB3;
WB3 rejection is covered by the text boundary and engine tests. Preserve the original
report unchanged; these are documentation corrections, not test failures.

## Remaining boundary

Three-platform managed and Native AOT validation is complete; submit for parent
implementation review. Acceptance remains pending. The one-byte escape-overhead
clarification above remains an explicit review point.
Keep the known WSLg startup abort, Windows-mounted-path installation uncertainty
and historical Linux viewport p95 exception visible. Local native filesystems do
not establish arbitrary-filesystem, clean-machine deployment or durability guarantees.
Stop after this bounded implementation's validation handoff; do not begin the
structured-result/projection experiment automatically.
