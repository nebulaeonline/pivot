# Validation connection setup

Current owner-reported state:

| Machine | SSH status |
| --- | --- |
| Pivot-Ubuntu-x64 | Key-based SSH working at robhorvath@192.168.5.183; full headless CI Passed |
| Pivot-Ubuntu-arm64 | Key-based SSH working at robhorvath@192.168.5.119; hostname Pivot-Linux-arm64; actual Rust test failure retained |
| Pivot-Win11-x64 | No SSH server/remoting configured |
| Pivot-Win11-arm64 | No SSH server/remoting configured |
| Mac | Key SSH works at robhorvath@192.168.5.103; native ARM64 full headless CI Passed; x64 tools need setup |

The orchestrator uses standard SSH, not PowerShell 7 remoting. Windows can use
built-in Windows PowerShell and OpenSSH. No guest service/firewall settings or keys
have been changed by this work. Key installation and service enablement are separate
setup actions; diagnostics below are read-only, not qualification gates.

Ubuntu x64 follow-up: the owner had read `.183` as `.103`. The correct address
accepts SSH connections; no firewall repair is needed for this tested host/port.
ssh.socket is active/listening and activates ssh.service on demand. Unprivileged
`sshd -T` cannot read private host keys; the actual server handshake succeeds and
its Ed25519 fingerprint matches the guest report.

A dedicated validation private key now resides in the host's normal `.ssh`
directory; only the public key is on the share at
`artifacts/validation-ssh/validation.pub`. Existing host key pairs are untouched.
A project-artifact known-hosts file pins the confirmed VM key; global host trust
files are unchanged. Key scan initially failed due to old Windows key-scan KEX
support; normal SSH negotiated successfully and verified the reported fingerprint.

Install the public key once as the normal Ubuntu user, without sudo:

```sh
bash /mnt/j/coding/pivot/scripts/ssh/install-validation-key.sh
```

The same command works on Ubuntu ARM64 if that share path is mounted. It appends
the public key idempotently, preserves existing keys, sets normal SSH-file
permissions and returns endpoint details automatically. It does not change
password login, server or firewall settings. The prepared host profile is
`artifacts/validation-ssh/targets.json`, selecting Hyper-V Ubuntu x64 instead of
WSL. Both public-key installations and actual automated SSH discovery/CI dispatch
have now completed; no password-login/service/firewall changes were needed.

Both Ubuntu profiles are in ROBPC's automatic
`artifacts/validation-config/RobPC.json` profile. Host keys were compared against
the public fingerprints returned from the guests before trusting them. Addresses
were DHCP-assigned; update the profile if the leases change.

## Windows guest setup

Inside each Windows guest, run from an ordinary Command Prompt:

```cmd
J:\coding\pivot\scripts\ssh\setup-windows.cmd
```

It asks before setup, stages locally and requests UAC. This avoids elevated sessions
losing the J: drive mapping. Built-in PowerShell 5.1 is sufficient. It installs
OpenSSH Server if absent, adds the validation public key without removing other
keys, requires key authentication, sets automatic service startup and restricts
the SSH firewall rule to the local subnet. It backs up/validates config, returns
the report through the original unelevated share session and retains native logs.
On failure it attempts to restore the prior SSH config; installed capability/keys
are left in place and reported. No changes occur until the owner runs and approves
the script. Setup has been syntax-checked but not executed in either guest yet.

The administrators key-file location/ACL follow
[Microsoft's Windows OpenSSH key guidance](https://learn.microsoft.com/en-us/windows-server/administration/openssh/openssh_keymanagement).
No PowerShell 7 remoting, VM control or machine execution-policy change is required.

On Ubuntu x64, diagnose the existing connection problem with:

```sh
bash /mnt/j/coding/pivot/scripts/ssh/inspect-linux.sh
```

The same script can collect Ubuntu ARM64's address/user once if those are unknown.
It records IP addresses, port22 listeners, service/socket state, firewall query and
public host fingerprint. Missing permission is retained; no sudo or firewall changes.
Reports return automatically under `artifacts/validation-endpoints/`; originals stay
under `~/PivotValidation/setup-reports/`. No private keys or authorized-key contents
are collected. Non-root ufw/sshd config queries may be unavailable; that is expected
diagnostic evidence, not a failed test.

For either Windows guest, optional one-time endpoint inventory without admin:

```cmd
J:\coding\pivot\scripts\ssh\inspect-windows.cmd
```

It uses built-in PowerShell, does not require Python, and records local addresses,
user/source path and any existing SSH service/listener. Policy bypass applies only
to this read-only script process, not machine policy. Reports return to the same
artifact directory; originals remain under LocalAppData/PivotValidation/setup-reports.

On the Mac, the source path from previous validation is `/Volumes/RobMac/coding/pivot`.
If that mount still exists, optional read-only inventory is:

```sh
python3 /Volumes/RobMac/coding/pivot/scripts/ssh/inspect_endpoint.py
```

Mac endpoint report confirms ARM64, robhorvath, the mounted share and active network
addresses .103 (wired) and .113 (Wi-Fi). Host connection to .103 succeeds. Install
the same validation public key once from the Mac terminal, without sudo:

```sh
bash /Volumes/RobMac/coding/pivot/scripts/ssh/install-mac-key.sh
```

Key installation and automated ARM64 discovery/full CI are now complete. Source is
uploaded over SCP to APFS and reports are retrieved over SSH, avoiding the SMB
permission error with Full Disk Access still off. Both native ARM64 and Rosetta
x64 passed all 13 existing headless CI gates. Owner installed x64 SDK10.0.403;
the authorized x64 Rust toolchain is installed with rustup. Python remains ARM64:
the runner verifies the selected Rust/.NET architectures independently. Direct
installed CLT Clang links x64 output, avoiding the xcrun launcher issue.
See [x64 toolchain and validation evidence](handoffs/2026-10-25-mac-x64-toolchain.md).

The installer preserves existing keys/access and returns updated endpoint details. The first
report lacked a public host-key file before the server's first activation; the real
handshake now presents a host key. Match the refreshed report before recording it
in the host's project known-hosts file. No global trust or Mac security settings change.

Next setup: establish reachable addresses, add the host validation public key to
the Ubuntu servers, enable key-only OpenSSH on the Windows guests with owner admin
approval, and configure the Mac endpoint. Private keys stay in the host's SSH
directory, not the repository/share. Existing password login on ARM64 need not be
disabled before key access is tested. Do not disable a working access path before
testing its replacement. GUI still requires explicit live consent after connection
setup; headless CI can run unattended through the same orchestrator.

### Windows guests: working SSH

Both guests now have key login: Hyper-V x64 at `User@192.168.5.118`, UTM ARM64 at
`robhorvath@192.168.5.125`. Server fingerprints match the setup evidence and are
recorded in project-specific known-hosts files. Windows x64 passed all 13 headless
CI gates in native NTFS storage. ARM64 also passed all 13 gates natively on NTFS.
See [Windows SSH evidence](handoffs/2026-10-25-windows-ssh-automation.md).

The service-start failure was resolved by normalizing private host-key ACLs.
`ssh-keygen` had created them with installer-user ownership/access; SYSTEM emitted
the private-key protection warning. Setup now retains key contents/fingerprints,
backs up ACL metadata, sets SYSTEM ownership and restricts access to SYSTEM and
Administrators. This follows [OpenSSH's host-key rules](https://github.com/PowerShell/Win32-OpenSSH/wiki/Security-protection-of-various-files-in-win32-openssh).
Correcting only the parent directory ACLs had not resolved startup. All original
failures remain preserved.

After ARM64 switched from NAT to bridged networking, its Public network conflicted
with a Private-only SSH firewall rule. Setup now enables the SSH rule on all
profiles while retaining LocalSubnet scope. No global firewall or UAC changes.
The earlier SYSTEM diagnostic ran successfully despite no ARM64 UAC prompt.
