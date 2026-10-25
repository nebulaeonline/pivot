# Ubuntu observer-corrected confirmation result

Update: [parent accepts direct runtime proof and retains overall Blocked](2026-10-25-ubuntu-confirmation-lifecycle-review.md).
Only bounded non-GUI lifecycle investigation/correction is authorized next. No replay.
Original result and analysis below remain within their stated evidence limits.

**Overall Blocked/2; original application runner exit0; runtime correlation Verified.**
The newly authorized confirmation is consumed. No retry, packet repair, observer
relaxation or extra guest action was performed. Return for parent direction.
[Authorization](2026-10-25-ubuntu-observer-correction-acceptance.md),
[owner command](2026-10-25-ubuntu-observer-attended-confirmation.md).

## What this actual run establishes

The complete unchanged managed suite passed again. All four applications report
managed mode, Avalonia.X11.X11Window and zero final charges. WB5 signature/payload
hash, unchanged source and before/after Linux context verify. Final strict scoped
snapshot is Verified, empty and without snapshot errors.

Unlike the first attempt, **direct main and all peer identities correlate**, with
raw argv, actual source cwd/device/inode, canonical frozen DLL, start ticks,
UID1000, executable, cgroup/ancestry/display and matching actual Rust children.
Main argv is actually observed as the relative DLL invocation; it is no longer
inferred from source or peer ancestry. All four records identify
`/usr/lib/dotnet/dotnet` and source cwd device2050/inode4325516.

| Application | PID | Start ticks | Rust child | Final charge |
| --- | ---: | ---: | ---: | ---: |
| Main | 34118 | 8719225 | 34323 | 0 |
| Clipboard/save | 34883 | 8719823 | 35156 | 0 |
| Discard | 35290 | 8720020 | 35560 | 0 |
| Clean | 35685 | 8720209 | 35960 | 0 |

Observer retains 90 direct process records and three corroborated pinned-original
exit events (32945, 32965, 33037). Exit events do not replace required runtime records.
The earlier compiler environ-read error did not recur; compiler environments are
not read under the reviewed contract. This does not explain the historical denial.

## Why qualification remains Blocked

Four original observation errors remain in `run.json`:

| PID | Original error |
| --- | --- |
| 33199 | Selected identity lost before pinned completion; /proc/33199/exe absent |
| 33285 | Selected identity lost before pinned completion; /proc/33285/cwd absent |
| 33772 | Errno3: No such process |
| 34118 | Selected identity lost before pinned completion; /proc/34118/exe absent |

Some of these PIDs have complete observations at other samples: 33285/33772 as Rust
compilers, and main34118 with verified direct application identity. PID33199 lacks
a retained direct record. Other successful samples do not erase these failures or
prove the state at the failed read. No PID reuse, live permission denial or clean
exit cause is inferred from these error strings alone.

Read-only source review shows uncovered read windows around the accepted pidfd
algorithm: selection/canonical cwd reads occur before pinning; `capture_scoped`
reads initial full identity, opens pidfd and performs an initial identity recheck
outside its inner exit-corroboration handler. A disappearance in those windows can
therefore remain an unresolved observation error even though failures inside the
handler produce corroborated exit events. This explains plausible algorithm gaps;
the retained errors do not record exact call sites, so their individual causes
are not proven. No correction has been made during result reconciliation.

Strict policy correctly retains overall Blocked despite functional/runtime identity
success and empty final survivors. Empty final scope is not complete historical
observation or a reason to discard the four errors.

## Provenance

Exact immutable packet receipt:
`77239bbc51aef1050a61e8184bfc7ea3bffb4d52d341f5a245d744a026089c95`.
All ten payloads verify; returned 545-source/272-input manifests match packet bytes.
Working-tree gate remains false. Original first and corrected packets stay intact.

Capture:
`artifacts/ubuntu-managed-implementation/captures/Pivot-Ubuntu-x64/ubuntu-capture-be3ca1e8a40b4677b5c53f68a021a5c7/`.
Direct native-report export:
`artifacts/ubuntu-managed-implementation/results/Pivot-Ubuntu-x64/ubuntu-attended-88dccc4f768e44908ebd310c68c7bfed/`.
Native originals remain in matching `~/pivot-preflight/ubuntu-capture-.../report`
and `~/pivot-preflight/ubuntu-attended-.../report` directories on ext4.

Thirty capture hash entries and all 28 overlapping nested entries verify. Direct
report export is byte-identical to nested report, not additional execution evidence.
Both transfer receipts match native origin hashes; no export errors. Capture/wrapper
exit2 is distinct from original functional runner exit0. Console/recorder output,
reports and original errors remain untouched.

Capture origin SHA-256:
`c73f9946174670fe9b4dcbfbf9fb2575d937d81ab4b2ecf0becb2747ddd0d189`.
Native-report origin SHA-256:
`bd19638c40a1800f9074c519c20a36927a5183ee914791daeb64b49d8fcb1534`.
Independent verifier and summary:
`artifacts/ubuntu-process-correction/verify_confirmation_result.py` and
`confirmation-result-verification.json`. These read existing evidence only.

## Parent review boundary

Review newly proven direct main/peer/child identities and successful unchanged
functional/backend/resource/context/integrity/final-snapshot results while retaining
overall Blocked. Proposed next work is narrowly bounded non-GUI analysis of process
selection/pinning/read-order race windows and more precise per-read diagnostics.
No blanket vanished-process suppression, inferred main identity, actor exemption,
extra cleanup wait, unrelated kill or repeated GUI attempt is proposed.

No further guest run is requested. Any changed candidate and later positive attempt
require separate review/authorization. Preserve both historical Blocked runs and
consumed receipts. Ubuntu GUI qualification remains open; AOT/unattended/ARM64/
Rosetta/orchestration remain separate. Windows/Ubuntu headless accepted gates closed;
G1/G2 closed; G3 held/unstarted. No product changes.
