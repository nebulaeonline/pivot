# Ubuntu x64 disabled implementation — accepted and closed

2026-10-25 parent decision, attachment
`4b5aa626-bc8a-46b3-ac9d-ed048a224ba4/Pasted text.txt`.

**Corrected operational implementation, observer scope and actual guest non-GUI
validation accepted. Disabled implementation gate closed.**
[Reviewed submission](2026-10-25-ubuntu-implementation-review-submission.md).
No additional non-GUI guest run is required for this acceptance. No actual Avalonia
application/peer or post-GUI cleanup has yet been demonstrated.

Accept fixed Linux preflight, unchanged UI runner and consent, separate owner
CONFIRM, native private roots, original-stream supervision, util-linux PTY recorder
and verified native export. Preserve Linux boot/session/seat/display/cgroup checks
and terminal outside direct logind membership; no Windows session assumptions.

Accept the precise cooperative observer scope: executable denial is outside
invocation only with valid start ticks preceding wrapper start, nonactor name and
readable command without private-root reference. Retain denials. Relevant actors,
root-referenced/new/unknown-start processes still Block on unreadability. Stop
ancestry at wrapper root. No named-service exceptions, survivor exclusions, extra
cleanup waits or unrelated kills. Preserve the first Blocked report unchanged.

Accept 24 actual guest frozen tests, seven origin entries/matching transfer hashes,
source integrity, checker identity, context/tools/libraries/ext4/recorder evidence.
Snapshot contains the checker, not post-application survivors. Three preexisting
service permission denials remain explicitly outside scope under the accepted rule.

Accepted immutable packet:
`artifacts/ubuntu-managed-implementation/ubuntu-disabled-observer-correction-20261025/`.
Receipt SHA-256:
`7a8450f4ccd5d1f1b39b6ecd10fb8e2fa87d319824ae9a439f8fb70a1843a4c3`.
544 sources, 271 inputs, ten payloads. Preserve every original byte and report.

## Next authorized scope

Prepare one new immutable enabled candidate from this exact snapshot. Only the
reviewed positive flag change and necessary fixed one-run packaging are permitted.
Keep working-tree positive path disabled. Preserve all other source, UI assertions,
consent/CONFIRM, Linux identity, observer scope, survivor policy, native roots,
private opt-out, caller restoration, report/export checks and failure classification.
An enabled flag does not grant execution authority.

Independently reconcile archive/input manifests, packet payloads and minimal diff.
Non-GUI refusals must cover missing/changed boot/user/graphical login/seat/socket/
guest/tools, valid terminal outside logind, denied/absent/noninteractive owner
acknowledgement, unchanged original runner's piped-input refusal, PID reuse/start
identity/scope/survivors, report paths/export failures and outcome classification.
No positive activation, application build or display launch during validation.

Return candidate receipt, diff, verifiers/tests and proposed fixed owner-command
design for separate parent review. **Do not run or provide an immediately executable
positive owner command before authorization.** Later execution must retain fresh
CONFIRM, separate live GUI consent, unchanged full suite, actual main/three peers,
independent backends, four zero charges, source integrity and empty scoped survivors.

Long Ubuntu commands stay in shared scripts with short launch instructions; native
ext4 work/originals, /mnt/j exchange only, no report transcription or routine sentinel.
No Ubuntu AOT, ARM64/Rosetta, unattended activation, new broker/dispatcher, lifecycle
automation or host fallback. Windows/Ubuntu headless accepted gates stay closed.
G1/G2 closed; G3 held/unstarted; G4–G7/production Undo/Redo deferred. No product changes.
