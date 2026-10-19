# Controlled core integration benchmark

This benchmark compares the reference and activated persistent backends through
the **same WorkingModel facade**. It does not call the public research Graph API.
See the [architecture/ownership contract](../architecture/controlled-core-integration.md)
and [results handoff](../handoffs/2026-10-19-controlled-core-integration.md).

## Reproduction

Windows PowerShell:

```powershell
$env:CARGO_TARGET_DIR = 'J:\coding\pivot\target\scaling-windows-checks'
python scripts/integration.py --output-dir artifacts/integration-final-windows
```

Ubuntu WSL2, from the repository:

```bash
export CARGO_TARGET_DIR=target/linux-wsl
export PIVOT_FILE_TEST_ROOT=/var/tmp/pivot-experimental-file-tests
python3 scripts/integration.py --output-dir artifacts/integration-final-linux
```

The standard-library runner builds the locked Release example with
`--features core-integration`, executes separate processes for each backend and
size, samples process RSS every 50 ms, and records environment, build output,
source SHA-256 manifest, CSV stdout and actual status. Default sizes are
1K/10K/100K. Each latency row has three warmups and 31 measured observations,
median/nearest-rank p95/max in nanoseconds. No custom LTO/target CPU, affinity,
frequency control, GUI, document load or network service participates.

## Fixture and semantic boundary

Construct N named decimal inputs through WorkingModel. Replace the final 1,000
with a chain rooted at n0 (999 formulas for the 1K fixture). Construction uses
existing-input replacement rather than quadratic reference add_computed directory
copying. Activation of the persistent backend is timed separately, including
reclaiming imported reference containers. This is an in-memory import bridge,
not a file-load or portable-transfer benchmark.

Every ordinary transaction performs expected-core-revision comparison, private
stage, edit, immutable capture, selected evaluation where specified, assertion of
the independent integer chain result, writer installation and temporary disposal.
The reference path includes actual obsolete model/snapshot destruction. The
persistent path includes handoff/backpressure and concurrent worker disposal.
Returned captures/results are dropped inside each timed pipeline. No hidden
reference model is retained for the activated backend.

| Row | Complete operation |
| --- | --- |
| unchanged_capture | Capture and dispose with no edit |
| input_capture | Stage, input assignment, capture, install and dispose; no calculation |
| input_chain | Stage, input assignment, capture, evaluate chain tail, install and dispose |
| formula_front/middle/tail | Replace a changing chain formula, capture, evaluate tail, install and dispose |
| invalid_draft_repair | Save missing-name draft, request exact full diagnosis, repair explicitly, capture/evaluate, install and dispose |
| named_add_formula | Add input and referring formula, capture/evaluate, install and dispose |

Formula positions are first/middle/final within the chain. Replacement alternates
the increment; expected results track the sum independently of the evaluator.
Named additions use unique names across warmups/samples; the final model therefore
contains 68 extra nodes. This modest growth also applies to retention measurements.
The same request sequence and assertions run against both backends.

The provisional 100K gates apply only to narrow **valid** operations: input/capture
below 5 ms p95; formula pipeline below 10 ms; input plus selected 1,000-formula
chain below 25 ms. Invalid diagnosis is O(N + E) and is intentionally a separate
row; it must not be described as a local formula-edit result.

## Memory and lifetime measurements

An always-on global allocator wrapper delegates to System and counts requested
live/peak Rust heap bytes across both caller and worker threads. It is independent
of the graph's allocation-domain counters. It does not track allocator metadata,
reserved arenas, native library allocations or committed OS pages. Instrumentation
is enabled for both backends in latency runs; this is not an uninstrumented latency
claim or a per-operation allocation profile.

For 16 and 32 retained captures, each iteration stages a changing structural formula,
captures and installs it. After the worker drains, report additional live requested
heap relative to the preceding settled state. Cleanup enqueue and worker drain are
separate observations. The 32-capture run follows disposal of the 16-capture run;
neither is cumulative. Model fixture/NodeId vector and facade metadata remain in
the process. These retention/disposal rows are single observations, not p95 values.

Final writer release measures enqueue and subsequent full runtime finish separately.
Compare their **sum** with synchronous reference release when assessing complete
destruction. The last-domain charge must be zero after finish. The full process
duration/RSS guard includes construction, activation, diagnostic fallbacks,
retention and final reclamation, rather than only a cheap queue operation.

The runtime's 2 GiB known-allocation budget and the runner's 8 GiB RSS stop are
different policies. Neither becomes an advertised application capacity. Several
models can share a runtime/domain; unrelated application buffers still need their
own aggregate ownership policy. No million-cell run is part of this stage.

Native macOS must rerun from a local APFS copy, especially for .NET. The previous
SMB source location remains suitable for shared reports, not macOS build output.
