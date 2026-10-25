"""Read-only SSH/network setup report. No sudo, firewall edits or key installation."""
import datetime
import json
import os
from pathlib import Path
import platform
import shutil
import socket
import subprocess
import uuid

def query(argv):
 try:
  r=subprocess.run(argv,stdin=subprocess.DEVNULL,capture_output=True,text=True,timeout=20)
  return {'command':argv,'exit_code':r.returncode,'stdout':r.stdout,'stderr':r.stderr}
 except (OSError,subprocess.SubprocessError) as e:return {'command':argv,'error':str(e)}
def main():
 root=Path(__file__).resolve().parents[2]
 name=socket.gethostname()+'-'+uuid.uuid4().hex[:8]
 report=Path.home()/'PivotValidation/setup-reports'/name;report.mkdir(parents=True)
 data={'hostname':socket.gethostname(),'user':os.environ.get('USER') or os.environ.get('USERNAME'),
  'home':str(Path.home()),'source_root':str(root),'platform':platform.platform(),'machine':platform.machine(),
  'changes_made':False,'authorized_keys_exists':(Path.home()/'.ssh/authorized_keys').is_file()}
 if platform.system()=='Linux':
  commands={'addresses':['ip','-j','address'],'listeners':['ss','-ltn','sport = :22'],
   'service':['systemctl','status','ssh.service','ssh.socket','--no-pager','-l'],
   'firewall':['ufw','status'],'effective_config':['/usr/sbin/sshd','-T']}
 elif platform.system()=='Darwin':
  commands={'addresses':['ifconfig'],'service':['launchctl','print','system/com.openssh.sshd'],
   'remote_login':['systemsetup','-getremotelogin']}
 else:commands={}
 data['queries']={k:query(v) for k,v in commands.items()}
 data['public_host_fingerprint']=query(['ssh-keygen','-lf','/etc/ssh/ssh_host_ed25519_key.pub']) if os.name!='nt' else None
 output=report/'endpoint.json';output.write_text(json.dumps(data,indent=2)+'\n')
 destination=root/'artifacts/validation-endpoints'/name
 try:destination.parent.mkdir(parents=True,exist_ok=True);shutil.copytree(report,destination)
 except OSError as e:print('Local original: '+str(report)+'; export failed: '+str(e));return 1
 print('Read-only SSH report returned: '+str(destination));return 0
if __name__=='__main__':raise SystemExit(main())
