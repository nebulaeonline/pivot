# Persistent core — standalone default transition

Historical milestone: the subsequent owner decision in [ADR-0013](../adr/0013-canonical-engine-consolidation.md)
retires the reference runtime. Constructor-selection recommendations and source
links below describe the pre-consolidation revision; benchmarks remain evidence.

Date: 2026-10-19. Implements [ADR-0011](../adr/0011-standalone-persistent-default.md).
The accepted Stage 2 engine, parser, binder and arithmetic executor remain canonical.

## Construction and lifecycle

```rust
use pivot_core::model::{WorkingModel, integration::IntegrationRuntime};

// Small application: a lazily initialized shared convenience runtime.
let mut simple = WorkingModel::new("Example")?;

// Preferred application/session policy: one owned runtime, many models.
let runtime = IntegrationRuntime::standalone()?;
let mut first = WorkingModel::new_in("First", &runtime)?;
let second = runtime.create_model("Second")?;
let x = first.add_input("x", 4.into())?;
let capture = first.capture_local()?;
let result = capture.evaluate(&[x])?;
// Release models, views, captures, results and snapshots before shutdown.
drop((first, second, capture, result));
assert!(runtime.finish().is_ok());
drop(simple);
IntegrationRuntime::with_convenience(|r| r.try_flush())??;
IntegrationRuntime::shutdown_convenience()?;
```

Snippet uses an application error context; it is illustrative, not a standalone main.
Default policy is 2 GiB admitted aggregate memory and 64 queued tasks per work lane;
configure explicitly with RuntimeConfig or share a domain with with_domain. No model
spawns a new runtime. Direct creation allocates empty persistent roots without
legacy snapshot construction, copying definitions or enabling a prepopulated model.
All model identities/revisions still originate in the existing identity allocator.

Convenience control serializes initialization and shutdown. Its callback must not
recursively call new or convenience control. Calls already holding models keep the
runtime alive; shutdown refusal retains it. Failed admission/initialization does not
select a different backend. A successful shutdown allows later lazy restart.
The fixed process recovery worker remains process-lifetime overhead even after
per-runtime workers join. Neither runtime ownership nor its budgets imply database,
service, registry or durability behavior.

## Constructor impact audit

| Boundary | Selection after transition | Compatibility scope |
| --- | --- | --- |
| Ordinary standalone WorkingModel::new | Persistent with core-integration (default feature) | Existing complete standalone tests run here |
| WorkingModel::new_in / runtime.create_model | Direct persistent, caller-owned shared runtime | Same engine and semantics, explicit budgets/lifecycle |
| WorkingModel::new_reference | Reference | Oracle and controlled fallback |
| Core without default features | Reference new unless another dependency enables integration | Explicit constructor avoids feature-unification ambiguity |
| pivot-host management create | Explicit reference | Authoring, revisions, receipts and publication unchanged |
| pivot-storage EditableGraph new | Explicit reference | Document sessions and files unchanged |
| Legacy transfer::import | Explicit reference | Exact validated restore; no hidden runtime activation |
| runtime.import_portable | Explicit persistent | Standalone opt-in import, frozen limits |
| pivot-workbook Workbook new and research shell | Explicit reference | Worksheet/graph transaction migration deferred |
| pivotd demo and legacy probes | Explicit reference | Existing daemon/CLI publication and IPC path |

Boundary tests assert reference selection in actual host/storage/workbook constructors
and storage restore even when persistent support is enabled. Research fixtures and
differential oracles explicitly use reference construction; ordinary core model,
evaluation, invocation and public interface tests continue to use new. No source,
binding, Decimal96, no-op, revision or stable-identity semantics are changed.

## Calculation, resources and failure policy

Stage 2's [compatibility/ownership contract](controlled-core-integration-stage2.md)
continues to apply. Local capture/closure failures never certify the whole graph;
complete capture and snapshot retain exact global diagnostics. Pinned inspection
must not materialize full arrays. Shared allocation domains account common roots
once; retained S remains readable after T edits. Admission refusal preserves the
writer and its revision. Cancellation does not undo an already committed edit and
results retain their originating revision for stale-result decisions by the caller.

Each runtime still has fixed bounded mutation/heavy/cleanup lanes. At capacity 64,
1,088 pre-admitted cleanup slots bound distinct live owners and pending retirement;
clones share tickets. Releasing owners can make later admission possible. Queue
full/disconnection returns explicit errors. Full snapshot copies reserve their own
budget/cleanup lease; portable Image copy-out becomes caller-owned. Full snapshot
and named-interface work retain their explicit broader costs. No exact RSS promise
is made: process recovery resources, OS/allocator overhead, external request/image
buffers and caller copies require separate application accounting.

Keep runtime control and finish off interactive callbacks. try_flush exposes cleanup
failure; recover_cleanup restores fixed receivers. finish refuses outstanding users
or owners, drains and joins after release. Process-global convenience is a small-app
policy, while application-owned runtimes provide independent shutdown and diagnostics.

See the [performance report](../performance/standalone-default-transition.md) and
[handoff](../handoffs/2026-10-19-standalone-default-transition.md).


Current transition validation passes on Windows, Linux WSL2 and native macOS ARM64,
including default construction, fallback, lifecycle and explicit downstream
boundaries. The [handoff](../handoffs/2026-10-19-standalone-default-transition.md)
records all ninety Release processes, bulk-construction/full-snapshot tradeoffs and
the remaining parent decision. The strategic sidebar does not expand this slice:
no native-operator, SQL comparison, workbook migration or new language work is added.
