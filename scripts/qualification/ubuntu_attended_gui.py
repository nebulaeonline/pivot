"""Disabled Ubuntu attended managed-GUI contract prototype; no activation CLI."""
from pathlib import Path
import runpy
import sys

POSITIVE_RUN_APPROVED=False
ENTRY='tests/application-session/ui_smoke.py'

def properties(query):
 if query.get('exit_code')!=0:raise ValueError('Required logind query unavailable')
 return dict(line.split('=',1) for line in query['stdout'].splitlines() if '=' in line)

def session_contract(data):
 """Correlate own display session and active seat, never infer from env alone."""
 if data.get('hostname')!='Pivot-Ubuntu-x64' or data.get('uid')!=1000:raise ValueError('Wrong guest/user')
 if data.get('user',{}).get('stdout','').strip()!='robhorvath':raise ValueError('Wrong user name')
 if not data.get('platform','').startswith('Linux-') or 'x86_64' not in data['platform']:raise ValueError('Native Linux x64 required')
 if data.get('hypervisor',{}).get('stdout','').strip()!='microsoft':raise ValueError('Expected Hyper-V evidence')
 if not data.get('boot_unchanged') or 'value' not in data.get('boot_before',{}):raise ValueError('Boot evidence unavailable/changed')
 if data['boot_before']!=data.get('boot_after'):raise ValueError('Boot changed')
 logind=data['logind'];user=properties(logind['queries']['user']);sid=user.get('Display')
 if not sid or sid not in user.get('Sessions','').split():raise ValueError('User display session unavailable')
 session=properties(logind['own_session_queries'][sid])
 expected={'Id':sid,'User':'1000','Name':'robhorvath','Remote':'no','Type':'wayland','Class':'user','Active':'yes','State':'active'}
 if any(session.get(k)!=v for k,v in expected.items()):raise ValueError('Active local Wayland user session not proven')
 seat=session.get('Seat')
 if not seat:raise ValueError('Seat unknown')
 state=properties(data['seats'][seat])
 if state.get('ActiveSession')!=sid or state.get('CanGraphical')!='yes':raise ValueError('Seat/session mismatch')
 env=data['environment']
 if env.get('XDG_RUNTIME_DIR')!='/run/user/1000' or env.get('WAYLAND_DISPLAY')!='wayland-0' or env.get('DISPLAY')!=':0':
  raise ValueError('Reviewed display endpoints changed')
 endpoints={r['path']:r for r in data['display_sockets']}
 for path in ['/run/user/1000/wayland-0','/tmp/.X11-unix/X0']:
  row=endpoints.get(path,{})
  if not row.get('socket') or row.get('uid')!=1000 or not row.get('inode'):raise ValueError('Display socket unavailable/wrong owner')
 return {'status':'CorrelatedForReview','session':sid,'seat':seat,'boot':data['boot_before']['value'],
         'terminal_self_session':properties(logind['queries']['self']).get('Id') if logind['queries']['self'].get('exit_code')==0 else None,
         'activation_approved':False}

def invoke_fixed(source,output=None,env=None):
 if not POSITIVE_RUN_APPROVED:raise ValueError('Ubuntu positive managed GUI disabled pending review')
 # Future reviewed activation must supply all live preconditions before this call.
 import os
 previous=(sys.argv[:],sys.path[:],Path.cwd(),dict(os.environ))
 try:
  os.chdir(source)
  if env is not None:os.environ.clear();os.environ.update(env)
  sys.argv=[ENTRY]+(['--output-dir',str(output)] if output is not None else [])
  sys.path.insert(0,str(source/'scripts'))
  return runpy.run_path(str(source/ENTRY),run_name='__main__')
 finally:
  sys.argv[:]=previous[0];sys.path[:]=previous[1];os.chdir(previous[2])
  os.environ.clear();os.environ.update(previous[3])

def main():
 if '--run-managed' in sys.argv and not POSITIVE_RUN_APPROVED:
  print('Blocked: Ubuntu positive managed GUI disabled pending review.',file=sys.stderr);return 2
 if len(sys.argv)==1:
  print('Blocked: fixed packet preflight arguments required.',file=sys.stderr);return 2
 from ubuntu_gui_runtime import main as runtime_main
 return runtime_main()

if __name__=='__main__':raise SystemExit(main())
