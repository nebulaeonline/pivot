# Persistent workbook production readiness

Historical milestone: the subsequent owner decision in [ADR-0013](../adr/0013-canonical-engine-consolidation.md)
retires the reference runtime. Constructor-selection recommendations and source
links below describe the pre-consolidation revision; benchmarks remain evidence.

Date: 2026-10-19. Parent accepted ADR-0012 and the bounded bulk foundation. This
follow-up investigates construction costs and recommends selection policy; it does
not switch production Workbook, host, document or storage constructors.

## Bounded change

The existing canonical AVL index now offers internal private-candidate insertion.
When an index node is uniquely owned, its inline payload and child links can be
updated in place. Shared nodes are copied before mutation. Rotations reuse owned
nodes and preserve the same ordering, height/length invariants and charged Node
representation. Heap-owning key/value payloads retain ordinary functional insertion;
no payload growth occurs inside an already charged allocation. This is the same
ownership principle already used by canonical graph pages, not a second worksheet
or graph representation.

Only the private workbook transaction uses this insertion for its existing four
indexes: positions, records, bindings and reverse node lookup. Ordinary Index.insert
retains its atomic functional contract. **insert_private may partially modify its
private index on admission failure: its caller must discard the entire candidate.**
Workbook does so through its admitted Reclaimable guard. The current published
state and any S captures remain shared/immutable until the final transaction commit.
Deletion retains the existing functional implementation. UUIDs, CellId/NodeId/
WorksheetId independence, formula provenance, draft semantics, quota checks,
cancellation polls, revision rules and atomic installation are unchanged.

A research-only cached PIVOT_IMMUTABLE_INDEX control selects the old functional
insertion through the same caller, binary, fixtures and runtime. It changes no
validation or limit policy. It is compiled only with scaling-experiment and is not
a production backend selector. Ordinary product builds use ownership-aware insertion
only inside the already opt-in persistent Workbook.

## Construction attribution

The readiness example includes the accepted bulk fixture/oracles rather than
forking them. --profile enables independent global cross-worker phase counters:
request construction; identity/anchor request planning; worksheet index updates;
spreadsheet parse/lowering; reference binding/reverse links; core install; graph
page and name-index work; validation; admission/quota accounting; publication; and
cleanup. Allocator bytes/calls use the active phase. Nested phases are exclusive
within a thread. Caller dispatch wait overlaps worker phases and cleanup can overlap
caller work: do not add all thread-wall times into a fictitious CPU total.

Fine-grained timers, allocator counters and atomic aggregation perturb timing,
particularly the per-allocation admission probe. Profile runs establish attribution;
ordinary non-profiled matched runs establish performance/distributions and gates.
Completed-child CPU and OS RSS high-water are recorded via wait4 on Unix. Windows
retains its process-peak query and labels CPU unavailable. On macOS live RSS admission
guarding is still unavailable: post-exit high-water is evidence, not a live limiter.
The native validation packet must retain external critical-memory-pressure stopping.

## Compatibility matrix

The accepted shared suites are executed unchanged through both constructors. The
identity-preserving 256-transaction differential test compares exact public/core
revisions, cell contents, spans/provenance, diagnostics and selected values.

| Supported behavior | Evidence |
|---|---|
| Sparse numeric creation and demanded referenced blanks | shared projection: empty_grid_is_sparse_and_referenced_blanks_are_explicit_demanded_anchors |
| Decimal/formula replacement; exact raw decimal parts | shared projection: content_replacement_preserves_ids_and_references; exact_submitted_decimals_are_metadata_separate_from_blank_zero_backing |
| Clear, move, retire, coordinate reuse, explicit rebind | shared projection: move_clear_retire_reuse_and_explicit_rebinding_have_independent_expected_results; persistent transaction S capture test |
| Invalid editable formulas; local versus global validation | shared formula suite; typed_cycle_drafts_are_diagnosable_and_global_snapshot_limit_is_explicit; persistent unrelated-draft test |
| Atomic multi-edit failure, identity/quota/stale rejection | shared projection rollback/resource/stale suites; persistent admission/cancellation tests |
| Original source, references, spans and revision provenance | shared formula suite and exact 256-step differential oracle |
| Immutable S/T values, definitions and positions | shared capture suite; persistent old-content test; retained-generation churn |
| No-op, net-noop and stale revision/session rules | shared projection: ordinary_and_net_noops_do_not_advance_public_state_or_graph_revision; stale_expected_revisions_and_sessions_reject_without_installation |
| Bounded rectangle reads and admitted copies | shared rectangle suite; persistent transient copy-out admission test |
| AVL invariants, shared roots and partial allocation failure | two independent ordered-map/private admission tests in canonical index |
| Repeated construction, failed candidates, cancellation and churn | four 1K-cell product configurations, late-invalid transaction, admitted-budget failure, 8,192 edits and 128 retained generations; per-round baseline and final zero charge |
| Host/storage/document feature-unification boundaries | full workspace regressions retain explicit reference constructors and frozen codecs |

Sources: [shared projection suite](../../crates/pivot-workbook/tests/suites/projection.rs),
[shared formula suite](../../crates/pivot-workbook/tests/suites/formulas.rs),
[transaction/lifecycle tests](../../crates/pivot-workbook/tests/persistent_transactions.rs),
[differential oracle](../../crates/pivot-workbook/src/persistent.rs) and
[canonical AVL tests](../../crates/pivot-core/src/model/structural/index.rs).

## Memory and caller scheduling

Graph pages, worksheet indexes/records, private candidates, journals and copy-out
reservations share the application runtime allocation domain. Incoming edit Vecs,
source Strings and returned caller-owned views live outside that ledger; request
reservations are conservative capacity admission, not a process RSS ceiling. The
counting allocator and OS high-water observe these buffers too. Returned copies
stop being charged when ownership transfers; retained captures stay charged until
last owner cleanup. Shared runtime admission and bounded queues retain aggregate
protection across models; last-owner root destruction remains off the caller.

Large worksheet preparation is still synchronous O(request) caller work. A future
C# application must schedule authoring on its session/service worker and publish
one atomic result; it must not paste on its UI thread. No C# or async API is added.
Retain the accepted native warning-pressure finding and distinguish new native
high-water evidence from live enforcement. Production caps remain unchanged.

See [matched results](../performance/persistent-workbook-readiness.md) and the
[review handoff](../handoffs/2026-10-19-persistent-workbook-readiness.md).


## Constructor assessment and application recommendation

| API/path | Current policy | Recommendation |
|---|---|---|
| Workbook.new | Reference implementation, production limits, existing call sites | Retain unchanged in this slice as compatibility entry point/oracle |
| Workbook.new_in | Persistent implementation with caller-owned shared runtime, same production caps | Preferred candidate for the next bounded application/session integration after review |
| Research constructors | Explicit larger fixtures and end-of-batch reference coordinator | Never substitute them into application/session constructors |
| Host/storage/document | Explicit reference boundaries | Do not migrate indirectly through feature unification |

The proposed adoption is limited to the currently supported sparse single-sheet,
scalar-decimal operation set. The application/session owner must retain the runtime,
schedule large authoring off the UI thread, bound outstanding input/result buffers
and release or drain captures before final runtime shutdown. Library domain admission
does not regulate arbitrary copies retained by a caller after return.

Do not implicitly give Workbook.new a hidden global runtime as part of a performance
fix. If the owner later chooses a default transition, settle the explicit reference
fallback and runtime lifecycle policy first, then change constructor selection in a
separately reviewed slice. The present recommendation is to prefer new_in for the
next application/session implementation, keep the fallback/oracle, and stop backend
optimization once current gates pass with an explained remaining cost. It is not a
claim of complete spreadsheet compatibility, file persistence or unconstrained size.


## Verification status

Frozen-source Windows/Linux/macOS full default CI passes (251/252/252 tests), all-feature
regression passes 300/301/301, fallback 21 each and workbook-only 49 each. Strict feature
lints, .NET, parity and actual IPC pass. All 234 ordinary/profile/core children
pass with identical 68 source hashes and zero final domain charge. The fresh native
APFS run matches all 296 copied source files before documentation edits. Normal
pressure in its 52 samples does not invalidate earlier warning-pressure findings;
completed-child high-water is evidence, not a native live RSS guard.
See the linked performance record for full distributions, instrumented cost
attribution, CPU/RSS accounting and the remaining construction/memory limitations.
The production constructor decision remains for review.
