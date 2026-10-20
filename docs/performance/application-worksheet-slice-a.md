# Application worksheet Slice A: boundary and UI observations

Final product/test/harness sources and root build files match the isolated Linux copy
(136 SHA-256 hashes). Source audit: [manifest](../../artifacts/application-source-manifest.json).
Final targeted boundary and GUI checks follow full CI; they include the last client
queue-size and displayed-revision/viewport guards. Rust and frozen hosted wire
behavior did not change between those checks. No concurrent build or benchmark
was running during the final standalone request timing loops.

## Boundary measurements

31 samples after three warmups. Edit is one numeric edit; viewport is a bounded
20x10 rectangle over the sparse test workbook; calculation selects the end of a
256-formula chain in retained S. These are product-scale fixtures (about 282 live
cells), not a 100K workbook or a GUI latency measure. Arithmetic, invalid drafts
and retained captures are real Rust operations, not a C# evaluator.

| Platform | Operation | Full C# median ms | p95 ms | Rust execution median ms | Serialization probe median ms | Framing/transport/scheduling residual median ms |
|---|---|---:|---:|---:|---:|---:|
| Windows | edit | 0.082 | 0.130 | 0.031 | 0.004 | 0.041 |
| Windows | viewport | 0.239 | 0.307 | 0.165 | 0.031 | 0.039 |
| Windows | calculate | 0.167 | 0.281 | 0.109 | 0.006 | 0.045 |
| WSL2 Linux | edit | 0.307 | 0.387 | 0.130 | 0.006 | 0.160 |
| WSL2 Linux | viewport | 0.628 | 0.906 | 0.352 | 0.046 | 0.165 |
| WSL2 Linux | calculate | 0.257 | 0.326 | 0.109 | 0.007 | 0.128 |
| Native macOS | edit | 0.054 | 0.117 | 0.022 | 0.002 | 0.023 |
| Native macOS | viewport | 0.193 | 0.220 | 0.149 | 0.022 | 0.022 |
| Native macOS | calculate | 0.108 | 0.119 | 0.079 | 0.002 | 0.022 |

C# request time includes admission, serialization, I/O, response decoding and caller
completion. Rust execution starts after request decoding and ends before encoding.
Serialization combines measured C# encode/decode and a Rust first-pass encode probe;
Rust then performs the final encode. Residual includes framing, Rust decode/final
encode, OS scheduling and pipe overhead, not pure kernel transport. Component
medians are not an additive exact decomposition. These are observations, not SLAs.

| Platform | Session startup ms | Shutdown ms | Final Rust charge |
|---|---:|---:|---:|
| Windows | 28.52 | 4.15 | 0 |
| WSL2 Linux | 81.63 | 17.64 | 0 |
| Native macOS | 138.48 | 11.06 | 0 |

Startup/shutdown are single observations, including child creation/initialization
and explicit close/worker joining respectively; they are not distributions.
[Windows boundary results](../../artifacts/application-boundary-final-windows/results.json),
[Linux boundary results](../../artifacts/application-boundary-final-linux/results.json).

## Actual window execution

Final Windows, WSLg and native macOS window self-checks pass and rendered PNGs were visually
inspected. They cover real entry/formula/repair/move/reuse/clear operations, stale
displayed-revision rejection, superseded viewport loads and scroll bounds. Each
closes with zero charge. The responsive 18x10 viewport in these windows is at most
20x10; resizing never creates a worksheet-sized control collection.

| Platform | UI observations | Edit to rendered frame ms | UI apply ms | Paint delay ms |
|---|---:|---:|---:|---:|
| Windows | 8 | 40.64–55.18 | 0.05–1.46 | 2.28–8.53 |
| WSLg Linux | 8 | 41.08–129.74 | 0.06–1.74 | 2.87–13.25 |
| Native macOS | 8 | 39.45–108.56 | 0.05–1.09 | 1.84–4.06 |

UI refresh includes a deliberate 35 ms coalescing delay plus dispatcher/renderer
scheduling. The first Linux action is a cold 129.74 ms observation; later actions
are 41.08–44.56 ms. Eight actions are not a p95 or a physical display/scanout
measurement. The self-check uses the real window/event workflow, not OS-level
mouse/keyboard automation or an accessibility certification.

[Windows UI report](../../artifacts/application-ui-final-windows/results.json),
[Windows rendered image](../../artifacts/application-ui-final-windows/results.png),
[Linux UI report](../../artifacts/application-ui-final-linux/results.json),
[Linux rendered image](../../artifacts/application-ui-final-linux/results.png).

## Preserved failures and limits

The first WSLg GUI attempt aborted before producing a report with native
`free(): invalid pointer` / SIGABRT:
[original log](../../artifacts/application-execution-linux.log). The diagnostic
[debugger attempt](../../artifacts/application-native-backtrace.log) completed
successfully and therefore captured no failure backtrace. One normal diagnostic
[repeat](../../artifacts/application-ui-linux-repeat/results.json) and the final-source
normal run above pass. No renderer workaround, dependency change or test weakening
was applied. The initial native startup abort is unresolved; later passes do not
establish its cause or unconditional Linux startup reliability. WSLg execution is
not a native Linux desktop deployment claim.

Native macOS application full CI and actual GUI execution passed without reruns.
All 294 copied-file hashes matched the workspace before documentation updates;
source/build/temp outputs were on internal APFS. The first macOS UI action was
108.56 ms; subsequent actions were 39.45–41.42 ms. Both normal close checks reported
zero charge. [Native report](../../artifacts/macos-validation-20261020T101213Z/REPORT.md),
[boundary results](../../artifacts/macos-validation-20261020T101213Z/ci/20261020T101437431933Z/application-session/results.json),
[UI results](../../artifacts/macos-validation-20261020T101213Z/ui/results.json),
[inspected image](../../artifacts/macos-validation-20261020T101213Z/ui/results.png). The original Linux backend viewport
p95 5.343 ms and its 3.381 ms repeat remain historical investigation evidence.
No AOT, installer, durable workbook recovery, file container, new formula features
or product capacity increase is included. Abrupt transport/process failure cannot
provide a graceful zero-charge receipt; the normal CLOSE runs above do.
