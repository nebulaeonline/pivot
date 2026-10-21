"""Regression checks for the shared clipboard-suite runner constraint."""
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'scripts'))
from desktop_test_lock import clipboard_lock


class ClipboardLockTests(unittest.TestCase):
    def test_an_independent_runner_cannot_enter_until_owner_releases(self):
        with tempfile.TemporaryDirectory() as directory:
            lock = Path(directory)/'clipboard.lock'
            program = 'import sys;sys.path.insert(0,sys.argv[1]);from desktop_test_lock import clipboard_lock\nwith clipboard_lock(sys.argv[2]):\n print("owned",flush=True);sys.stdin.readline()'
            child = subprocess.Popen([sys.executable, '-c', program, str(ROOT/'scripts'), str(lock)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
            try:
                self.assertEqual(child.stdout.readline().strip(), 'owned')
                with self.assertRaises(TimeoutError):
                    with clipboard_lock(lock, timeout=0.1):
                        self.fail('Concurrent runner entered')
                child.communicate('\n', timeout=5)
                self.assertEqual(child.returncode, 0)
                with clipboard_lock(lock, timeout=0.1):
                    self.assertTrue(lock.exists())
                self.assertFalse(lock.exists())
            finally:
                if child.poll() is None:
                    child.kill()
                    child.wait()

    def test_failed_suite_releases_its_lock(self):
        with tempfile.TemporaryDirectory() as directory:
            lock = Path(directory)/'clipboard.lock'
            with self.assertRaisesRegex(ValueError, 'suite failed'):
                with clipboard_lock(lock):
                    raise ValueError('suite failed')
            self.assertFalse(lock.exists())

    def test_unknown_or_stale_owner_is_not_silently_stolen(self):
        with tempfile.TemporaryDirectory() as directory:
            lock = Path(directory)/'clipboard.lock'
            lock.write_text('partial owner record', encoding='utf-8')
            with self.assertRaisesRegex(TimeoutError, 'No lock was stolen'):
                with clipboard_lock(lock, timeout=0):
                    self.fail('Unknown owner was replaced')
            self.assertEqual(lock.read_text(), 'partial owner record')


if __name__ == '__main__':
    unittest.main()
