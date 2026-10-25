"""Pure collector classification checks; no Linux/GUI queries executed."""
import importlib.util
from pathlib import Path
import unittest

path=Path(__file__).resolve().parents[2]/'scripts/qualification/ubuntu_gui_preparation.py'
spec=importlib.util.spec_from_file_location('ubuntu_preparation',path)
collector=importlib.util.module_from_spec(spec);spec.loader.exec_module(collector)

class SessionTests(unittest.TestCase):
 def test_failed_query_cannot_supply_session_properties(self):
  self.assertEqual(collector.properties({'exit_code':1,'stdout':'Active=yes\nUser=1000'}),{})
 def test_only_own_active_local_user_graphical_sessions_are_candidates(self):
  baseline={'User':'1000','Remote':'no','Type':'wayland','Active':'yes','State':'active','Class':'user'}
  self.assertEqual(collector.own_graphical([baseline],1000),[baseline])
  for key,value in [('User','1001'),('Remote','yes'),('Type','tty'),('Active','no'),('State','closing'),('Class','greeter')]:
   with self.subTest(key=key):self.assertEqual(collector.own_graphical([dict(baseline,**{key:value})],1000),[])
 def test_missing_fields_do_not_imply_graphical_identity(self):
  self.assertEqual(collector.own_graphical([{'User':'1000','Type':'wayland'}],1000),[])

if __name__=='__main__':unittest.main()
