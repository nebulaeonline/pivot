# Named Window identity, references and binding

Status: **Architecture accepted with clarifications under [ADR-0039](../adr/0039-named-windows-and-document-composition.md), 2026-11-01**.
Nothing in this document is an implemented API.

## Identity and authority

| Identity | Meaning / lifetime |
| --- | --- |
| WindowId | Non-reused workbook object UUID, independent of name/placement/presentation |
| CellId in Grid scope | Authored anchor, scope-qualified by WorksheetId or unbound WindowId |
| ProducerId + configuration revision | Authorized source definition, independent of views |
| ProducerGenerationId | Immutable successfully published source snapshot |
| Source item + FieldId | Producer-scoped stable record/field, not a displayed ordinal |
| PlacementId | Document-layout placement, no calculation ownership |
| DocumentGenerationId | Immutable coherent output capture, not a workbook revision |

Window creation/deletion, source changes and producer publication stage complete
candidates. Refusal leaves current heads and existing captures untouched. Ownership
mode change is not a toggle that converts content: detach requires explicit snapshot
materialization into a **new unbound Window**, with new owned IDs and no reference
rebinding. Rebinding to another producer similarly creates a new Window; deleting
the old one leaves its references faulted. Renaming never repairs retired identities.

## Reference grammar and resolution

Examples: `Sheet1!A1`, `LineItems|$A$9`, `'Customer Notes'|A1`, and
`'Bob''s items'|A1`. Quote names containing spaces, punctuation or ambiguous address
tokens; doubled apostrophes escape apostrophes. `!` selects worksheet namespace,
`|` selects Window namespace. Both endpoints of an explicit range resolve to one
Grid scope. Unqualified references use the formula's authored Grid scope.

Names resolve only during an explicit authoring transaction at expected revision.
Store the resolved owner and target identities, submitted source, spans and `$` flags.
Display uses current names/addresses; Save retains authoritative identities, not just
rendered text. Never treat `|` as an arithmetic operator or permit cross-workbook paths.

| Target | Bound meaning |
| --- | --- |
| Unbound Grid scalar | Existing CellId identity and canonical graph definition |
| Worksheet-bound Grid scalar | Window lifetime guard + source CellId; holes demand the source's ordinary blank anchor only when scalar binding needs it |
| External-bound Grid scalar | Window guard + ProducerId/item/FieldId; shared Decimal-or-fault captured leaf |
| Grid range in unbound scope | Spatial descriptor in that scope, using established insertion/deletion semantics |
| Worksheet-bound Grid range | Composed Window-local rectangle and authorized source SelectionId; candidate derives membership and checks cycles |
| External-bound Grid range | Unsupported computational operand initially; presentation selection remains allowed |
| Text A1 | Whole text object for presentation binding; not a scalar formula value |

Bound Grid display axes are generated from a declared producer order and field
schema. A reference authored at A9 resolves the item then displayed there. Reordering
or refreshing preserves that item's reference; it does **not** retarget to the next
occupant of A9. Formula display follows the item's new address. An absent item produces
a missing-item fault; it can resume only if the producer's contract guarantees the
same key names the same record. Sources without durable keys use generation-qualified
item IDs, so a new generation invalidates old item references instead of guessing.
Worksheet sources use existing CellIds; sorting and moves follow those identities.

A bound scalar also requires that its item remains in the Window's current producer
membership. Moving a source cell outside that membership yields an out-of-view fault,
even if the source CellId is still live elsewhere. Returning the same live identity
can restore availability; deleting it and reusing its address cannot. Source item
absence is distinct from permanent identity retirement. Formula display renders the
fault when no current Window address exists, retaining the original authority.

Worksheet aliases lower to the source's existing computational target while the
Window/membership guards are valid. Deleting the Window or changing membership
stages affected formula bindings to explicit reference faults in the same candidate;
it does not delete the source node. This is binding maintenance, not a second
calculation pass or a cloned value graph. Captured scalar contexts likewise include
Window guard state alongside the pinned producer generation. Old captures retain
their old guards and values without consulting current mutable directories.

Copy/fill transforms relative coordinates against the destination Grid scope and
then explicitly binds the target there. Absolute coordinates suppress translation;
an explicit scope qualifier remains that scope. Cross-scope copies of unqualified
references resolve in the destination scope. Out-of-bounds, missing-source or
unsupported-type targets refuse the copy transaction rather than silently changing
meaning. Copying an already faulted binding retains its fault authority, not a fresh
name lookup. Range operations retain the existing function's operand/type rules;
this architecture does not make a Window a general collection-valued scalar.

In unbound scopes, structural movement preserves surviving CellIds and retirement
faults removed targets. Bound axes are not directly editable; source structural edits
transform producer geometry under existing SelectionId rules. A deleted descriptor
or producer produces explicit source faults, even if its old name/coordinates recur.
Window deletion faults its guarded references even if source cells survive. An old
capture remains valid against its pinned lifetimes. No automatic target repair.

## Binding and publication

Worksheet-range producers pin committed workbook roots and exact typed values;
hidden rows remain source members unless an explicitly reviewed producer says
otherwise. Bindings carry a rectangular descriptor, not a copied set of cell values.
Source membership, projection and dependent calculation publish as one coherent
coordinator generation. The complete combined source/graph dependency candidate is
cycle-validated; a Window cannot disguise feedback through a producer.

External producers declare schema, durable-key contract, order, authorization
reference and refresh policy. Explicit refresh validates keys, field types, counts
and admission before atomic publication. Schema incompatibility rejects publication;
it does not relabel old fields. An unsuccessful refresh retains the selected last
successful snapshot, records failure/attempt metadata, and never publishes partial
rows. No previous snapshot means unavailable. Disconnection is visible on reopen.

Decimal outputs share one captured source leaf per producer/item/field binding,
with exact selected-generation context and lifetime guards. This extends the bounded
ADR-0020 pattern, not its experimental fixed limits or private authoring syntax.
Strings/Booleans/Blank remain exact in presentation. External non-Decimal formula
binding is rejected before graph installation; source faults propagate through the
canonical evaluator, never as cached numeric substitutes. Display formatting cannot
change the computational value. No per-projected-value CellId is manufactured.

## Editor and persistence

Chrome identifies name, ownership, live/snapshot generation and stale/failure state.
An upper-right properties control exposes binding/layout; refresh appears only for
explicit-refresh producers. Bound content is read-only. Selection/hit testing returns
a tagged authored cell, projected item/field/generation, Text position or chrome;
stale map targets are refused. Chrome is omitted from output unless explicitly added
as printable content. Removing chrome never hides source provenance from diagnostics.

Save stores Window/producer IDs, selected snapshot/schema and item identities,
authoritative formula guards, scopes, geometry and bindings. Reopen validates them
without query or name-based repair. Missing authorization keeps snapshots readable
under document policy but blocks refresh; document rendering still applies its source
admission policy. Sensitive snapshot persistence requires explicit document consent
and source policy; a source that forbids persistence makes Save refuse until removed,
never silently omit its values. Credentials and refresh permissions do not travel
with copied document content.
