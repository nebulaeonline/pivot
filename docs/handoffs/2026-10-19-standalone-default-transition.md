# Persistent Core — Standalone Default Transition

Date: 2026-10-19. Base: `532f60d7cd8283c92752ef0d5f082aef48c0cd05`.
Implemented and validated on Windows, Linux WSL2 and native macOS ARM64;
stop for transition review. The new native run validates this transition source.

## Decision and implementation

[ADR-0011](../adr/0011-standalone-persistent-default.md) records the default policy.
Persistent integration is now a default pivot-core feature. WorkingModel::new starts
directly on persistent state and shares one lazily initialized convenience runtime,
not six workers per model. Applications/session hosts should own one runtime using
standalone()/configured() and create models with new_in or runtime.create_model.
All use the accepted canonical paged engine, parser, binder and arithmetic executor.

Defaults: 2 GiB aggregate admission, capacity 64 per work lane, six fixed runtime
workers and 1,088 pre-admitted cleanup slots. Allocation ceiling is not eagerly
allocated and is not an RSS limit. Convenience control supports diagnostics, flush,
receiver recovery and shutdown refusal/restart. Application-owned runtime finish
retains its existing deterministic drain/join and outstanding-owner rules.
No global mutable model registry is introduced.

Explicit new_reference and no-default-features retain controlled fallback and the
compatibility oracle. Different-runtime activation now reports a mismatch instead
of silently accepting a runtime that does not own the model. Direct construction
uses empty persistent roots, without a legacy snapshot/import pass.

Local and complete capture remain distinct accepted contracts. Bounded pinned
inspection remains the interactive path; full snapshot/diagnosis remains explicit,
admitted and broader. Existing Decimal96, source/binding, public ports, no-op,
stable identities and actual public core revisions are preserved.

## Final Windows/Linux validation

Full default CI passed on Windows/Linux, including 241/242 default workspace tests,
warning-free .NET builds, 10,099 decimal vectors, protocol parity and real IPC.
Combined suites passed 264/265 tests; reference-only core passed 21 on both.
Strict normal/combined/reference-only/research-only lint configurations passed.
The sixty verified Release children match final-source manifests and pass all
original default-facing targets. Native transition validation also passed below.
Full logs and summaries are linked from the performance report.

## Native macOS transition validation

The [native report](../../artifacts/macos-validation-20261019T191136Z/REPORT.md)
records a fresh internal APFS copy on Apple M1, 16 GiB RAM, macOS 27.0.1 ARM64.
Source, targets, temporary files and initial reports stayed local; only reports
were returned to SMB. All 274 copied source hashes matched the reviewed tree before
this documentation update. No product changes or validation reruns were needed.

Full default CI passed: 242 tests, warning-free .NET Release build, 10,099 decimal
vectors, protocol/frozen-v1.0 compatibility and actual Unix daemon/CLI/REPL IPC.
Combined tests passed 265, integration-only 242 and fallback 21; all requested lint
configurations passed, including genuinely reference-only and research-only core.
The workspace research-feature run also passed 265; it includes integration through
default features and must not be described as a reference-only test run.

All thirty native Release children passed, bringing this transition to ninety
successful processes across three platforms. At 100K, native persistent p95 was
0.013 ms input/capture, 0.331 ms input/chain, 0.491 ms tail formula edit/calculation,
0.007 ms bounded inspection, 48.748 ms full snapshot and 48.510 ms complete diagnosis.
All original targets passed. Full snapshot remains slower than native reference
(30.313 ms). Every small convenience and shared session final domain charge was zero.

The 100K edit-fixture construction cost was 1.300 seconds persistent versus
0.0467 seconds reference; the bulk-construction tradeoff remains material on Mac.
Small convenience initialization/edit/calculation observations ranged 0.113–0.684 ms
(one per child, not a p95 estimate). Thirty-two retained captures added 133,474
requested bytes. The unchanged runner has no native RSS helper/guard; eight separate
samples observed normal pressure and a 241.39 MiB sampled process maximum. These
are sparse observations, not per-child high-water marks or equivalent RSS-cap proof.

## Performance result and disclosed setup cost

All thirty default-facing Release processes passed on each of Windows and Linux,
with matching final-source hashes. At 100K, persistent p95 input/capture was
0.042/0.134 ms (Windows/Linux), input plus chain 0.584/0.685 ms, and tail formula
edit/calculation 0.974/1.035 ms. All original interactive targets passed. Every
small convenience-runtime and session-runtime final domain charge was zero.

A distinct setup tradeoff is now visible: constructing the 100K edit fixture via
100K individually dispatched input insertions plus formula setup took 3.013 seconds
Windows / 11.580 seconds Linux, versus about 0.082/0.080 seconds reference. These
are repeated public editing operations, not empty-model creation or a legacy
activation. Small default model initialization/edit/calculation was observed at
0.387–1.004 ms Windows / 3.530–4.809 ms Linux (one observation per child). Bulk
construction is not included in the interactive p95 rows or hidden as free setup.
Its queued-edit overhead should be an explicit review tradeoff; no new batching
API or public capacity increase is introduced to conceal it. Full validated import
remains a separate, frozen-limit operation. Cleanup totals include construction.

## Downstream boundary

The [constructor audit](../architecture/standalone-default-transition.md) lists every
production path. Host management, storage EditableGraph, workbook constructors and
research shell, the legacy transfer importer, daemon demo and legacy probes explicitly
select reference. Restore and constructor boundary tests assert that selection even
when features unify. Hosted revisions/receipts, file/session behavior, frozen
storage/protocol fixtures and daemon/REPL contracts remain under full regression.
No workbook, host, document-format or IPC migration is included.

## Validation and decision gate

See the [performance/validation report](../performance/standalone-default-transition.md)
for final test counts, benchmark gates, exact source manifests and resource evidence.
New lifecycle tests cover default construction without activation, shared session
admission, independent identities, private staging, stale revision rejection,
S/T retention, default ports, local versus complete validation, validated transfer,
malformed input, allocation refusal/retry, shutdown refusal, zero-charge release
and convenience restart. The complete ordinary standalone suite runs through new,
while Stage 2 differential/cancellation/queue/recovery tests remain retained.

The controlled standalone selection policy now passes current three-platform
validation, including this transition’s changed defaults and lifecycle entry points.
The remaining gate is parent review of the policy and disclosed costs.

After this transition is accepted, the next separately reviewed milestone is real
workbook transactions on persistent state: demanded anchors, cell moves, retirement
and atomic worksheet/graph edits. Hosted persistence, ranges, new spreadsheet syntax,
C# IPC and Avalonia remain outside this slice. Do not start workbook migration here.


## Recommendation

Accept the implementation as the controlled standalone default policy, subject to
review of the explicit bulk-construction and full-snapshot costs. Compatibility and
default-facing performance gates pass on all three target platforms. The standalone
transition can be considered complete upon acceptance; macOS validation is no longer
an outstanding gate.
[Copy-ready native instructions](2026-10-19-macos-standalone-transition-instructions.md)
cover APFS staging, default/feature/fallback CI, all thirty Release children and
return of results to the SMB artifacts directory. No further application migration
is part of this handoff.
