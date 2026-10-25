"""Windows guest-local preflight. Positive GUI execution remains review-gated.

No VM dispatch or new permit protocol. Never supplies the UI runner's consent.
"""
import argparse
import ctypes
from ctypes import wintypes
import getpass
import json
import os
from pathlib import Path
import platform
import runpy
import stat
import subprocess
import sys
import time
import traceback
import uuid

from guest_preflight import filesystem, windows_architectures
from run_headless_ci import owned_processes, read_json, same_source, sha, unpack_verified, verify_files, write_json

VM_ID = 'f89ff908-16dc-44c4-af54-008967e48443'
ENTRY = 'tests/application-session/ui_smoke.py'
# Changing this requires parent implementation/profile review and positive-run approval.
POSITIVE_RUN_APPROVED = False
MANAGED_REVIEW_BOOT = '2026-10-25T00:37:42.0920420-04:00'
MANAGED_REVIEW_SESSION = 1


def local_tree(path):
    """Reject reparse points before resolving, including existing ancestors."""
    path = Path(os.path.abspath(path))
    for item in [path, *path.parents]:
        if item.exists() or item.is_symlink():
            if item.lstat().st_file_attributes & stat.FILE_ATTRIBUTE_REPARSE_POINT:
                raise ValueError('Reparse point in work path: '+str(item))
    info = filesystem(path)
    if not info.get('native_work_root') or info.get('type', '').upper() != 'NTFS':
        raise ValueError('Fixed guest-local NTFS required: '+str(path))
    return path


def desktop_identity():
    """Read current process/session and input-desktop metadata; no input injection."""
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    user = ctypes.WinDLL('user32', use_last_error=True)
    kernel.ProcessIdToSessionId.argtypes = [wintypes.DWORD, ctypes.POINTER(wintypes.DWORD)]
    session = wintypes.DWORD()
    if not kernel.ProcessIdToSessionId(os.getpid(), ctypes.byref(session)) or session.value == 0:
        raise ValueError('Interactive nonzero process session required')
    user.GetProcessWindowStation.restype = wintypes.HANDLE
    user.GetThreadDesktop.argtypes = [wintypes.DWORD]
    user.GetThreadDesktop.restype = wintypes.HANDLE
    user.OpenInputDesktop.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    user.OpenInputDesktop.restype = wintypes.HANDLE
    user.CloseDesktop.argtypes = [wintypes.HANDLE]
    user.GetUserObjectInformationW.argtypes = [wintypes.HANDLE, ctypes.c_int,
                                              wintypes.LPVOID, wintypes.DWORD,
                                              ctypes.POINTER(wintypes.DWORD)]

    def name(handle):
        buf = ctypes.create_unicode_buffer(256); needed = wintypes.DWORD()
        if not handle or not user.GetUserObjectInformationW(handle, 2, buf, ctypes.sizeof(buf), ctypes.byref(needed)):
            raise ValueError('Desktop/window-station name unavailable')
        return buf.value

    station = name(user.GetProcessWindowStation())
    current = name(user.GetThreadDesktop(kernel.GetCurrentThreadId()))
    input_desktop = user.OpenInputDesktop(0, False, 1)  # DESKTOP_READOBJECTS
    if not input_desktop:
        raise ValueError('Input desktop inaccessible (locked or unavailable)')
    try:
        active = name(input_desktop)
    finally:
        user.CloseDesktop(input_desktop)
    if station.lower() != 'winsta0' or current.lower() != 'default' or active.lower() != current.lower():
        raise ValueError('Current desktop is not the interactive default input desktop')
    return {'pid': os.getpid(), 'session_id': session.value, 'window_station': station,
            'thread_desktop': current, 'input_desktop': active, 'user': getpass.getuser()}


def guest_identity():
    if sys.platform != 'win32' or windows_architectures() != ('x64', 'x64'):
        raise ValueError('Native Windows x64 required')
    if platform.node().lower() != 'windev2407eval':
        raise ValueError('Expected qualified Windows guest hostname')
    query = ("$ErrorActionPreference='Stop'; $c=Get-CimInstance Win32_ComputerSystem; "
             "$p=Get-CimInstance Win32_ComputerSystemProduct; $o=Get-CimInstance Win32_OperatingSystem; "
             "[ordered]@{manufacturer=$c.Manufacturer;model=$c.Model;uuid=$p.UUID;"
             "boot=$o.LastBootUpTime.ToString('o')} | ConvertTo-Json -Compress")
    result = subprocess.run(['powershell.exe', '-NoProfile', '-NonInteractive', '-Command', query],
                            capture_output=True, text=True, timeout=30, check=True)
    info = json.loads(result.stdout)
    if (info.get('manufacturer') != 'Microsoft Corporation' or info.get('model') != 'Virtual Machine'
            or not info.get('boot') or not info.get('uuid')):
        raise ValueError('Hyper-V guest metadata unavailable or unexpected')
    # Firmware UUID is evidence, not assumed equal to Hyper-V management VM ID.
    return dict(info, hostname=platform.node(), desktop=desktop_identity())


def clean_environment(work):
    env = {k: v for k, v in os.environ.items()
           if k.upper() != 'PIVOT_DESKTOP_PERMIT' and not k.upper().startswith('PIVOT_UI_')
           and not k.upper().startswith('PIVOT_')}
    for key, folder in {'CARGO_TARGET_DIR': 'target', 'CARGO_HOME': 'cargo-home',
                        'NUGET_PACKAGES': 'nuget', 'DOTNET_CLI_HOME': 'dotnet-home',
                        'TEMP': 'tmp', 'TMP': 'tmp', 'TMPDIR': 'tmp',
                        'PIVOT_FILE_TEST_ROOT': 'fixtures'}.items():
        target = work/folder; target.mkdir(exist_ok=True); local_tree(target)
        env[key] = str(target)
    # .NET bin/obj stay in the freshly extracted native source tree.
    env.update(DOTNET_GENERATE_ASPNET_CERTIFICATE='false', DOTNET_CLI_TELEMETRY_OPTOUT='1',
               DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE='true', RUSTUP_AUTO_INSTALL='0')
    return env


def invoke_managed(source, report, env):
    if not POSITIVE_RUN_APPROVED:
        raise ValueError('Positive managed GUI execution awaits separate parent approval')
    # Execute only the unchanged fixed runner. Its own desktop_session obtains
    # consent from the real terminal. No answer, pipe or token is supplied.
    # In-process execution also preserves Python assertion tracebacks in the
    # bootstrap's tee log without making the runner's stdout a non-TTY pipe.
    old_cwd, old_env, old_args, old_path = Path.cwd(), dict(os.environ), sys.argv[:], sys.path[:]
    try:
        os.chdir(source); os.environ.clear(); os.environ.update(env)
        sys.argv = [ENTRY, '--output-dir', str(report/'managed')]
        runpy.run_path(str(source/ENTRY), run_name='__main__')
        return 0
    except SystemExit as error:
        if isinstance(error.code, int): return error.code
        if error.code is None: return 0
        print(str(error.code), file=sys.stderr)
        return 1
    except Exception:
        traceback.print_exc()
        return 1
    finally:
        os.chdir(old_cwd); os.environ.clear(); os.environ.update(old_env)
        sys.argv, sys.path[:] = old_args, old_path


def managed_evidence(report):
    """Additional completeness checks; never substitute for the runner's assertions."""
    main = read_json(report/'managed/results.json')
    peers = [read_json(report/f'managed/results.{mode}.json') for mode in ['clipboard-save', 'discard', 'clean']]
    records = [main, *peers]
    if any(r.get('status') != 'passed' or r.get('remaining_charge') != 0 for r in records):
        raise ValueError('Missing passing main/peer zero-charge evidence')
    sessions = {r.get('process_session_id') for r in records}
    if sessions != {desktop_identity()['session_id']}:
        raise ValueError('Actual application/peer session does not match preflight')
    if any(not r.get('graphical_backend') or not r.get('process_id') for r in records):
        raise ValueError('Missing actual graphical backend/process evidence')
    return {'application_reports': len(records), 'zero_charge_fields': len(records),
            'process_ids': [r['process_id'] for r in records],
            'graphical_backends': [r['graphical_backend'] for r in records]}


def execute_managed(source, report, env, rows, identity):
    if not POSITIVE_RUN_APPROVED:
        raise ValueError('Positive managed GUI execution awaits separate parent approval')
    before = guest_identity()
    if before != identity:
        raise ValueError('Identity changed before fixed managed invocation')
    if (before.get('boot') != MANAGED_REVIEW_BOOT
            or before.get('desktop', {}).get('session_id') != MANAGED_REVIEW_SESSION):
        raise ValueError('Managed packet boot/session differs from reviewed current Windows evidence')
    start = time.time()
    code = invoke_managed(source, report, env)
    result = {'status': 'Passed' if code == 0 else 'Blocked' if code == 2 else 'Failed',
              'runner_exit_code': code, 'runner_invoked': True,
              'started_unix': start, 'elapsed_seconds': time.time()-start,
              'entry_point': ENTRY, 'evidence_errors': []}
    # Preserve an executed assertion failure even if later evidence checks fail.
    for name, action in [('source_after', lambda: same_source(source, rows)),
                         ('survivors', lambda: owned_processes(source.parent)),
                         ('identity_after', guest_identity)]:
        try: result[name] = action()
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            result['evidence_errors'].append(name+': '+str(error))
    if code == 0:
        try: result['application_evidence'] = managed_evidence(report)
        except (OSError, ValueError, KeyError) as error:
            result['evidence_errors'].append(str(error))
        if (result.get('source_after') or result.get('identity_after') != identity
                or result.get('survivors', {}).get('status') != 'Verified'
                or result.get('survivors', {}).get('matches') or result['evidence_errors']):
            result['status'] = 'Blocked'
    write_json(report/'managed-run.json', result)
    return 0 if result['status'] == 'Passed' else 1 if result['status'] == 'Failed' else 2


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle-dir', type=Path, required=True)
    parser.add_argument('--receipt-sha256', required=True, help='Reviewed packet digest; integrity only, not GUI authority')
    parser.add_argument('--work-root', type=Path, required=True, help='Existing local NTFS directory')
    parser.add_argument('--run-managed', action='store_true', help='Review-gated; currently always refused')
    args = parser.parse_args()
    report = None
    try:
        if args.run_managed and not POSITIVE_RUN_APPROVED:
            raise ValueError('Positive GUI gate is disabled pending review')
        if not sys.stdin.isatty() or not sys.stdout.isatty():
            raise ValueError('Run from a real terminal in the intended guest desktop')
        identity = guest_identity()
        parent = local_tree(args.work_root)
        if not parent.is_dir():
            raise ValueError('Existing work-root required')
        bundle = args.bundle_dir.resolve(strict=True)
        if sha(bundle/'bundle-receipt.json') != args.receipt_sha256.lower():
            raise ValueError('Packet receipt does not match reviewed digest')
        receipt = read_json(bundle/'bundle-receipt.json')
        verify_files(bundle, receipt['files'])
        # The wrapper and both imported helpers must be covered by the packet.
        for filename in ['windows_attended_gui.py', 'guest_preflight.py', 'run_headless_ci.py']:
            if not any(row['path'] == filename for row in receipt['files']):
                raise ValueError('Helper absent from packet receipt: '+filename)
            if sha(Path(__file__).parent/filename) != sha(bundle/filename):
                raise ValueError('Executed helper differs from packet: '+filename)
        rows = read_json(bundle/'source-manifest.json')
        inputs = read_json(bundle/'build-input-manifest.json')
        if ENTRY not in {row['path'] for row in inputs}:
            raise ValueError('Fixed UI entry point absent from build-input manifest')
        work = parent/('attended-'+uuid.uuid4().hex); work.mkdir()
        report = work/'report'; report.mkdir(); source = work/'source'; source.mkdir()
        unpack_verified(bundle/'source.zip', source, rows)
        verify_files(source, inputs)
        # No archive-generated links; reject any unexpected reparse point now.
        for path in source.rglob('*'):
            if path.lstat().st_file_attributes & stat.FILE_ATTRIBUTE_REPARSE_POINT:
                raise ValueError('Reparse point in extracted source')
        env = clean_environment(work)
        print(f'Owner check: ROBPC / Pivot-Win11-x64 / VM {VM_ID}.')
        print('Confirm current Basic Session, reviewed resource settings and clipboard separation remain unchanged.')
        print('J: is exchange only. This acknowledgement is NOT GUI consent.')
        if input('Type CONFIRM to record these current conditions: ').strip() != 'CONFIRM':
            raise ValueError('Owner conditions not confirmed')
        current = guest_identity()
        if current != identity:
            raise ValueError('Boot/user/session/desktop changed during preflight')
        result = {'status': 'PreflightPassed', 'GUI_executed': False, 'guest': identity,
                  'owner_host_vm_binding': {'host': 'ROBPC', 'vm_id': VM_ID, 'attested': True},
                  'packet_receipt_sha256': args.receipt_sha256.lower(),
                  'source_files': len(rows), 'build_inputs': len(inputs), 'entry_point': ENTRY,
                  'native_work': str(work), 'dotnet_outputs': 'source/**/bin and obj',
                  'environment_roots': {k: v for k, v in env.items() if v.startswith(str(work))},
                  'positive_execution': 'disabled pending separate review', 'G3': 'held'}
        write_json(report/'preflight.json', result)
        write_json(report/'origin-hashes.json', [{'path': 'preflight.json', 'sha256': sha(report/'preflight.json')}])
        print('Native preflight report: '+str(report))
        code = execute_managed(source, report, env, rows, identity) if args.run_managed else 0
        # Hash all original evidence after the fixed run or metadata-only preflight.
        write_json(report/'origin-hashes.json', [
            {'path': p.relative_to(report).as_posix(), 'sha256': sha(p)}
            for p in sorted(report.rglob('*')) if p.is_file() and p.name != 'origin-hashes.json'])
        return code
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError, EOFError) as error:
        result = {'status': 'Blocked', 'reason': str(error),
                  'GUI_executed': None if args.run_managed else False}
        if report is not None:
            write_json(report/'blocked.json', result)
            write_json(report/'origin-hashes.json', [
                {'path': p.relative_to(report).as_posix(), 'sha256': sha(p)}
                for p in sorted(report.rglob('*')) if p.is_file() and p.name != 'origin-hashes.json'])
        print(json.dumps(result), file=sys.stderr)
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
