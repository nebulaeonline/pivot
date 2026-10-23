# Bounded rectangular Fill

ADR-0022 implements Up/Down/Left/Right as trusted same-document duplication, not
clipboard interchange, series generation or history. The direction picks bottom/
top/right/left source edge respectively. Every destination maps directly to its
aligned source in one immutable pre-operation capture. Source edges never mutate.

`pivot-workbook/src/fill.rs` shares internal source-to-destination planning with
`duplicate_structured_block`. Typed content is retained; valid formula syntax is
translated using effective bound coordinates, existing dollar flags and range-kind
metadata. Structured bindings retain portable producer/output identities. Rejected
drafts remain non-executable. C# sends a typed direction/rectangle/expected state;
it neither parses nor duplicates formulas. No OS clipboard reads/writes are needed.

Before staging any candidate edits, compare translated typed definitions with
existing content and authoritative bindings. Equivalent range geometry/kind and
existing bindings reuse the destination definition without creating SelectionIds.
Equal displayed numbers alone do not establish equality. Blank is distinct from
explicit zero; blank clears retain destination CellIds and do not materialize absent
cells. No-op fills retain revision, saved checkpoint, dirty flag and graph identities.

For real fills, selected area including source edge is at most 256. Degenerate
single-row vertical/single-column horizontal operations validate state, shape,
bounds and cancellation but do not enumerate work or allocate identities. Shared
planning reserves bounded 8 MiB transient admission and enforces 128 KiB source/text,
16,384 token and 8,192 reference work. Repeated-source work counts; capacities do
not increase. Text/formula, graph, selection and experiment limits remain intact.

One cancel-aware transaction stages all changed destinations. The staged canonical
graph receives cycle-only dependency validation before live installation. This uses
core dependency/range-member enumeration and bounded/cancellable traversal, never
numeric evaluation or a second reducer. Existing editable drafts/missing references
are leaves for cycle validation; unrelated invalid drafts do not block Fill. Scalar
and range self-cycles reject. Ordinary entry/paste draft semantics remain unchanged.
No partial root, identity-ledger, revision, path or checkpoint installation occurs.
Temporary failed candidate owners use existing deferred reclamation; noninteractive
runtime flush/finish tests verify release without changing interactive cleanup rules.

The private v4 addition is FILL31 and typed direction field28; WB5, CB2 and hosted
v1 stay unchanged. A preserved pre-Fill v4 executable explicitly rejects opcode31
without changing its revision/value and closes with zero charge. Client request
cancellation after dispatch drains/acknowledges committed state and never replays;
pre-install core cancellation leaves the original book intact.

Desktop exposes an accessible Fill submenu and grid-scoped Ctrl/Cmd+D, Shift+D,
R, Shift+R. It excludes active editors, pending mutations, drag selection and
projected records. Logical rectangles cross presentation bands without including
records/chrome. Fill preserves selection orientation, active cell, scroll and grid
focus. Commands do not secretly commit drafts. One changed Fill is one document
transition, a foundation for later Undo/Redo design rather than history itself.
