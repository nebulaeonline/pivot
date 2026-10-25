"""Linux wrapper refusal/integrity tests; no actual GUI or builds."""
import contextlib
import copy
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'scripts/qualification'),str(ROOT/'tests/execution-safety')]
import ubuntu_gui_runtime as runtime
import launch_ubuntu_gui as launcher
from test_ubuntu_attended_gui import fixture

def context():
 data=fixture();data['caller_cgroup']={'value':'0::/user.slice/user-1000.slice/user@1000.service/app.slice/terminal.scope'}
 for row in data['display_sockets']:row['device']=87
 return data

class RuntimeTests(unittest.TestCase):
 def test_context_allows_terminal_outside_logind(self):
  self.assertEqual(runtime.compare_context(context(),context())['status'],'Verified')
 def test_changed_or_missing_required_context_blocks(self):
  for key in ['inode','device','uid','socket']:
   data=context();del data['display_sockets'][0][key]
   with self.subTest(key=key),self.assertRaises((ValueError,KeyError)):runtime.compare_context(context(),data)
  for key,value in [('boot_before',{'value':'other'}),('uid',99),('hostname','other')]:
   data=context();data[key]=value
   with self.subTest(key=key),self.assertRaises(ValueError):runtime.compare_context(context(),data)
  data=context();data['seats']['seat0']['stdout']='ActiveSession=3\nCanGraphical=yes'
  with self.assertRaises(ValueError):runtime.compare_context(context(),data)
 def test_ambiguous_live_sessions_block(self):
  with patch.object(runtime.sys,'platform','linux'),patch.object(runtime.platform,'machine',return_value='x86_64'), \
       patch.object(runtime.os,'getuid',return_value=1000,create=True),patch.object(runtime,'read',return_value={'value':'boot'}), \
       patch.object(runtime,'query',return_value={'exit_code':0,'stdout':'Sessions=2 4'}), \
       patch.object(runtime,'own_graphical',return_value=[{},{}]):
   with self.assertRaisesRegex(ValueError,'Exactly one'):runtime.live_context()
 def test_invalid_native_storage(self):
  with patch.object(runtime,'filesystem',return_value={'native_work_root':False,'type':'cifs'}):
   with self.assertRaises(ValueError):runtime.local_tree(ROOT)
 def test_missing_tool_refuses(self):
  with patch.object(runtime.shutil,'which',return_value=None):
   with self.assertRaisesRegex(ValueError,'Missing tool'):runtime.tools(ROOT,{})
 def test_positive_entry_guard_before_adapters(self):
  with patch.object(runtime.contract,'POSITIVE_RUN_APPROVED',False),patch.object(runtime,'live_context') as adapter:
   with self.assertRaises(ValueError):runtime.execute(ROOT,ROOT,{},context(),[])
   adapter.assert_not_called()
 def test_recorder_positive_guard_and_report_path_validation(self):
  args=['launcher','--packet-dir','.','--receipt-sha256','fixture','--run-managed']
  with patch.object(sys,'argv',args),patch.object(launcher.subprocess,'call') as child,contextlib.redirect_stderr(io.StringIO()):
   self.assertEqual(launcher.main(),2);child.assert_not_called()
  with patch.object(runtime,'local_tree',side_effect=lambda p:p):
   with self.assertRaises(ValueError):launcher.native_report('Native report: /elsewhere/report',Path('/native'))
   with self.assertRaises(ValueError):launcher.native_report('Native report: /native/a/report\nNative report: /native/b/report',Path('/native'))
   self.assertEqual(launcher.native_report('CONFIRM: Native report: /native/a/report',Path('/native')),Path('/native/a/report'))
 def test_noninteractive_preflight_refuses_before_adapters(self):
  args=['wrapper','--packet-dir','.','--receipt-sha256','fixture']
  with patch.object(sys,'argv',args),patch.object(sys.stdin,'isatty',return_value=False), \
       patch.object(runtime,'live_context') as adapter,contextlib.redirect_stderr(io.StringIO()):
   self.assertEqual(runtime.main(),2);adapter.assert_not_called()
 def test_failed_outcome_never_becomes_blocked(self):
  self.assertEqual(runtime.classify(1,['observer failed']),'Failed')
  self.assertEqual(runtime.classify(0,['survivor']),'Blocked')
  self.assertEqual(runtime.classify(2,[]),'Blocked');self.assertEqual(runtime.classify(0,[]),'Passed')
 def records(self):
  source=Path('/native/source');display={'DISPLAY':':0','WAYLAND_DISPLAY':'wayland-0','XDG_RUNTIME_DIR':'/run/user/1000'}
  reports=[{'process_id':p,'status':'passed','execution_mode':'managed','remaining_charge':0,
   'graphical_backend':'Actual.Linux.Backend','executable':'/usr/bin/dotnet'} for p in range(100,104)]
  rows=[{'pid':p,'uid':1000,'start_ticks':str(p+100),'ppid':50,'observer_pid':50,
   'ancestry':[{'pid':50,'start_ticks':'1','uid':1000}],'cgroup':'/user-1000.slice/app.scope',
   'executable':'/usr/bin/dotnet','command':str(source/'clients/dotnet/Pivot.Desktop/bin/Release/net10.0/Pivot.Desktop.dll'),
   'managed_dll':str(source/'clients/dotnet/Pivot.Desktop/bin/Release/net10.0/Pivot.Desktop.dll'),
   'display':display.copy()} for p in range(100,104)]
  rows.extend({'pid':p+1000,'uid':1000,'start_ticks':str(p+200),'ppid':p,
   'executable':str(source.parent/'target/release/pivot-session')} for p in range(100,104))
  return source,display,reports,rows
 def test_runtime_identity_and_pid_reuse(self):
  source,display,reports,rows=self.records()
  self.assertEqual(runtime.correlate(rows,reports,source,display)['zero_charge_fields'],4)
  rows.append(dict(rows[0],start_ticks='999'))
  with self.assertRaisesRegex(ValueError,'reused'):runtime.correlate(rows,reports,source,display)
 def test_missing_start_ownership_ancestry_and_display(self):
  for key,value in [('start_ticks',''),('uid',0),('ancestry',[]),('display',{}),('executable','/wrong'),('cgroup','')]:
   source,display,reports,rows=self.records();rows[0][key]=value
   with self.subTest(key=key),self.assertRaises(ValueError):runtime.correlate(rows,reports,source,display)
 def test_backend_and_charge_required(self):
  for key,value in [('graphical_backend',None),('remaining_charge',1),('execution_mode','native-aot')]:
   source,display,reports,rows=self.records();reports[0][key]=value
   with self.subTest(key=key),self.assertRaises(ValueError):runtime.correlate(rows,reports,source,display)
 def test_missing_or_mismatched_rust_child_blocks(self):
  source,display,reports,rows=self.records();rows.pop()
  with self.assertRaises(ValueError):runtime.correlate(rows,reports,source,display)
  source,display,reports,rows=self.records();reports[1]['rust_process_id']=9999
  with self.assertRaises(ValueError):runtime.correlate(rows,reports,source,display)
 def test_missing_process_start_ticks_refuses(self):
  with tempfile.TemporaryDirectory() as temp:
   proc=Path(temp);folder=proc/'123';folder.mkdir();(folder/'stat').write_text('123 (dotnet) S 1 2')
   with self.assertRaisesRegex(ValueError,'start ticks'):runtime.proc_identity(123,proc)
 def test_unreadable_preexisting_nonactor_scope_is_narrow(self):
  work=Path('/private/fresh-invocation')
  self.assertTrue(runtime.preexisting_nonactor('systemd','/usr/lib/systemd/systemd --user','10','20',work))
  for name,command,start in [('dotnet','unrelated','10'),('systemd',str(work),'10'),
                             ('systemd','unrelated','20'),('systemd','unrelated','30'),('systemd','unrelated','')]:
   with self.subTest(name=name,command=command,start=start):
    self.assertFalse(runtime.preexisting_nonactor(name,command,start,'20',work))
 def test_checker_ancestry_stops_at_wrapper_root(self):
  row={'pid':os.getpid(),'ppid':999,'uid':1000,'start_ticks':'123'}
  with patch.object(runtime,'proc_identity') as query:
   self.assertEqual(runtime.process_ancestry(row,Path('/unused')),[{'pid':os.getpid(),'uid':1000,'start_ticks':'123'}])
   query.assert_not_called()
 def test_execute_nonempty_survivors_and_observer_errors_block(self):
  for code in [0,1]:
   with tempfile.TemporaryDirectory() as temp,patch.object(runtime.contract,'POSITIVE_RUN_APPROVED',True), \
        patch.object(runtime,'live_context',return_value=context()),patch.object(runtime,'compare_context',return_value={'status':'Verified'}), \
        patch.object(runtime,'same_source',return_value=[]),patch.object(runtime,'Observer') as observer, \
        patch.object(runtime.contract,'invoke_fixed',side_effect=SystemExit(code)), \
        patch.object(runtime,'scoped_snapshot',return_value={'status':'Verified','matches':[{'pid':123}]}):
    observer.return_value.errors=[];observer.return_value.records={};observer.return_value.exits=[]
    result=runtime.execute(Path(temp),Path(temp),{},context(),[])
    self.assertEqual(result,1 if code else 2)
    self.assertEqual(runtime.read_json(Path(temp)/'run.json')['status'],'Failed' if code else 'Blocked')
 def test_export_hashes_and_transfer_failure(self):
  with tempfile.TemporaryDirectory() as temp:
   base=Path(temp);report=base/'native/report';report.mkdir(parents=True);(report/'data.json').write_text('{}')
   with contextlib.redirect_stdout(io.StringIO()):runtime.export(report,base/'returned')
   destination=base/'returned/native'
   self.assertEqual(runtime.sha(destination/'data.json'),runtime.sha(report/'data.json'))
   receipt=runtime.read_json(destination/'transfer-receipt.json')
   self.assertEqual(receipt['native_manifest_sha256'],receipt['returned_manifest_sha256'])
   with patch.object(runtime.shutil,'copytree',side_effect=OSError('share unavailable')):
    with self.assertRaises(OSError):runtime.export(report,base/'failed')
   self.assertTrue((report/'origin-hashes.json').exists())
 def test_source_tamper_detected_and_unsafe_manifest_path(self):
  with tempfile.TemporaryDirectory() as temp:
   root=Path(temp);(root/'file').write_text('first');rows=[{'path':'file','sha256':runtime.sha(root/'file')}]
   (root/'file').write_text('changed');self.assertTrue(runtime.same_source(root,rows))
   with self.assertRaises(ValueError):runtime.verify_files(root,[{'path':'../escape','sha256':'x'}])
 def test_original_piped_yes_refuses_before_output(self):
  with tempfile.TemporaryDirectory() as temp:
   output=Path(temp)/'absent';env={k:v for k,v in os.environ.items() if not k.upper().startswith('PIVOT_')}
   run=subprocess.run([sys.executable,str(ROOT/runtime.contract.ENTRY),'--output-dir',str(output)],cwd=ROOT,
    env=env,input='yes\n',text=True,capture_output=True,timeout=30)
   self.assertEqual(run.returncode,2,run.stdout+run.stderr);self.assertFalse(output.exists())
 def test_fixed_invocation_preserves_streams_and_restores_state(self):
  with tempfile.TemporaryDirectory() as temp:
   source=Path(temp);before=(Path.cwd(),dict(os.environ),sys.argv[:],sys.path[:],sys.stdin,sys.stdout)
   def fake(path,run_name):
    self.assertEqual(path,str(source/runtime.contract.ENTRY));self.assertIs(sys.stdin,before[4]);self.assertIs(sys.stdout,before[5])
    self.assertEqual(dict(os.environ),{'DISPLAY':':0'});raise SystemExit(2)
   with patch.object(runtime.contract,'POSITIVE_RUN_APPROVED',True),patch.object(runtime.contract.runpy,'run_path',side_effect=fake):
    with self.assertRaises(SystemExit):runtime.contract.invoke_fixed(source,source/'report',{'DISPLAY':':0'})
   self.assertEqual((Path.cwd(),dict(os.environ),sys.argv[:],sys.path[:]),before[:4])

if __name__=='__main__':unittest.main()
