# Ubuntu x64 disabled managed-GUI implementation

Update: first guest adapter check Blocked; see the
[preserved result and new disabled observer correction](2026-10-25-ubuntu-observer-scope-correction.md).
The short non-GUI launcher now pins that correction packet. Original packet and
evidence below remain historical and unchanged; no GUI activation is authorized.

Implementation and frozen non-GUI checks prepared under the
[accepted direction](2026-10-25-ubuntu-disabled-contract-acceptance.md).
Actual Hyper-V guest adapter checks are pending. **Positive managed GUI and AOT
remain hard-disabled and unauthorized.** No positive owner command exists.

## Authorized guest non-GUI command

In the intended **Pivot-Ubuntu-x64 desktop terminal**, without sudo:

```sh
bash /mnt/j/coding/pivot/scripts/qualification/ubuntu-wrapper-checks.sh
```

All longer commands are in shared scripts. This verifies the pinned packet and
copies it to a fresh ext4 directory under `~/pivot-preflight/`. It runs 22 frozen
non-GUI tests, read-only live session/seat/display/tool/library/process adapters
and recorder-version discovery. It never invokes the GUI entry, opens a display,
builds Pivot, installs tools or accesses the clipboard. It does not request GUI
consent or repeat clipboard isolation checks.

Native originals remain in
`~/pivot-preflight/ubuntu-wrapper-nongui-<id>/report`; copies return automatically to
`artifacts/ubuntu-managed-implementation/non-gui-results/Pivot-Ubuntu-x64/`.
Report includes original checker bytes, context, raw tool queries/errors, test
logs, source-after and origin hashes. Retain any first Blocked/Failed result; no
automatic retry. If transfer fails, preserve the printed local report for copy-only
recovery. No output transcription required.

## Implementation and packet

[Architecture and implementation contract](../architecture/ubuntu-attended-managed-gui.md).
`ubuntu_gui_runtime.py` implements fixed packet/live preflight, native private
environment, exact original managed runner supervision, scoped process observation
and hash-verified export. `launch_ubuntu_gui.py` records the native child console
through installed util-linux `script` while preserving a real interactive PTY.
Original runner/assertions/consent remain unchanged. Owner CONFIRM remains separate.
`POSITIVE_RUN_APPROVED=False` guards CLI, supervisor and fixed invocation.

Packet:
`artifacts/ubuntu-managed-implementation/ubuntu-disabled-implementation-final-20261025/`.
Receipt SHA-256:
`322def344aa89f1032d9662fcb08c59dfd17a5d0d454e899acac0a3a63849900`.
544 source files, 271 inputs and ten receipt-covered payloads. Relative to accepted
prototype receipt `39c2a5de162a0f632a31f4d531a9f98e1dd9629a81b8f20a3236f51a4aa88c4b`,
only the contract helper and architecture document change; four additions are
runtime helper, original collector source, runtime tests and fixed recorder.
All other prior source bytes remain identical. Full source/input manifests, exact
diff and reconciliation are retained in the artifacts directory. Prototype and
intermediate disabled packet are preserved; only the final named packet is proposed.

Twenty-two frozen host-side non-GUI checks pass (three contract, nineteen runtime),
with a real disabled-positive refusal/2. Nineteen working-tree runtime tests also
pass on Linux in WSL. Initial sandbox denial of WSL access was resolved through
normal tool approval, not guest configuration changes. This is Linux test evidence,
not actual Hyper-V GUI or adapter qualification. Frozen verification is reproducible
with `verify_frozen_non_gui.py`; host logs and summaries are retained.

Checks cover context/session/seat/socket change, terminal outside direct logind,
ambiguous graphical sessions, native storage/tools, start identity/PID reuse,
ancestry/ownership/display, actual Rust child matching, backend/charge requirements,
survivor/failure classification, source hashes/unsafe paths, export failure,
noninteractive refusal and piped yes to the unchanged original runner. Recorder
positive refusal/report-path validation and real-stream/state restoration are also
covered. Polling mocks test contracts, not actual application execution.

## Runtime evidence limits and next gate

Linux main/peers must be observed using actual UID/PID/start ticks, executable,
command, ancestry/cgroup and display context. Process-session fields remain null
on Linux and are not logind IDs. A 10ms cooperative observer may miss a process;
missing/reused identity Blocks rather than being inferred. Final scoped snapshot
has no extra cleanup delay. Library discovery does not prove the actual future
Avalonia backend; each process must report its own. DMI UUID/serial denial remains
disclosed; owner-confirmed management binding remains cooperative.

Return actual guest non-GUI results and this immutable implementation for parent
review before enabling or running GUI. No accepted Windows or Ubuntu headless suite
is rerun. Ubuntu AOT/ARM64/Rosetta/unattended/orchestration remain separate.
G1/G2 closed; G3 held; no product changes.
