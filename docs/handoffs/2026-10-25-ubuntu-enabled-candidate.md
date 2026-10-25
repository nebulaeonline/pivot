# Ubuntu x64 enabled managed-GUI candidate — review submission

Update: [parent accepts the candidate](2026-10-25-ubuntu-enabled-candidate-acceptance.md)
and authorizes one attended attempt after final verification. No attempt has run.
Submission below retains its original pre-approval context.

**Candidate preparation complete; positive execution remains unapproved.**
Derived from the [accepted corrected disabled implementation](2026-10-25-ubuntu-disabled-implementation-acceptance.md).
No application build, GUI launch or fresh guest run occurred. Working-tree gate
remains false. No executable positive owner command/script has been created.

## Immutable packet and minimal delta

Packet: `artifacts/ubuntu-enabled-candidate/ubuntu-enabled-review-20261025/`.
Receipt SHA-256:
`0c2b9817c0ef685038581f0deecc82d7d8c049d9e4357907e4b47cab3c4095aa`.
Accepted parent receipt:
`7a8450f4ccd5d1f1b39b6ecd10fb8e2fa87d319824ae9a439f8fb70a1843a4c3`.

Same 544 archived source paths, 271 input paths and ten receipt-covered payloads.
Exactly one source-line change, in `scripts/qualification/ubuntu_attended_gui.py`:
`POSITIVE_RUN_APPROVED=False` becomes `POSITIVE_RUN_APPROVED=True`.
Flat helper, source archive and affected manifest hashes match that change.
Receipt approval/scope/parent-provenance metadata explicitly describes an enabled
candidate requiring separate execution approval. No launch payload was needed at
this stage; every other source and packet payload stays byte-identical.

Independent verification checks every archived source hash, input hash, path
inventory, flat helper identity, all receipt payloads, AST flag value, one-line
delta and disabled working tree. Original runner/assertions/consent, Linux identity
adapters, observer scope, strict survivors, ext4/private environment, caller state
restoration, recorder and failure/export policies remain unchanged.

Artifacts: `candidate-source.diff`, `candidate-verification.json`,
`independent-verification.json`, `verify_candidate.py` and
`check_candidate_refusals.py` under `artifacts/ubuntu-enabled-candidate/`.
Accepted disabled packets and original Blocked and successful guest reports remain
untouched. Enabled code is not execution authority.

## Frozen non-GUI checks

Twenty-six checks pass on the Windows host and on Linux in WSL: 21 unchanged
applicable frozen tests and five additional enabled-state refusal tests. Three
disabled-state-only tests are explicitly excluded; historical disabled coverage is
preserved. The combined recorder-guard/path test is excluded because its guard
assumes false; strict path checks are separately exercised in the enabled suite.
Frozen test source itself is unchanged.

Checks cover changed/missing boot/user/guest/login/seat/socket, ambiguous graphical
sessions, valid terminal outside direct logind membership, invalid storage/tools,
PID reuse/start/ownership/ancestry/display/child identity, bounded nonactor scope,
backend/charges, survivors and Failed/Blocked/Passed classification. Owner denial
and EOF are driven through enabled preflight with tools/context mocked and positive
supervisor asserted uncalled. Noninteractive positive requests refuse before live
adapters. Changed live context refuses before original entry. The unchanged UI
runner actually rejects piped `yes` before output creation. Path validation,
source tamper/unsafe manifests, export failure and real-stream restoration remain
covered. Positive original entry is never activated; no display is opened.

Latest `candidate-refusals.log` and verification summary reflect the Linux run;
host run also passed with 26 checks. These are refusal/control-flow checks, not
actual GUI/backend/cleanup proof. Accepted actual guest adapter evidence remains
the source-matched disabled implementation's evidence; no repeat was required for
the flag-only candidate. No expanded qualification claim follows from WSL tests.

## Proposed fixed owner command design — not issued

After separate parent authorization, prepare one short shared-storage shell script
for an ordinary terminal inside Pivot-Ubuntu-x64's intended VMConnect Basic Session.
The actual script/launch command is intentionally absent until authorization.
It should initialize no new settings, install nothing and perform these fixed steps:

1. Verify this exact receipt and every payload before importing helpers, copy the
   pinned packet to a fresh native ext4 location and verify native copies.
2. Retain a local exclusive one-attempt receipt before the sole fixed-launcher
   invocation, preventing accidental replay. This is not GUI consent or a new
   authorization broker.
3. Invoke only accepted `launch_ubuntu_gui.py` with the pinned packet/receipt and
   managed gate. Preserve live terminal/display environment. Require current
   owner CONFIRM and the original runner's separate interactive affirmative consent.
4. Let unchanged live preflight, full suite, observer, recorder and exporter retain
   native originals and return verified evidence automatically. Stop on the first
   outcome; no retry or same-approval reuse. Export recovery is copy-only.

The future positive gate must prove real managed main/three peers, actual independent
backend identities and Linux process correlation, four zero charges, unchanged
functional/WB5 assertions, source integrity and an empty strict survivor snapshot.
The earlier checker snapshot is not post-GUI cleanup evidence. No speculative
Avalonia backend selection or invented logind membership is permitted.

## Review boundary

Request separate parent review of exact candidate, refusals and proposed fixed
one-run command design before positive activation. No guest action is requested
now. Long Ubuntu commands stay in scripts with short instructions; reports return
without transcription. No repeat isolation sentinel absent concrete changes.
Ubuntu AOT/ARM64/Rosetta/unattended/orchestration remain separate. Windows and Ubuntu
headless accepted gates closed; G1/G2 closed; G3 held/unstarted. No product changes.
