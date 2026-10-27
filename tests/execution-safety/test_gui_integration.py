"""Headless tests of GUI prerequisites, attendance and report collection."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
import gui_session
import desktop_test_lock
import attend_validation
from execution_policy import Blocked
spec=importlib.util.spec_from_file_location('gui_runner',ROOT/'scripts/validate.py')
runner=importlib.util.module_from_spec(spec);spec.loader.exec_module(runner)

class GuiIntegrationTests(unittest.TestCase):
 def test_short_launcher_does_not_fall_back_after_a_completed_run(self):
  with tempfile.TemporaryDirectory() as temp:
   root=Path(temp)
   for name,status in [('20260101-old','NeedsGraphicalSession'),('20260102-new','Passed')]:
    folder=root/'artifacts/validation-matrix'/name;folder.mkdir(parents=True)
    (folder/'matrix.json').write_text(json.dumps({'suite':'managed','targets':[{'rid':'win-x64','status':status}]}))
   self.assertIsNone(attend_validation.latest_plan(root,'win-x64','managed'))
 def test_short_launcher_refuses_missing_target_source(self):
  with tempfile.TemporaryDirectory() as temp:
   root=Path(temp);folder=root/'artifacts/validation-matrix/example';folder.mkdir(parents=True)
   (folder/'matrix.json').write_text(json.dumps({'suite':'managed','targets':[{'rid':'win-x64','status':'NeedsGraphicalSession','activation':{'argv':[str(root/'missing-python'),str(root/'missing-source')],'run_id':'example-attended'}}]}))
   with self.assertRaisesRegex(ValueError,'unavailable'):attend_validation.latest_plan(root,'win-x64','managed')
 def test_display_variable_alone_cannot_qualify_remote_linux_session(self):
  with patch.object(gui_session.sys,'platform','linux'),patch.object(gui_session.os,'getuid',return_value=1000,create=True),patch.dict(gui_session.os.environ,{'DISPLAY':':0'},clear=True),patch.object(gui_session.subprocess,'check_output',return_value='Active=yes\nRemote=yes\nType=tty\nUser=1000\n'):
   self.assertFalse(gui_session.inspect_session()['ready_for_consent'])
 def test_locked_linux_session_refused(self):
  with patch.object(gui_session.sys,'platform','linux'),patch.object(gui_session.os,'getuid',return_value=1000,create=True),patch.dict(gui_session.os.environ,{'DISPLAY':':0'},clear=True),patch.object(gui_session.subprocess,'check_output',return_value='Active=yes\nRemote=no\nType=wayland\nUser=1000\nLockedHint=yes\n'):
   self.assertFalse(gui_session.inspect_session()['ready_for_consent'])
 def test_local_linux_session_is_only_ready_for_consent(self):
  with patch.object(gui_session.sys,'platform','linux'),patch.object(gui_session.os,'getuid',return_value=1000,create=True),patch.dict(gui_session.os.environ,{'DISPLAY':':0'},clear=True),patch.object(gui_session.subprocess,'check_output',return_value='Active=yes\nRemote=no\nType=wayland\nUser=1000\nLockedHint=no\n'):
   result=gui_session.inspect_session();self.assertTrue(result['ready_for_consent']);self.assertNotIn('passed',result.values())
 def test_local_user_manager_resolves_logind_primary_graphical_session(self):
  responses=['Active=yes\nRemote=no\nType=unspecified\nClass=manager\nUser=1000\n','Display=2\n','Active=yes\nRemote=no\nType=wayland\nUser=1000\nLockedHint=no\n']
  with patch.object(gui_session.sys,'platform','linux'),patch.object(gui_session.os,'getuid',return_value=1000,create=True),patch.dict(gui_session.os.environ,{'DISPLAY':':0'},clear=True),patch.object(gui_session.subprocess,'check_output',side_effect=responses):
   result=gui_session.inspect_session();self.assertTrue(result['ready_for_consent']);self.assertEqual(result['selected_graphical_session'],'2')
 def test_ssh_cannot_use_primary_display_fallback(self):
  with patch.object(gui_session.sys,'platform','linux'),patch.object(gui_session.os,'getuid',return_value=1000,create=True),patch.dict(gui_session.os.environ,{'DISPLAY':':0','SSH_CONNECTION':'synthetic'},clear=True),patch.object(gui_session.subprocess,'check_output',return_value='Active=yes\nRemote=no\nType=unspecified\nClass=manager\nUser=1000\n') as query:
   self.assertFalse(gui_session.inspect_session()['ready_for_consent']);self.assertEqual(query.call_count,1)
 def test_activation_reenters_worker_with_fresh_run_and_live_consent_flag(self):
  args=type('Args',(),dict(run_id='example',worker='osx-x64',suite='aot',allow_emulation=True,dotnet_root=None))()
  plan=runner.activation_plan(args,ROOT)
  self.assertEqual(plan['run_id'],'example-attended');self.assertIn('--gui',plan['argv']);self.assertIn('--allow-emulation',plan['argv'])
  self.assertNotIn('PIVOT_DESKTOP_PERMIT',' '.join(plan['argv']))
 def test_missing_attended_report_does_not_become_pass(self):
  original={'suite':'managed','targets':[{'rid':'osx-arm64','status':'NeedsGraphicalSession','remote_source_root':'/missing','activation':{'run_id':'example-attended'}}]}
  config={'osx-arm64':{'transport':'local'}}
  with tempfile.TemporaryDirectory() as temp:
   result=runner.collect_attended(original,config,Path(temp))[0]
  self.assertEqual(result['status'],'NeedsGraphicalSession')
 def test_mismatched_attended_report_is_not_accepted(self):
  with tempfile.TemporaryDirectory() as temp:
   root=Path(temp);remote=root/'source';report=remote/'artifacts/validation-runs/example-attended/osx-arm64';report.mkdir(parents=True)
   (report/'result.json').write_text(json.dumps({'rid':'osx-x64','suite':'managed','run_id':'example-attended','status':'Passed'}))
   out=root/'out';out.mkdir()
   original={'suite':'managed','targets':[{'rid':'osx-arm64','status':'NeedsGraphicalSession','remote_source_root':str(remote),'activation':{'run_id':'example-attended'}}]}
   result=runner.collect_attended(original,{'osx-arm64':{'transport':'local'}},out)[0]
   self.assertEqual(result['status'],'ExportFailed')
 def test_matching_completed_report_replaces_pending_error(self):
  with tempfile.TemporaryDirectory() as temp:
   root=Path(temp);remote=root/'source';report=remote/'artifacts/validation-runs/example-attended/osx-arm64';report.mkdir(parents=True)
   (report/'result.json').write_text(json.dumps({'rid':'osx-arm64','actual_rid':'osx-arm64','suite':'managed','run_id':'example-attended','status':'Passed','application':{'zero_charges':4}}))
   ui=report/'desktop-ui';ui.mkdir()
   for i,name in enumerate(['results.json','results.clipboard-save.json','results.discard.json','results.clean.json']):
    (ui/name).write_text(json.dumps(dict(process_id=100+i,status='passed',execution_mode='managed',remaining_charge=0,graphical_backend='test backend',process_architecture='arm64')))
   out=root/'out';out.mkdir()
   original={'suite':'managed','targets':[{'rid':'osx-arm64','status':'NeedsGraphicalSession','error':'old session missing','remote_source_root':str(remote),'activation':{'run_id':'example-attended'}}]}
   result=runner.collect_attended(original,{'osx-arm64':{'transport':'local'}},out)[0]
   self.assertEqual(result['status'],'Passed');self.assertNotIn('error',result);self.assertNotIn('activation',result)
 def test_gui_process_architecture_must_match_target(self):
  with tempfile.TemporaryDirectory() as temp:
   output=Path(temp);folder=output/'desktop-ui';folder.mkdir()
   for i,name in enumerate(['results.json','results.clipboard-save.json','results.discard.json','results.clean.json']):
    (folder/name).write_text(json.dumps(dict(process_id=100+i,status='passed',execution_mode='managed',remaining_charge=0,graphical_backend='test backend',process_architecture='x64')))
   with self.assertRaisesRegex(ValueError,'architecture'):runner.check_gui(output,'managed',None,'arm64')
 @unittest.skipIf(sys.platform=='win32','POSIX host flock; also run on the Mac')
 def test_clipboard_override_cannot_bypass_machine_gui_lock(self):
  with tempfile.TemporaryDirectory() as temp:
   root=Path(temp);original=desktop_test_lock.machine_desktop_lock
   def fixed_lock(timeout=300):return original(root/'machine.lock',timeout)
   with patch.object(desktop_test_lock.sys,'platform','darwin'),patch.object(desktop_test_lock,'machine_desktop_lock',side_effect=fixed_lock):
    with patch.dict(desktop_test_lock.os.environ,{'PIVOT_CLIPBOARD_LOCK':str(root/'first.lock')}):
     with desktop_test_lock.gui_suite_lock(timeout=0):
      with patch.dict(desktop_test_lock.os.environ,{'PIVOT_CLIPBOARD_LOCK':str(root/'second.lock')}):
       with self.assertRaises(Blocked):
        with desktop_test_lock.gui_suite_lock(timeout=0):self.fail('Concurrent GUI suite')
      self.assertFalse((root/'second.lock').exists())
 @unittest.skipIf(sys.platform=='win32','POSIX host flock; also run on the Mac')
 def test_independent_processes_share_machine_lock_and_release_on_exit(self):
  with tempfile.TemporaryDirectory() as temp:
   path=Path(temp)/'host.lock'
   code='import sys;sys.path.insert(0,sys.argv[1]);from desktop_test_lock import machine_desktop_lock\nwith machine_desktop_lock(sys.argv[2],timeout=0):\n print("owned",flush=True);sys.stdin.readline()'
   child=subprocess.Popen([sys.executable,'-c',code,str(ROOT/'scripts'),str(path)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
   try:
    self.assertEqual(child.stdout.readline().strip(),'owned')
    with self.assertRaises(Blocked):
     with desktop_test_lock.machine_desktop_lock(path,timeout=0):self.fail('Concurrent desktop entry')
    inode=path.stat().st_ino
    child.kill();child.wait(timeout=5)
    with desktop_test_lock.machine_desktop_lock(path,timeout=0):self.assertEqual(path.stat().st_ino,inode)
   finally:
    if child.poll() is None:child.kill();child.wait()
    child.stdin.close();child.stdout.close()

if __name__=='__main__':unittest.main()
