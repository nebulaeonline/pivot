# Controlled production core integration — Stage 2

Date: 2026-10-19. Implemented for explicit standalone activation; review required before default selection. Supersedes the Stage 1 operation/runtime restrictions, without changing the default constructor.

## Compatibility matrix

| Contract | Activated standalone core |
| --- | --- |
| Inputs, formula submission/replacement, stored invalid bindings | Supported through WorkingModel and canonical parser/binder |
| Node rename, retirement, name reuse, explicit stable-ID repair | Supported; historical bindings never rebind by name |
| Public input/output ports, rename/remove/retarget, defaults and patches | Supported; existing validation and no-op rules preserved |
| Staging, rollback, expected revision and installation | Shared private roots; only successful changes advance the actual public core revision |
| Raw overrides and named invocation | Existing validation, tagged decimal values, captured defaults and arithmetic executor |
| Immutable old S / editable new T | Captures, results, views and full snapshots pin their own generation |
| Complete snapshot and global structural diagnostics | Explicit heavy materialization; accepted diagnostic ordering and payloads |
| Portable import/export | Shared accepted importer/exporter; exact stored source/bindings, decimal parts and retired ledgers |
| Workbook, host/service publication and persistence | Unchanged reference paths; migration requires separate review |

No second public graph engine, dependency, parser or arithmetic implementation is added. Core remains independent of files, IPC, workbooks and UI. The research Graph API remains separately feature gated.

## Capture, inspection and diagnostics

`capture_calculation()` retains the old complete global-validation contract. A certified persistent root captures cheaply; an uncertified root performs exact heavy diagnosis first. Callers requiring interactive behavior must explicitly choose `capture_local()`: it pins the draft without global diagnosis, and invocation validates the requested dependency closure. Unrelated invalid definitions do not prevent that calculation. `CalculationFailure::LocalInvalid` is a representative local witness, not a complete graph report. Complete diagnosis still uses the accepted snapshot validator. A repaired initially-invalid root is not globally materialized by local editing or capture; certification stays conservative until explicit complete validation succeeds.

The same local contract exists on the reference backend, using a staged source and the accepted selected-subgraph snapshot/evaluator. Reference capture copies the source; it is an oracle and compatibility adapter, not a cheap persistent capture.

Activated `inspect()` returns a pinned `DefinitionView`. Point lookup borrows definitions and bound tokens. Node pages admit at most 256 headers and scan at most 4,096 slots plus a lookahead; reverse/forward edge pages admit at most 256 IDs plus a lookahead. Cursors belong to an exact view nonce, so equal revisions on independent staged branches cannot mix pages. Node and reverse-index cursors seek directly into persistent indexes. Port metadata is borrowed through the existing interface iterators. Large token slices are borrowed rather than copied; consuming them remains caller work.

`snapshot()` and view `snapshot(cancel)` explicitly construct complete legacy arrays and adjacency/order metadata. They reserve a conservative copy budget before allocation and run on the heavy lane. Snapshot clones share the resulting allocation and cleanup lease. There is no hidden claim that this operation is constant time. `validate_complete(cancel)` has the same whole-graph scope. Materialization checks cancellation between records and validation steps; traversal checks between work items. Cancellation does not roll back a previously committed edit.

## Runtime and ownership

Each runtime has two fixed mutation workers, two heavy workers and two cleanup workers, each with a 256 KiB reserved stack. Work queues have caller-selected bounded capacity, capped at 4,096. Submission uses `try_send`: busy/disconnected queues return an error and preserve the model. Heavy imports and diagnosis cannot occupy the mutation workers. CPU and allocator contention across lanes remain possible.

A bounded pool pre-admits cleanup ownership before a root, result, capture, task or full snapshot can be published. Its size is `capacity * 16 + 64` (maximum 65,600); each owner also reserves metadata. Shared clones share ownership. The last owner moves its payload into the reserved slot and notifies cleanup, rather than recursively destroying a generation on the calling/UI thread. Ordinary notification is nonblocking. Rejected or canceled queued work also retires its captured payload without executing the task.

A second fixed cleanup receiver handles primary receiver loss. If both receivers fail, a bounded process recovery worker retains and drains parked payloads. Its 256 KiB stack and fixed channel are process overhead outside individual allocation domains. A full emergency recovery channel can backpressure catastrophic failure handling; it never falls back to caller-thread generation destruction. New owners are rejected while cleanup is failed. `try_flush()` reports the failure; `recover_cleanup(&mut runtime)` restarts the fixed receivers. `finish()` refuses outstanding users/owners, drains work and joins workers. Dropping the runtime handle leaves existing owners safe; deterministic shutdown uses explicit finish.

Admission covers shared persistent allocations, copy reservations, worker stacks and bounded ownership metadata, rather than pretending to cap RSS. Portable Image copy-out becomes caller-owned after return. Named invocation preparation scales with interface width and uses a conservative reservation; port editing is measured separately. Cleanup statistics report sums of individual destruction wall intervals and maximum duration, not sampled CPU time.

## Transfer and boundaries

The portable limits remain frozen: 4,096 live nodes and 8,192 allocated node/port slots, plus the accepted field/token limits. Imports use the accepted validator without reparsing historical source, then activate privately on the heavy lane. Failure preserves the caller's model. Larger benchmark fixtures deliberately test transfer rejection rather than bypass limits. Service/receipt revisions, storage-v1 bytes, generated protocols and file behavior are unchanged.

See the [Stage 2 performance report](../performance/controlled-core-integration-stage2.md) and [review handoff](../handoffs/2026-10-19-controlled-core-integration-stage2.md).
