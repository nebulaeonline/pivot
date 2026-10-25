"""Early-pinning lifecycle races; synthetic branches and native non-GUI fixture."""
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts/qualification'))
import ubuntu_gui_runtime as runtime

class LifecycleTests(unittest.TestCase):
 def observe(self,fail=None,exited=False,tokens=None,pin_error=None,corroboration_error=None):
  token={'pid':123,'uid':1000,'start_ticks':'200'}
  token_values=tokens or [token.copy(),token.copy(),token.copy()]
  command='dotnet clients/Pivot.Desktop.dll '
  row={'pid':123,'uid':1000,'start_ticks':'200','executable':str(Path('/usr/bin/dotnet')),'command':command}
  def capture(folder,work,managed,step=None):
   if fail and fail.startswith('capture.'):
    return step(fail,lambda:(_ for _ in ()).throw(FileNotFoundError(2,'fixture exit')))
   return dict(row),None
  original_resolve=Path.resolve
  def resolve(p,*args,**kwargs):
   if p.name=='exe':
    if fail=='selection.executable':raise FileNotFoundError(2,'fixture exited')
    return Path('/usr/bin/dotnet')
   return original_resolve(p,*args,**kwargs)
  with tempfile.TemporaryDirectory() as temp:
   folder=Path(temp)/'123';folder.mkdir();(folder/'comm').write_text('dotnet');(folder/'cmdline').write_bytes(command.replace(' ','\0').encode())
   with patch.object(runtime,'process_token',side_effect=token_values),patch.object(runtime.os,'getuid',return_value=1000,create=True), \
        patch.object(os,'pidfd_open',side_effect=pin_error,return_value=77,create=True),patch.object(os,'close'), \
        patch.object(Path,'resolve',resolve),patch.object(runtime,'process_arguments',side_effect=PermissionError(13,'live argv denied') if fail=='selection.argv' else None,return_value=['dotnet','clients/Pivot.Desktop.dll']), \
        patch.object(runtime,'cwd_identity',side_effect=FileNotFoundError(2,'cwd gone') if fail=='selection.cwd' else None,return_value={'path':'/private/source','device':1,'inode':2}), \
        patch.object(runtime,'canonical_managed',side_effect=ValueError('path escaped') if fail=='selection.canonical_dll' else None,return_value='/private/source/clients/Pivot.Desktop.dll'), \
        patch.object(runtime,'capture_scoped',side_effect=capture), \
        patch.object(runtime,'exited_original',side_effect=corroboration_error,return_value=exited):
    return runtime.observe_candidate(folder,Path('/private'),100)
 def test_exit_before_any_original_identity_stays_unresolved(self):
  result=self.observe(tokens=[FileNotFoundError(2,'enumerated PID vanished')],exited=True)
  self.assertEqual(result['kind'],'unresolved');self.assertFalse(result['record']['pidfd_acquired'])
  self.assertEqual(result['record']['operation'],'pin.preliminary_stat_uid')
 def test_pidfd_acquisition_ESRCH_is_not_clean_exit(self):
  result=self.observe(pin_error=ProcessLookupError(3,'no process'),exited=True)
  self.assertEqual(result['kind'],'unresolved');self.assertFalse(result['record']['original_identity_bound'])
  self.assertEqual(result['record']['errno'],3)
 def test_exit_during_initial_binding_is_not_proven(self):
  token={'pid':123,'uid':1000,'start_ticks':'200'}
  result=self.observe(tokens=[token,FileNotFoundError(2,'binding missing')],exited=True)
  self.assertEqual(result['kind'],'unresolved');self.assertTrue(result['record']['pidfd_acquired'])
  self.assertFalse(result['record']['original_identity_bound'])
 def test_PID_reuse_while_binding_blocks(self):
  a={'pid':123,'uid':1000,'start_ticks':'200'};b=dict(a,start_ticks='300')
  result=self.observe(tokens=[a,b],exited=True)
  self.assertEqual(result['kind'],'unresolved');self.assertIn('changed',result['record']['error'])
 def test_selection_failures_inside_bound_exit_window(self):
  for op in ['selection.executable','selection.cwd']:
   with self.subTest(op=op):
    result=self.observe(fail=op,exited=True)
    self.assertEqual(result['kind'],'exit');self.assertTrue(result['record']['exit_corroborated'])
    self.assertEqual(result['record']['operation'],op);self.assertFalse(result['record']['substitute_for_runtime_identity'])
 def test_initial_and_post_capture_reads_inside_outer_lifecycle(self):
  for op in ['capture.initial.identity.executable','capture.initial_recheck.identity.stat','capture.post.identity.cwd']:
   with self.subTest(op=op):
    result=self.observe(fail=op,exited=True)
    self.assertEqual(result['kind'],'exit');self.assertEqual(result['record']['operation'],op)
 def test_live_unreadability_and_path_errors_stay_blocked(self):
  for op in ['selection.argv','selection.canonical_dll']:
   with self.subTest(op=op):
    result=self.observe(fail=op,exited=False)
    self.assertEqual(result['kind'],'unresolved');self.assertEqual(result['record']['operation'],op)
  self.assertEqual(self.observe(fail='selection.canonical_dll',exited=True)['kind'],'unresolved')
 def test_reuse_during_exit_corroboration_stays_unresolved(self):
  result=self.observe(fail='selection.cwd',corroboration_error=ValueError('PID reused'))
  self.assertEqual(result['kind'],'unresolved');self.assertIn('PID reused',result['record']['corroboration_error']['error'])
 def test_full_observation_keeps_binding_diagnostics(self):
  result=self.observe();self.assertEqual(result['kind'],'observation')
  self.assertTrue(result['record']['observation_lifecycle']['original_identity_bound'])
 def test_unavailable_pidfd_support_is_unresolved(self):
  with patch.object(os,'pidfd_open',None,create=True),patch.object(runtime.os,'getuid',return_value=1000,create=True), \
       patch.object(runtime,'process_token',return_value={'pid':123,'uid':1000,'start_ticks':'200'}):
   result=runtime.observe_candidate(Path('/proc/123'),Path('/private'),100)
   self.assertEqual(result['kind'],'unresolved');self.assertFalse(result['record']['pidfd_acquired'])
 def test_native_child_pidfd_lifetime_without_GUI_or_build(self):
  if sys.platform!='linux' or os.getuid()!=1000:self.skipTest('Native Linux UID1000 process fixture')
  import subprocess,select
  child=subprocess.Popen([sys.executable,'-c','import sys; print("ready",flush=True); sys.stdin.read()'],
   stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
  fd=None
  try:
   self.assertEqual(child.stdout.readline().strip(),'ready')
   folder=Path('/proc')/str(child.pid);before=runtime.process_token(folder)
   fd=os.pidfd_open(child.pid,0);self.assertEqual(runtime.process_token(folder),before)
   poll=select.poll();poll.register(fd,select.POLLIN);self.assertEqual(poll.poll(0),[])
   child.stdin.close();child.wait(timeout=5)
   self.assertTrue(runtime.exited_original(folder,before,fd))
  finally:
   if fd is not None:os.close(fd)
   if child.poll() is None:child.stdin.close();child.wait(timeout=5)
   child.stdout.close()

if __name__=='__main__':unittest.main()
