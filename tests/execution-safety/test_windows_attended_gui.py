"""Non-GUI preflight tests. No Pivot window, clipboard access or positive suite."""
import contextlib
import io
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'scripts/qualification'))
import windows_attended_gui as worker


class AttendedTests(unittest.TestCase):
    def test_positive_path_is_disabled_before_any_preflight_or_child(self):
        argv = ['worker', '--bundle-dir', '.', '--receipt-sha256', '0'*64,
                '--work-root', '.', '--run-managed']
        with patch.object(sys, 'argv', argv), patch.object(worker, 'guest_identity') as identity, \
                patch.object(worker.subprocess, 'run') as run, contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(worker.main(), 2)
            identity.assert_not_called(); run.assert_not_called()
        with patch.object(worker.subprocess, 'run') as run, self.assertRaises(ValueError):
            worker.invoke_managed(Path('.'), Path('.'), {})
        run.assert_not_called()

    def test_noninteractive_refuses_before_machine_queries(self):
        argv = ['worker', '--bundle-dir', '.', '--receipt-sha256', '0'*64, '--work-root', '.']
        with patch.object(sys, 'argv', argv), patch.object(sys.stdin, 'isatty', return_value=False), \
                patch.object(worker, 'guest_identity') as identity, contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(worker.main(), 2); identity.assert_not_called()

    def test_fixed_future_invocation_inherits_terminal_without_consent_injection(self):
        with patch.object(worker, 'POSITIVE_RUN_APPROVED', True), patch.object(worker.subprocess, 'run') as run:
            run.return_value.returncode = 2
            self.assertEqual(worker.invoke_managed(Path('source'), Path('report'), {'SAFE': 'value'}), 2)
            args, options = run.call_args
            self.assertEqual(args[0], [sys.executable, worker.ENTRY, '--output-dir', str(Path('report/managed'))])
            self.assertEqual(options, {'cwd': Path('source'), 'env': {'SAFE': 'value'}, 'check': False})

    def test_wrong_platform_hostname_and_non_hyperv_metadata_refuse(self):
        with patch.object(worker.sys, 'platform', 'linux'), self.assertRaises(ValueError):
            worker.guest_identity()
        with patch.object(worker.sys, 'platform', 'win32'), \
                patch.object(worker, 'windows_architectures', return_value=('x64', 'x64')), \
                patch.object(worker.platform, 'node', return_value='ROBPC'), self.assertRaises(ValueError):
            worker.guest_identity()
        with patch.object(worker.sys, 'platform', 'win32'), \
                patch.object(worker, 'windows_architectures', return_value=('x64', 'x64')), \
                patch.object(worker.platform, 'node', return_value='WinDev2407Eval'), \
                patch.object(worker.subprocess, 'run') as run, self.assertRaises(ValueError):
            run.return_value.stdout = json.dumps({'manufacturer': 'Other', 'model': 'Physical'})
            worker.guest_identity()

    def test_real_windows_desktop_metadata_adapter_only(self):
        if sys.platform != 'win32':
            self.skipTest('Windows OS adapter only')
        # Read metadata only. An unavailable desktop is a valid refusal, not a pass.
        try:
            result = worker.desktop_identity()
        except ValueError as error:
            self.assertTrue(str(error)); return
        self.assertGreater(result['session_id'], 0)
        self.assertEqual(result['window_station'].lower(), 'winsta0')
        self.assertEqual(result['thread_desktop'].lower(), result['input_desktop'].lower())

    def test_native_root_rejects_network_and_reparse(self):
        if sys.platform != 'win32':
            self.skipTest('Windows filesystem attributes only')
        with tempfile.TemporaryDirectory() as tmp:
            self.assertEqual(worker.local_tree(tmp), Path(os.path.abspath(tmp)))
            with patch.object(worker, 'filesystem', return_value={'native_work_root': False}), self.assertRaises(ValueError):
                worker.local_tree(tmp)
            original = Path.lstat
            def fake_stat(path, *args, **kwargs):
                value = original(path, *args, **kwargs)
                if str(path) == str(Path(tmp)):
                    from types import SimpleNamespace
                    return SimpleNamespace(st_file_attributes=0x400)
                return value
            with patch.object(Path, 'lstat', fake_stat), self.assertRaises(ValueError):
                worker.local_tree(tmp)

    def test_environment_never_inherits_pivot_authority(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(worker, 'local_tree', side_effect=lambda p: p), \
                patch.dict(os.environ, {'PIVOT_DESKTOP_PERMIT': 'stale', 'PIVOT_UI_PEER_REPORT': 'bad',
                                        'PIVOT_SESSION_PATH': 'bad', 'PIVOT_FILE_TEST_ROOT': 'J:/bad'}):
            env = worker.clean_environment(Path(tmp))
            for key in ['PIVOT_DESKTOP_PERMIT', 'PIVOT_UI_PEER_REPORT', 'PIVOT_SESSION_PATH']:
                self.assertNotIn(key, env)
            self.assertEqual(env['PIVOT_FILE_TEST_ROOT'], str(Path(tmp)/'fixtures'))
            self.assertEqual(env['DOTNET_GENERATE_ASPNET_CERTIFICATE'], 'false')

    def test_full_synthetic_preflight_and_refusals_no_children(self):
        for scenario in ['pass', 'decline', 'changed-session', 'bad-receipt', 'bad-input', 'bad-payload', 'changed-helper']:
            with self.subTest(scenario=scenario), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp); bundle = root/'bundle'; bundle.mkdir(); work = root/'work'; work.mkdir()
                payload = b'# synthetic fixed entry point; never executed\n'
                rows = [{'path': worker.ENTRY, 'sha256': __import__('hashlib').sha256(payload).hexdigest()}]
                with zipfile.ZipFile(bundle/'source.zip', 'w') as archive:
                    archive.writestr(worker.ENTRY, b'tampered' if scenario == 'bad-payload' else payload)
                worker.write_json(bundle/'source-manifest.json', rows)
                worker.write_json(bundle/'build-input-manifest.json', rows if scenario != 'bad-input' else [])
                for filename in ['windows_attended_gui.py', 'guest_preflight.py', 'run_headless_ci.py']:
                    shutil.copyfile(ROOT/'scripts/qualification'/filename, bundle/filename)
                if scenario == 'changed-helper':
                    (bundle/'windows_attended_gui.py').write_text('# differs from executing helper')
                files = [{'path': p.name, 'sha256': worker.sha(p)} for p in bundle.iterdir()]
                worker.write_json(bundle/'bundle-receipt.json', {'files': files})
                digest = worker.sha(bundle/'bundle-receipt.json') if scenario != 'bad-receipt' else '0'*64
                argv = ['worker', '--bundle-dir', str(bundle), '--receipt-sha256', digest, '--work-root', str(work)]
                identities = [{'boot': 'one'}, {'boot': 'two' if scenario == 'changed-session' else 'one'}]
                with patch.object(sys, 'argv', argv), \
                        patch.object(sys.stdin, 'isatty', return_value=True), \
                        patch.object(sys.stdout, 'isatty', return_value=True), \
                        patch.object(worker, 'guest_identity', side_effect=identities), \
                        patch.object(worker, 'local_tree', side_effect=lambda p: Path(p)), \
                        patch.object(worker.stat, 'FILE_ATTRIBUTE_REPARSE_POINT', 0x400, create=True), \
                        patch('builtins.input', return_value='no' if scenario == 'decline' else 'CONFIRM'), \
                        patch.object(worker.subprocess, 'run') as child, \
                        contextlib.redirect_stderr(io.StringIO()):
                    # Non-Windows stat results have no file attributes. Patch only
                    # the extracted-source inspection, not the actual zip/hash I/O.
                    original = Path.lstat
                    def attrs(path, *args, **kwargs):
                        result = original(path, *args, **kwargs)
                        if hasattr(result, 'st_file_attributes'): return result
                        class WithAttrs:
                            st_file_attributes = 0
                            def __getattr__(self, key): return getattr(result, key)
                        return WithAttrs()
                    with patch.object(Path, 'lstat', attrs):
                        self.assertEqual(worker.main(), 0 if scenario == 'pass' else 2)
                    child.assert_not_called()
                if scenario == 'pass':
                    reports = list(work.glob('*/report/preflight.json'))
                    self.assertEqual(len(reports), 1)
                    result = worker.read_json(reports[0]); self.assertFalse(result['GUI_executed'])
                    worker.verify_files(reports[0].parent, worker.read_json(reports[0].parent/'origin-hashes.json'))


if __name__ == '__main__':
    unittest.main()
