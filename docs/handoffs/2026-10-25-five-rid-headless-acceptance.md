# Five-RID headless acceptance

Date: 2026-10-25. Parent review accepts and closes Windows x64 and Windows ARM64
SSH integration and native headless validation. No new implementation or gate
rerun was performed to record this decision.

| RID | Designated environment | Accepted headless result |
| --- | --- | --- |
| win-x64 | Hyper-V Pivot-Win11-x64 | Passed 13/13 |
| linux-x64 | Hyper-V Pivot-Ubuntu-x64 | Passed 13/13 |
| win-arm64 | UTM Pivot-Win11-arm64 | Passed 13/13 |
| linux-arm64 | UTM Pivot-Ubuntu-arm64 | Failed, Rust exit 101 |
| osx-arm64 | Mac mini, native ARM64 | Passed 13/13 |
| osx-x64 | Mac mini, Rosetta x64 | Passed 13/13 |

ROBPC remains controller-only. Local ROBPC/WSL diagnostics do not satisfy any
designated target. Existing SSH profiles, native-local source/build roots,
architecture verification and automatic report retrieval remain the workflow.
Windows setup failures and corrections remain ordinary diagnostic history, with
original reports preserved. No further Windows connectivity qualification.

The next assignment is focused diagnosis of
`model::structural::selection_transaction_tests::late_selection_walk_cancellation_preserves_all_roots`,
whose `runtime.finish().is_ok()` assertion failed at
`crates/pivot-core/src/model/structural.rs:2113` on Linux ARM64. Its cause remains
unknown. Inspect shutdown outcomes, outstanding ownership, cancellation, worker
completion and resource accounting using targeted native ARM64 diagnostics.
Preserve the original Failed result and assertion; do not retry full CI to green
or weaken the test. Any correction must address an established runtime or test
lifecycle defect and receive targeted verification on relevant architectures.

After targeted verification, run the original full Linux ARM64 headless suite.
Once that passes, execute one unified six-target headless matrix through
`scripts/validate.py`, using only designated environments and collecting actual
results. Failed or incomplete targets must remain reported accordingly. A complete
passing matrix closes the headless automation milestone.

GUI/Native AOT integration follows headless completion. Preserve live GUI consent,
actual graphical backends, existing functional assertions, zero charges and owned
child cleanup. Native AOT requires actual publish and execution; no coverage is
claimed from discovery, cross-compilation or publish alone. No forensic packets,
unrelated-process qualification or per-run parent approvals are reintroduced.

G1/G2 remain accepted and closed. G3 remains held. Workbook/formula semantics,
persistence, protocols and product limits remain outside scope except a separately
identified, demonstrably necessary runtime correction.
