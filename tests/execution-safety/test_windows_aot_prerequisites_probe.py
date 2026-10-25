"""Collector classification and metadata-only supplement; no build or GUI."""
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts/qualification'))
import windows_aot_prerequisites_probe as probe


class PrerequisiteCollectorTests(unittest.TestCase):
    def test_verified_missing_and_blocked_are_separate(self):
        self.assertEqual(probe.classify({'status':'Verified'}),'Verified')
        for reason in ['Required installed tool missing: cl','Native library missing: x','Native Rust target library missing: x']:
            self.assertEqual(probe.classify({'status':'Blocked','reason':reason}),'Missing')
        for reason in ['Native x64 tool required: cl','Access denied','Native Rust target query failed']:
            self.assertEqual(probe.classify({'status':'Blocked','reason':reason}),'Blocked')

    def test_target_and_tool_metadata_only_queries_preserve_private_environment(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder=Path(tmp);(folder/'libstd-example.rlib').write_bytes(b'standard-library fixture')
            base={'status':'Verified','tools':{'cl':{'path':'fixture/cl.exe'}}}
            env={'RUSTUP_AUTO_INSTALL':'0','AVALONIA_TELEMETRY_OPTOUT':'1'}
            with patch.object(probe.aot,'prerequisites',return_value=base), \
                    patch.object(probe.aot,'image_arch',return_value='x64'), \
                    patch.object(probe,'tool_version',return_value={'exit_code':0,'version':{'file_version':'fixture'}}), \
                    patch.object(probe.aot.common,'local_tree',side_effect=lambda p:p), \
                    patch.object(probe.subprocess,'run',return_value=SimpleNamespace(returncode=0,stdout=str(folder),stderr='')) as query:
                result=probe.collect(folder,env)
                self.assertEqual(result['qualification_status'],'Verified')
                self.assertEqual(query.call_args.args[0],['rustc','--print','target-libdir','--target',probe.TARGET])
                self.assertIs(query.call_args.kwargs['env'],env)
                self.assertEqual(result['tools']['cl']['architecture'],'x64')
                self.assertEqual(len(result['rust_target']['standard_library_hashes']),1)
                self.assertEqual(env,{'RUSTUP_AUTO_INSTALL':'0','AVALONIA_TELEMETRY_OPTOUT':'1'})

    def test_missing_prerequisites_do_not_attempt_supplemental_commands(self):
        with patch.object(probe.aot,'prerequisites',return_value={'status':'Blocked','reason':'Required installed tool missing: cl'}), \
                patch.object(probe,'tool_version') as version,patch.object(probe.subprocess,'run') as query:
            self.assertEqual(probe.collect(Path('.'),{})['qualification_status'],'Missing')
            version.assert_not_called();query.assert_not_called()


if __name__=='__main__':unittest.main()
