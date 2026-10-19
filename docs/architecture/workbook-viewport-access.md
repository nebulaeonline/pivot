# Viewport and bulk worksheet access proposal

Status: Accepted bounded-rectangle direction under ADR-0010. Slice A provides only basic
in-process content reads; caching, geometry transport, RPC and GUI remain future work.

Review clarification: ADR-0010's structural architecture is Accepted. Reference
transforms, range boundaries/movement, blank/text coercion, reduction order and
shared writable aliases remain separately reviewable compatibility semantics.
[Slice A](workbook-projection-foundation.md) implements only the bounded typed
single-sheet proof; future mechanisms described here are not claimed implemented.

## Bounded rectangular reads

A read identifies SessionId, WorksheetId, expected document/projection state and a
rectangle in the current axis ordering, plus requested field groups and explicit
byte/cell/work budgets. A result identifies the exact captured roots and calculation
generation, returns sparse records with rectangular coverage, and indicates whether
numeric values are current, stale, pending or failed. It must not silently combine
definitions from T with results from S and call the combination current.

Fields can be requested separately: content/editor source, computed values/errors,
style IDs/format metadata and geometry. Do not retransmit full formulas/styles for
every unchanged visible value. Include stable CellIds where materialized and axis
identity/order information needed to reconcile positions. Empty-region coverage is
explicit even when no cell records are returned; absence cannot ambiguously mean
the server omitted data because a budget was exhausted.

Split large replies into bounded chunks pinned to one immutable read capture.
Continuation state binds rectangle, requested fields, revision and progress. Reject
or restart after expiry rather than continuing against unrelated roots. Specify
explicit minimum-progress or item-too-large errors so a long source cannot cause an
endless empty page. The initial in-process contract can be an iterator over bounded
owned batches; a future wire contract needs its own negotiation and security review.

No one-request-per-cell design, unlimited payload, or increase of the current 1 MiB
IPC cap is proposed. Future bulk transport and document transfer remain distinct
contracts. Bounded result chunks should work over ordinary IPC before shared-memory
transport is considered.

## Cache and asynchronous delivery

Cache entries identify session/sheet, rectangle coverage, requested fields and
provenance. Initially DocumentStateId can invalidate the whole entry; later separate
projection/content/format/geometry component versions allow selective reuse without
inventing incompatible meanings for old tags. Value caches additionally require
graph revision and calculation generation/override provenance.

Tag requests with client-local viewport generation. When scrolling or structural
edits supersede an outstanding request, cancellation is best effort; reject late
results by generation/state before installing them into the active viewport.
Old immutable entries can remain reusable if deliberately addressed as old state.

Prefetch a bounded neighboring region under an explicit memory and concurrency
budget. Cap outstanding requests and total retained bytes; rapid scrolling must
not queue an unbounded sheet scan. Eviction discards derived client caches, not
authoritative graph definitions. Optional change descriptions can invalidate
rectangles/IDs, but a missing event or generation gap requires refresh.

## Geometry, selection and editing

Geometry reads provide row/column run lengths, default/exception sizes, hidden
intervals and cumulative offsets sufficient for pixel-to-axis lookup. Avoid one
height query per row and avoid allocating all row objects merely to compute scroll
position. Geometry belongs to workbook presentation, with indexed prefix sums over
run metadata as a candidate implementation.

Selection, active cell, drag preview and uncommitted editor text are client overlays.
The formula bar can display editable submitted source or rendered bound references,
but commits must carry the state against which that display was resolved. Submit
one bounded edit/paste operation; preserve the local editor on conflicts instead
of silently resubmitting against new coordinates.

Formatting/rendering belongs in the client or a shared presentation formatter,
using exact authoritative values and format metadata. It must not interpret or
calculate formulas. Future Avalonia virtualization can consume these batches and
geometry independently of core graph internals. Drawing and calculation run without
holding the workbook edit gate.
