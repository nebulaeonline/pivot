# ADR-0041 — bound Grid Windows design handoff

Date: 2026-11-02.
Disposition: **Design approved with four mandatory clarifications incorporated. No implementation or platform validation.**

The parent accepts the architecture. All four required contract refinements below
are now incorporated in the ADR; no additional design-confirmation cycle is required.
Implementation still requires separate owner authorization.

[ADR-0041](../adr/0041-worksheet-range-producers-and-bound-grid-windows.md) specializes
ADR-0039's second scope on the accepted ADR-0040 tagged-owner/capture foundation.
It introduces read-only worksheet-range views and explicit-refresh fixture views,
without a database framework or another calculation engine.

Review these decisions:

1. Worksheet ranges use existing SelectionId structural affinity. Local A1 maps the
   physical top-left, with no header inference or visibility compaction. Blanks and
   hidden positions remain members. Source formatting is read-only; bound views have
   no independent filter/edit/style/sort/structure commands.
2. Scalars alias source CellIds with Window/producer/membership guards, not new alias
   nodes. Sorting follows source identity. Leaving membership faults; returning the
   same live identity may resume. Retirement/address reuse never repairs authority.
   Ranges compose into guarded source SelectionIds; partial exclusion faults the
   operand. Candidate cycle validation includes producer relationships and dormant
   reactivatable bindings, even when unrelated invalid drafts exist.
3. Fixture schemas and key policy are immutable configuration. Durable exact keys
   can resume the same contracted record; generation-qualified keys never retarget.
   Strings/leading zeros, Blank/empty String and Decimal metadata stay exact. Only
   declared Decimal fields use shared captured leaves; other types are presentation
   only, with explicit formula refusal. External computational ranges stay deferred.
4. Refresh begins explicitly. Session/configuration/connection/attempt-qualified
   tickets reject obsolete completion. Validate the whole payload and dependent
   candidate before one install. Failure preserves last good data with visible stale
   status; reads never refresh. Captures pin source roots/guards/contexts with the
   same canonical calculation generation.
5. Delete, disconnect, detach and replacement differ. Disconnect retains snapshots;
   detach materializes exact values into a new unbound Window under existing limits;
   changing source creates replacement objects. None silently retarget references.
6. Proposed WB15/private v15 cuts encode producer generations and guard authority;
   final numbers require implementation audit. Reopen never fetches and external
   snapshots open disconnected. Hosted v1 and CB5/CB6 stay unchanged, with explicit
   native Window clipboard refusal. Credentials/capabilities/tickets are not saved.
7. Keep aggregate limits. Proposed 32 producers/100,000 logical view rows specialize
   ADR-0039; external schemas have 64 fields and 65,536 aggregate record-field slots.
   Payload shares existing source-byte admission, metadata shares workbook/domain
   budgets and demanded anchors/leaves consume existing node/identity allowances.
   Selected, retained and staged generations are charged concurrently. These caps
   are refusal boundaries to qualify, not promises that all maxima fit together.

Compared with ADR-0040, this design adds producer ownership variants, source membership
maintenance, immutable external contexts, refresh status/authority and read-only view
protocol. It retains client-only selection, one canonical graph/capture, tagged
identity, atomic lifecycle and aggregate admission. It does not extend ADR-0039 into
Text, geometry, layout, PDF or invocation.

The validation plan includes source edit/sort/structure/visibility, dense holes,
exact types, key generations, refresh races, old captures, full candidate cycles,
deletion and malicious Save/reopen authority, plus staged resource checkpoints and
six-target headless/managed/AOT GUI qualification. macOS x64 is explicitly Rosetta.
ADR-0040's original Windows ARM64 cold timing failure stays unresolved; no new design
or later pass reclassifies it. Owner-reported Rust nightly updates are noted for
future receipts; no current-platform validation is claimed.

## Mandatory clarifications incorporated

1. **Positional versus identity authority.** The normative `Data!B2:B4` / `Items`
   example combines a scalar bound to c3, spatial SUM and ALLOCATEWEIGHTEDRANGE.
   Insertion preserves physical Blank slots; sort follows scalar identity while
   changing ordered range weights. Moving c3 out/back can fault/resume the same live
   scalar; deletion/replacement cannot. Whole descriptor retirement remains permanent.
   A live partially excluded range faults as a whole and can resume only on restored
   containment of the same descriptor. Ordinary nested structural transformations
   normally preserve containment; state-oracle cases do not authorize source rebind.
   Combined command/oracle and Save/reopen evidence is required for each fault state.
2. **Refresh state, values and freshness.** An explicit matrix separates selected
   generation, latest attempt status, workbook expected state, canonical calculation
   revision and future render admission. Failed/cancelled attempts keep G1 calculable
   with visible failed/cancelled status and no selected G2. Obsolete completion changes
   neither selected data nor newer status. Successful changed contexts publish affected
   dependencies atomically; status-only transitions do not advance calculation revision.
   Future renderer policy may refuse retained data; no renderer/TTL is implemented here.
3. **Native authority gate.** Freeze WB15/private v15 only after auditing relationships
   among producer/configuration/Window, schema/fields, key policy/generations, selection
   membership, guarded/retired/reactivatable faults and shared output contexts. A valid
   payload digest cannot authorize those relationships. Hostile digest-recomputed
   fixtures must still refuse forged authority atomically. Saved external snapshots
   reopen disconnected/readable; credentials, capabilities, tickets and executable
   provider instructions never travel. Unsupported state cannot become unbound cells.
4. **One ownership/charge ledger.** The ADR assigns owner, identity responsibility and
   actual byte/lifetime charges to metadata, keys, selections/guards, shared leaves,
   payload/indexes, old captures, superseded staging and decoder/validation scratch.
   Shared allocations charge once; independent references/indexes/pins pay separately.
   Identity counts remain distinct from bytes. A required 32-producer/64-Window
   coexistence workload uses one 128-row selected fixture generation, 31 worksheet
   producers, 1,248 logical view rows and 1,088 source/consumer/leaf nodes. Old capture
   and staged replacement variants must fit unchanged limits, with reconciled ledger
   totals. Failure returns measured evidence for review, never a silent budget increase.

The exact transition matrix, ownership table and native acceptance gate are normative
implementation requirements, not optional documentation examples. Preserve the
approved six-target plan, source qualification, archive/cleanup cadence and unresolved
ADR-0040 Windows ARM64 cold-run timing issue. Stop here for owner implementation
authorization. No subsequent design or implementation is started by this handoff.
