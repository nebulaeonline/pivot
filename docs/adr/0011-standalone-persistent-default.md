# ADR-0011: Standalone persistent default and shared runtime lifecycle

Date: 2026-10-19. Status: Accepted. Implementation: complete; verified on Windows, Linux and native macOS.

## Context

Stage 2 was accepted after current Windows, Linux and native macOS validation.
The parent authorized a controlled standalone default transition, while reserving
workbook, hosted-model and document-session migration for separate reviews.
Six bounded runtime workers must be shared rather than allocated per model.

## Decision

Enable core-integration in pivot-core's default features. `WorkingModel::new`
starts directly on persistent storage and lazily shares one process convenience
runtime. It stores execution resources, never a global mutable model registry.
`WorkingModel::new_in(name, &runtime)` and `runtime.create_model(name)` are the
preferred application/session path: callers own the runtime, its aggregate budget,
recovery and deterministic shutdown. `IntegrationRuntime::standalone()` chooses
ordinary settings; `configured(RuntimeConfig)` exposes budget and queue policy.

Defaults are a 2 GiB aggregate allocation-admission budget and capacity 64 per
work lane, with Stage 2's two mutation, two heavy and two cleanup workers. Stacks
are reserved at 256 KiB each; cleanup owner capacity is 1,088. This is an admission
ceiling, not preallocated 2 GiB and not an RSS cap. Several sessions can deliberately
share an allocation domain through the existing with_domain constructor.

Convenience runtime control is available through `with_convenience`: diagnostics,
try_flush and cleanup recovery remain reachable. Its callback holds the control
lock and must not re-enter default construction/control. `shutdown_convenience`
refuses outstanding owners without losing the runtime; successful shutdown joins
its workers, and later construction may lazily start a fresh runtime. Initialization
failure is retryable. Applications should use their own runtime to avoid unrelated
libraries sharing the convenience budget and shutdown boundary.

`new_reference` explicitly retains the oracle/fallback representation. Building
pivot-core with no default features also retains reference `new`, but feature
unification can enable persistent support; consumers requiring reference behavior
must use the explicit constructor. BackendKind reports actual selection. Existing
reference models may activate explicitly; activation on a different already-owned
runtime now returns an error instead of silently claiming to transfer ownership.

Host, storage, workbook and daemon constructors explicitly use new_reference.
The accepted transfer importer remains a reference compatibility boundary;
`runtime.import_portable` explicitly returns persistent standalone models. Frozen
wire/storage encodings and capacity limits are unchanged.

## Consequences

Local capture certifies only the requested closure at its immutable generation;
complete capture retains global diagnostics. Bounded pinned inspection is the
interactive read path. Complete snapshots/diagnosis remain explicit, admitted
O(N + E) compatibility work, including their slower-than-reference cost.

Stage 2 ownership, cancellation, cleanup failure recovery and fixed worker lanes
remain canonical. Shared-domain pressure rejects work atomically; it does not
silently fall back to reference. Large-generation reclamation remains off caller
threads. Transfer Images and some caller-owned buffers, the process recovery worker,
allocator overhead and OS resident memory remain outside exact domain accounting.

The default transition must pass normal standalone tests through persistent new,
reference-oracle/fallback tests, downstream regression and default-facing Release
measurements. Native platform evidence is specific to the tested source, not
inherited from Stage 2. Workbook/host/document integration, ranges, persistence,
C# IPC and Avalonia are outside this decision.


## Transition validation

Current Windows/Linux/native macOS default CI and compatibility gates passed,
including 264/265/265 combined tests, 21 fallback core tests per platform and all
ninety default-facing Release processes. Native source/build/test files stayed on
internal APFS and copied source hashes matched. See the
[transition handoff](../handoffs/2026-10-19-standalone-default-transition.md).
The parent accepted this decision; measured per-edit bulk construction
and complete-materialization costs are explicit acceptance tradeoffs. No native
per-child RSS high-water or RSS-guard claim is made.
