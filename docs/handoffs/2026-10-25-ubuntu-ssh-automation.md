# Actual Ubuntu VM SSH automation

Both public-key installations completed. Noninteractive authentication succeeds
from ROBPC without passwords. Guest-reported public host fingerprints match actual
handshakes. No SSH service, password-login or firewall settings changed on Ubuntu.

| RID/environment | SSH endpoint | Automated discovery | Existing headless CI |
| --- | --- | --- | --- |
| Hyper-V Pivot-Ubuntu-x64 | robhorvath@192.168.5.183 | Available, native x64 | Passed all 13 steps |
| UTM Pivot-Ubuntu-arm64 (hostname Pivot-Linux-arm64) | robhorvath@192.168.5.119 | Available, native aarch64 | Failed in actual Rust test |

Both have native ARM64/x64 .NET10/Rust toolchains and guest-local ext4 work roots.
Discovery: `artifacts/validation-matrix/20261025T201808Z-2ffca608/`.
Parallel actual guest CI: `artifacts/validation-matrix/20261025T201831Z-64511277/`.
Returned logs: `artifacts/validation-runs/20261025T201831Z-64511277/<rid>/`.
SCP report retrieval and shared report export both worked. No GUI or AOT run.

ARM64 failure is
`model::structural::selection_transaction_tests::late_selection_walk_cancellation_preserves_all_roots`,
at `crates/pivot-core/src/model/structural.rs:2113`, assertion
`runtime.finish().is_ok()`. This stops CI at rust-tests (exit101); formatting and
strict clippy passed. Later .NET/parity/IPC/session gates did not execute on ARM64.
Preserve the first failure; no pass-producing retry or assertion/engine change.
Runtime finish has multiple failure branches; this assertion alone does not identify
which fired. Older Linux evidence records a different clipboard test failing at
runtime.finish, but does not establish a shared cause. Track as a real test/runtime
investigation, not infrastructure-forensics or successful ARM64 qualification.

ROBPC's automatically selected profile is
`artifacts/validation-config/RobPC.json`, with both Ubuntu SSH endpoints and dedicated
key/known-hosts options. No `--config` is required for ordinary host commands. Other
machines fall back to template or their own host profile. Default six-target
discovery reports unavailable environments honestly. Private key stays in ROBPC's
normal .ssh directory; only public key/configuration is on the share.

Windows x64 local headless CI previously passed. Both Windows guest SSH setup scripts
are now prepared with owner consent/UAC, local staging, key-only auth and automatic
report return; syntax-checked, not executed. Mac SSH address/Remote Login state is
still needed; its source path was recovered from existing results. These are setup
tasks, not additional parent qualification gates. No new immutable packet or
one-attempt ritual. G3 remains held; no Pivot product changes.
