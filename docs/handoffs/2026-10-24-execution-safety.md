# Optional validation — execution-safety milestone

2026-10-24. **First execution-safety milestone accepted and closed.**
[Acceptance and next qualification boundary](2026-10-24-execution-safety-acceptance.md).
VM orchestration and expanded target acceptance remain deferred.
History G2's completed three-platform packet is separate and unchanged.

## Result and boundaries

The common Python execution policy now gates CI desktop mode, direct managed UI
acceptance and Native AOT acceptance before suite work. A TTY warning requires an
affirmative answer. Noninteractive/declined/invalid permits return Blocked/exit 2,
not a failed product assertion. No guest is approved for unattended GUI yet.

A local, ephemeral consent broker lets nested wrappers and existing managed/AOT
clipboard peers validate the same live approval. C# verifies its challenge before
Avalonia initialization for both self-check activation variables. No blanket
environment flag, stored JSON/hostname, new VM CLI or remote service can bypass
consent. Ordinary manual application startup and workbook behavior are unchanged.

Headless CI strips inherited self-check variables and permit authority. The manual
launcher also strips them. Existing graphical assertions, native/managed acceptance,
clipboard lock and fixture-contained Rust fault tests remain intact. No graph,
history, persistence, protocol or Cargo feature change belongs to this milestone.

The [complete transitive audit and owner inventory](../architecture/validation-execution-safety.md)
separates byte-payload session tests from the OS clipboard and enumerates actual
GUI entry/peer paths, Rust test child processes and research fault workers.
[Contributor commands and refusal behavior](../testing.md) require no VM packages.

## Verification scope

Evidence root: `artifacts/validation-safety/`; early progress logs are also preserved
under `artifacts/validation-infrastructure-proposal/`. The console probe links the
actual C# policy source without any Avalonia dependency. Synthetic tests cover
default/no/EOF/nonterminal refusal, bogus isolation flags/names, malformed/wrong/
expired/revoked permits, nested live approval, exception cleanup, headless authority
stripping, wrapper refusal before any spawn, deliberate accepted local CI routing,
and real managed main/peer policy checks.

Windows native AOT publication additionally compiles the actual desktop guard;
direct managed and native executables are exercised only in refusal mode. This is
not positive graphical acceptance or certification of OS dialogs/clipboard behavior.
No GUI windows, host clipboard, sentinel test, personal data or VM operations are used
for this milestone's automated evidence. Synthetic mocked consent is confined to
the non-GUI probe and cannot authorize actual desktop tests.

Retain the initial failures: the first sandboxed Python run could not access its
new private temporary directory; an unrestricted rerun passed. The first Linux
suite exposed a synthetic fixture leak: a fake inherited permit from the routing
test remained when the test next simulated fresh local consent. The guard correctly
refused. The test now removes its deliberately fake token before that independent
scenario; no guard or product assertion was weakened. Initial source/manifests and
failed Linux results remain separate from final-source evidence.

Final Windows and native WSL headless contributor CI both pass all **13 steps**,
including nine policy tests each, seven-project .NET builds, decimal/protocol parity
and actual IPC/application-session execution. Default Rust totals are **347 Windows /
348 Linux**. WSL source/test/report data is on native ext4 under
`/var/tmp/pivot-validation-safety-20261024/`; the Cargo cache is on J:. This is not
Hyper-V validation. All **247 code/build inputs** match the final tested source;
**339 native Linux evidence hashes** were verified after transfer.

Windows publication is warning-free native x64 AOT. Eleven actual refusal checks
cover three Python entry points plus smoke/peer activation with missing/invalid
permits in both the managed and Native AOT desktop images. They all return Blocked/
exit 2 before creating a self-check report. No positive GUI acceptance is claimed.
Console-probe tests separately prove live approved main/peer startup authorization.

Final reports: `artifacts/validation-safety/windows-final/`, `linux-final/`,
`refusals/verification.json` and `linux-copy-verification.json`. Original pre-correction
Windows success and Linux failure are retained. Fresh native Mac headless validation
passes the same 13 steps, nine safety tests and 348 Rust tests.
[Mac reconciliation](2026-10-24-macos-execution-safety-validation.md) verifies 366
returned evidence hashes and 247 inputs; shared documentation drift exactly matches
the recorded G2 acceptance update while the tested local tree remained unchanged.
Earlier G2 results are not reused. No new manual desktop testing is requested.
The parent accepts and closes this safety milestone; no guest isolation is approved.

## Next boundary

This gate is accepted/closed. The next bounded deliverable is primarily read-only
guest qualification; it has not begun. Before any unattended guest activation, obtain
host-side integration/redirection settings, guest/session/runtime verification and
a scheduled harmless clipboard sentinel test. The owner has provisioned the four
named VMs in the audit; no guest is yet declared isolated or platform-validated.
Do not alter the legacy Ubuntu 18.04 build environment, install guest tools, add
remote dispatch, VM lifecycle automation or select VM flags at this gate.
