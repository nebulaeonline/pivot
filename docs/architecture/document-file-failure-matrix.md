# Local document file safety and failure matrix

Status: Proposed ADR-0009 contract; no file adapter or fault tests implemented.
Applies to explicit document saves, not ADR-0008 hosted database commits.

## Proposed safe-save sequence

1. Reserve the session/target for one save. Capture immutable document state S,
   fresh SavedFileVersion V, intended target and expected fingerprint F. Keep edits
   available; do not hold a graph edit lock during disk I/O.
2. Serialize and validate the complete candidate under bounded budgets. Create a
   private exclusive temporary file in the destination directory/on the same
   filesystem. Never truncate the original as a staging step.
3. Finish writing, flush through the platform adapter, and verify the candidate's
   version/digest. Preserve required permissions and metadata. Check the target
   still matches F while holding the cooperative writer reservation. For a new
   target, require a no-clobber create/install rather than an implicit overwrite.
4. Replace the destination using the supported platform primitive. A retained prior
   copy can assist recovery, but a backup is not permission to ignore errors. Finish
   the platform-specific directory/metadata durability steps where supported.
5. Verify the installed file identifies V and the captured digest, obtain its new
   fingerprint, then acknowledge S as saved. Release the reservation. If later edits
   exist, leave dirty. Associate a Save As target only after this completion.

These are logical requirements, not a claim that one portable syscall sequence
works everywhere. A chosen SQLite document container may use a private staging
database or another reviewed snapshot/transaction strategy. It must preserve
explicit Save boundaries and produce a self-contained saved artifact; no raw copy
of an active database/WAL pair. Incremental saving needs a separate measured design.

## Outcomes

| Event | Required outcome and recovery |
| --- | --- |
| Open fails validation, compatibility or quota | No new usable session; existing sessions/files untouched; distinguish corrupt data from unsupported features and editable invalid drafts |
| Original changes during Open | Stable-read checks fail or retry a bounded number of times; never combine sections from different file versions |
| Encode fails or cancellation precedes staging | Original unchanged; current session dirty; release candidate resources |
| Disk full, permission failure or flush error before replace | Original unchanged; retain unsaved state; clean temporary data only when safe |
| Baseline mismatch, deletion or competing writer | Do not overwrite; report conflict; offer Save As or explicit reload/overwrite of newly observed baseline |
| New Save As target appears concurrently | No-clobber installation fails; no automatic replace consent |
| Replace reports failure | Inspect documented platform outcomes; do not assume the original path is unchanged; locate original/candidate/backup and report actionable state |
| Replace succeeds but metadata sync/verification fails | Save outcome uncertain; keep session and captured export identity; inspect file before deciding saved status; no blind replay |
| Broker/IPC response lost | Reconcile V/digest against the target and live export ticket; matching content permits acknowledgment; otherwise retain unknown/dirty status |
| App crashes before replace | Previous complete saved version remains under supported profile; candidate may be orphaned; no automatic promotion by filename |
| App crashes after replace | Reopen and validate whichever complete version is present; recovery checkpoint can be offered separately; no promise beyond tested filesystem profile |
| Edits continue while S saves | File contains S; saved checkpoint becomes S; newer live state remains dirty |
| Save As fails/uncertain | Preserve existing association and original file; reconcile target before changing association or deleting anything |
| Revert load fails | Keep prior session, unsaved edits and results; never clear first and then attempt load |
| Close requests Save and save fails | Keep document open; require an explicit later Discard to lose edits |
| Optional recovery checkpoint fails | Warn under application policy; main file/checkpoint untouched; explicit Save remains available |
| Recovery checkpoint base differs from current file | Offer recovered content as an unsaved copy; do not overwrite newer external content |
| Network/cloud/removable storage loses guarantees | Report unsupported or weaker save profile; retain state; do not promise local-filesystem atomicity or power-loss durability |

## Platform proof obligations

Atomic visibility, successful API return and survival of power loss are different
guarantees. Implementation must publish its tested OS/filesystem support profile.
Windows replacement has documented error-specific outcomes and metadata behavior;
the write-through replacement flag is not supported. Validate actual flush and
recovery behavior rather than assuming that flag provides durability. See
[Microsoft ReplaceFileW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew).

On Linux, syncing file data does not itself persist the containing directory entry;
review directory synchronization with the selected replacement strategy. See
[Linux fsync](https://www.man7.org/linux/man-pages/man2/fsync.2.html).
Apple documents a distinction between fsync and F_FULLFSYNC; native macOS validation
must establish the promised profile. See
[Apple fsync manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fsync.2.html).
These references inform requirements; no platform durability has been proved here.

The cooperative-lock plus fingerprint model cannot prevent an uncooperative writer
in the final check/replace window. State that limit in supported behavior. Test
symlink/hard-link aliases, sharing violations, ACLs, locked files and replacement
failures; never silently downgrade to truncate-and-rewrite. Recovery cleanup must
verify ownership and paths, and must not delete files based on a temporary-looking
name alone. Autosave retention and backup policy require separate user-facing choices.
