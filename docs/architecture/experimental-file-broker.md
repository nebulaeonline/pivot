# Experimental local file broker — Slice B

Implemented 2026-10-19 above the computation core in pivot-storage. This is a small
headless Rust broker over the [provisional envelope](experimental-document-format.md).
It does not add a hosted store, permanent edit history or daemon document protocol.

## Lifecycle and ownership

`FileDocument::new` owns one in-memory DocumentSession and no path. `open` performs
a bounded stable read, validates the envelope/digest and graph, restores a fresh
session and records its verified initial checkpoint and target fingerprint.
New documents are dirty; a successful Open is clean. Invalid but well-formed graph
drafts can be opened and saved even though calculation reports structural errors.

Graph edits, validation, selected-node evaluation and named invocation use the
existing DocumentSession API. The session and core remain free of file handles or
paths. File association, pending writes and uncertain outcomes belong to FileDocument.
All newly required SHA-256/libc/Windows API dependencies are in pivot-storage;
pivot-core and the daemon/REPL dependency graph are unchanged.

| API | Behavior |
| --- | --- |
| `save` | Save to the associated target against its observed baseline; no association returns NeedsPath |
| `save_as(path, New)` | Install a new file without clobbering any existing target |
| `save_as(path, ReplaceObserved(fingerprint))` | Explicit consent to replace exactly the observed target, not an unconditional overwrite flag |
| `observe(path, limits)` | Obtain an opaque physical/content fingerprint for explicit replace consent |
| `prepare_save` / `prepare_save_as` | Capture immutable S and return one private SavePlan; allow continued graph editing |
| `SavePlan::commit` | Perform encoding and filesystem work without borrowing the graph/session; return an opaque outcome retaining the writer reservation |
| `finish_save(outcome)` | Verify the pending ticket and acknowledge the exact capture only on verified installed content; then associate the target |
| `reconcile` | Recheck and sync an uncertain installed version/digest before acknowledging it; different content remains a conflict |
| `cancel_prepared(plan)` | Cancel a plan before commit and clear its pending ticket; no file is written |
| `close(discard)` | Reject an active plan; require clean state or explicit discard, then release the session |

Only one plan may be active per document. Dropping an uncommitted plan is not an
implicit cancellation or authorization to begin another write; cancel it explicitly.
Revert, autosave, automatic recovery-file promotion and interactive dialogs are not
implemented. A caller can explicitly discard/close an unresolved session and Open
the file again, but the broker never presents that as proof an earlier save rolled back.

## Installation sequence

1. Encode the immutable capture under separate logical/encoded budgets before
   touching the original. Allocate a fresh saved-file version and integrity digest.
2. Acquire a nonblocking OS reservation on a stable per-directory lock file. Check
   the target fingerprint or nonexistence. Create an exclusive random temporary
   file in that same directory, initially private to the current owner.
3. Write the complete candidate, sync its file handle, apply the supported permission
   policy, sync again, reread and verify exact staged bytes and envelope integrity.
4. Recheck the baseline immediately before installation. Existing-file saves use
   Rust's platform rename/replace operation. New targets use a hard-link installation
   of the completed stage, which fails if any destination exists; remove the temporary
   link before verification. No truncate-and-rewrite fallback exists.
5. Perform the supported parent-directory synchronization, reread the installed
   target, and verify exact bytes, saved version and digest. Return a verified
   fingerprint while retaining the cooperative reservation through acknowledgment.
6. Acknowledge S and associate the verified target. If current state is T, keep T
   dirty. Dropping the outcome explicitly unlocks before closing the lock handle.

The stable lock is `.pivot-experimental-write.lock`, serializing cooperative writes
within one directory. It is deliberately coarse and retained after unlocking to
avoid multiple lock identities. Broker targets cannot use that reserved name or
the `.pivot-exp-` temporary prefix. No global model registry is involved.

Unix unlock is explicit: concurrent child-process creation can briefly inherit the
open-file description even with close-on-exec, so relying only on closing the parent
descriptor is insufficient. Process termination releases the OS reservation; orphaned
staging files are not automatically promoted or deleted on a later Open.

## Conflicts and uncertain outcomes

Fingerprints include physical identity (device/inode or volume/file index), length,
modified time, permission mode and SHA-256 of full content. Metadata alone is never
the comparison token. Changed/replaced/deleted targets reject Save, preserving
unsaved edits. Save As failures retain the old association; a new target that appears
during staging is not overwritten. No silent refresh, merge or force-overwrite retry.

Every attempted replacement failure, or failure after successful installation but
before final verification, is conservatively uncertain. No saved checkpoint or new
association is acknowledged. The session remains dirty and subsequent saves are
blocked until reconciliation or explicit discard. Reconciliation requires the exact
captured saved-file version and file digest, not merely a structurally valid model.
If it succeeds while T is current, only S becomes the saved checkpoint.

Pre-install encoding/write/flush/conflict errors preserve the original and mark the
session unsaved. Direct successful encoding is never a save acknowledgment. The
low-level DocumentSession acknowledgment remains a trusted in-memory primitive;
applications using the broker should let the broker decide when to call it.
Some failure details remain coarse in this experimental Rust API; it is not a
finished user-facing diagnostic or recovery protocol.

## Supported profile and limits

This proof targets ordinary local, current-user-controlled directories and regular
single-link files. Canonical parent paths coordinate directory aliases. Symlink
targets and hard-linked files are rejected. Unix opens use no-follow/nonblocking
flags and require current-user ownership. Windows verifies the handle's regular-file
identity and link count; adversarial reparse-point/parent-directory races are not a
supported security boundary. No filesystem capability is delegated to a daemon.

Cooperative reservations do not exclude arbitrary external applications. An
uncooperative writer can race the final baseline check and rename. This is not a
portable content compare-and-swap, collaborative editing system or protection against
an attacker controlling the parent directory. Nonlocal/network/cloud-synchronized
filesystems, cross-volume installation and filesystems without hard-link support
are outside the supported proof profile; failures are surfaced, not bypassed.

| Platform | Actual strategy and qualification |
| --- | --- |
| Windows | Owner-rights-only protected DACL on temporary/reservation files; sync_all for staged bytes; platform rename for replacement. Replaced files inherit the private candidate's ACL rather than preserving arbitrary original ACLs/alternate streams. No directory-flush or power-loss guarantee is claimed. |
| Linux | Mode 0600 stage; retain ordinary existing Unix mode bits; fsync staged file and destination directory. UID/GID/ACL/xattr preservation beyond this policy is not promised. Final WSL file tests use native disk-backed `/var/tmp` rather than `/mnt/j` or memory-backed `/tmp`. |
| macOS | Same Unix code path, pending native validation. No F_FULLFSYNC/power-loss claim. |

Rust documents [platform rename behavior](https://doc.rust-lang.org/std/fs/fn.rename.html)
and [file locks and synchronization](https://doc.rust-lang.org/std/fs/struct.File.html).
Linux documents why directory entries need separate synchronization in
[fsync](https://www.man7.org/linux/man-pages/man2/fsync.2.html).
Windows creation uses [CreateFileW security attributes](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew)
with a protected owner-rights DACL. These references inform the implementation;
test evidence is narrower than a universal filesystem durability guarantee.

The tests terminate processes at defined pre/post-install boundaries; this proves
restart behavior for completed operations under the tested profile, **not survival
of power loss or an OS/storage crash**. No stronger durability label is advertised.

## Reproducible headless proof

```text
cargo test --locked -p pivot-storage --test files process -- --nocapture
```

The integration harness starts distinct executable processes for create, reopen,
calculate/invoke, edit/save, reopen again, Save As, independent-copy edits and
pre/post-install termination. It requires no background daemon or database. Full
workspace CI also executes this harness. See the
[Slice B handoff](../handoffs/2026-10-19-document-foundation-slice-b.md) for exact logs.

## Review disposition and production follow-ups

Slice B is accepted as an experimental proof. Preserve its implementation and
regression tests; stop expanding the broker while workbook projection is designed.
Windows ACL/alternate-stream preservation, the uncooperative-writer race, native
macOS behavior, power-loss guarantees and current full-copy/small graph ceilings
remain explicit production issues, not Slice B blockers. See
[ADR-0010's review packet](../handoffs/2026-10-19-workbook-projection-design.md).
No production container or hosted-persistence work is authorized by that direction.
