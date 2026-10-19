# Structural mutation feasibility

Status: bounded experiment, behind `scaling-experiment`. Production replacement
requires a separate review. The previous assessment B is accepted; this slice
investigates its unresolved structural-edit path.

## Representation comparison

| Candidate | Local mutation and capture | Retention / risk |
| --- | --- | --- |
| Mutable indexed arena | Excellent locality, cheap slot access | Captures need full copies, a journal or a separate versioning layer; slots must not become public identities |
| Immutable chunks, copied flat directory | Copy one chunk plus O(N/chunk size) pointers; root capture cheap | Simple and compact, but directory work still grows with total graph size |
| Persistent ordered map / prior treap | Expected logarithmic replacement, cheap root capture | Per-entry allocation, pointer chasing, randomized balance; input-only results do not prove structural editing |
| Paged arena with persistent fixed-radix directory | Copy one 64-slot page and at most 16 small directory branches per changed page; capture pins roots | More directory overhead than a flat array; deterministic bounded path, no balancing assumption |
| Mutable arena plus MVCC log | Potentially compact writes | Reader visibility, historical lookups, compaction and safe reclamation add substantially more machinery |

Test the paged/radix option. This is a reasoned candidate selection, not a measured
victory over every candidate. No dependency package or new production ABI is added.
The older treap remains a separate, reproducible input-only experiment.

## Ownership and identity

`pivot_core::model::structural::Graph` owns immutable record pages, separate input
value pages, a shared name directory, and a small diagnostic directory. Each live
record holds an immutable compiled definition and a reverse dependency set.
Definitions contain the accepted bound postfix tokens and explicit forward IDs.
Physical pages are indexed by the process-local node sequence; `NodeId` still
includes model identity. A slot is never recycled into a retired identity.

The version contains model identity, aggregate commit, topology revision and input
generation. Captures pin those roots and that version. Results belong to the capture
that produced them; there is no persistent result cache or automatic dirty-output
selection. The same production `evaluate_expression` executes arithmetic and error
propagation. Selected closure scheduling is an adapter to that executor, not a
second decimal evaluator. Core has no workbook, file, IPC or UI dependency.

One serialized writer owns a version domain. Clones are private transaction
candidates, as with the accepted WorkingModel::stage seam; publishing divergent
mutable clones as independent resources with the same domain is unsupported.
A future public fork operation must allocate a distinct model/version identity.

The fixed-projection workbook adapter atomically installs a graph candidate and
metadata page roots under the graph's aggregate version. Its immutable projection
pins stable cell/node mappings, axes and positions. Exact submitted DecimalInput
parts remain in metadata, separately from effective core numeric values. Original
formula text and bound reference locations remain provenance. Old accepted content
is explicitly tagged `Original`, rather than inventing new accepted Expected tags.

## Structural transactions

The candidate begins by cloning roots. Replacing a formula replaces its definition
and changes reverse links only for the symmetric difference of old/new dependencies.
Unchanged definitions retain their Arc identities. Values change through their own
pages. Conversion from formula to input removes the old outgoing edges.

Validation walks the changed definition's dependency closure using explicit stacks,
then rechecks previously diagnosed roots so repairs can restore a calculable state.
Missing targets, parse/binding failures and cycles are editable drafts; capture
rejects the entire invalid graph. No obsolete topological order is silently reused.
The cycle diagnostic identifies a detected active path, not the production Kahn
diagnostic's complete set of cycle members plus affected dependents. Diagnostic
categories and rejection agree; exact node-list equivalence is not claimed.

A shared visit budget bounds validation work per transaction. Exhaustion is a hard
failure, not a clean validation result; all graph and metadata candidates are
discarded. Stale expected versions, malformed trusted transfers, unsupported edits
and quota failures also install nothing. Existing parser/binder and the accepted
trusted-transfer stack checker are reused. Name-based source binds at submission;
later name reuse never repairs an existing stable-ID reference implicitly.

Retiring a node leaves a bounded tombstone and its inbound references. Its outbound
reverse links are removed. Retired identities cannot be resurrected; a new formula
with the same name receives a new ID. Explicit formula resubmission can bind the
new target. Formula addition supports existing names and self-reference diagnosis.

## Known broad operations and scope limits

- Reverse sets are Arc<BTreeSet<NodeId>>. Changing one edge can copy the full set
  at a high-degree target. A paged edge index is a production follow-up.
- Name addition and retirement copy the BTree name directory when shared. Adding
  formulas is supported for semantics, not demonstrated as a localized name-index
  operation. A persistent name index is still needed.
- Validation follows dependencies; a long chain or broad selected dependency closure
  can visit the whole graph. This is a bounded traversal, not a constant-time cycle
  oracle. Existing invalid roots can make repair expensive.
- The workbook prototype handles existing anchored cells and decimal/formula text
  replacement. New workbook anchors, moves, retirements and axis changes remain on
  the accepted path and are not claimed optimized. Core addition/retirement tests
  exercise the underlying stable-ID semantics separately.
- Core import requires a valid source model without public ports. Port validation,
  editable invalid import, storage transfer/publication and native workbook adapters
  require explicit integration before production adoption.
- Allocated node and definition-byte quotas are incremental logical limits, not RSS
  admission. Names, indexes, old generations and allocator overhead are additional.
  The fixed-projection adapter's byte accounting is a research metadata allowance,
  not the accepted complete workbook quota formula.

## Retention and production policy

Arc roots reclaim unreachable pages on the last drop. Cleanup is measured explicitly;
the last owner can pay work proportional to the freed generation. No background
reclaimer is implemented. The benchmark retains 16 structural generations and an
older S, rather than only repeatedly changing a value under one topology.

Before adoption, require a session-level physical-memory policy: cap live captures
and retained generations, charge shared pages once to their owning store, reserve
candidate headroom before installation, reject work before OS exhaustion, and run
large reclamations off interactive threads. Tombstones need an identity-preserving
compaction policy; their slots cannot be reused while historical references exist.
The runner's 8 GiB process guard is an experiment safety limit, not that policy.

Proposed 100K local-formula target: p95 below 10 ms for prepare, bind, edit,
validate, capture and selected output evaluation. This leaves room within a 16.7 ms
interactive frame for client work. It applies to short formulas with small
dependency closures; broad pastes, huge reverse sets and long graph traversals
need explicit work budgets and separate expectations. Existing provisional input
and capture targets remain 5 ms, and the 1,000-formula input pipeline remains 25 ms.

See the [experiment specification](../performance/structural-experiment.md) and
[handoff](../handoffs/2026-10-19-structural-mutation-feasibility.md) for evidence and
the production recommendation.
