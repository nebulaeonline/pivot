"""Read-only Native AOT fixture verification; no toolchain or desktop launch."""
import hashlib
from pathlib import Path
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'scripts'))
from verify_native_aot import verify_workbook_fixture

class WorkbookFixtureTests(unittest.TestCase):
    def fixture(self):
        payload=b'independent fixture payload'
        return b'PIVOTWB11'+len(payload).to_bytes(8,'little')+hashlib.sha256(payload).digest()+payload
    def test_current_format(self):
        verify_workbook_fixture(self.fixture())
    def test_obsolete_formats(self):
        for version in range(1,11):
            with self.subTest(version=version),self.assertRaises(RuntimeError):
                verify_workbook_fixture(b'PIVOTWB'+str(version).encode()+self.fixture()[9:])
    def test_corrupt_and_truncated_envelopes(self):
        original=self.fixture()
        for bad in [original[:48],original+b'extra',original[:-1]+b'X',original[:9]+(1).to_bytes(8,'little')+original[17:]]:
            with self.subTest(length=len(bad)),self.assertRaises(RuntimeError):verify_workbook_fixture(bad)
if __name__=='__main__':unittest.main()
