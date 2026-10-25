# Hyper-V x64 — full headless CI evidence review

Review workstream: 2026-10-24 local project date; execution receipts below use
**2026-10-25 UTC** timestamps. **Both supplied guest runs passed; the parent accepts and closes both full
headless CI gates. GUI isolation is not approved; G3 is held.**
[Acceptance and next boundary](2026-10-24-hyperv-headless-ci-acceptance.md).
[Headless qualification acceptance](2026-10-24-x64-headless-qualification-acceptance.md),
[execution instructions](2026-10-24-hyperv-headless-ci-instructions.md).

## Actual guest execution

| Gate | Pivot-Win11-x64 | Pivot-Ubuntu-x64 |
| --- | --- | --- |
| Native root | C: NTFS | `/dev/sda2` ext4 |
| Frozen source / build inputs | 508 / 253 | 508 / 253 |
| Existing CI steps / exit | 13 passed / 0 | 13 passed / 0 |
| Default Rust passed / failed / ignored | 347 / 0 / 0 | 348 / 0 / 0 |
| Execution-safety tests | 9 passed | 9 passed |
| Seven-project .NET build, parity, IPC/session | Passed | Passed |
| Decimal vectors / value or scale failures | 10,099 / 0 | 10,099 / 0 |
| Reported session charge fields | 10 zero | 10 zero |
| Scoped owned-process survivors | None reported | None reported |
| Native-origin report hashes verified | 346 | 346 |

Both guests executed the frozen `scripts/ci.py` headless pipeline, not host/WSL
substitutes. Formatting, strict default lint, actual Rust tests, .NET build,
aggregate reference, Rust/C# decimal and protocol parity, real local IPC,
application-session and execution-safety gates all exited zero. Protocol reports
include 58 decimal layouts, 118 authoring fixtures and frozen hosted-v1 codec/
current-host descriptor coverage. No test or source fix/rerun was performed during
this review; the shared return tree contains one supplied full CI run per guest.

Windows reports native x64 OS/Python on `WinDev2407Eval`, the owner-associated guest
name for `Pivot-Win11-x64`. Source, target, temp and original report are under
`C:\PivotWork\ci-20261025T011804Z-c36a79c1\`. .NET SDK is 10.0.402.
Linux reports `Pivot-Ubuntu-x64`, x86_64 generic Ubuntu kernel 7.0.0-38 and glibc
2.43, not a WSL kernel. Native work is under
`/home/robhorvath/pivot-work/ci-20261025T012417Z-1883dd92/`; .NET SDK is 10.0.113.
Both report Rust 1.101.0-nightly (`48f4b0553`, 2026-10-23).

Cargo targets/caches, NuGet caches, .NET obj/bin and temporary persistence fixtures
are private guest-local paths. The share was only the frozen input/evidence exchange
location. Existing Linux transient IPC sockets use guest `/tmp`; this is not an
SMB/network-storage persistence validation claim. Source, targets, caches and native
fixtures remain in the guests; the transferred report subtree is the review input.

## Independent reconciliation

Windows-side review independently checks **692 native-origin file hashes** against
the two returned manifests and verifies each transfer receipt's before/after manifest
hash. Source/build manifests and bundle receipts exactly match the frozen packet
`artifacts/guest-headless-ci/x64-20261024/`; both helpers match its frozen fingerprint.
The 508-file source trees were verified before/after each guest run with no changed,
missing or added source files. All 253 current code/build inputs still matched at
review time; subsequent documentation reconciliation is outside the tested snapshot.

Raw Rust totals, nine-test policy logs, decimal/protocol results and all twenty
remaining-charge values were independently checked. Process snapshots report no
scoped survivors and Linux reports no unreadable current-user processes. This is
a scoped known-tool/work-root check, not a whole-machine security-sandbox claim.
The existing tests retain their own explicit runtime finish and cleanup assertions.

No log reports generating an ASP.NET development certificate in these runs; the
frozen helper disables generation before invoking .NET. Earlier probe side effects
remain recorded; no certificate cleanup or trust change was attempted.

Original results:

- [Windows wrapper result](../../artifacts/guest-headless-ci/results/Pivot-Win11-x64/ci-20261025T011804Z-c36a79c1/result.json)
- [Windows CI summary](../../artifacts/guest-headless-ci/results/Pivot-Win11-x64/ci-20261025T011804Z-c36a79c1/ci/20261025T011810316437Z/summary.json)
- [Ubuntu wrapper result](../../artifacts/guest-headless-ci/results/Pivot-Ubuntu-x64/ci-20261025T012417Z-1883dd92/result.json)
- [Ubuntu CI summary](../../artifacts/guest-headless-ci/results/Pivot-Ubuntu-x64/ci-20261025T012417Z-1883dd92/ci/20261025T012417217097Z/summary.json)
- [Independent verification receipt](../../artifacts/guest-headless-ci/guest-ci-review-verification.json)
- [Review script](../../artifacts/guest-headless-ci/review_results.py)

Step durations total about 361 seconds on Windows and 98 seconds on Ubuntu. These
are execution diagnostics including restore/build work, not comparable performance
benchmarks, guest capacity policy or a production speedup claim.

## Accepted headless scope and open GUI boundaries

The guests remain HeadlessQualified by the preceding parent decision. The parent has now accepted and closed the full
headless CI execution gates; collector success alone did not grant this acceptance. No all-feature research suite, managed desktop or Native AOT
publish-and-execute acceptance was run in this slice. Console policy tests simulate
consent only for the non-GUI probe and stubbed routing; they do not start Avalonia.

GUI isolation evidence is unchanged and incomplete: host Enhanced Session capability
and integration services are enabled, but actual connection/resource-redirection
settings have not been established. Windows Console/session 3 and Linux Wayland
environment hints are insufficient session/backend isolation proof. Bidirectional
clipboard sentinels were not executed. No guest GUI profile or permit is approved.

UTM ARM64/Rosetta acceptance remains separate. No remote dispatch, lifecycle automation,
provisioning, package/SDK installation or final VM flags were added. Preserve earlier
collector permission/name-shadowing errors, incomplete metadata captures, linker-help
exit 1100 and original probe certificate side effects; clean headless passes do not
diagnose prior intermittent GUI/clipboard failures.

The parent accepts this handoff with the existing qualification and safety records.
No repeat of these passing suites is required. Stop before
GUI activation or remote orchestration. G1/G2 and execution safety remain closed;
G3 remains held and has not started; G4–G7 remain deferred.
