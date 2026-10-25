"""Focused /proc/cwd correction regressions; no Avalonia or application build."""
import copy
import os
from pathlib import Path
import select
import sys
import tempfile
import unittest
from unittest.mock import patch,Mock
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts/qualification'))
import ubuntu_gui_runtime as runtime

class CorrectionTests(unittest.TestCase):
 def target(self,base):
  source=base/'source';dll=source/'clients/dotnet/Pivot.Desktop/bin/Release/net10.0/Pivot.Desktop.dll'
  dll.parent.mkdir(parents=True);dll.write_bytes(b'fixture, never executed');return source,dll
 def test_relative_and_absolute_fixed_DLL_with_target_cwd(self):
  with tempfile.TemporaryDirectory() as temp,patch.object(runtime,'local_tree',side_effect=lambda p:p):
   source,dll=self.target(Path(temp))
   self.assertEqual(runtime.canonical_managed(['dotnet',str(dll.relative_to(source))],source,source),str(dll.resolve()))
   self.assertEqual(runtime.canonical_managed(['/usr/bin/dotnet',str(dll)],source,source),str(dll.resolve()))
 def test_outside_missing_and_wrong_cwd_refuse(self):
  with tempfile.TemporaryDirectory() as temp,patch.object(runtime,'local_tree',side_effect=lambda p:p):
   base=Path(temp);source,dll=self.target(base);outside=base/'Pivot.Desktop.dll';outside.write_bytes(b'outside')
   for argv,cwd in [(['dotnet','../Pivot.Desktop.dll'],source),(['dotnet','missing.dll'],source),
                    (['dotnet',str(dll.relative_to(source))],base),(['dotnet',str(dll)],base/'absent')]:
    with self.subTest(argv=argv,cwd=cwd),self.assertRaises((OSError,ValueError)):
     runtime.canonical_managed(argv,cwd,source)
 def test_symlink_boundaries_refuse(self):
  if sys.platform!='linux':self.skipTest('Linux-native symlink check')
  with tempfile.TemporaryDirectory() as temp:
   base=Path(temp);source,dll=self.target(base);alias=base/'alias';alias.symlink_to(source,target_is_directory=True)
   with self.assertRaises(ValueError):runtime.canonical_managed(['dotnet',str(dll)],alias,source)
 def row(self):
  return {'pid':123,'uid':1000,'start_ticks':'456','ppid':50,'executable':'/usr/bin/dotnet',
   'command':'dotnet fixed.dll','cgroup':'/user-1000.slice/app.scope'}
 def capture(self,managed=True,failure=None,exited=False,changed=None,changed_cwd=False):
  with tempfile.TemporaryDirectory() as temp:
   folder=Path(temp)/'123';folder.mkdir();(folder/'environ').write_bytes(b'DISPLAY=:0\0')
   row=self.row();after=copy.deepcopy(row)
   if changed:after.update(changed)
   original=Path.read_bytes
   def read_bytes(p):
    if p.name=='environ' and failure:raise failure
    return original(p)
   with patch.object(runtime,'proc_identity',side_effect=[row,copy.deepcopy(row),after]), \
        patch.object(os,'pidfd_open',return_value=99,create=True),patch.object(os,'close'), \
        patch.object(runtime,'process_arguments',return_value=['dotnet','fixed.dll']), \
        patch.object(runtime,'cwd_identity',side_effect=[{'path':temp,'device':1,'inode':2},{'path':temp,'device':1,'inode':3 if changed_cwd else 2}]), \
        patch.object(runtime,'canonical_managed',return_value='/private/source/fixed.dll'), \
        patch.object(runtime,'process_ancestry',return_value=[{'pid':os.getpid(),'uid':1000,'start_ticks':'1'}]), \
        patch.object(Path,'read_bytes',read_bytes),patch.object(runtime,'exited_original',return_value=exited):
    return runtime.capture_scoped(folder,Path(temp),managed)
 def test_build_actor_does_not_read_unneeded_environment(self):
  row,event=self.capture(managed=False,failure=PermissionError('compiler environ denied'))
  self.assertIsNone(event);self.assertIn('environment not read',row['display_evidence'])
 def test_live_managed_environment_denial_blocks(self):
  with self.assertRaises(PermissionError):self.capture(failure=PermissionError('live denied'),exited=False)
 def test_pinned_exit_event_is_not_runtime_identity(self):
  row,event=self.capture(failure=ProcessLookupError('exited'),exited=True)
  self.assertIsNone(row);self.assertTrue(event['pidfd_exit']);self.assertFalse(event['substitute_for_runtime_identity'])
 def test_reuse_exec_and_context_changes_block(self):
  for changed in [{'start_ticks':'999'},{'executable':'/other'},{'command':'other'},{'cgroup':'other'}]:
   with self.subTest(changed=changed),self.assertRaises(ValueError):self.capture(changed=changed)
 def test_changed_cwd_metadata_blocks(self):
  with self.assertRaisesRegex(ValueError,'cwd changed'):self.capture(changed_cwd=True)
 def test_pidfd_exit_requires_fresh_matching_or_absent_process(self):
  with tempfile.TemporaryDirectory() as temp:
   folder=Path(temp)/'123';folder.mkdir();fields=['Z']+['0']*18+['456'];(folder/'stat').write_text('123 (actor) '+' '.join(fields))
   poll=Mock();poll.poll.return_value=[(99,1)]
   with patch.object(select,'poll',return_value=poll,create=True),patch.object(select,'POLLIN',1,create=True):
    self.assertTrue(runtime.exited_original(folder,self.row(),99))
    fields[19]='999';(folder/'stat').write_text('123 (actor) '+' '.join(fields))
    with self.assertRaises(ValueError):runtime.exited_original(folder,self.row(),99)
    poll.poll.return_value=[];self.assertFalse(runtime.exited_original(folder,self.row(),99))
 def test_unknown_pidfd_identity_does_not_pass(self):
  with tempfile.TemporaryDirectory() as temp,patch.object(runtime,'proc_identity',return_value=self.row()), \
       patch.object(os,'pidfd_open',side_effect=PermissionError('no pinned identity'),create=True):
   with self.assertRaises(PermissionError):runtime.capture_scoped(Path(temp)/'123',Path(temp),False)
 def test_linux_native_readonly_pidfd_and_process_adapters(self):
  if sys.platform!='linux':self.skipTest('Linux-native proc/pidfd adapter')
  before=runtime.proc_identity(os.getpid());fd=os.pidfd_open(os.getpid(),0)
  try:
   runtime.stable_identity(before,runtime.proc_identity(os.getpid()))
   poll=select.poll();poll.register(fd,select.POLLIN);self.assertEqual(poll.poll(0),[])
   cwd=runtime.cwd_identity(Path('/proc')/str(os.getpid()))
   self.assertEqual(cwd['path'],str(Path.cwd().resolve()));self.assertTrue(cwd['inode'])
  finally:os.close(fd)

if __name__=='__main__':unittest.main()
