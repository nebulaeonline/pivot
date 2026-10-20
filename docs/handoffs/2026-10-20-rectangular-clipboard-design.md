# Rectangular clipboard and formula copy — design review packet

Date: 2026-10-20. **Proposal only; no implementation or feature tests yet.**
[ADR-0017](../adr/0017-rectangular-clipboard-and-formula-copy.md) is Proposed.
The parent explicitly requires semantic review before formula translation is built.
The accepted rectangular-selection milestone and existing active-cell operations
remain unchanged.

## Decisions requested

1. Admit at most 256 coordinate slots per copied/pasted block. A single destination
   cell expands to the source shape; an already selected region must match exactly.
   No tiling, truncation, cut, copy/fill or implicit capacity increase.
2. Native formula copy transforms authoritative **effective coordinates in source S**,
   not historical source spellings. `$` locks the named copy axes; it does not alter
   stable-CellId move behavior. For B2→D5, A2/$A2/A$2/$A$2 become
   C5/$A5/C$2/$A$2. A moved target at A5 is copied from A5 even if the old formula
   still says A1. This is the key policy choice, not an implementation detail.
3. SUM endpoints transform independently with the same axis flags; copy creates
   fresh destination descriptors. Move still preserves existing identities/bounds.
4. Snapshot the whole block before overlap writes. Blanks clear destinations;
   preserve existing destination identities, allocate fresh identities for new
   destinations, and submit one atomic existing Rust workbook batch.
5. Reject a whole copy for retired/unplaced bound targets or unsupported opaque typed
   formulas. Reject a whole paste for translated out-of-bounds references. Preserve
   syntax-invalid drafts as invalid, rather than guessing at their text. Arithmetic/
   cycle errors remain normal editable Rust graph states.
6. Carry versioned native typed bytes plus external TSV. Rust owns decoding,
   formatting, transformation, binding and mutation; C# only transports bytes and
   manages UI context. Native copied formulas translate; external TSV formulas bind
   literal supplied coordinates because no reliable source origin exists.
7. Preserve exact decimal parts and formula bytes outside rewritten references.
   Use reviewed bounded payload/token/scratch ceilings, not unbounded blank-area
   enumeration. No new durable clipboard roots, schema migration or evaluator.

The ADR specifies examples, TSV blank-row rules, destination selection behavior,
error policies and exact proposed admission ceilings. The difference between native
relative copy and literal external text must be clear in the implementation/manual
checks. Existing native single-cell formula copy intentionally adopts the reviewed
new translation policy; external single-cell text behavior remains literal.

## Source findings and implementation plan (after approval)

The workbook already has immutable captures, stable-position metadata and atomic
apply. Limits remain 256 edits, 1024 live / 8192 allocated cells, 4096 source bytes/
tokens per formula, 1 MiB owned-byte estimate and a 1 MiB private session frame.
Current scalar references retain original UTF-8 spans plus bound CellIds; SUM stores
bound geometry but its AST currently has only a whole-call span. Spreadsheet `$`
syntax is currently rejected. These are the real extension points, not a new graph.

1. Extend shared spreadsheet parsing with per-axis markers and range-endpoint spans;
   leave general model syntax alone. Write pure Rust transformation tables before
   wiring UI/OS data. Define the reviewed private clipboard profile and limits.
2. Add Rust workbook capture-to-clipboard image conversion using effective scalar
   positions and bound SUM geometry. Validate occurrences/flags and exact numeric
   parts; encode bounded private bytes and TSV without retaining a live capture.
3. Implement Rust clipboard decode/plan against one expected destination revision.
   Prepare the complete overlap-safe batch, reserve/admit all work and invoke the
   existing candidate apply once. Preserve destination identities and rejected-draft
   states. Tests must cover late hard failure/cancellation with exact rollback.
4. Add separately versioned private COPY_BLOCK/PASTE_BLOCK payload operations and
   real Rust/C# boundary tests. Frozen hosted schemas are not extended; unsupported
   capabilities fail explicitly. Keep queue/framing/revision/response-drain rules.
5. Connect C# selection/clipboard transport and successful destination highlighting.
   Preserve editor text paste, focus, cancellation/interaction-generation guards and
   the bounded painted viewport. No address rewriting in C#.
6. Run focused Rust tests, actual application/OS clipboard checks, full Windows/WSLg
   CI and fresh native APFS validation with source hashes, manual acceptance and
   zero-charge cleanup. Produce the implementation handoff, then stop for review.

### Compatibility gate identified during design

WB2 decodes an invalid formula by recomputing its diagnostic. A previously saved
`=$A$1` invalid draft would become valid syntax under the proposed grammar. The
plan must preserve its stored invalid/non-executable state on reopen **and native
copy**, until explicit resubmission by the user. The ADR proposes a legacy diagnostic
mode of the same parser and an internal rejected-draft batch path, with fixtures,
not silent promotion or weaker marker validation. Current valid bindings still
receive full source/token/selection consistency validation. If a format revision
is actually necessary, stop and obtain review rather than casually changing WB2.

## Required evidence

Cover all four copy-axis modes, moved source formulas/targets, blank/retired targets,
SUM reversed/mixed-absolute bounds, overlap, native cross-workbook/process copy,
external TSV, exact signed-zero/scale, provenance/spans, invalid drafts, out-of-bounds
rollback, shape mismatch and all profile quotas. Test late clipboard context changes,
stale destination revisions, cancellation before versus after dispatch and no split
commits. Include WB2 reopened `$` formulas, old invalid-dollar drafts, hosted
capability rejection and no leaked roots/captures after failure.

No existing feature/performance behavior is claimed from this proposal. Retain
WSL mounted-path uncertain installation, historical native SIGABRT, Linux viewport
p95 exception and earlier validation failures. Multi-cell clipboard is not delivered
until the proposal is approved and separately implemented/validated.
