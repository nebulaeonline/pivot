"""Non-GUI preflight tests. No Pivot window, clipboard access or positive suite."""
import contextlib
from copy import deepcopy
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
import launch_windows_managed as exporter


def reviewed_fixture():
    return {'manufacturer': 'Microsoft Corporation', 'model': 'Virtual Machine',
            'uuid': worker.REVIEWED_FIRMWARE_UUID, 'hostname': 'WinDev2407Eval',
            'boot': '2026-10-25T00:37:42.0932210-04:00',
            'desktop': {'pid': 123, 'user': 'User', 'session_id': 1,
                        'window_station': 'WinSta0', 'thread_desktop': 'Default', 'input_desktop': 'Default'},
            'boot_observation': deepcopy(worker.REVIEWED_SAMPLE)}


class AttendedTests(unittest.TestCase):
    def test_corrected_exporter_same_line_failure_copy_and_hash_rejection(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(worker, 'local_tree', side_effect=lambda p: p):
            parent = Path(tmp); native = parent/'attended-test/report'; native.mkdir(parents=True)
            worker.write_json(native/'preflight.json', {'status': 'PreflightPassed'})
            worker.write_json(native/'origin-hashes.json', [{'path': 'preflight.json', 'sha256': worker.sha(native/'preflight.json')}])
            log = 'Type CONFIRM: Native preflight report: '+str(native)+'\n'
            for code in [0, 1, 2]:
                target = parent/f'capture-{code}'; target.mkdir()
                self.assertEqual(exporter.collect_native_report(log, code, parent, target, worker), [])
                self.assertTrue((target/'native-report/preflight.json').is_file())
            (native/'preflight.json').write_text('tampered')
            target = parent/'bad'; target.mkdir()
            self.assertTrue(exporter.collect_native_report(log, 0, parent, target, worker))
            self.assertFalse((target/'native-report').exists())
            self.assertEqual(exporter.capture_status({'status': 'Failed'}, 1, ['export failure']), 'Failed')
            self.assertEqual(exporter.capture_status({'status': 'Passed'}, 0, ['export failure']), 'Blocked')

    def test_exporter_tee_preserves_real_tty_and_output(self):
        for tty in [True, False]:
            terminal, log = io.StringIO(), io.StringIO()
            terminal.isatty = lambda: tty
            tee = exporter.Tee(terminal, log)
            self.assertEqual(tee.isatty(), tty)
            tee.write('scratch'); tee.flush()
            self.assertEqual(terminal.getvalue(), log.getvalue())
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
        with tempfile.TemporaryDirectory() as tmp, patch.object(worker, 'POSITIVE_RUN_APPROVED', True), \
                patch.object(worker.runpy, 'run_path') as run:
            original_env, original_args, original_path = dict(os.environ), sys.argv[:], sys.path[:]
            streams = (sys.stdin, sys.stdout)
            def check_runner(*args, **kwargs):
                self.assertEqual(args, (str(Path(tmp)/worker.ENTRY),))
                self.assertEqual(kwargs, {'run_name': '__main__'})
                self.assertEqual(dict(os.environ), {'SAFE': 'value'})
                self.assertEqual(sys.argv, [worker.ENTRY, '--output-dir', str(Path(tmp)/'report/managed')])
                self.assertEqual((sys.stdin, sys.stdout), streams)
                raise SystemExit(2)
            run.side_effect = check_runner
            self.assertEqual(worker.invoke_managed(Path(tmp), Path(tmp)/'report', {'SAFE': 'value'}), 2)
            self.assertEqual(dict(os.environ), original_env)
            self.assertEqual(sys.argv, original_args); self.assertEqual(sys.path, original_path)

    def test_staged_managed_outcomes_preserve_failure_and_cleanup_distinctions(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for code, expected in [(0, 'Passed'), (1, 'Failed'), (2, 'Blocked')]:
                with self.subTest(code=code), patch.object(worker, 'POSITIVE_RUN_APPROVED', True), \
                        patch.object(worker, 'guest_identity', return_value=reviewed_fixture()), \
                        patch.object(worker, 'invoke_managed', return_value=code), \
                        patch.object(worker, 'same_source', return_value=[]), \
                        patch.object(worker, 'owned_processes', return_value={'status': 'Verified', 'matches': []}), \
                        patch.object(worker, 'managed_evidence', return_value={'application_reports': 4}):
                    self.assertEqual(worker.execute_managed(root, root, {}, [], reviewed_fixture()), code)
                    self.assertEqual(worker.read_json(root/'managed-run.json')['status'], expected)
            with patch.object(worker, 'POSITIVE_RUN_APPROVED', True), \
                    patch.object(worker, 'guest_identity', return_value=reviewed_fixture()), \
                    patch.object(worker, 'invoke_managed', return_value=0), \
                    patch.object(worker, 'same_source', return_value=['changed']), \
                    patch.object(worker, 'owned_processes', return_value={'status': 'Verified', 'matches': []}), \
                    patch.object(worker, 'managed_evidence', return_value={}):
                self.assertEqual(worker.execute_managed(root, root, {}, [], reviewed_fixture()), 2)

    def test_managed_evidence_requires_actual_session_backend_and_zero_charge(self):
        good = {'status': 'passed', 'remaining_charge': 0, 'process_session_id': 1,
                'process_id': 100, 'graphical_backend': 'Actual.WindowImpl'}
        with patch.object(worker, 'read_json', return_value=good), \
                patch.object(worker, 'desktop_identity', return_value={'session_id': 1}):
            self.assertEqual(worker.managed_evidence(Path('.'))['zero_charge_fields'], 4)
        for delta in [{'remaining_charge': 1}, {'graphical_backend': None}, {'process_session_id': 2}]:
            with self.subTest(delta=delta), patch.object(worker, 'read_json', return_value=dict(good, **delta)), \
                    patch.object(worker, 'desktop_identity', return_value={'session_id': 1}), self.assertRaises(ValueError):
                worker.managed_evidence(Path('.'))

    def test_new_boot_or_session_cannot_reuse_prepared_managed_profile(self):
        changed_boot, changed_session = reviewed_fixture(), reviewed_fixture()
        changed_boot['boot_observation']['measurement']['start_event']['record_id'] += 1
        changed_session['desktop']['session_id'] = 99
        for identity in [changed_boot, changed_session]:
            with tempfile.TemporaryDirectory() as tmp, self.subTest(identity=identity), patch.object(worker, 'POSITIVE_RUN_APPROVED', True), \
                    patch.object(worker, 'guest_identity', return_value=identity), \
                    patch.object(worker, 'invoke_managed') as invoke, self.assertRaises(ValueError):
                worker.execute_managed(Path(tmp), Path(tmp), {}, [], identity)
            invoke.assert_not_called()

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
                for filename in ['windows_attended_gui.py', 'guest_preflight.py', 'run_headless_ci.py', 'windows_boot_continuity.py']:
                    shutil.copyfile(ROOT/'scripts/qualification'/filename, bundle/filename)
                if scenario == 'changed-helper':
                    (bundle/'windows_attended_gui.py').write_text('# differs from executing helper')
                files = [{'path': p.name, 'sha256': worker.sha(p)} for p in bundle.iterdir()]
                worker.write_json(bundle/'bundle-receipt.json', {'files': files})
                digest = worker.sha(bundle/'bundle-receipt.json') if scenario != 'bad-receipt' else '0'*64
                argv = ['worker', '--bundle-dir', str(bundle), '--receipt-sha256', digest, '--work-root', str(work)]
                identities = [reviewed_fixture(), reviewed_fixture()]
                if scenario == 'changed-session': identities[1]['desktop']['session_id'] = 2
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
