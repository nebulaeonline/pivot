# Rectangular clipboard, formula copy and WB3 — implementation acceptance

Date received: 2026-10-20. **Accepted; workstream closed.**
The parent review supplied by the user accepts the implementation of
[ADR-0017](../adr/0017-rectangular-clipboard-and-formula-copy.md), separately from its
earlier design acceptance. [Submitted packet](2026-10-20-rectangular-clipboard-review-packet.md),
[implementation](2026-10-20-rectangular-clipboard.md),
[desktop automation](2026-10-20-desktop-acceptance-automation.md).

## Accepted scope

Native copy translates effective captured scalar positions and SUM geometry with
independent dollar-axis flags. Ordinary scalar binding/move behavior remains based
on stable identity. Existing destination CellIds remain; new bindings are local.
Overlap reads all source slots before one bounded atomic Rust mutation, with complete
staged-destination binding and no partial installation on hard failure.

Rust owns untrusted decoding, numeric interpretation, translation, binding and edits;
C# owns clipboard transport and presentation. Native formulas translate; external TSV
formulas bind literal addresses. Malformed or unavailable native data never silently
falls back to text. Retain adversarial payload, exact sign/scale, source/span, identity,
overlap, late-failure/cancellation and cross-process regression coverage permanently.

PIVOTWB3 is the current native format. Keep explicit WB1/WB2 rejection fixtures, no
legacy readers/parser modes, and constrained non-executable rejected drafts until
explicit resubmission. Preserve graph/selection validation and frozen hosted v1.

`python scripts/ci.py --desktop` is the accepted ordinary UI acceptance workflow.
Retain real-window, separate-process clipboard, actual close Save/Discard/Cancel,
reopened-generation and zero-charge checks. Routine manual five-group replay on each
OS is no longer required; targeted physical-input, native-picker, IME/accessibility
and clipboard-manager spot checks remain appropriate when relevant.

Plain inline-editor arrows commit and move; modified arrows and formula-bar arrows
retain text editing. Hard rejected edits remain, and unchanged F2 exit must preserve
historical-source bindings. This usability addition is accepted.

## Validation accepted

Windows: 309 default / 325 all-feature Rust tests. Linux/WSLg: 310 / 326. Native Mac
ARM64: 310 / 326. Full .NET, numeric/protocol parity, IPC and desktop checks pass, with
zero-charge main/peer/boundary close. Native Mac used APFS, matched 339 copied source
hashes and 173 build/test inputs, and returned 391 transfer-verified files without
source fixes or gate reruns. No enlarged capacity, arbitrary clipboard-provider
reliability, physical-input certification or power-loss durability claim is implied.

## Required operational follow-up and stop point

Enforce sequential Windows/WSLg desktop clipboard execution where they share the
system clipboard. The runner currently relies on documented sequencing; enforcement
is **not yet implemented**. This narrow follow-up must not be represented as complete
merely because prior validation was run sequentially.

Retain bounded native-byte read retries, interaction guards and explicit failure.
The intermittent availability issue remains unexplained; do not expand it into an
elaborate clipboard service without a reproducible defect.

Keep the historical WSLg native startup abort, mounted-Windows-path installation
uncertainty and Linux viewport p95 exception visible and separate. They do not block
this accepted milestone and are not resolved by it.

Stop before selecting the next feature. No further graph, storage, codec or clipboard
infrastructure work is authorized merely to refine this slice. Cut/fill, text values,
additional functions, formatting, structural operations and higher capacity remain
separate product choices; no priority among them has been selected.
