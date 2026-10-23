# ADR-0023 Stage 4 — workbook/history association feasibility

Date: 2026-10-23. **Windows/Linux/native macOS APFS research validated; complete feasibility review
pending. No production integration, general filesystem guarantee or verified physical
power-loss durability.**
[Stage 3 acceptance and Stage 4 safeguards](2026-10-23-history-durability-stage3-review.md),
[ADR-0023](../adr/0023-document-transaction-history.md),
[experiment plan](../architecture/disk-history-feasibility-plan.md),
[native macOS validation and independent evidence review](2026-10-23-macos-history-association-validation.md).

## Result and stop gate

The tested workbook and history authorities remain separate through saved Open,
explicit recovery, declined recovery, late Save, Clear, copies, replacement,
rename/relocation, close policies and interrupted publication. A saved WB5 remains
independently openable when history is missing, corrupt, incompatible, copied or
locked by another owner. Recovery is explicit and does not replace the saved file.

Each platform passes **69 cases**, **207 total**, including 24 parent-terminated
fault workers and a separate two-process writer-exclusion check per platform.
Windows default/all-feature Rust suites pass **346 / 382**; Linux **347 / 383**.
Mac default/all-feature Rust suites pass **347 / 383**. Formatting and strict
all-target/all-feature Clippy pass on all three. Original native journals, metadata
and oracles produce **884 hashes per platform, 2,652 total**.

Stop for parent review of the complete feasibility findings. This does not choose
a production database, journal/container, sidecar layout or retention implementation.
Production history, session RPC, desktop Undo/Redo and Save integration remain deferred.

**Existing native-export limitation remains:** making a *new* WB5 export from
historical B after later D-only identities have been committed is rejected by the
unchanged portable graph-transfer validator. Stage 4 neither changes that contract
nor substitutes the cold codec for native Save. A negative test confirms rejection
leaves the file, history and pending-save state intact. Production Save after
historical activation needs separately reviewed work. The successful late-save cases
capture B while B is current, retain that actual immutable export through D/Undo/Clear,
and complete it when live content is A. This is the prepared-save contract being proved.

## Research implementation and authorities

`pivot-workbook-document::history_association` is gated by `history-experiment`.
The native file adapter gains only a feature-gated local identity-token observation
using its existing regular-file/ownership/one-link checks. The journal exposes its
existing logical instance UUID. WB5, CB2, private v4, hosted v1, graph NodeId semantics
and workbook/runtime limits are unchanged. No compatibility readers were added.

A research history folder contains the Stage 3 journal epochs plus an append-only
control stream. This layout is an experiment, not a product packaging decision.
The control header binds a logical authority UUID and the control file's physical
identity token. State names the active epoch's relative filename, logical journal
instance UUID and physical identity token. Copying a whole folder or replacing an
epoch with a copied journal does not create attachment authority.

A saved receipt binds the ticket, historical ContentStateId, native WB5 checkpoint,
exact file SHA-256, physical file identity and path hint. Verification reads WB5
through the real native adapter, opens its checkpoint through `Document::open_document`,
and checks the observed fingerprint again. Path and DocumentId alone are insufficient.
A same-filesystem rename can match at a new proposed path because the physical file
identity is unchanged. A byte-identical independent copy/replacement cannot match
merely because its checkpoint/digest/DocumentId match.

These tokens describe a local filesystem observation, not cryptographic origin or
a portable capability. Inode/file-ID reuse and arbitrary malicious metadata rewriting
are not certified against; checksums detect damage, not authentication. No assertion
is made about every possible unobserved external filesystem mutation or network share.

The saved ContentStateId is an external receipt: WB5 itself does not contain the
history ContentStateId. Ordinary WB5 Open receives its normal independent runtime
identity; the experiment does not stamp it with a guessed historical identity.
An explicit matching recovery reconstructs historical state and the latest lifetime
ledger through the accepted cold adapter. It does not modify the saved workbook.

## Save ordering and uncertain installation

Prepare uses an actual `Document::export_document` capture while that content is the
committed cursor. It persists an immutable intent containing the ticket, checkpoint,
content identity, exact WB5 bytes, target and original target observation. A ticket
is scoped to the research authority and remains independent of the live root/epoch.
The native adapter retains its original optimistic replacement checks and uncertainty
handling; the production session code is untouched.

Before installation, the intent is durably marked attempted. `native::install_with`
performs the real temporary-file write/sync/installation path. Only after read-back,
WB5 Open, checkpoint/byte/identity verification and control publication does the
research owner record a saved receipt. Journal head/cursor alone cannot mark content
clean. The clean predicate also compares live ContentStateId to the verified saved
receipt and refuses clean status while an intent remains unresolved.

Failures after attempted installation block associated edits, Clear, policy changes
and close. Reconciliation is an explicit fresh native read/Open of the exact intended
checkpoint, not an automatic success flag or command replay. If the actual file is
still the verified original observation (or an intended new target remains absent),
explicit abort can remove the uninstalled intent. Unexpected file contents/identity
remain unresolved. A completed receipt can be reconciled idempotently by ticket ID
without reinstalling bytes. This is a research reopen/verification seam, not a new
production override of installation-uncertainty safeguards or a power-loss promise.

Prepared intent survives restart. Explicit resume is allowed only before an attempted
installation and after rechecking the original file identity/digest or absent target;
it does not automatically retry an uncertain install. A lost prepare response can
reconcile control first, then discover its committed pending ticket. A resumed ticket
reports no newly measured encoding time, since the WB5 bytes were already captured.

The native export's capture/encode timings remain available on an original ticket;
Stage 3 export/validation/construction/write/sync/reconciliation costs remain separate
through Candidate and journal metrics. No monolithic new interactive-latency claim
or new performance benchmark is presented by this stage.

## Control publication and recovery

Control snapshots are checksummed prepare/payload/decision frames with sequence,
operation UUID and prior digest, using payload sync before decision sync. Partial
uncommitted tails can be trimmed under the exclusive owner lock. Complete corrupted
records reject without silent rollback. The current research header is `PVH4CTL2`;
the earlier local prototype schema is rejected, without a migration reader.

An ambiguous control error blocks authoritative control reads and mutation until
that exact operation UUID is reconciled against the persisted stream. Wrong IDs
cannot unblock it. Reopening also validates the physical control/epoch observations.
File-install uncertainty remains separately represented by an attempted save intent;
reconciling control cannot invent a file-install receipt. Stage 3's journal-pending
barrier remains intact and has its own explicit reconciliation seam.

This is still bounded-resident *linear-scan research*: control startup scans its
stream, and the Stage 3 transaction-ID lookup/startup costs remain. No production
receipt index, incremental encoder, fast startup or constant-history-length commit
latency is established. Metadata snapshots repeat a pending WB5 export while present;
this is intentionally simple research storage, not a space-efficiency endorsement.

## Retention, Clear and relocation

Clear creates and synchronizes a new epoch whose baseline is the current content
with the full latest committed ledger, then atomically publishes the epoch switch
through control. It preserves live content, saved receipt/path authority, lifetime
floors and pending valid save bytes/tickets. The old Redo timeline disappears from
the active epoch only after a committed switch. Old epochs/orphans remain as research
archives; no collection, compaction or secure-erasure claim is made.

The tests finish a prepared B save after Undo to A and after Clear. The installed
file is B while live history is A and remains dirty. The new epoch still reserves
B/D-only identities. Interrupted Clear either preserves the previous Redo or selects
the complete new baseline; the ticket remains usable in a fresh process in both cases.

Retained close preserves unsaved D for explicit reopen/recovery. Session-only close
publishes a retirement tombstone only on successful close; unsaved work additionally
requires explicit discard. Cancelled close and outstanding/uncertain saves do not
retire history. Tombstoned journal bytes are retained for audit by this research
harness; physical reclamation/privacy/secure deletion are not implemented.

An explicit break-retention operation first durably retires the authority, then
returns the live content with its full ledger for an explicitly unretained edit.
It cannot bypass a pending save, uncertain journal/control outcome or unavailable
store. There is no silent fallback to no-history on storage error. No production
unretained-edit UI/protocol or policy is introduced.

Relocation is a persisted intent followed by a real same-filesystem directory rename,
parent synchronization where supported, physical-instance verification and completion.
Before-move interruption can be explicitly cancelled at the source; after-move
interruption can be reconciled at the verified destination. A copied directory fails
the identity checks. Cross-filesystem migration/copy-based relocation is not implemented
or claimed. The workbook is outside the research history folder; this keeps its file
and history locations independent.

Windows prevented renaming an actively locked history directory. The test records
that outcome, releases the owner, then verifies unavailable storage and later restore.
Linux and macOS additionally exercise a live namespace move: further operations fail until the
verified original namespace returns. Neither platform silently changes retention policy.

Save As explicitly rebinds this research authority to the newly installed file after
verification; the old file remains independently openable and is no longer its matching
attachment. A deliberately created independent history for a copied workbook is a new
authority. This is an explicit experimental choice, not a settled product copy/Save As
policy or automatic history-discovery implementation.

## Executed matrix and evidence

Per platform:

| Cases | Count |
| --- | ---: |
| Matching, late-save/Clear, cancellation, retention, copies/rename/replacement/Save As, relocation, unavailable/unretained, export-limit rejection | 14 |
| Save receipt: five control points plus after-install and before-receipt, kill and returned error | 14 |
| Clear: five control points, kill and returned error | 10 |
| Session-only close: five control points, kill and returned error | 10 |
| Actual native adapter: before write, partial write, before install, after install, before verify; kill and returned error | 10 |
| Relocation: before/after move; kill and returned error | 4 |
| Two declines in independent processes, followed by deliberate restart recovery | 1 |
| Missing history, corrupt control version/decision, damaged epoch, copied store | 5 |
| Separate owner process excludes attachment while ordinary WB5 Open succeeds | 1 |
| **Total** | **69** |

Decline checks every history file hash before/after both ordinary Opens and subsequent
recovery. Recovery compares the exact D capsule and independent typed cell/binding,
diagnostic ordering, invalidation and ContentStateId oracle, plus the latest identity
floor. Corrupt/unavailable history tests verify ordinary valid WB5 Open and no mutation
of the tested history bytes. The failed save/Clear cases compare old/new cursor/Redo,
actual installed WB5 and surviving tickets rather than success booleans.

Each matrix terminates 24 fault workers with SIGKILL/TerminateProcess plus one
held-owner process. Each platform has 150 completed worker logs (including seed and
decline workers), all with FINISH 0; the 24 fault logs intentionally have no finish.
Maximum observed whole-run charge is about **27.21 MB** on all three platforms, including native-export staging and simultaneous owners, not RSS or
steady-state cache allocation. Snapshot/ticket buffers are admitted separately;
prepare reserves a conservative two-native-image bound before export, not a production
memory default or exact heap profile. No capacity limit was increased.

Final evidence: `artifacts/history-stage4/{windows,linux}-results-current`, raw logs,
summary operation/return-code receipts, native-original hashes, source/build/native
manifests, toolchains and evidence checksum receipt. The native Mac return is
`artifacts/history-stage4/macos-validation-20261023T222012Z`, with 1,092 APFS-origin
evidence hashes, verified manifest/transfer checksums, 166 matching inputs and 462
matching source hashes (461 frozen files plus the ignored GPT baseline). A separate
`macos-review-20261023/verification.json` records the independent reconciliation.
Each platform contributes 884 native-original hashes; native data is hashed before copying. Earlier failed attempts
and Stage 1–3 evidence/manifests remain separate and intact.

Initial failures are preserved: the historical-export fixture, Windows open-directory
rename refusal, native-install sandbox denial and early compiler/lint corrections.
The final prepared-save fixture captures B before later allocations and includes the
explicit historical-export negative case; assertions and quotas were preserved.

## Scope of the filesystem findings

Windows source/data: local fixed NTFS J:. Linux source/data: native WSL `/var/tmp`,
with build cache/copied receipts on J:. Mac source, target, temporary files and
journals were on internal APFS, using a native ARM64 binary; only frozen results
were copied to SMB. Its first diskutil directory probe failed before testing and
was corrected by verifying the mounted APFS volume/device IDs. No gates were rerun
or assertions weakened. Tests kill processes while the OS remains
alive; injected errors are not device flush failures. Windows control/directory
namespace durability, physical power loss, valid-suffix removal, malicious rewriting,
network shares remain unverified. Native APFS passed the listed research cases;
this does not certify a general macOS durability contract. The research control stream
inherits the trusted-anchor/suffix-loss qualification from Stage 3.

Same-filesystem identity observations and these controlled association cases do not
establish every combination of external rename/replacement during an uncertain install.
Native APFS evidence is now available for complete feasibility review. General
cross-platform association or durability guarantees still require a separately
reviewed filesystem contract. No recurring desktop/manual/.NET run was needed for
these feature-only Rust changes.

No WB5 format change or ordinary portable-transfer relaxation, production coordinator,
compatibility adapter, capacity increase, identity recycling, compaction, Optimize
Worksheet, JIT/SIMD or production packaging choice is part of this work. Stop for
complete feasibility review before proposing production integration.
