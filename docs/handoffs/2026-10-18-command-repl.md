# Pivot Handoff — Persistent command REPL

Date: 2026-10-18
Branch / commit: uncommitted
Status: Complete

## Objective

Implement the owner-authorized command REPL slice, first separating idle and frame
deadlines, then retaining a negotiated IPC connection with explicit reconnect.
Accept ADR-0005 as requested by the authorized architecture review.

## Implemented

- `pivot-transport::FrameTimeouts`: separate idle, partial-frame read, and write
  policies; first byte starts a fixed remaining-frame deadline.
- Daemon configuration flags for these deadlines, connection cap, and work budget.
- `pivot-cli repl` (also default when no command is supplied): help, models,
  describe, invoke, reconnect, and exit over the existing public protocol.
- Session-local terminal history and line editing through rustyline 19.0.0 with
  default features disabled; no history is written to disk.
- Shared parsing/execution with existing one-shot CLI. No engine path added.
- Broken-connection state and explicit reconnect; ambiguous invocations are never
  automatically replayed. A malformed response also invalidates the connection.
- Three timeout regression tests and extended real IPC/REPL integration coverage.
- ADR-0005 and index marked Accepted; operational values remain policy, not protocol.
- Updated usage/status notes in `docs/architecture/command-repl.md` and local IPC docs.

## Interface / semantics

Defaults: 300-second idle, five-second partial-frame read, five-second write;
CLI response first-byte wait is ten seconds. Partial header and payload share one
deadline, so slow incremental bytes cannot extend it indefinitely. Daemon flags
configure positive seconds (maximum 86400), connection count (1–1024), and positive
work budget. Existing byte framing and endpoint security remain unchanged.

Describe/invoke take explicit per-command model/revision selection; invoke can select
public outputs. Unknown/invalid commands recover without discarding a healthy session.
I/O or response failure drops the connection; offline commands send nothing. Explicit
reconnect creates a fresh handshake, optionally at another endpoint. No automatic
retry, idempotency promise, expression grammar, or model editing is introduced.

TTY mode supports cursor/backspace editing and Up/Down history. Ctrl+C cancels input;
EOF/exit closes the session. Pipe mode shares command behavior without editing.
REPL exit after command errors returns 0; one-shot CLI exit codes remain unchanged.

## Tests actually run

- `python scripts/ci.py` — PASS on Windows: 44 Rust tests, formatting, strict Clippy,
  .NET Release solution build, 10,099 decimal cases, Protobuf fixtures, and extended
  IPC/REPL integration. Report: `artifacts/ci/20261018T174940270917Z/summary.json`.
- WSL2 `cargo clippy --locked --workspace --all-targets -- -D warnings` — PASS.
- WSL2 `cargo test --locked --workspace --quiet` — PASS: 44 Linux tests.
- WSL2 `python3 tests/ipc/verify.py --output-dir artifacts/repl-linux` — PASS.
- WSL2 pseudo-terminal smoke test — PASS: actual terminal Up-arrow history,
  backspace editing, and decimal invocation. Transcript: `artifacts/repl-linux/terminal-repl.log`.
- `cargo fmt --all -- --check` and `git diff --check` — PASS.

Regression tests use virtual time for separate idle, partial-header/payload, and
blocked-write deadlines. Integration waits for initial REPL commands to complete,
idles six seconds, then invokes successfully. A fake peer accepts exactly one
session and disconnects after receiving Invoke; the test confirms one invocation,
no automatic reconnect/replay, an offline command sending nothing, and explicit
reconnect to a healthy daemon. Existing one-shot CLI and malformed traffic tests pass.

## Decisions made or proposed

- **Accepted:** ADR-0005 after owner authorization, preserving cross-user denial,
  Native AOT, fuzzing, fairness, and cancellation as remaining work.
- **Implementation choices:** whitespace-delimited command arguments, per-command
  revision selection, in-memory history, offline REPL recovery, and piped REPL exit
  semantics. No new wire contract or computational language was selected.

## Open issues / risks

The new slice is verified on Windows and Linux WSL2. macOS previously passed the
preceding full CI slice but has not rerun these REPL changes. Windows terminal key
handling was not manually exercised; terminal editing/history smoke was Linux-only.
The protocol/decimal policy did not change. Full Linux .NET CI was not repeated for
this Rust-only slice; Windows full CI includes both cross-language suites.

The REPL notices idle-server disconnect when it next performs I/O, then requires
explicit reconnect. No heartbeat, automatic safe-query retry, completion, persisted
history, command quoting, expression evaluation, transient definitions, model edits,
scripting, C# IPC client, Native AOT, GUI, or publication infrastructure was added.
Calculation cancellation and full resource fairness remain open.

## Questions for ChatGPT / owner

No blocking questions. Rerun macOS from a local APFS checkout before claiming the
new REPL has three-platform verification. Review future editable-model sessions
separately from the command parser and daemon transport.

## Next recommended slice

Review the REPL and design working-model session APIs, or explicitly choose C# IPC /
Native AOT verification. Stop here until the next slice is authorized.
