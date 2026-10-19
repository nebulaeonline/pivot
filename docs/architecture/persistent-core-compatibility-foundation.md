# Persistent core compatibility foundation

Status: opt-in compatibility foundation, not a replacement WorkingModel.
Recommendation: **Further focused restructuring required** before production
integration. The persistent graph direction remains supported.

This extends the accepted [structural experiment](structural-mutation-feasibility.md).
No persisted layout or public storage ABI is selected. Core remains independent of
workbook coordinates, files, hosted resources, transport and UI.

## Persistent structural indexes

Names and reverse dependencies now use immutable AVL indexes with deterministic
balancing and shared child roots. Updating one edge or one name allocates paths
proportional to tree height rather than copying the directory or adjacency set.
Sorted traversal is available for diagnostics/export. Forward dependencies remain
inside shared immutable compiled definitions; replacing a formula replaces its
own tokens/dependency vector and updates only changed reverse links.

The record/value page directory remains the fixed-radix, 64-slot research design.
Stable NodeId values identify logical nodes, never physical addresses. Unrelated
compiled definitions retain their shared identities. Retired IDs are never reused.
AVL tests compare rotations, deletion, ordering and retained roots against an
independent BTreeMap; benchmarks exercise one, medium and near-total graph degrees.

## Existing engine semantics

The prototype now supports named inputs/formulas, stable-reference renaming,
addition/replacement/removal, malformed or unresolved editable drafts, public input
and output ports, required inputs, optional defaults, port updates/removal and
temporary invocation overrides. Capture results are immutable and generation tagged;
results retain the capture they evaluated. A new capture allocates one charged
shared state, and cloning that capture shares it.

To avoid a second set of authoring rules:

- Accepted name binding delegates to a shared lookup-based binder. The old path
  supplies its BTree directory; the experiment supplies its persistent directory.
- Input-override checking, named request preparation, interface target validation
  and public error conversion are shared with the accepted evaluator.
- Port authoring uses accepted WorkingModel port methods through a private proxy
  containing the interface and relevant target classifications, not every graph
  definition. Saved invalid port targets survive omitted-field patches as before.
- The existing postfix arithmetic executor remains the only arithmetic evaluator.

Standalone source binds at submission. Renaming or reusing a retired name does not
rebind historical tokens. Explicit formula repair uses the current directory.
Same-name renames and empty port patches preserve the experimental version.
Input assignments retain the accepted standalone model's advancing behavior.

One serialized writer owns a version domain; clones are private candidates.
Aggregate commit, topology and input generations remain experimental tags. A
materialized WorkingModel gets a synthetic revision derived from import origin and
commits; this is not a promised numerical equivalence to per-operation revisions or
the hosted service's public batch revisions. Their integration requires review.

## Restoration and diagnostics

The private adapter imports through accepted transfer::import, then copies stored
definitions/bindings without parsing source. Export materializes private working
fields and calls accepted transfer::export. Allocated node/port ledgers, retired
references, original source, numeric parts, port requirements and invalid bindings
survive. Existing transfer admission caps remain unchanged: a large in-memory graph
does not thereby become an admitted portable payload. Images returned to callers
are caller-owned copy-out data, not a newly persisted representation.

Initially invalid models are accepted. Structural repairs use a budgeted full
accepted snapshot check until the imported draft becomes valid. Ordinary changes
to valid graphs use bounded dependency-closure validation and recheck diagnosed
roots. Valid capture checks public ports and pins roots without copying all nodes.

Graph::diagnose provides the accepted complete Kahn diagnostic, including affected
dependents, through guarded materialization. Capture rejection can still contain a
representative local cycle path; exact detail equivalence is not claimed for that
research API. Production integration must choose an explicit diagnostic contract.
Full diagnosis and initially invalid restoration can perform O(total graph) work;
they are separate from the ordinary valid formula-edit gate.

## Allocation admission

Each graph has a configurable shared allocation domain, defaulting to 2 GiB for the
experiment. Clones, retained generations, captures, input/record pages, radix/AVL
nodes, compiled buffers, names, port structures and reclaimer jobs share that
domain. An allocation object is charged once and releases its charge on last drop;
sharing a root does not charge every referenced object again.

Known object sizes, Arc control headers and owned buffer capacities are charged.
Port BTree storage uses conservative fixed-root and per-entry allowances. Admission
also reserves conservative space for parsing, candidate headroom, validation,
calculation, materialization and import. Work reservations follow returned results
until result drop. Charged captures can fail explicitly with CaptureError::Admission.
Hard rejection discards private candidates without installing graph changes.
Headroom includes trusted token counts, prior outgoing dependencies, affected
retirement dependents, existing invalid roots and transaction bookkeeping; a tiny
replacement source does not hide a large pre-existing adjacency operation.

MemoryStats distinguishes allocated charges from reserved headroom. The sum is the
admission counter; its peak is **not** actual allocator heap or RSS. The rejection
benchmark deliberately reserves almost the full budget and therefore pushes that
counter near its limit without allocating corresponding physical memory.

This is an initial bound on known requested engine storage plus work allowances,
not an exact global allocator or OS resident-memory limit. It excludes allocator
arenas/reserves, the small domain ledger itself, caller-owned request/copy-out data,
accepted source models outside the domain, application containers and most frozen
workbook projection state. New worksheet metadata pages use the core domain, but
the whole worksheet is not yet governed by it. Multiple graph domains also require
application-level aggregate admission. The separate OS RSS guard remains a
research safeguard, not a substitute for those missing policies.

## Retention and reclamation

Retained generations pin charged objects; releasing one owner frees only unreachable
paths. Retired record/value metadata is charged and bounded by allocated-node and
byte admission. It is not compacted away or allowed to recycle an identity.

An explicit bounded Reclaimer accepts obsolete owners for destruction on a worker.
Queue-full, worker-disconnected and admission failures return the original owner;
they do not silently destroy a large generation on the submitting thread. Worker
stack/queue/job headroom is reserved. Shutdown joins the worker and belongs off the
interactive path. Benchmarks measure enqueue, join and ordinary last-owner cleanup.

Ordinary Rust Drop remains synchronous if callers bypass the reclaimer. Wiring all
publication, cancellation and shutdown ownership paths to a reviewed policy is
still production work. There is no claim that all reclamation is constant time.

## Integration gate

Before a controlled production switch, settle diagnostic/version/no-op adapters,
host final-directory batch binding and publication, interface scaling with many
ports, global memory/copy-out budgets, retirement compaction, invalid-draft repair
work limits, cancellation and reclaimer ownership. Extend the fixed workbook
projection's mutation coverage separately. Preserve frozen codecs and document
provenance; no production data-format migration is needed for this ownership change.

Incremental result caching remains excluded. Requests still evaluate their selected
dependency closures using the accepted executor. C#, Avalonia, new syntax/ranges,
workbook files and hosted persistence remain held.

See the [benchmark specification](../performance/compatibility-foundation.md) and
[review handoff](../handoffs/2026-10-19-compatibility-foundation.md).
