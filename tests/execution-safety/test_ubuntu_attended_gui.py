"""Disabled Linux contract checks; never connect to a display or execute GUI."""
import copy
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('ubuntu_contract',ROOT/'scripts/qualification/ubuntu_attended_gui.py')
worker=importlib.util.module_from_spec(spec);spec.loader.exec_module(worker)
def q(**values):return {'exit_code':0,'stdout':'\n'.join(k+'='+v for k,v in values.items())}
def fixture():
 return {'hostname':'Pivot-Ubuntu-x64','uid':1000,'user':{'stdout':'robhorvath\n'},'platform':'Linux-test-x86_64',
  'hypervisor':{'stdout':'microsoft\n'},'boot_unchanged':True,'boot_before':{'value':'boot'},'boot_after':{'value':'boot'},
  'logind':{'queries':{'self':{'exit_code':1},'user':q(Display='2',Sessions='3 2')},
   'own_session_queries':{'2':q(Id='2',User='1000',Name='robhorvath',Remote='no',Type='wayland',Class='user',Active='yes',State='active',Seat='seat0')}},
  'seats':{'seat0':q(ActiveSession='2',CanGraphical='yes')},
  'environment':{'XDG_RUNTIME_DIR':'/run/user/1000','WAYLAND_DISPLAY':'wayland-0','DISPLAY':':0'},
  'display_sockets':[{'path':p,'socket':True,'uid':1000,'inode':1} for p in ['/run/user/1000/wayland-0','/tmp/.X11-unix/X0']]}
class ContractTests(unittest.TestCase):
 def test_display_seat_correlation_does_not_invent_terminal_membership(self):
  result=worker.session_contract(fixture());self.assertEqual(result['session'],'2')
  self.assertIsNone(result['terminal_self_session']);self.assertFalse(result['activation_approved'])
 def test_wrong_context_and_missing_evidence_refuse(self):
  for field,value in [('hostname','host'),('uid',0),('boot_unchanged',False),('display_sockets',[])]:
   data=fixture();data[field]=value
   with self.subTest(field=field),self.assertRaises(ValueError):worker.session_contract(data)
  data=fixture();data['seats']['seat0']=q(ActiveSession='3',CanGraphical='yes')
  with self.assertRaises(ValueError):worker.session_contract(data)
 def test_disabled_entry_never_calls_original_runner(self):
  with patch.object(worker.runpy,'run_path') as runner:
   with self.assertRaises(ValueError):worker.invoke_fixed(ROOT)
   self.assertEqual(worker.main(),2);runner.assert_not_called()

if __name__=='__main__':unittest.main()
