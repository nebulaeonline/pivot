"""Synthetic refusal/routing tests: no window, clipboard or input API is used."""
from contextlib import ExitStack
import importlib.util
import os
from pathlib import Path
import runpy
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
import execution_policy as policy


class PolicyTests(unittest.TestCase):
    def setUp(self):
        self.environment = patch.dict(os.environ, {}, clear=True)
        self.environment.start()

    def tearDown(self):
        self.environment.stop()

    def interactive(self, answer='yes'):
        stack = ExitStack()
        stack.enter_context(patch('sys.stdin.isatty', return_value=True))
        stack.enter_context(patch('sys.stdout.isatty', return_value=True))
        stack.enter_context(patch('builtins.input', return_value=answer))
        return stack

    def test_noninteractive_flags_and_vm_names_never_authorize(self):
        with patch('sys.stdin.isatty', return_value=False):
            for name in ['PIVOT_ISOLATED', 'CI', 'PIVOT_VM_NAME']:
                os.environ[name] = 'Pivot-Win11-x64'
                with self.assertRaises(policy.Blocked):
                    with policy.desktop_session('synthetic'): self.fail('authorized')

    def test_default_no_eof_and_nonterminal_output_refuse(self):
        for answer in ['', 'no']:
            with self.interactive(answer), self.assertRaises(policy.Blocked):
                with policy.desktop_session('synthetic'): self.fail('authorized')
        with self.interactive(), patch('builtins.input', side_effect=EOFError), self.assertRaises(policy.Blocked):
            with policy.desktop_session('synthetic'): self.fail('authorized')
        with self.interactive(), patch('sys.stdout.isatty', return_value=False), self.assertRaises(policy.Blocked):
            with policy.desktop_session('synthetic'): self.fail('authorized')

    def test_live_permit_nested_use_revocation_and_wrong_token(self):
        with self.interactive(), policy.desktop_session('synthetic'):
            permit = os.environ[policy.PERMIT]
            self.assertTrue(policy.verify_permit(permit))
            port, secret = permit.split(':')
            wrong = port + ':' + ('0' if secret[0] != '0' else '1') + secret[1:]
            self.assertFalse(policy.verify_permit(wrong))
            with patch('builtins.input', side_effect=AssertionError('second prompt')):
                with policy.desktop_session('nested'): pass
        self.assertNotIn(policy.PERMIT, os.environ)
        self.assertFalse(policy.verify_permit(permit))
        with patch.dict(os.environ, {policy.PERMIT: permit}), self.assertRaises(policy.Blocked):
            with policy.desktop_session('stale'): self.fail('authorized')

    def test_expiration_and_exception_cleanup(self):
        with self.interactive():
            with self.assertRaisesRegex(RuntimeError, 'cancel'):
                with policy.desktop_session('synthetic'):
                    permit = os.environ[policy.PERMIT]
                    with patch('execution_policy.time.monotonic', return_value=float('inf')):
                        self.assertFalse(policy.verify_permit(permit))
                    raise RuntimeError('cancel')
            self.assertNotIn(policy.PERMIT, os.environ)
            self.assertFalse(policy.verify_permit(permit))

    def test_headless_environment_removes_automation_and_authority(self):
        os.environ.update({policy.PERMIT:'fake', 'PIVOT_UI_PEER_REPORT':'fake',
                           'PIVOT_UI_SMOKE_REPORT':'fake', 'PATH':'keep'})
        self.assertEqual(policy.headless_environment(), {'PATH':'keep'})

    def test_malformed_permits_fail_closed(self):
        for permit in ['', 'yes', 'true', '0:'+'a'*64, '999999:'+'a'*64,
                       'localhost:1:secret', '12:'+'z'*64]:
            self.assertFalse(policy.verify_permit(permit))

    def test_direct_python_entries_refuse_before_any_child(self):
        for script, args in [('scripts/ci.py', ['--desktop']),
                             ('scripts/verify_native_aot.py', []),
                             ('tests/application-session/ui_smoke.py', [])]:
            with self.subTest(script=script), patch('sys.argv', [script, *args]), \
                    patch('sys.stdin.isatty', return_value=False), \
                    patch('subprocess.Popen', side_effect=AssertionError('spawn before consent')), \
                    patch('subprocess.run', side_effect=AssertionError('run before consent')):
                with self.assertRaises(SystemExit) as result:
                    runpy.run_path(str(ROOT/script), run_name='__main__')
                self.assertEqual(result.exception.code, 2)

    def test_ci_headless_no_vm_and_desktop_authority_routing(self):
        spec = importlib.util.spec_from_file_location('pivot_ci', ROOT/'scripts/ci.py')
        ci = importlib.util.module_from_spec(spec); spec.loader.exec_module(ci)
        calls = []
        class Child:
            stdout = []
            def wait(self): return 0
        def spawn(*args, **kwargs): calls.append(kwargs['env']); return Child()
        with tempfile.TemporaryDirectory() as tmp, patch('subprocess.Popen', side_effect=spawn):
            os.environ.update({policy.PERMIT:'synthetic', 'PIVOT_UI_SMOKE_REPORT':'polluted'})
            ci.run_step('dotnet-build', ['synthetic'], Path(tmp))
            ci.run_step('desktop-ui', ['synthetic'], Path(tmp))
        self.assertNotIn(policy.PERMIT, calls[0])
        self.assertNotIn('PIVOT_UI_SMOKE_REPORT', calls[0])
        self.assertEqual(calls[1][policy.PERMIT], 'synthetic')
        self.assertNotIn('PIVOT_UI_SMOKE_REPORT', calls[1])
        with patch('sys.argv', ['ci.py']), patch.object(ci, 'execute', return_value=0) as execute, \
                patch.object(ci, 'desktop_session', side_effect=AssertionError('headless prompt')):
            self.assertEqual(ci.main(), 0); self.assertFalse(execute.call_args.args[0].desktop)
        def accepted(args):
            self.assertTrue(args.desktop)
            self.assertTrue(policy.verify_permit(os.environ[policy.PERMIT]))
            return 0
        # The preceding routing check deliberately installed a fake inherited
        # permit. A new local-consent scenario starts without that stale token.
        os.environ.pop(policy.PERMIT, None)
        with self.interactive(), patch('sys.argv', ['ci.py', '--desktop']), \
                patch.object(ci, 'execute', side_effect=accepted):
            self.assertEqual(ci.main(), 0)


class ManagedProbeTests(unittest.TestCase):
    def test_real_managed_guard_manual_refusal_and_live_child_peers(self):
        probe = os.environ.get('PIVOT_POLICY_PROBE')
        self.assertIsNotNone(probe, 'Run tests/execution-safety/verify.py to build the real guard probe')
        def run(env):
            return subprocess.run(['dotnet', probe], env=env, capture_output=True, text=True, timeout=10)
        clean = policy.headless_environment()
        self.assertEqual(run(clean).returncode, 0)
        for flag in ['PIVOT_UI_SMOKE_REPORT', 'PIVOT_UI_PEER_REPORT']:
            env = dict(clean, **{flag:'synthetic-not-a-report'})
            self.assertEqual(run(env).returncode, 2)
            env[policy.PERMIT] = '12345:invalid'
            self.assertEqual(run(env).returncode, 2)
        with patch('sys.stdin.isatty', return_value=True), patch('sys.stdout.isatty', return_value=True), \
                patch('builtins.input', return_value='yes'), policy.desktop_session('synthetic probe'):
            for flag in ['PIVOT_UI_SMOKE_REPORT', 'PIVOT_UI_PEER_REPORT']:
                env = dict(os.environ, **{flag:'synthetic-not-a-report'})
                self.assertEqual(run(env).returncode, 0)
            retained = dict(env)
        self.assertEqual(run(retained).returncode, 2)


if __name__ == '__main__': unittest.main()
