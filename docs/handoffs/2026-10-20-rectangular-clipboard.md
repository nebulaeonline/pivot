# Bounded rectangular clipboard and formula copy

Date: 2026-10-20. **Implemented; three-platform automated validation complete; parent implementation review pending.**
[ADR-0017](../adr/0017-rectangular-clipboard-and-formula-copy.md),
[design acceptance](2026-10-20-rectangular-clipboard-review.md),
[fresh Mac packet](2026-10-20-macos-rectangular-clipboard-instructions.md).

## Behavior and ownership

Copy captures the normalized rectangle from one immutable Rust generation, up to
256 coordinate slots including blanks. Native paste translates effective scalar
bindings, not historical formula text. `$A1`, `A$1`, `$A$1` lock copy axes while
retaining stable scalar identity during ordinary edits/moves. SUM translates actual
bound endpoint geometry with independent flags/orientation and fresh selections.
Moved formula positions provide the current copy origin. Retired/unplaced bound
targets reject copy instead of reviving historical coordinate occupants.

Rust owns parsing, effective annotations, portable encoding, exact decimal formatting,
TSV decoding, reference translation and one existing atomic apply. Existing destination
CellIds remain; new destinations receive local identities. Formula references to cells
created later in row-major order bind their final staged identities. All source slots
are read before any destination mutation, so overlap never cascades. Blank slots clear.
Bounds, shape, stale state, cancellation/admission and late candidate failures install
nothing. There is no alternate evaluator, clipboard graph, retained source ticket or
per-cell IPC sequence. Source clipboard bytes survive source workbook mutation/close.

Paste outside editing commits immediately. One selected origin expands to source shape
and highlights top-left through bottom-right after success; an existing rectangle must
match exactly and retains its orientation. No tiling/broadcast. Failure retains the
selection. Late reads check interaction, selection, document, revision and editor state;
queued requests retain exact-revision/cancellation/response-drain rules. A rejected
single-cell external source can remain a correctable draft. Inside an editor, paste
inserts text without commit and rejects rectangular text. Edit/Clear/Delete/Move remain
active-cell operations; Copy/Paste now use the selection. Cut is absent.

## Native profile, TSV and rejected states

The private session adds COPY_BLOCK/PASTE_BLOCK. Frozen hosted v1 remains unchanged.
`PIVOTCB1` bytes contain little-endian u32 source row/column, height/width, then row-major
slots: tag 0 blank; tag 1 five exact decimal parts (lo/mid/hi/sign/scale); tag 2 formula;
tag 3 preserved rejected draft. Formula slots contain length-prefixed UTF-8 source,
annotation count and ordered effective row/column pairs. Source syntax supplies flags
and spans; normal formula occurrence count must match annotations. Rejected slots
require zero annotations. No graph, CellId or SelectionId is serialized.

Every received field is untrusted. Validate lengths, UTF-8, dimensions, decimal parts,
source syntax/span correspondence, coordinate bounds, counts, trailing bytes and
aggregate quotas. Effective coordinates express portable content, never source-workbook
authority. Malformed/unsupported native data rejects explicitly with no TSV fallback.
Validated typed Rust formulas can copy; arbitrary opaque typed provenance is rejected.

The OS format is MIME `application/x-pivot-cell-block-v1` on Windows/Linux and UTI
`org.pivot.cell-block-v1` on macOS. Both also publish text. External TSV formulas bind
literal addresses, without an invented source origin. Tabs and LF/CRLF delimit fields;
remove at most one terminal row separator, preserve empty rows/columns, reject ragged
rows/bare CR/unsupported text. Empty text means one blank. Exact scale and signed zero
survive text input/output without floating-point conversion. Native formula source
outside rewritten spans remains exact; TSV changes embedded formula tab/CR/LF whitespace
to spaces so it cannot accidentally create extra cells. Loss of the native format
means subsequent paste deliberately uses literal external-text semantics.

Copied rejected drafts are explicitly quarantined, even if their text parses under
current grammar. This is a constrained state with a fixed diagnostic, no bound tokens
or references, and an exactly matching non-executable graph definition. It asserts no
historical parser error and requires explicit user resubmission to execute. Ordinary
syntax-invalid drafts retain strict recomputed validation. Unknown markers and graph/
selection inconsistencies remain rejected.

## WB3 and resource bounds

Native files now use PIVOTWB3. WB1 and WB2 return explicit unsupported-version errors;
both frozen rejection fixtures are retained. No old reader, migration or legacy parser
mode exists. WB3 consistently uses the new grammar and distinguishes preserved rejected
state from ordinary syntax rejection. Combined graph/selection validation remains.

The ceilings are 256 slots/edits, 256 KiB combined copy output and accepted paste payload,
4096 UTF-8 bytes/tokens per formula before/after translation, 128 KiB aggregate formula
source, 16,384 parsed tokens and 8192 annotated occurrences/endpoints. Each clipboard
operation reserves 8 MiB additional planning scratch in the shared allocation domain;
batch reservations remain separate. Existing live/allocated/owned/selection/runtime
quotas can reject a smaller block. Caller/process clipboard bytes are separate from
continuing engine charges. Arbitrary OS provider allocation before receipt is not
bounded by this profile. No capacity increase or process-RSS guarantee is claimed.

## Validation and retained failures

Focused Rust tests cover all scalar axes, moved target/origin, reversed/mixed SUM,
overlap, identity retention/freshness, later-created destination bindings, exact
numbers/signed zero, external TSV, rejected drafts, source independence, retired
targets, forged payloads, bounds/shape/cancellation/stale state and late quota rollback.
WB3 tests include historical rejection and graph/selection corruption. C# boundary
checks close the source before cross-document paste, verify WB3 dollar/rejected-state
reopen and explicit resubmission, and prove zero-charge close on both children.

Actual-window checks exercise OS typed/text clipboard, immediate rectangle commit,
Rust scalar/SUM translation, destination highlighting, mismatch rollback and malformed
native no-fallback, alongside prior editor/selection/focus/save checks. These use routed
input and the real window/Rust child; they do not certify physical keys or IMEs.

Full Windows/Linux CI passes (309/310 default Rust tests); strict all-feature lint
and 325/326 all-feature tests also pass. Actual-window clipboard checks pass on both. These are the clipboard foundation
results before the subsequent automation/arrow follow-up; current evidence and the
updated acceptance workflow are in the [automation handoff](2026-10-20-desktop-acceptance-automation.md).
Fresh native Mac/APFS automation now passes; see the integrated results below.
[Windows CI](../../artifacts/rectangular-clipboard-ci-windows/20261020T232812124606Z/summary.json),
[window report](../../artifacts/rectangular-clipboard-windows/results.json),
[clipboard image](../../artifacts/rectangular-clipboard-windows/results.clipboard.png),
[172 build/test input hashes](../../artifacts/rectangular-clipboard-build-inputs.json),
[Linux CI](../../artifacts/rectangular-clipboard-ci-linux/20261020T232810429297Z/summary.json),
[WSLg window](../../artifacts/rectangular-clipboard-linux/results.json),
[Windows final boundary](../../artifacts/rectangular-clipboard-final-boundary-windows/clipboard.json),
[Linux final boundary](../../artifacts/rectangular-clipboard-final-boundary-linux/clipboard.json).
The final two client/test follow-ups enforce explicit nonempty copy/paste revisions;
full application-session and actual-window workflows were rerun on both platforms
without changing the Rust sources already covered by full CI/all-feature gates.
All 172 then-current foundation build/test inputs matched Windows and the isolated
ext4 Linux copy; the automation follow-up now verifies 173 inputs.
[Windows hash check](../../artifacts/rectangular-clipboard-windows-input-verification.json),
[Linux hash check](../../artifacts/rectangular-clipboard-linux-input-verification.json).
Linux source and .NET outputs are under `/var/tmp/pivot-rectangular-clipboard-20261020`,
Cargo cache under `target/ranges-linux`, and document fixtures under native
`/var/tmp/pivot-experimental-file-tests`. No .NET tests ran from the Windows mount.
Windows and WSLg clipboard images were inspected: E15:F16 extent, active F16,
translated formula, values and action-scope label are readable without observed clipping.

[Initial findings](../../artifacts/rectangular-clipboard-initial-findings.md) summarize
the earliest tool-only parser/window output; subsequent initial logs remain intact.
Retain the initial parser diagnostic-offset test failure, stale version-corruption
test expectation, occupied test-fixture target and lint failure in those records.
The first window launch exposed Avalonia rejecting a MIME identifier in its application
format API; corrected to the platform API with the Mac UTI mapping. Full CI then caught
TSV negative-zero sign loss; the adapter now explicitly preserves submitted zero sign.
All-feature sandbox IPC tests reported access denied; the unsandboxed run is separate.
Final resource auditing also added counted parser work for malformed TSV formulas,
so failed parses cannot bypass aggregate admission. Preserved rejected drafts are
not parsed during copying/installing. The new adversarial test and full CI/all-feature
runs pass after that tightening. No failed logs have been removed. Keep prior WSLg native abort, Windows-mounted-path
uncertain installation and historical Linux viewport p95 exception unresolved.

## Optional manual inspection reference

The owner requested automation of these five groups. Use `python scripts/ci.py --desktop`;
the [automation handoff](2026-10-20-desktop-acceptance-automation.md) records coverage,
remaining targeted spot checks and directional edit commits. Repeating this checklist
on every platform is no longer a routine gate. Launch `python scripts/run_desktop.py`
only when you want an interactive spot check (Mac: from native APFS).

1. Select B2 and paste this external 2×2 block; it must commit without Enter/editor:

   ```text
   2	=B2+1
   3	=SUM($B$2:B3)
   ```

   B2:C3 should show 2,3 / 3,5. Copy B2:C3 with the button and Ctrl+C (Cmd+C on
   Mac), select E5 and paste. E5:F6 should show 2,3 / 3,18; F5 is `=E5+1`, F6 is
   `=SUM($B$2:E6)`. Successful paste selects E5:F6 without opening an editor.
2. Paste into a differently sized region: reject without changes. Paste into an exact
   2×2 destination: succeed and preserve orientation. Paste a source containing a blank
   over an occupied cell: clear it. Copy A1:A2 values 1,2 into A2: expect 1,1,2.
3. In a fresh workbook enter A1=10 and B1=`=A1+1`. Move A1 to A5 using Move selected.
   Copy B1 into B2: its formula should be `=A6+1`; B1 retains its historical source.
   Try `$A1`, `A$1`, `$A$1` in formulas and confirm locked axes during copy.
4. Copy a native block, open a second Pivot window/workbook and paste; bindings must
   be local. Also copy before closing/replacing the source workbook, keeping the
   clipboard provider application alive. Paste an external literal `=A1+1`: it stays
   that text. Do not infer OS clipboard persistence after provider process exit.
5. F2/formula-bar paste inserts text without committing; Escape cancels. Check shortcuts,
   focus/caret, Shift selection, immediate commit, Save/reopen and dirty-close on native
   storage. Save a workbook containing `$` scalar/SUM formulas and rejected drafts and
   reopen it. Prior WB2 files must be rejected clearly, with the current workbook intact.

Record any optional spot-check feedback separately; do not infer physical input coverage.
No Cut, fill, structural operations, formatting/text-value expansion, generalized
clipboard framework, alternate graph, SQLite or capacity increase. Stop for parent
implementation review now that three-platform automated evidence is integrated.

## Acceptance workflow update

The owner's subsequent request replaces routine manual replay with deterministic
automated acceptance and optional inspection. Previous pending manual checks are
not marked passed. Cross-process clipboard, real dirty-close choices and arrow commits
now have automation, including the fresh native Mac/APFS run recorded below.
[Automation and final evidence](2026-10-20-desktop-acceptance-automation.md).

## Native Mac validation integrated — 2026-10-20

The fresh [native APFS report](../../artifacts/macos-validation-20261020T235001Z/REPORT.md)
passes integrated `python3 scripts/ci.py --desktop`, strict all-feature lint and
310 default / 326 all-feature Rust tests. No setup failures, product fixes or gate
reruns were needed. Source, Cargo/.NET outputs, temporary files, document fixtures
and original reports stayed under internal APFS before reports returned to SMB.
[CI summary](../../artifacts/macos-validation-20261020T235001Z/ci/20261020T235011455134Z/summary.json),
[environment](../../artifacts/macos-validation-20261020T235001Z/environment.json).

All seven window result fields pass, including clipboard_checklist and arrow_edit.
Separate-process private UTI receipt and actual Cancel→Save, Discard and clean close
pass; the coordinator reopens Save/Discard files to verify their installed generations.
The main window, all three peers and source/destination boundary children report zero
remaining Rust charge; the report records no surviving owned processes.
[Window](../../artifacts/macos-validation-20261020T235001Z/ci/20261020T235011455134Z/desktop-ui/results.json),
[clipboard peer](../../artifacts/macos-validation-20261020T235001Z/ci/20261020T235011455134Z/desktop-ui/results.clipboard-save.json),
[boundary](../../artifacts/macos-validation-20261020T235001Z/ci/20261020T235011455134Z/application-session/clipboard.json).
WB3 header/length/digest checks pass; suites cover dollar scalar/SUM reopen,
preserved rejected drafts/resubmission and explicit WB1/WB2 rejection. Sixteen
returned document fixtures include deliberately malformed/rejected cases, not an
assertion that every fixture is a valid workbook.

Before this documentation integration, all 339 copied source hashes and 173 reference
build/test hashes independently matched this tree. The final reference inputs still
match the implementation. The transfer record reports SHA-256 agreement for all
391 returned files and no missing report links; 391 files are present here.
[Source manifest](../../artifacts/macos-validation-20261020T235001Z/source-manifest.json),
[source checks](../../artifacts/macos-validation-20261020T235001Z/source-final-verification.json),
[reference checks](../../artifacts/macos-validation-20261020T235001Z/reference-input-final-verification.json),
[transfer](../../artifacts/macos-validation-20261020T235001Z/transfer-verification.json).

The Mac clipboard PNG was inspected here: E15:F16, active F16, values 2,3 / 3,18,
translated SUM, arrow-commit help and action-scope label are clear without observed
clipping. The Mac inspector also records distinguishable inactive/active selections
and synchronized editor/formula-bar contents.
[Clipboard image](../../artifacts/macos-validation-20261020T235001Z/ci/20261020T235011455134Z/desktop-ui/results.clipboard.png),
[inspection record](../../artifacts/macos-validation-20261020T235001Z/image-inspection.json).

No optional manual interaction was performed. Routed keys/buttons do not certify
physical input, IMEs/accessibility, native pickers or clipboard-manager behavior.
The carried `prior-manual-failure.json` describes the earlier editor milestone, not
a new manual failure in this run; retain it as historical evidence. Routine five-group
manual replay is not required under the owner's updated acceptance workflow.
Known WSLg native abort, mounted-path save uncertainty and historical Linux viewport
p95 exception remain unresolved. No new capacity, filesystem or performance guarantee.

Windows/Linux/Mac automated validation is complete for the bounded scope. Ready for
parent implementation review; implementation acceptance has not yet been granted.
