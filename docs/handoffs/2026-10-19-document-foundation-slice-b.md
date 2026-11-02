# Document Foundation — Slice B review handoff

Date: October 19, 2026.
Disposition: Experimental headless local model-file workflow implemented;
stop after Slice B for review.

## Result

ProfitModel can be created, saved, closed, reopened in another process, calculated,
invoked through its named interface, edited, saved again and copied with Save As.
Copies retain logical identity but have separate sessions and mutable runtime state.
The workflow uses no daemon, hosted registry, publication, receipts, StoreId,
RecoveryEpochId or database.

This builds on accepted Slice A rather than replacing its graph machinery.
The computation core, formula grammar/binder/evaluator and hosted storage-v1 record
shape/fixtures remain unchanged. The ordinary authoring daemon stays MemoryOnly.

## Implemented scope

- A provisional single-graph envelope with explicit magic/version, bounded length,
  SavedFileVersion and SHA-256 integrity over header and payload.
- Shared deterministic CBOR graph-body primitives, without encoding a hosted Record
  or inventing placeholder store metadata.
- A Rust FileDocument broker for New, Open, Save, Save As and Close. Explicit Revert,
  autosave and automatic crash-recovery promotion are deferred.
- Canonical target association and physical/content fingerprints outside the core.
- Separate capture, disk commit and acknowledgment; graph edits may advance S to T
  before the exact S capture is acknowledged.
- Private same-directory staging, staged flush/verification, observed-baseline checks,
  no-clobber new-file installation and platform replacement of existing targets.
- Verification of installed bytes/version/digest before saved acknowledgment.
- Conservative uncertainty after attempted replacement or post-install failure,
  blocked further saves, and explicit reconciliation against the exact captured file.
- A cross-process executable test harness and pre/post-install fault/termination tests.

No production .pivot format or extension is established. The small binary envelope
is an experimental proof, not a decision against ZIP or SQLite as future document
containers. No further hosted persistence work was started.

## Review files

| Deliverable | File |
| --- | --- |
| Experimental envelope specification | [Format](../architecture/experimental-document-format.md) |
| Broker API, ownership, installation and limitations | [Broker notes](../architecture/experimental-file-broker.md) |
| Envelope and broker implementation | [experimental_file.rs](../../crates/pivot-storage/src/experimental_file.rs) |
| OS-specific creation, identity and sync | [platform.rs](../../crates/pivot-storage/src/experimental_file/platform.rs) |
| Shared graph codec factoring | [codec.rs](../../crates/pivot-storage/src/codec.rs) |
| File, process and failure tests | [files.rs](../../crates/pivot-storage/tests/files.rs) |
| Decision and roadmap | [ADR-0009](../adr/0009-portable-documents-and-local-lifecycle.md), [status](../STATUS.md) |

## Save invariants and limitations

Encoding success is never saved success. Only verified installation acknowledges
the immutable capture. Failed Save As preserves the prior association; uncertain
Save As does too until reconciliation succeeds. S saved while T is current leaves
T dirty. A different valid file at the target cannot satisfy reconciliation merely
because it has the same logical DocumentId.

Cooperative writes reserve a stable per-directory OS lock, retained through the
outcome/acknowledgment. It is explicitly unlocked before close; this avoids a Unix
fork/close-on-exec descriptor briefly extending lock lifetime. No model registry or
persistent mutation receipt is involved. The lock file is not unlinked on release.

Target fingerprints include physical file identity, length, modified time, ordinary
mode and a full-file digest. New installation uses a hard-link create, which cannot
replace an existing target. Unsupported filesystems fail rather than falling back
to truncation. Existing replacement has a final check/rename race against
uncooperative writers; no universal content compare-and-swap or collaboration is
claimed. Parent directories must be current-user-controlled, non-adversarial local
directories. Symlink targets and multiply linked files are rejected.

Windows stages use an owner-rights-only protected DACL. Replacement adopts that
private candidate's ACL; arbitrary prior ACLs, alternate streams and metadata are
not preserved. Unix stages start mode 0600 and preserve ordinary existing mode bits;
general ACL/xattr/UID/GID preservation is not promised. Windows has no claimed
directory-flush guarantee here. macOS's Unix code path remains unverified natively.

The process-exit tests exercise complete old/new file visibility across process
restart at explicit boundaries. They are **not power-loss or OS/storage-crash proof**.
No stronger durability capability is advertised. Uncertain outcomes remain dirty
and unresolved until verified, or the caller explicitly discards/closes the session.

## Validation evidence

Full Windows CI passed on the final source: **185 Rust tests**, formatting, workspace
Clippy with warnings denied, .NET 10.0.402 Release build, 10,099 decimal vectors,
Rust/C# protocol and legacy compatibility, and actual named-pipe daemon/CLI/REPL
integration. Tests used the local fixed NTFS J: volume.

- [Final Windows CI summary](../../artifacts/ci/20261019T081301061382Z/summary.json)
- [Windows Rust test log](../../artifacts/ci/20261019T081301061382Z/rust-tests.log)
- [Windows IPC results](../../artifacts/ci/20261019T081301061382Z/ipc/results.txt)
- [Explicit Windows process-workflow output](../../artifacts/document-files-windows-processes.log)

Ubuntu WSL2 formatting, workspace Clippy, **186 Rust tests** and actual Unix-socket
IPC/REPL checks passed. The extra test is Unix-only permission/alias verification.
File tests used `/var/tmp/pivot-experimental-file-tests` on the native Linux
filesystem reported by `stat` as `ext2/ext3`, not the Windows-mounted `/mnt/j`.
Kernel: `6.18.54.1-microsoft-standard-WSL2`.

- [Final Linux Rust log](../../artifacts/document-files-linux-tests.log)
- [Explicit Linux process-workflow output](../../artifacts/document-files-linux-processes.log)
- [Linux IPC results](../../artifacts/document-files-linux-ipc/results.txt)

The file target contains 18 Windows / 19 Unix cases, including the internal process
worker entry point. New tests cover cross-process create/reopen/edit/invoke/copy,
restart at pre/post-install boundaries, exact decimals, rename/name reuse,
retired identities, incomplete repairable drafts, malformed/version/digest/length
rejection, bounded arbitrary data, encoding/write failures, external changes even
with equal length/time, target-creation races, cooperative locking, S/T saves,
uncertain reconciliation, failed Save As and cancellation/Close guards.

The original 11 frozen hosted storage-v1 fixtures still pass unchanged. No fixture
bytes or manifest were regenerated. Default graph resource budgets and full-copy
staging are unchanged; these are not spreadsheet-scale claims. The file cap is
independent of the unchanged IPC frame cap.

Early runs surfaced sandbox denial of owner-only Windows files and the Unix
inherited-descriptor lock-release issue; permitted Windows execution and explicit
unlock resolved them. WSL `/tmp` was found to be tmpfs, so final Linux file tests
were deliberately moved to the disk-backed path above. Earlier tmpfs checks are
not presented as disk durability evidence.

No Linux .NET rerun was needed for the Rust-only change. Native macOS validation,
power-loss tests, network/cloud filesystems and general metadata preservation
remain outside verified coverage. Artifact reports are local ignored evidence.

## Reproduce the proof

```text
cargo test --locked -p pivot-storage --test files process -- --nocapture
```

On Linux, set `PIVOT_FILE_TEST_ROOT` to a supported native disk directory when the
default temporary directory is memory-backed. The harness spawns independent test
executables and checks persisted state after each process terminates. It requires
no long-running service or external infrastructure.

## Dependencies and exclusions

pivot-storage now directly uses already-locked sha2, libc on Unix and windows-sys
on Windows for integrity/private file creation and identity. No new package versions
were fetched or selected. The small shared codec refactor retains hosted field order
and validation; the new document decoder delegates graph semantics to existing
validation and transfer machinery. The only session change is an internal way for
the broker to mark a failed/uncertain save unsaved.

No database, production container, new protocol operation, C# broker/client feature,
GUI, workbook, cell/range syntax, incremental calculation or large-scale optimization
was added. No file I/O was introduced into pivot-core or graph calculation locks.

After review, prioritize workbook projection and C# application integration; do not
automatically resume hosted persistence or expand this envelope into a storage
platform. Production format selection remains a separate decision. Stop here.
