# Unbound Grid Windows

Status: **Accepted and closed by parent implementation review (2026-11-01).**
The [accepted ADR](../adr/0040-unbound-grid-windows.md) is authoritative. All six
targets passed managed/AOT GUI qualification. Windows ARM64 headless is accepted
with a separate retained-cache pass; its original cold timing failure remains Failed
and unexplained. See the [qualification](../performance/unbound-grid-windows.md).

## Ownership and captures

GridScopeId tags distinct WorksheetId and WindowId types. Sparse positions, cell
references, formatting, filters, ranges, structural requests and transfer records
carry that owner. Windows share one canonical graph and allocation domain; they
are not extra worksheets, independent models or page rectangles. Up to 64 live
Windows coexist with up to sixteen worksheets without multiplying aggregate cell,
identity, byte or work limits. Closing/selecting an editor changes no content.

The private client stores selected scope locally. Every scope-dependent request
carries its ID and worksheet/Window discriminator. Queued reads retain the original
target across client selection changes. ScopeCapture pins the canonical calculation
generation and persistent roots containing axes, sparse indexes, formatting and
visibility; cross-scope dependencies use those same roots. Retired Window captures
can still be evaluated without consulting the current directory.

## Axis identity and admission

An empty axis needs one namespace/length descriptor, with no per-position objects.
After structural edits, immutable charged runs map logical ranks to stable sequence
identities; binary-search prefix indexes resolve positions. Inserted identities and
new tail slots consume fresh sequence numbers. Surviving axes preserve identity;
removed sequences never name later occupants. Run images and allocation cursors
persist in WB14. Old captures retain their old run roots and charges.

Declared dimensions, actual run/index allocations, cell anchors, graph identities
and retained retired metadata remain distinct. Metadata copies and validation scratch
are admitted before allocation. The dense structural oracle checks both axes through
320 edits; a retained-capture test exhausts remaining domain admission while an old
root remains pinned. Sixty-four maximum-dimension empty Windows allocate no cells or
graph nodes. These checks do not claim process-RSS measurements.

Legacy worksheet commands retain the accepted empty structural no-op policy.
Window structural edits publish axis identity changes even without authored cells.
For worksheet edits that already change content/format/ranges, axis runs preserve
the surviving identities too. Existing worksheet admission may grow on current-format
reopen; that appends fresh axis sequences. Explicit Window dimensions remain fixed.

## References and retirement

`Sheet!A1` and `Window|A1` select separate namespaces. Quoted names and `$` copy flags
retain original source and bound identities. Qualified bounds use the target scope,
not the author's dimensions. Rename updates rendered qualifiers without rebinding.
Deletion stages owned retirement and inbound scalar/range faults atomically. Required
tombstone authority survives Save/reopen; name/address reuse cannot repair it.

Workbooks containing live Windows cycle-validate the complete candidate graph before
publication/import, using cycle-only validation so unrelated invalid drafts do not
mask cycles. Creating the first Window refuses an already cyclic candidate. Legacy
worksheet-only editable-cycle behavior is unchanged. No second evaluator is added.

Trusted same-document copy translates relative references and creates destination
definitions. Explicit qualifiers retain their owner; unqualified references bind in
the destination. The private cross-scope copy currently refuses filtered source or
destination rather than bypassing exact-operation visibility confirmation. Cross-scope
cut/move remains unavailable. Ordinary in-scope Fill and movement retain their rules.

## Persistence, clipboard and editor

WB14 is the sole native format; WB13/older are refused without migration. Private
session v14 carries tagged targets; hosted v1 remains unchanged. Native validation
checks owner/qualifier namespaces, global UUID uniqueness, scope ordering, axis-run
overlap/counts/cursors, references/faults, graph membership and retired authority.

CB5/CB6 remain worksheet formats. Native Window Copy/Cut, native paste into Windows,
and worksheet exports containing Window references/faults refuse before OS clipboard
mutation. Imported native formulas cannot introduce Window authority through those
formats. Copy Values explicitly exports typed scalar text, preserving String prefixes
and leading-zero identifiers. Paste Text explicitly chooses textual interpretation;
ordinary preferred-native paste never silently falls back to text.

The Desktop has a separate Windows list, name and integer dimensions, create/rename,
open/select, deletion review and a virtualized Grid editor. Scope dimensions govern
viewport, navigation, headers, selection and structural bands. Deletion presents the
exact inbound occurrence count qualified by an expected-state, one-use ticket. A
failed/stale count or ticket refuses deletion. No binding/layout/refresh controls exist.

Producer bindings, Text, section geometry, document composition, PDF and history
integration remain outside this implementation. Experimental history/structured
paths refuse unsupported Window ownership rather than silently discard it.
