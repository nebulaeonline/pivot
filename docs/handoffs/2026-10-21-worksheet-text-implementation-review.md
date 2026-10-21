# Worksheet text — implementation acceptance and scope closure

Date received: 2026-10-21. Parent review supplied by the user.
**Disposition: Accepted. Bounded Document-Owned Worksheet Text is closed.**
[ADR-0018](../adr/0018-document-owned-worksheet-text.md),
[implementation and evidence](2026-10-21-worksheet-text-implementation.md).
This implementation acceptance is separate from the preceding semantic design approval.

## Accepted behavior

LiteralText remains canonical document-owned content with stable CellId, revision,
atomic edits, immutable captures and saved checkpoints. The graph contains only a
payload-free NonNumeric marker; numeric scalar references and SUM consuming text
produce explicit type errors. Text itself is valid content. Preserve strict numeric
precision, invalidation, copy translation and snapshot consistency.

Rust owns classification and exact decimal admission. Accept reversible apostrophe
escaping, exact supported UTF-8, single-line/control/line-separator rejection,
native typed text and mixed atomic rectangles with existing admission/context guards.

The governing bound is **4,096 UTF-8 bytes of stored text**. Exactly one leading
apostrophe used solely as forced-text editor/TSV syntax may make submitted source
**4,097 bytes**. Count the entire submitted source against aggregate work/clipboard
budgets. The parent explicitly accepts this clarification; it supersedes the earlier
contradictory per-entry wording in ADR-0018 without increasing stored-text capacity.

Accept PIVOTWB4, private application-session version 2 and PIVOTCB2/cell-block-v2.
The Protobuf package remains application.v1; the negotiated version is the private
compatibility boundary. WB1–WB3 remain unsupported with no migration reader. Failed
Open preserves the live workbook, revision, dirty state and content. Hosted v1 wire
and storage stay unchanged; unsupported scalar exports reject the marker explicitly.

## Accepted validation record

| Platform | Default Rust tests | All-feature tests | Managed / Native AOT |
| --- | ---: | ---: | --- |
| Windows x64 | 314 | 330 | Passed |
| Linux/WSLg x64 | 315 | 331 | Passed |
| Native macOS ARM64 | 315 | 331 | Passed |

Retain warning-free publication, exact native main/helper image execution, real
C#–Rust boundary, mixed clipboard, WB4 persistence, dirty-close and zero-charge
cleanup evidence. Mac APFS verification includes 359 source hashes, 183 build/test
inputs and 438 returned files. This accepts the submitted evidence; it does not
claim the parent independently audited the unprovided source code.

Preserve original failed logs and the Mac report's stale clipboard-v1 sentence;
the corrected handoff and tested version evidence establish v2. Keep the WSLg
startup abort, mounted-Windows-path installation uncertainty and historical Linux
viewport exception separate. Deployment, physical input and durability limitations
are not resolved by this acceptance.

## Stop point and candidate next direction

Do not begin the next implementation automatically. No computational strings,
string functions, dates, booleans, structured records, projections, connectors,
permanent `#` grammar, formatting, Undo/Redo or capacity changes are authorized by
this acceptance.

A design-only structured-result/projection experiment using fixed typed records
is the next candidate. Review producer identity, immutable result generations,
record identity, presentation projection identity and the ordinary worksheet
address space before any implementation authorization. Dynamic presentation must
coexist with ordinary cells without renumbering or mutating their identities.
No SQL connectivity or immediate `#` grammar implementation is selected.
