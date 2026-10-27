"""Cooperative execution safety. No VM dispatch or unattended guest approval yet."""
from contextlib import contextmanager
from enum import Enum
import hashlib
import hmac
import json
import os
from pathlib import Path
import secrets
import socket
import socketserver
import sys
import threading
import time

PERMIT = 'PIVOT_DESKTOP_PERMIT'
WARNING = ('Pivot desktop acceptance will open application windows, manipulate keyboard '
           'focus and access the operating-system clipboard. Do not use other applications '
           'during this test. Continue? [y/N] ')


class ExecutionClass(str, Enum):
    HEADLESS = 'headless'
    GUI = 'gui-isolated-or-interactive'
    FIXTURE_FAULT = 'fixture-contained-fault'
    DESTRUCTIVE = 'destructive-fault'
    TRANSLATED = 'compat-translated'


class Blocked(RuntimeError):
    """Infrastructure/consent unavailable, distinct from a failed product test."""


# Audited entry points. Headless and fixture-fault capabilities are independent;
# none of these fixture tests require system disruption or a VM rollback.
ENTRY_POINTS = {
    'scripts/ci.py': (ExecutionClass.HEADLESS, ExecutionClass.FIXTURE_FAULT),
    'scripts/verify_native_aot.py': (ExecutionClass.HEADLESS, ExecutionClass.GUI),
    'scripts/run_desktop.py': (ExecutionClass.GUI,),
    'scripts/attend_validation.py': (ExecutionClass.GUI,),
    'scripts/desktop_test_lock.py': (ExecutionClass.HEADLESS,),
    'scripts/bulk_workbook.py': (ExecutionClass.HEADLESS,),
    'scripts/research_history_matrix.py': (ExecutionClass.HEADLESS,),
    'scripts/research_history_durability.py': (ExecutionClass.HEADLESS, ExecutionClass.FIXTURE_FAULT),
    'scripts/research_history_acknowledgments.py': (ExecutionClass.HEADLESS,),
    'scripts/research_history_association.py': (ExecutionClass.HEADLESS, ExecutionClass.FIXTURE_FAULT),
    'scripts/research_mutation_records.py': (ExecutionClass.HEADLESS, ExecutionClass.FIXTURE_FAULT),
    'tests/application-session/verify.py': (ExecutionClass.HEADLESS, ExecutionClass.FIXTURE_FAULT),
    'tests/application-session/ui_smoke.py': (ExecutionClass.HEADLESS, ExecutionClass.GUI),
    'tests/application-session/test_desktop_lock.py': (ExecutionClass.HEADLESS,),
    'tests/decimal-parity/verify.py': (ExecutionClass.HEADLESS,),
    'tests/decimal-parity/aggregate_reference.py': (ExecutionClass.HEADLESS,),
    'tests/protocol-parity/verify.py': (ExecutionClass.HEADLESS,),
    'tests/ipc/verify.py': (ExecutionClass.HEADLESS, ExecutionClass.FIXTURE_FAULT),
    'tests/execution-safety/verify.py': (ExecutionClass.HEADLESS,),
    'tests/execution-safety/test_policy.py': (ExecutionClass.HEADLESS,),
}


def declare_execution(path):
    root = Path(__file__).resolve().parents[1]
    entry = Path(path).resolve().relative_to(root).as_posix()
    classes = ENTRY_POINTS[entry]  # Unknown entry requires audit, not a safe default.
    print('Execution classes: ' + ', '.join(c.value for c in classes), flush=True)
    return classes


def desktop_environment():
    """Retain live authority but never inherit stale self-check/peer activation."""
    return {k: v for k, v in os.environ.items()
            if not k.startswith('PIVOT_UI_')}


def headless_environment():
    return {k: v for k, v in desktop_environment().items() if k != PERMIT}


def _line(stream):
    data = stream.readline(256)
    if not data.endswith(b'\n') or len(data) > 200:
        raise ValueError('Invalid permit message')
    return data.strip().decode('ascii')


def verify_permit(value):
    """Check live loopback capability, not an isolation flag or saved consent file."""
    try:
        port, secret = value.split(':')
        port = int(port)
        if not 1 <= port <= 65535 or len(secret) != 64 or len(bytes.fromhex(secret)) != 32:
            return False
        nonce = secrets.token_hex(32)
        with socket.create_connection(('127.0.0.1', port), timeout=2) as connection:
            connection.settimeout(2)
            with connection.makefile('rwb') as stream:
                stream.write((secret + '\n' + nonce + '\n').encode('ascii'))
                stream.flush()
                response = _line(stream)
        expected = hmac.new(bytes.fromhex(secret), nonce.encode('ascii'), hashlib.sha256).hexdigest()
        return hmac.compare_digest(response, expected)
    except (OSError, ValueError, TypeError):
        return False


@contextmanager
def desktop_session(label):
    """Consent once per live suite; nested runners/GUI peers inherit the capability.

    Not a hostile same-user security boundary. No JSON/hostname/environment based
    unattended isolation override is accepted. Guest approval is a later gate.
    """
    inherited = os.environ.get(PERMIT)
    if inherited is not None:
        if not verify_permit(inherited):
            raise Blocked('Invalid or expired desktop permit; no graphical work started.')
        yield {'class': ExecutionClass.GUI.value, 'mode': 'inherited-live-interactive-consent'}
        return
    if not sys.stdin.isatty() or not sys.stdout.isatty():
        raise Blocked('Desktop acceptance requires interactive consent. Unattended guest '
                      'approval is not configured; no host fallback or graphical work started.')
    print(f'Execution class: {ExecutionClass.GUI.value}; {label}', flush=True)
    try:
        answer = input(WARNING)
    except EOFError:
        answer = ''
    if answer.strip().lower() not in ('y', 'yes'):
        raise Blocked('Desktop acceptance declined; no graphical work started.')
    secret = secrets.token_hex(32)
    expires = time.monotonic() + 4 * 60 * 60

    class Handler(socketserver.StreamRequestHandler):
        def handle(self):
            try:
                self.request.settimeout(2)
                token, nonce = _line(self.rfile), _line(self.rfile)
                if (time.monotonic() >= expires or not hmac.compare_digest(token, secret)
                        or len(nonce) != 64 or len(bytes.fromhex(nonce)) != 32):
                    return
                reply = hmac.new(bytes.fromhex(secret), nonce.encode('ascii'), hashlib.sha256).hexdigest()
                self.wfile.write((reply + '\n').encode('ascii'))
            except (OSError, ValueError):
                return

    class Server(socketserver.ThreadingTCPServer):
        daemon_threads = True

    with Server(('127.0.0.1', 0), Handler) as server:
        thread = threading.Thread(target=server.serve_forever, kwargs={'poll_interval': .05}, daemon=True)
        thread.start()
        os.environ[PERMIT] = f'{server.server_address[1]}:{secret}'
        try:
            yield {'class': ExecutionClass.GUI.value, 'mode': 'interactive-consent',
                   'unattended_isolation': 'NotConfigured'}
        finally:
            os.environ.pop(PERMIT, None)
            server.shutdown()
            thread.join(timeout=3)


def blocked_exit(error):
    print(json.dumps({'status': 'Blocked', 'class': ExecutionClass.GUI.value,
                      'reason': str(error), 'graphical_tests': 'not started',
                      'unattended_isolation': 'NotConfigured'}), file=sys.stderr)
    return 2
