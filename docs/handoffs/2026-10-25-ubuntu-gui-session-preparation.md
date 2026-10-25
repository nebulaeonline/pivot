# Ubuntu x64 graphical qualification — read-only session preparation

Update: [collection reconciled and disabled proposal prepared](2026-10-25-ubuntu-session-and-disabled-proposal.md).
No repeat collection requested. Instructions below retain their original stage context.

Under the [parent's bounded preparation authorization](2026-10-25-windows-native-aot-acceptance.md),
the first deliverable is a frozen metadata collector. Actual guest results are
pending; disabled GUI-runner design and its immutable review packet follow the
session/seat review. **No Ubuntu GUI or Native AOT execution is authorized.**

## Owner instruction

In **Pivot-Ubuntu-x64**, open a terminal in the intended desktop through VMConnect
Basic Session. Run this short command, without sudo:

```sh
bash /mnt/j/coding/pivot/scripts/qualification/ubuntu-gui-preparation.sh
```

The script contains all longer commands. It verifies the frozen collector hash,
copies the collector to native storage under `~/pivot-preflight/`, then collects
metadata there. Do not run from SSH or an unrelated text login: the intended
desktop terminal is part of the evidence. No clipboard sentinel is requested.

Results return automatically to
`artifacts/ubuntu-gui-preparation/results/Pivot-Ubuntu-x64/ubuntu-gui-preparation-<id>/`.
Native originals remain under
`~/pivot-preflight/ubuntu-gui-preparation-<id>/report`.
If transfer fails, retain the printed local location for copy-only recovery; do not
rerun merely to export. No manual transcription of command output is required.

## Collection scope

Frozen collector:
`artifacts/ubuntu-gui-preparation/session-collector-20261025/ubuntu_gui_preparation.py`.
SHA-256: `1297f43d656e08b9d3b1220c12915adccd9f8d209c611f3e387c9067248ff600`.
Three pure non-GUI tests pass for failed-query rejection, strict own/local/active
graphical-session filtering and incomplete-field rejection. These are collector
logic checks, not guest qualification.

Retain raw logind caller/user/list queries and detailed properties only for the
current UID's sessions; inspect candidate seats separately. An environment hint
alone does not establish identity. One candidate is marked ReviewRequired, not
automatically approved. Missing/failed queries and unresolved session/seat remain
visible. DMI data may be permission-restricted; no privilege change is attempted.

Capture boot UUID before/after, user/hostname/hypervisor/DMI, allowlisted session
environment, display socket inode/owner/mode metadata, current-user process names
and selected desktop process stat/cgroup identity, native block-backed filesystem,
installed .NET/Rust metadata and library-cache listing. No process environments,
clipboard contents, display connection, application launch, compile, restore,
install, VM setting changes or executable acceptance probes. Tool metadata uses
private .NET home with certificate/telemetry/workload updates disabled and Rust
auto-install disabled. Query failures are retained for review.

Native report includes collector bytes/hash and origin hashes; export verifies
every payload and records matching native/returned manifest digests. This is
cooperative evidence, not an authorization mechanism or proof of the actual
Avalonia backend. Owner must correlate console/VM binding; process snapshots do
not establish cleanup qualification.

## Following collection

Review actual Linux session/seat/display relationship, native execution
prerequisites and relevant process identity. Then prepare the fixed managed GUI
entry retaining unchanged UI runner and existing affirmative interactive consent,
applicable refusal tests and immutable disabled packet for parent review. Do not
transplant Windows boot/desktop assumptions. Positive GUI and AOT remain separate
future gates. Preserve accepted Ubuntu headless and Windows qualifications.
No repetitive sentinel, unattended authority, new protocol, dispatcher or lifecycle
automation. G3 held; no product changes.
