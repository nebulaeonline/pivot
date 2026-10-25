# Windows AOT guest prerequisites and enabled candidate

Update: [parent accepts and closes these preparation gates](2026-10-25-windows-aot-candidate-acceptance.md)
and authorizes one attended positive attempt after final verification. No positive
attempt has run. The submission below retains its original review-stage context.

Ready for parent review. Actual guest prerequisites **Verified**; immutable
enabled candidate prepared and checked. **Positive Native AOT publish/execution
remains unapproved.** No positive owner command is provided.

This completes the bounded preparation authorized by the
[parent decision](2026-10-25-windows-aot-preparation-acceptance.md), following the
[read-only collection](2026-10-25-windows-aot-prerequisite-collection.md).

## Actual guest result

Returned evidence:
`artifacts/windows-aot-prerequisite-qualification/results/Pivot-Win11-x64/aot-prerequisites-5172ba97044b4afb9644b41bd1352452/`.
Captured 2026-10-25 18:07:55 UTC in WinDev2407Eval, reviewed firmware UUID,
User/session 1/WinSta0/Default. Before/after context and reviewed boot continuity
verify. Source unchanged; all nine helper hashes match the frozen collector.

Installed prerequisites verified:

- .NET SDK **10.0.402**, host/runtime **10.0.13**, win-x64.
- Rust **1.101.0-nightly (48f4b0553, 2026-10-23)**, native
  `x86_64-pc-windows-msvc` host and installed target standard library.
- VS 2022 Community **17.14.37806.7**, required x86/x64 C++ component present.
- HostX64/x64 compiler **19.44.35229.0**, linker **14.44.35229.0**;
  compiler/linker share the selected MSVC directory.
- Windows SDK **10.0.26100.0**, x64 resource compiler **10.0.26100.7705**;
  kernel32, UCRT and MSVC libcmt library paths/hashes retained.

All six selected tool images are x64. Rust/cargo executable paths identify rustup
shims; their file-version resources are null, not invented versions. The rustc
metadata query and resolved target-library path/hashes establish the selected Rust
toolchain. No claim of a stable Rust release or successful AOT link is made.

Four origin entries verify, including original source/input/packet manifests.
Native and returned origin-manifest digests match:
`6c0109d1d845bb59817683dca39a3c11845a19572031affcfe3cc486a9432565`.
The native report remains under
`C:\PivotWork\aot-prerequisites-5172ba97044b4afb9644b41bd1352452\report`.
Reconciliation: `artifacts/windows-aot-prerequisite-qualification/guest-evidence-verification.json`.
No tools were installed; no build, publish, Avalonia or managed GUI suite ran.

## Candidate and exact delta

Immutable packet:
`artifacts/windows-native-aot-candidate/windows-aot-enabled-review-20261025/`.
Receipt SHA-256:
`6989a33a4438fc049a67c6f9ae09f65f3f67c28d1b5ebb1d7bf515669163b7ec`.

Derived from the unchanged accepted disabled packet receipt
`e74a9fee3c44874cf2823becf181b61b37d163b79ca83371f6b2ae946133ad64`.
Same 537 source paths, 265 build-input paths and fifteen receipt-covered files.
Exactly one source line changes in `scripts/qualification/windows_attended_aot.py`:
`AOT_EXECUTION_APPROVED=False` becomes `AOT_EXECUTION_APPROVED=True`.
The corresponding flat helper, source archive, source/input hashes and receipt are
updated. All other source and packet payloads remain byte-identical. No new launch
packaging was necessary; no execution CMD was created. Working-tree approval
constant stays false. Disabled packet and its evidence are preserved.

Original Native AOT runner/profile, warning checks, functional assertions, WB5
checks, four-process evidence, consent broker, boot checks, private Avalonia opt-out,
strict scoped survivor checks and original timeouts remain unchanged.

Exact-delta verifier and diff:
`artifacts/windows-aot-prerequisite-qualification/reconcile_and_prepare_candidate.py`,
`artifacts/windows-native-aot-candidate/candidate-source.diff`,
`artifacts/windows-native-aot-candidate/candidate-verification.json`.

## Non-GUI validation and review boundary

Eleven frozen-candidate checks pass: seven unchanged applicable preparation tests
plus four enabled-path refusals. These cover invalid context before tool/runner
entry, invalid prerequisites before positive runner entry, changed identity before
original runner entry, and a real piped `yes` refused by original GUI consent before
output creation. The unchanged applicable tests also include real noninteractive
runner refusal, rejected arbitrary command/RID/managed switches, prerequisite
failures, artifact/peer contracts and strict survivor classification. Mocked checks
prove control flow, not native publish success or guest GUI behavior.

Two disabled-state-only tests are explicitly excluded from this enabled packet;
their accepted disabled-packet evidence remains intact. No assertions were changed
in the frozen test source. Log and reproducible checker:
`artifacts/windows-native-aot-candidate/candidate-negative-tests.log` and
`check_candidate_negatives.py`. No positive path is invoked by these checks.

Request parent review of actual prerequisites and enabled candidate before any
owner-attended warning-free win-x64 AOT publish and native-image/peer execution.
Existing CONFIRM and separate live GUI consent must remain in any later run.
Windows managed/headless gates stay closed; no repetitive clipboard sentinel.
Ubuntu GUI/AOT, ARM64/Rosetta and unattended activation remain separate.
G1/G2 closed; **G3 held**. No product changes.
