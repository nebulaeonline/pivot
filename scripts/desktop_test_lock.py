"""Serialize suites sharing one system clipboard, including Windows and WSLg.

Atomic exclusive file creation works across Windows/DrvFS; flock and msvcrt locks
do not provide a common cross-OS contract. Never guess that an existing owner is
dead: timeout with its metadata instead of stealing a potentially live lock.
"""
import argparse
from contextlib import contextmanager, nullcontext
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import uuid


def default_lock_path():
    override = os.environ.get('PIVOT_CLIPBOARD_LOCK')
    if override:
        return Path(override).resolve()
    if os.name == 'nt':
        return Path(os.environ['LOCALAPPDATA']) / 'Pivot' / 'desktop-clipboard.lock'
    if sys.platform == 'darwin':
        return Path.home() / 'Library' / 'Caches' / 'Pivot' / 'desktop-clipboard.lock'
    release = Path('/proc/sys/kernel/osrelease')
    wsl = 'WSL_INTEROP' in os.environ or (release.exists() and 'microsoft' in release.read_text().lower())
    if wsl:
        # Resolve the Windows user's actual local-data directory. Fail closed when
        # interop is unavailable; a Linux-only fallback could race Windows tests.
        command = '[Console]::OutputEncoding=[Text.UTF8Encoding]::new();[Environment]::GetFolderPath("LocalApplicationData")'
        try:
            windows = subprocess.check_output(['powershell.exe', '-NoLogo', '-NoProfile', '-NonInteractive', '-Command', command], encoding='utf-8', timeout=15).strip()
            if not windows:
                raise ValueError('Empty Windows local-data directory')
            local = subprocess.check_output(['wslpath', '-u', windows], encoding='utf-8', timeout=5).strip()
            return Path(local) / 'Pivot' / 'desktop-clipboard.lock'
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            raise RuntimeError('Cannot resolve the shared Windows/WSLg clipboard lock. Enable Windows interop or set PIVOT_CLIPBOARD_LOCK to the same Windows file on both sides.') from error
    return Path(os.environ.get('XDG_RUNTIME_DIR', f'/tmp/pivot-{os.getuid()}')) / 'desktop-clipboard.lock'


@contextmanager
def machine_desktop_lock(path=None, timeout=300):
    """One fixed Mac host lock, independent of RID, checkout and user override."""
    import fcntl
    from execution_policy import Blocked
    path = Path(path) if path is not None else Path('/private/var/tmp/pivot-native-desktop.lock')
    try:
        descriptor = os.open(path, os.O_RDWR | os.O_CREAT | getattr(os, 'O_NOFOLLOW', 0), 0o666)
    except OSError as error:
        raise Blocked('Cannot acquire the machine-wide Mac desktop lock: '+str(error)) from error
    started = time.monotonic()
    try:
        while True:
            try:
                fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except BlockingIOError:
                if time.monotonic() - started >= timeout:
                    raise Blocked('The Mac desktop is owned by another GUI suite; ARM64/Rosetta GUI cannot overlap')
                time.sleep(.05)
        yield {'path': str(path), 'scope': 'machine-wide native Mac desktop', 'pid': os.getpid()}
    finally:
        # Keep the inode: unlinking a flock file can create two independent locks.
        os.close(descriptor)


@contextmanager
def gui_suite_lock(timeout=300):
    with machine_desktop_lock(timeout=timeout) if sys.platform == 'darwin' else nullcontext() as machine:
        with clipboard_lock(timeout=timeout) as owner:
            if machine: owner['machine_desktop_lock'] = machine
            yield owner


@contextmanager
def clipboard_lock(path=None, timeout=300):
    path = Path(path) if path is not None else default_lock_path()
    path.parent.mkdir(parents=True, exist_ok=True)
    owner = {'token': uuid.uuid4().hex, 'pid': os.getpid(), 'platform': sys.platform, 'created_unix': time.time(), 'lock_path': str(path)}
    started = time.monotonic()
    while True:
        try:
            # Do not expose a partial owner record to a second owner: the existence
            # itself is the lock, and an empty/partial record is still locked.
            descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            break
        except FileExistsError:
            if time.monotonic() - started >= timeout:
                try:
                    detail = path.read_text(encoding='utf-8')[:2048]
                except OSError:
                    detail = 'Owner metadata unavailable'
                raise TimeoutError(f'Clipboard suite already owns {path}: {detail}. No lock was stolen. If a runner crashed, verify its suite and children have stopped before removing this lock.')
            time.sleep(0.05)
    try:
        with os.fdopen(descriptor, 'w', encoding='utf-8') as stream:
            json.dump(owner, stream)
        owner['wait_seconds'] = time.monotonic() - started
        yield owner
    finally:
        # Delete only the exact ownership record created by this acquisition.
        try:
            if json.loads(path.read_text(encoding='utf-8')).get('token') == owner['token']:
                path.unlink()
        except FileNotFoundError:
            pass


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--timeout', type=float, default=1)
    args = parser.parse_args()
    with clipboard_lock(timeout=args.timeout) as record:
        print(json.dumps(record))
