# Atomic workbook edit proposal

Status: Accepted atomicity direction under ADR-0010. Slice A implements in-process
coordinated batches; future service/wire behavior below is not implemented.

Review clarification: ADR-0010's structural architecture is Accepted. Reference
transforms, range boundaries/movement, blank/text coercion, reduction order and
shared writable aliases remain separately reviewable compatibility semantics.
[Slice A](workbook-projection-foundation.md) implements only the bounded typed
single-sheet proof; future mechanisms described here are not claimed implemented.

## One coherent installation boundary

The workbook session owns a coherent aggregate root containing sheet directory,
axis order, sparse chunks, anchor/binding directories, named scopes, presentation
and graph-definition roots with portable/runtime maps. Requests identify SessionId
and expected document state. Graph revision additionally pins computational work.
Do not install a worksheet candidate first and patch the graph afterward.

Proposed operation flow:

1. Validate session/revision, command dimensions, byte/work limits and targets.
2. Capture the current roots, then stage private changed chunks, axes and graph
   definitions. Allocate provisional identities that cannot escape a rejected edit.
3. Compute structural anchor/reference transforms and the final name/position
   directory. Resolve only newly submitted or explicitly copied formula references.
   Existing bound references retain their identities.
4. Validate candidate integrity and ownership; collect editable draft diagnostics.
   Syntax/cycle/reference errors can be draft content, while broken internal maps,
   quota failures and conflicting expected state reject the whole command.
5. Recheck expected head, then install one aggregate root and its revision tags
   under a brief coordinator gate. Publish one committed change description.
6. Schedule or capture calculation and deliver events outside that gate. File I/O
   and full calculation never run while holding the edit installation lock.

Concurrent staging does not imply two writers can both commit against one expected
head. A conflict installs nothing and requires explicit refresh/retry. There is no
automatic client-intent replay, hosted attachment or durable receipt requirement.
Ambiguous IPC outcomes will need a separate later client contract; this proposal
does not inherit ADR-0008 durable recovery or allocate new wire fields.

## Worked edit boundaries

| Command | Atomic changes |
| --- | --- |
| Set a formula | Sparse content/source, reference descriptors, staged core definition, diagnostics and changed dependencies |
| Paste a rectangle | Destination chunk changes, new/preserved anchors, copied references and all graph edits; no partially applied visible paste |
| Insert a row | Axis-order run, geometry/default metadata, range boundary/membership transforms and affected graph revision where required |
| Move cells | Source/destination directory and reverse anchor positions, stable binding relocation, memberships and invalidation |
| Delete a row/column | Axis retirement, content/anchor tombstones, missing references, membership updates and changed computation |
| Change widths/styles | Presentation/geometry roots and document state; graph revision stays unchanged unless an explicit semantic setting changed |

Reject a move with occupied destinations in the first proof. Later overwrite moves
must explicitly define displaced-target retirement. Large paste can be prepared in
bounded batches internally, but either commits as one logical operation or reports
an explicit different multi-operation contract; do not silently weaken atomicity.

Preserve exact decimal representations in staged portable content independently
of normalized runtime arithmetic. The graph remains a computational projection,
not a competing source for presentation metadata or original source text.

## Revision and snapshot rules

Every real committed document edit advances document state. A computational edit
also advances graph revision. Moving a cell with unchanged scalar dependencies may
change projection state only; if it changes a range's members, computational state
changes too. Inserting a blank row inside a bound selection changes membership
generation even if its current aggregate value happens to be unchanged. Numeric
representation changes can conservatively advance graph revision as today.

Snapshots pin graph definitions, captured inputs, relevant model definitions,
selection membership, settings and projection provenance from the same installed
root. A calculation on S can complete while edits install T. Results remain tagged
S and cannot be installed as T's current values. This preserves ADR-0002's immutable
calculation model without freezing the editable workbook for a whole calculation.

The first implementation can compose core WorkingModel::stage and its snapshot
builder for small models. They currently copy/rebuild whole graph structures.
Worksheet path-copy sharing alone does not make them cheap. Persistent/chunked
graph definitions and reusable dependency roots require separate core work and
measurements before spreadsheet-scale claims.

Undo, if added, restores coherent roots rather than reversing half the worksheet
and half the graph. Captured/saved states can retain old roots; retention must be
bounded by explicit policy. The existing S/T saved-checkpoint rule applies to the
whole future workbook capture, with no permanent file write per edit.
