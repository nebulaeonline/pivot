"""Read-only graphical-session prerequisites; actual GUI reports remain decisive."""
import ctypes
import os
from pathlib import Path
import subprocess
import sys


def inspect_session():
    info = {'ready_for_consent': False, 'kind': 'read-only prerequisites'}
    try:
        if sys.platform == 'win32':
            from ctypes import wintypes
            kernel = ctypes.WinDLL('kernel32', use_last_error=True)
            user = ctypes.WinDLL('user32', use_last_error=True)
            session = wintypes.DWORD()
            kernel.ProcessIdToSessionId.argtypes = [wintypes.DWORD, ctypes.POINTER(wintypes.DWORD)]
            if not kernel.ProcessIdToSessionId(os.getpid(), ctypes.byref(session)):
                raise OSError('Cannot inspect Windows session')
            info['session_id'] = session.value
            if session.value == 0:
                raise ValueError('Windows session 0 cannot run attended desktop tests')
            user.GetProcessWindowStation.restype = wintypes.HANDLE
            user.GetThreadDesktop.argtypes = [wintypes.DWORD]
            user.GetThreadDesktop.restype = wintypes.HANDLE
            user.OpenInputDesktop.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
            user.OpenInputDesktop.restype = wintypes.HANDLE
            user.CloseDesktop.argtypes = [wintypes.HANDLE]
            user.GetUserObjectInformationW.argtypes = [wintypes.HANDLE, ctypes.c_int, wintypes.LPVOID, wintypes.DWORD, ctypes.POINTER(wintypes.DWORD)]
            def name(handle):
                text = ctypes.create_unicode_buffer(256); needed = wintypes.DWORD()
                if not handle or not user.GetUserObjectInformationW(handle, 2, text, ctypes.sizeof(text), ctypes.byref(needed)):
                    raise OSError('Desktop metadata unavailable')
                return text.value
            station = name(user.GetProcessWindowStation())
            current = name(user.GetThreadDesktop(kernel.GetCurrentThreadId()))
            desktop = user.OpenInputDesktop(0, False, 1)
            try:
                active = name(desktop)
            finally:
                if desktop: user.CloseDesktop(desktop)
            info.update(window_station=station, current_desktop=current, input_desktop=active)
            if station.lower() != 'winsta0' or current.lower() != 'default' or active.lower() != current.lower():
                raise ValueError('An unlocked interactive Windows input desktop is required')
        elif sys.platform == 'darwin':
            console = Path('/dev/console').stat().st_uid
            manager = subprocess.check_output(['launchctl', 'managername'], text=True, timeout=10).strip()
            info.update(console_uid=console, bootstrap_manager=manager)
            if console != os.getuid() or manager != 'Aqua':
                raise ValueError('Run from Terminal in the logged-in Mac graphical session')
        elif sys.platform == 'linux':
            session = os.environ.get('XDG_SESSION_ID', 'self')
            def fields_for(identifier):
                output = subprocess.check_output(['loginctl', 'show-session', identifier, '-p', 'Active', '-p', 'Remote', '-p', 'Type', '-p', 'User', '-p', 'LockedHint', '-p', 'Class'], text=True, stderr=subprocess.STDOUT, timeout=10)
                return dict(line.split('=', 1) for line in output.splitlines() if '=' in line)
            try: fields = fields_for(session)
            except subprocess.SubprocessError as error:
                info['caller_session_error'] = str(error); fields = {}
            info['caller_session'] = fields
            remote_shell = bool(os.environ.get('SSH_CONNECTION') or os.environ.get('SSH_TTY'))
            if fields.get('Type') not in ('x11', 'wayland') and fields.get('Remote') != 'yes' and not remote_shell:
                # GNOME Terminal may live in the user's manager/app scope rather
                # than a logind session. Resolve the same user's primary display.
                output = subprocess.check_output(['loginctl', 'show-user', str(os.getuid()), '-p', 'Display'], text=True, stderr=subprocess.STDOUT, timeout=10)
                primary = dict(line.split('=', 1) for line in output.splitlines() if '=' in line).get('Display')
                if primary:
                    fields = fields_for(primary); info['selected_graphical_session'] = primary
            info['login_session'] = fields
            if remote_shell or fields.get('Active') != 'yes' or fields.get('Remote') != 'no' or fields.get('Type') not in ('x11', 'wayland') or fields.get('User') != str(os.getuid()):
                raise ValueError('Run from a terminal in the active local Linux graphical session')
            if fields.get('LockedHint') == 'yes':
                raise ValueError('Unlock the Linux graphical session before GUI testing')
            if not (os.environ.get('DISPLAY') or os.environ.get('WAYLAND_DISPLAY')):
                raise ValueError('Graphical-session display connection is unavailable')
        else:
            raise ValueError('Unsupported graphical-session platform')
        info['ready_for_consent'] = True
        info['note'] = 'Prerequisites only; actual backend and functional execution must still pass'
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        info['reason'] = str(error)
    return info
