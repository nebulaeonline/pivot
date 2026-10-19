# Controlled Production Core Integration — Stage 2

Date: 2026-10-19. Base: `bafff65443e216d404b4487c2450ce5473988f22`.
Recommendation: advance to a concrete standalone default-backend selection review.
The default remains reference; stop after this slice for review.

## Implemented

Explicitly activated WorkingModel now supports standalone input/formula replacement,
renaming, retirement, name reuse, stable-ID repair, every public port operation,
optional defaults, raw/named invocation, staging and portable transfer. There is
no shadow reference model. Public core revisions retain their accepted meaning;
internal generation tags never replace core or hosted publication revisions.

Cheap `capture_local()` and pinned bounded definition/edge inspection are separate
from exact global validation and complete legacy snapshot construction. Local
calculation accepts unrelated invalid drafts and reports a representative requested
closure error. Existing complete capture/snapshot diagnostics remain exact.
Initially-invalid repair stays local rather than invoking a hidden full import.

Six fixed per-runtime workers separate mutation, heavy and reclamation work.
Pre-admitted cleanup slots preserve last-owner responsibility through queue
rejection, cancellation and receiver failure. Reserved fallback and bounded process
recovery prevent generation destruction on the submitter. Explicit failure reporting,
recovery, draining and shutdown are tested. Emergency backpressure and process-wide
recovery overhead are disclosed in the ownership contract.

The accepted importer/exporter preserves original source, bound identities,
retired identity ledgers and exact decimal representations. Existing portable
capacity and validation limits remain frozen. No storage/protocol bytes or
workbook/host behavior changed.

## Validation outcome

Windows: 259 combined feature tests and 209 default tests passed. Linux WSL2:
260 combined feature tests and 210 default tests passed. All three feature lint
configurations and full default CI passed on both platforms, including .NET,
10,099 decimal vectors, protocol parity and real IPC. All sixty Release benchmark
processes passed with final-source hashes matching their manifests.

At 100K, persistent input edit/capture p95 was 0.026 ms Windows / 0.143 ms Linux;
formula edit/calculation remained below 1 ms on both. Complete snapshot p95 was
153 / 129 ms and slower than reference construction; bounded inspection remained
about 0.011 / 0.009 ms. All original and proposed broader-operation budgets passed.
Thirty-two retained formula captures added about 0.127 MiB requested heap versus
947.54 MiB for staged reference captures. Final domain charge was zero in every
process. Full tables, raw artifacts and reclamation costs are in the report.

Native macOS ARM64 validation also passed on an Apple M1 with 16 GiB RAM,
macOS 27.0.1, Rust nightly 1.101.0 and .NET 10.0.402. The dirty working tree was
copied to local APFS; source, targets, temporary files and initial reports stayed
local. All 265 copied source hashes matched the reviewed tree before this
results-only documentation update. Default CI passed (210 Rust tests), as did
integration-only (237), research-only (234) and combined (260) tests, all three
strict feature lint configurations, .NET/parity and real Unix-socket IPC.

All thirty native Release processes passed, bringing Stage 2 to ninety successful
processes across three platforms. At 100K, native persistent p95 was 0.014 ms for
input edit/capture, 0.318 ms for input/chain, at most 0.485 ms for tested formula
edits/calculation, 0.008 ms for bounded inspection, 50.476 ms for a complete snapshot
and 80.430 ms for complete diagnosis. All review targets passed. Complete snapshot
construction remains slower than native reference (30.402 ms), so the existing
local-versus-complete contract remains important. Final domain charge was zero
in every native process; 32 retained formula captures added 133,474 requested bytes.

The runner has no macOS RSS support or effective native RSS guard. Separate sparse
monitoring observed normal memory pressure; its 323.36 MiB sampled process maximum
is not a per-child high-water mark and must not be compared as one. This is a
measurement limitation, not a remaining correctness-validation gap. No product
changes or validation reruns were required. Evidence:
[macOS report](../../artifacts/macos-validation-20261019T182116Z/REPORT.md).

## Evidence and review boundaries

See the [compatibility and ownership contract](../architecture/controlled-core-integration-stage2.md)
and [performance/validation report](../performance/controlled-core-integration-stage2.md)
for measured results, reproduction commands and complete validation status.

Tests compare accepted reference and activated facade behavior, including complete
diagnostic payloads, actual revisions, selected results and independent arithmetic
expectations. Coverage includes invalid drafts, retirement/repair, defaults,
overrides, staging rollback, old S/new T, bounded paging, cursor isolation,
transfer rejection/round trips, allocation refusal and zero final domain charges.
Runtime fault tests hold heavy and cleanup work while edits proceed, reject full
and disconnected queues without executing tasks, and recover receiver loss.

Remaining decisions before standalone default selection:

- Approve the explicit local-diagnostic versus complete-snapshot API contract.
- Decide runtime ownership/configuration for ordinary standalone construction;
  activation currently requires an explicitly owned runtime and allocation domain.
- Accept conservative copy reservations and the measured full-snapshot cost;
  complete inspection is broader than local editing.

No default switch is included. Workbook projection transactions and hosted service
publication need their own compatibility reviews. C# IPC, Avalonia, ranges,
persistence and capacity expansion remain outside this slice.
