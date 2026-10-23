# ADR-0021 final implementation acceptance and scope closure

Date: 2026-10-23. **Parent disposition: accepted. Bounded Spreadsheet Aggregate
Functions milestone closed.** This is implementation acceptance, separate from the
earlier design approval and interim conditional review.
[Corrected implementation and final evidence](2026-10-23-wb5-only-aggregate-correction.md).

## Accepted result

MIN, MAX, AVERAGE and COUNT accept one rectangular range and compose with ordinary
arithmetic. Direct blank/text members are ignored, explicit numeric zero and valid
numeric formulas included, computed errors propagated. COUNT with no numbers is
zero; MIN/MAX/AVERAGE return NoNumericValues. Canonical extrema, the private exact
256-bit AVERAGE reducer with one final midpoint-away rounding, and 544 independent
rational reference cases are accepted. Strict SUM, stable bindings/range identity,
immutable captures and resource-accounting behavior remain intact.

WB5 is the sole native workbook read/write format. WB1–WB4 explicitly reject; no
obsolete reader, migration decoder or blank reconstruction compatibility path
remains. Current-format tests retain blank/zero identity, SUM, aggregate metadata,
CellIds/bindings, text, rejected drafts, saved checkpoints and atomic failed Open.
Experimental workbook/clipboard/private protocols support their current version
only unless an identified consumer and explicit support commitment justify more.
CB2/private application v4 and frozen hosted v1 retain their established contracts.

## Accepted evidence and limits

Windows: 339 default / 355 all-feature Rust tests. Linux/WSLg and native Mac/APFS:
340 / 356 each. Managed CI, strict all-feature checks, actual Native AOT, all eleven
desktop fields and main/helper workflows pass with no native publication warnings
and zero remaining charge. Final Mac run `macos-validation-20261023T090419Z`
matched 407 source and 206 build-input hashes. Its receipt reports 458 files
compared against APFS originals; per-file original hashes were not retained for an
independent Windows repeat of that transfer comparison. Windows independently
verified source/input manifests and inventoried returned files. The parent accepts
this provenance limitation. Prior WB4-reading evidence remains historical only.

The intermittent Windows ClipboardChecklistAsync null destination remains unresolved.
Earlier WSLg startup allocator abort, mounted-Windows-filesystem Save uncertainty
and historical Linux viewport p95 exception remain open. Successful runs do not
establish root causes. No test weakening or unrelated remediation is authorized.

No MEDIAN/MODE, aliases, formatting, Undo/Redo, permanent structured integration,
connectors, scheduler, structural edits or capacity increases are included.
Do not automatically begin another implementation milestone.

## Follow-up discussion — proposed sequence, not implementation started

1. Four-direction rectangular Fill: command-driven Up/Down/Left/Right over logical
   worksheet coordinates, using the opposite source edge (bottom/top/right/left
   respectively), copying each aligned source directly to every destination.
   Reuse authoritative formula duplication and relative/absolute translation;
   source capture and the entire fill are one atomic document mutation.
2. Undo/Redo architecture: design transaction-level history after Fill, including
   stable identities, retained generations/ownership, bounded resource quotas,
   saved checkpoints, dirty-state behavior and branching after Undo. A whole Fill
   must be one undoable transaction. Start with design, not history code.

The proposed Fill semantics are duplication, not series generation or drag handles.
Single-cell selections are no-ops; source blanks clear, exact literal text copies,
invalid drafts remain non-executable. Invalid translated references or quota failures
reject the entire operation. Multi-cell edges preserve alignment. Ordinary coordinates
exclude projected records. Trusted same-document structured duplication preserves
ProducerId/OutputId and existing quotas; only ordinary A1 operands translate.
Private-session requests should carry direction, selection and expected state;
keyboard/menu integration and actual desktop/three-platform validation are proposed.
Detailed bounds, request shape and no-op behavior still need concrete design review.
