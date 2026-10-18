# Local IPC integration

Run `python tests/ipc/verify.py` from the repository root. This builds the Rust
daemon, CLI, and raw-frame probe, starts a unique endpoint, and exercises real
process boundaries. Windows and Ubuntu WSL2 Linux are verified; macOS execution remains pending.

Coverage includes discovery, description, required/defaulted inputs, decimal
values/scales, selected outputs, exact revisions, errors, concurrent clients,
successful/failed handshake, repeated requests, invalid/oversized/zero/truncated
frames, abandoned requests, slow partial reads, survival, and graceful shutdown.

Logs go to `artifacts/ipc/` or `--output-dir`. Subprocess timeouts bound failures;
cleanup requests supervised stdin shutdown and force-kills only the test daemon if
needed. Unix cleanup removes only its unique empty test directory, never recursive
paths. Actual other-user access rejection is not exercised here. Windows ACL and
endpoint exclusivity tests run through `cargo test -p pivot-transport`.
