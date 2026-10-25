"""Practical orchestration contracts; no GUI or application builds."""
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('matrix_runner',ROOT/'scripts/validate.py')
runner=importlib.util.module_from_spec(spec);spec.loader.exec_module(runner)

class MatrixTests(unittest.TestCase):
 def test_private_environment_shared_temp_aliases_and_optout(self):
  with tempfile.TemporaryDirectory() as temp:
   env=runner.environment(Path(temp))
   self.assertEqual(env['TMP'],env['TMPDIR']);self.assertEqual(env['TEMP'],env['TMP'])
   self.assertEqual(env['AVALONIA_TELEMETRY_OPTOUT'],'1')
 def test_tool_architecture_mismatch_is_setup_failure(self):
  probes={'rustc':{'stdout':'host: x86_64-unknown-linux-gnu'},'dotnet':{'stdout':'Architecture: x64'},'sdk':{'stdout':'10.0.113'}}
  runner.validate_tools('linux-x64',probes)
  with self.assertRaises(ValueError):runner.validate_tools('linux-arm64',probes)
 def test_unconfigured_target_never_executes(self):
  args=type('Args',(),{'suite':'headless','gui':False})()
  with patch.object(runner.subprocess,'call') as call:
   result=runner.run_target('osx-arm64',{'transport':'ssh','host':None},args,'run',ROOT)
   self.assertEqual(result['status'],'NotConfigured');call.assert_not_called()
 def test_missing_report_does_not_mean_pass(self):
  with tempfile.TemporaryDirectory() as temp,patch.object(runner,'owned_alive',return_value=False):
   with self.assertRaises(FileNotFoundError):runner.check_gui(Path(temp),'managed',None)
 def test_real_owned_survivor_fails(self):
  with tempfile.TemporaryDirectory() as temp:
   output=Path(temp);folder=output/'desktop-ui';folder.mkdir()
   for pid,name in enumerate(['results.json','results.clipboard-save.json','results.discard.json','results.clean.json'],1):
    (folder/name).write_text(json.dumps({'process_id':pid,'status':'passed','execution_mode':'managed','remaining_charge':0,'graphical_backend':'actual'}))
   with patch.object(runner,'owned_alive',return_value=False):self.assertEqual(runner.check_gui(output,'managed',None)['zero_charges'],4)
   with patch.object(runner,'owned_alive',side_effect=lambda p:p==2):
    with self.assertRaisesRegex(ValueError,'survived'):runner.check_gui(output,'managed',None)
 def test_headless_entry_does_not_enable_desktop(self):
  command=runner.command_for('headless',Path('/native/reports'))
  self.assertIn('scripts/ci.py',command);self.assertNotIn('--desktop',command)
 def test_native_aot_recognizes_all_six_execution_RIDs(self):
  sys.path.insert(0,str(ROOT/'scripts'))
  import verify_native_aot as aot
  for os_name,machine,rid in [('win32','AMD64','win-x64'),('win32','ARM64','win-arm64'),
   ('linux','x86_64','linux-x64'),('linux','aarch64','linux-arm64'),('darwin','arm64','osx-arm64'),('darwin','x86_64','osx-x64')]:
   arch=rid.rsplit('-',1)[1]
   with self.subTest(rid=rid),patch.object(aot.sys,'platform',os_name),patch.object(aot.platform,'machine',return_value=machine), \
        patch.object(aot.subprocess,'check_output',return_value='Host:\n Architecture: '+arch+'\n'):
    self.assertEqual(aot.host_rid(),rid)
 def test_arm_python_can_control_explicit_mac_x64_tools(self):
  identity={'native_architecture':'arm64','execution':'native'}
  runner.validate_controller('osx-x64','osx-arm64',identity,True)
  with self.assertRaises(ValueError):runner.validate_controller('osx-x64','osx-arm64',identity,False)
  with self.assertRaises(ValueError):runner.validate_controller('linux-x64','osx-arm64',identity,True)
 def test_aot_target_follows_dotnet_not_arm_python(self):
  sys.path.insert(0,str(ROOT/'scripts'))
  import verify_native_aot as aot
  with patch.object(aot.sys,'platform','darwin'),patch.object(aot.platform,'machine',return_value='arm64'), \
       patch.object(aot.subprocess,'check_output',return_value='Host:\n Architecture: x64\n'):
   self.assertEqual(aot.host_rid(),'osx-x64')
 def test_mac_x64_environment_selects_tools_without_global_changes(self):
  import os
  before=dict(os.environ)
  with tempfile.TemporaryDirectory() as temp,patch.object(runner.sys,'platform','darwin'), \
       patch.object(Path,'is_dir',return_value=True),patch.object(Path,'is_file',side_effect=lambda p:p.name=='clang',autospec=True):
   env=runner.environment(Path(temp),'osx-x64')
   self.assertEqual(env['RUSTUP_TOOLCHAIN'],'nightly-x86_64-apple-darwin')
   self.assertEqual(env['DOTNET_ROOT'],str(Path('/usr/local/share/dotnet/x64')))
   self.assertTrue(env['CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER'].endswith('clang'))
  self.assertEqual(dict(os.environ),before)
 def test_mac_native_work_rejects_network_mount(self):
  with tempfile.TemporaryDirectory() as temp,patch.object(runner.sys,'platform','darwin'):
   good=[{'exit_code':0,'stdout':'Filesystem blocks Mounted on\n/dev/disk3s1 100 /System/Volumes/Data\n'},
         {'exit_code':0,'stdout':'/dev/disk3s1 on /System/Volumes/Data (apfs, local, journaled)\n'}]
   with patch.object(runner,'query',side_effect=good):self.assertEqual(runner.native_root(Path(temp))['type'],'apfs')
   bad=[{'exit_code':0,'stdout':'Filesystem blocks Mounted on\n//server/share 100 /Volumes/share\n'},
        {'exit_code':0,'stdout':'//server/share on /Volumes/share (smbfs, nodev)\n'}]
   with patch.object(runner,'query',side_effect=bad):
    with self.assertRaises(ValueError):runner.native_root(Path(temp))

if __name__=='__main__':unittest.main()
