# Ubuntu x64 session evidence and disabled managed-GUI proposal

Update: [parent accepts this preparation gate](2026-10-25-ubuntu-disabled-contract-acceptance.md)
and authorizes bounded operational-wrapper implementation/non-GUI checks only.
No positive Ubuntu execution is authorized. Submission below is historical context.

Ready for parent review under the
[bounded Ubuntu preparation direction](2026-10-25-windows-native-aot-acceptance.md).
Actual metadata reconciled; disabled contract prototype/proposal frozen. **No Ubuntu
GUI, AOT, build, clipboard access or acceptance run occurred.** This packet is not
an execution-ready wrapper awaiting only a flag flip.

## Actual session evidence

Returned original:
`artifacts/ubuntu-gui-preparation/results/Pivot-Ubuntu-x64/ubuntu-gui-preparation-22632339a7a14b8f9ba2f22762b66e8c/`.
Captured 2026-10-25 18:29:13 UTC. Native work root is
`/home/robhorvath/pivot-preflight/ubuntu-gui-preparation-22632339a7a14b8f9ba2f22762b66e8c`.
Filesystem is ext4 on `/dev/sda2`; native originals are in its `report` directory.
Both origin entries and transfer digest independently verify. Manifest SHA-256:
`e70e251799cface6ee3919292912645524ac8345e6215ecc176b96852c5ac5ee`.
Returned collector matches frozen SHA-256
`1297f43d656e08b9d3b1220c12915adccd9f8d209c611f3e387c9067248ff600`.

| Observation | Actual result |
| --- | --- |
| Guest/user | Pivot-Ubuntu-x64; robhorvath, UID/GID 1000 |
| OS | Ubuntu 26.04.1 LTS, Linux 7.0.0-38-generic x86_64 |
| Virtualization | microsoft; Microsoft Corporation / Virtual Machine |
| Boot UUID before/after | 616f92de-4c4d-4a06-8937-3f64765a039d; unchanged |
| User display session | Display=2; Sessions=3 2 |
| Graphical session 2 | User 1000, local, active, Class=user, Type=wayland, tty2, seat0 |
| Seat | seat0: ActiveSession=2, CanGraphical=yes |
| Session 3 | systemd-user manager; Type=unspecified, no seat |
| Display endpoints | /run/user/1000/wayland-0 and /tmp/.X11-unix/X0; sockets owned by UID 1000 |
| Desktop processes | gnome-shell PID 3718; Xwayland PID 4862; raw start/stat/cgroup retained |
| .NET | SDK 10.0.113, host 10.0.13, x64 |
| Rust | 1.101.0-nightly, x86_64-unknown-linux-gnu; cargo 1.101.0-nightly |

The earlier session/seat Unknown is now resolved **for the user's graphical login**
by consistent user/session/seat queries. It is not a claim that the terminal belongs
to logind session 2: `show-session self` fails because the terminal is in a
user@1000.service application scope. GNOME/Xwayland also use user-service scopes.
XDG_SESSION_ID/SEAT/VTNR are unset. Preserve the failed query and this distinction.
Wayland environment and existing Xwayland sockets do not prove the eventual
Avalonia backend. No display connection was made.

DMI product_uuid/product_serial reads are permission-denied. No sudo or privilege
change was attempted. Hostname/vendor/model do not uniquely prove Hyper-V VM ID;
the proposal uses existing reviewed host evidence plus explicit owner binding.
Tool/library-cache metadata is discovery, not GUI dependency loading or executable
architecture/runtime proof. Existing native headless evidence remains separate.

## Disabled packet and checks

Packet: `artifacts/ubuntu-gui-preparation/ubuntu-disabled-review-20261025/`.
Receipt SHA-256:
`39c2a5de162a0f632a31f4d531a9f98e1dd9629a81b8f20a3236f51a4aa88c4b`.
540 source entries, 267 inputs, six receipt-covered payloads. Derived from the
accepted disabled Windows AOT source snapshot; all 537 prior source bytes remain
unchanged. Exactly three additions: Linux contract helper, its tests and
[Linux architecture proposal](../architecture/ubuntu-attended-managed-gui.md).
No product/assertion/consent changes. Ancestor Windows files confer no Linux or
renewed Windows execution authority.

The Linux helper has `POSITIVE_RUN_APPROVED=False`; its CLI unconditionally refuses
and fixed original-runner function independently checks that flag. It correlates
own active display session with seat, boot and owned display sockets; it never
treats an environment hint or manager session as graphical identity.

Three pure collector tests and three frozen contract tests pass. Two real frozen
refusal invocations return 2: disabled positive request and piped `yes` into the
unchanged original managed UI runner, with no output directory created. Frozen
checks ran on the Windows host without builds/GUI; they are not Linux runtime
qualification. Logs and reproducible check scripts are under
`artifacts/ubuntu-gui-preparation/`; reconciliation files are
`session-evidence-verification.json`, `disabled-packet-verification.json` and
`non-GUI-checks.json`.

## Review requested and remaining implementation

Review session/seat correlation, owner-confirmed VM binding despite restricted DMI,
and the proposed fixed managed entry/native private environment/evidence contract.
Actual Linux backend and process identity must be observed later, not inferred from
Windows session IDs or Wayland hints. Future implementation must add live packet/
tool/context preflight, original runner supervision, Linux process-start/cgroup
correlation, strict scoped cleanup and verified report export before activation.
Those are disclosed proposal work, not completed runtime safeguards in this packet.

No positive owner command exists. Return for parent direction before implementation
promotion or Ubuntu GUI activation. Any longer Ubuntu commands will be scripts in
shared storage with short launch instructions; results return without transcription.
No repeated clipboard sentinel absent relevant changes. Windows gates and Ubuntu
headless remain closed; Ubuntu AOT, ARM64/Rosetta, unattended activation and
orchestration separate. G3 held; no product changes.
