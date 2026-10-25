# Third Ubuntu managed-GUI attempt — overall Blocked/2

**Original UI runner exit0 and runtime identity Verified; qualification Blocked/2.**
The third authorization is consumed. No retry, packet modification or new code
correction was performed during reconciliation. [Authorization](2026-10-25-ubuntu-lifecycle-correction-acceptance.md),
[command](2026-10-25-ubuntu-lifecycle-attended-confirmation.md).

## Actual successes and remaining result

All unchanged functional assertions pass again: main/three peers, managed mode,
independent Avalonia.X11.X11Window reports, four zero charges, WB5 signature/payload,
source integrity and before/after Linux context. Direct PID/start/UID/executable/
canonical DLL/actual cwd/cgroup/ancestry/display and Rust-child correlation verifies.
Final strict scoped snapshot is Verified/empty with no final-snapshot errors.

| Application | PID | Start ticks | Rust child | Final charge |
| --- | ---: | ---: | ---: | ---: |
| Main | 38218 | 8830283 | 38372 | 0 |
| Clipboard/save | 38753 | 8830899 | 38957 | 0 |
| Discard | 39064 | 8831102 | 39271 | 0 |
| Clean | 39368 | 8831300 | 39577 | 0 |

The main's relative argv and peers' absolute argv canonicalize to the frozen native
DLL. All report `/usr/lib/dotnet/dotnet`; source cwd device2050/inode4334799 is
retained. Eighty complete process records and three corroborated exit events exist.
They do not erase unresolved observations. Overall wrapper/capture is correctly
Blocked/2, distinct from runner exit0 and successful runtime identity correlation.

## New diagnostics: 121 unresolved observations

| Failed operation | Count | Binding established? |
| --- | ---: | --- |
| pin.preliminary_stat_uid | 108 | No |
| selection.executable | 9 | Yes; scope not yet determined |
| capture.post_cwd | 1 | Yes |
| capture.post.identity.executable | 1 | Yes |
| capture.initial.identity.executable | 1 | Yes |
| capture.display_environment | 1 | Yes |

There are 120 FileNotFoundError/ENOENT records and one PermissionError/EACCES record.
The 108 preliminary failures have unknown start and no acquired pidfd; their
invocation membership is not established. They must not be described as 108 proven
Pivot child failures or corroborated exits. Source review confirms enumeration
visits numeric /proc candidates before UID/scope can be established. The new
conservative policy retains these unknown candidates as unresolved errors.

Thirteen failures occurred after original-identity binding. Nine occurred during
executable selection before scope was determined. Four were in capture:
PID37305 post-cwd loss, PID37881 post-identity executable loss, PID39064 initial
identity executable loss, and main38218 display-environment denial. Main38218 and
peer39064 also have complete verified records at other samples. The actual error
for main is `/proc/38218/environ`, errno13; it is not the historical compiler denial.
No later successful sample removes this error.

These failure stages are now observed directly rather than reconstructed. Exit
corroboration did not establish a permitted exit event for any of the 121 failures.
The evidence does not prove they were all clean exits, harmless races, PID reuse
or persistent restrictions. Three separate successful corroborated exit events
do not establish the state of these failed samples. Detailed timing/readiness
within unsuccessful corroboration is not fully retained beyond the flags/errors.

The lifecycle change improved diagnostics and preserved direct GUI correlation,
but **did not make observation error-free under this workload**. Non-GUI branch
tests and the small native child fixture did not establish behavior under the real
build/runtime process churn. No claim of corrected overall qualification is made.

## Provenance and evidence

Packet receipt:
`0e79adba5a46c1d2687c9070a7a126adaf7e5a039d3d659b11089abc8fc9b3c6`.
All ten payloads and returned 546-source/273-input manifests verify; source-after
is unchanged. Accepted packet bytes and working-tree disabled gate remain intact.

Capture:
`artifacts/ubuntu-managed-implementation/captures/Pivot-Ubuntu-x64/ubuntu-capture-1bf5bf684f0642a7a82dc9c3c5c8a3f1/`.
Native-report export:
`artifacts/ubuntu-managed-implementation/results/Pivot-Ubuntu-x64/ubuntu-attended-2fe70faddfa64fddb1b15780387a6b24/`.
Originals remain in the corresponding `~/pivot-preflight/ubuntu-capture-.../report`
and `~/pivot-preflight/ubuntu-attended-.../report` directories on native ext4.

Thirty capture hashes and all 28 overlapping nested hashes verify. The separate
native export is byte-identical to the nested report, not independent execution
evidence. Both transfer receipts match native-origin digests; no export errors.

Capture origin SHA-256:
`47b6edbaf283ae288aefb1dd12154649fd772091601d733cb2757ca33c06f64e`.
Native-report origin SHA-256:
`1a7a8ce85ad1dd5745049cd44a61c840218c33db360c2fd8a30d6e71b368509a`.
Independent read-only verifier and summary:
`artifacts/ubuntu-lifecycle-correction/verify_third_result.py` and
`confirmation-result-verification.json`. Original console, reports, timeline,
exception details and exact exits remain unchanged.

## Parent review requested

Retain overall Blocked and all successful evidence within its scope. I recommend
reviewing the observer's contract and acquisition boundary before another GUI
attempt: global enumeration candidates without established invocation membership
are currently treated as unresolved invocation failures, while some bound processes
also lose metadata before exit readiness is positively established.

Any proposed change needs a separate non-GUI investigation with realistic short-
lived process churn fixtures and explicit rules for unavailable pre-selection
candidates versus incomplete required scoped identities. That is a contract review,
not permission to ignore ENOENT or weaken actual main/peer/child evidence. Live
managed environment denial must remain explicitly investigated as well.

No fourth GUI command, automatic retry, broader exception suppression, extra cleanup
wait, actor exclusion or unrelated termination is proposed. No guest action is
requested now. All three original attempts remain independently Blocked/2 and
their receipts consumed. Further work requires parent direction; Ubuntu GUI open,
AOT/unattended/ARM64/Rosetta separate. Windows/Ubuntu headless gates closed; G1/G2
closed; G3 held/unstarted. No product changes.
