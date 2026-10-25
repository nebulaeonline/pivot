"""Read-only Ubuntu graphical-session evidence. Never launch GUI, build or install."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import socket
import stat
import subprocess
import sys
import uuid

PROPERTIES=['Id','User','Name','Type','Class','Remote','State','Active','Seat','Display','Leader','TTY','Desktop','Service']
ENVIRONMENT=['XDG_SESSION_ID','XDG_SESSION_TYPE','XDG_CURRENT_DESKTOP','XDG_SESSION_DESKTOP',
 'XDG_RUNTIME_DIR','DISPLAY','WAYLAND_DISPLAY','XDG_SEAT','XDG_VTNR','SSH_TTY']

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def query(command):
 try:
  run=subprocess.run(command,stdin=subprocess.DEVNULL,capture_output=True,text=True,timeout=20)
  return {'command':command,'exit_code':run.returncode,'stdout':run.stdout,'stderr':run.stderr,
          'status':'Observed' if run.returncode==0 else 'Unavailable'}
 except (OSError,subprocess.SubprocessError) as error:
  return {'command':command,'status':'Unavailable','error':str(error)}
def properties(result):
 if result.get('exit_code')!=0:return {}
 return dict(line.split('=',1) for line in result['stdout'].splitlines() if '=' in line)
def own_graphical(sessions,uid):
 return [s for s in sessions if s.get('User')==str(uid) and s.get('Remote')=='no'
         and s.get('Type') in ['wayland','x11'] and s.get('Active')=='yes'
         and s.get('State')=='active' and s.get('Class')=='user']
def filesystem(path):
 result=query(['findmnt','--json','--target',str(path),'--output','TARGET,SOURCE,FSTYPE'])
 if result.get('exit_code')!=0:raise ValueError('Native filesystem unavailable: '+json.dumps(result))
 row=json.loads(result['stdout'])['filesystems'][0]
 if row['fstype'] not in ['ext4','xfs','btrfs'] or not row['source'].startswith('/dev/'):
  raise ValueError('Native block-backed Linux filesystem required: '+json.dumps(row))
 return row
def read(path):
 try:return {'value':Path(path).read_text().strip()}
 except OSError as error:return {'error':str(error)}
def socket_metadata(path):
 try:
  info=path.stat()
  return {'path':str(path),'socket':stat.S_ISSOCK(info.st_mode),'uid':info.st_uid,
          'device':info.st_dev,'inode':info.st_ino,'mode':oct(stat.S_IMODE(info.st_mode))}
 except OSError as error:return {'path':str(path),'error':str(error)}
def main():
 if sys.platform!='linux' or platform.machine()!='x86_64':raise ValueError('Linux x64 guest only')
 if len(sys.argv)==1:
  # Copy collector to native storage before collection; no builds or executable probes.
  home=Path.home().resolve();filesystem(home)
  work=home/'pivot-preflight'/('ubuntu-gui-preparation-'+uuid.uuid4().hex)
  work.mkdir(parents=True);filesystem(work)
  target=work/'collector.py';shutil.copy2(Path(__file__),target)
  if sha(target)!=sha(Path(__file__)):raise ValueError('Native collector copy mismatch')
  return subprocess.call([sys.executable,str(target),'--collect'],cwd=work)
 if sys.argv[1:]!=['--collect']:raise ValueError('No configurable command or GUI execution option')
 work=Path(__file__).resolve().parent;fs=filesystem(work)
 report=work/'report';report.mkdir()
 boot=read('/proc/sys/kernel/random/boot_id')
 result={'scope':'Ubuntu x64 read-only GUI preparation; not activation approval',
  'captured_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'hostname':socket.gethostname(),
  'uid':os.getuid(),'gid':os.getgid(),'user':query(['id','-un']),
  'platform':platform.platform(),'filesystem':fs,'native_work':str(work),'collector_sha256':sha(Path(__file__)),
  'boot_before':boot,'environment':{k:os.environ.get(k) for k in ENVIRONMENT},
  'os_release':read('/etc/os-release'),'hypervisor':query(['systemd-detect-virt']),
  'DMI':{n:read('/sys/class/dmi/id/'+n) for n in ['sys_vendor','product_name','product_uuid','product_serial']},
  'caller_cgroup':read('/proc/self/cgroup'),'parent_cgroup':read('/proc/'+str(os.getppid())+'/cgroup'),
  'GUI_executed':False,'build_or_publish_executed':False,'clipboard_accessed':False,
  'activation_approved':False,'G3':'held'}
 queries={'self':query(['loginctl','show-session','self']+sum([['-p',p] for p in PROPERTIES],[])),
  'user':query(['loginctl','show-user',str(os.getuid()),'-p','Display','-p','Sessions','-p','State']),
  'list':query(['loginctl','list-sessions','--no-legend','--no-pager'])}
 # Inspect properties only for sessions logind associates with this UID.
 ids=set(properties(queries['user']).get('Sessions','').split())
 for line in queries['list'].get('stdout','').splitlines():
  parts=line.split()
  if len(parts)>=2 and parts[1]==str(os.getuid()):ids.add(parts[0])
 details={sid:query(['loginctl','show-session',sid]+sum([['-p',p] for p in PROPERTIES],[])) for sid in sorted(ids)}
 result['logind']={'queries':queries,'own_session_queries':details}
 candidates=own_graphical([properties(r) for r in details.values()],os.getuid())
 result['graphical_candidates']=candidates
 result['session_correlation']='ReviewRequired' if len(candidates)==1 else 'Unknown'
 seats=sorted({s['Seat'] for s in candidates if s.get('Seat')})
 result['seats']={s:query(['loginctl','show-seat',s,'-p','Id','-p','ActiveSession','-p','Sessions','-p','CanGraphical','-p','CanTTY']) for s in seats}
 runtime=os.environ.get('XDG_RUNTIME_DIR','');wayland=os.environ.get('WAYLAND_DISPLAY','')
 result['display_sockets']=[]
 if runtime and wayland:
  p=Path(wayland) if Path(wayland).is_absolute() else Path(runtime)/wayland
  result['display_sockets'].append(socket_metadata(p))
 display=os.environ.get('DISPLAY','')
 if display.startswith(':') and display[1:].split('.')[0].isdigit():
  result['display_sockets'].append(socket_metadata(Path('/tmp/.X11-unix/X'+display[1:].split('.')[0])))
 result['own_desktop_processes']=query(['ps','-u',str(os.getuid()),'-o','pid=,ppid=,sid=,tty=,comm='])
 # Process start ticks and cgroups, without collecting environments or command lines.
 result['desktop_process_identity']={}
 for line in result['own_desktop_processes'].get('stdout','').splitlines():
  parts=line.split()
  if len(parts)>=5 and parts[-1] in ['gnome-shell','Xorg','Xwayland','gnome-session-b'] and parts[0].isdigit():
   pid=parts[0];result['desktop_process_identity'][pid]={'stat':read('/proc/'+pid+'/stat'),'cgroup':read('/proc/'+pid+'/cgroup')}
 env=dict(os.environ,DOTNET_CLI_HOME=str(work/'dotnet-home'),DOTNET_CLI_TELEMETRY_OPTOUT='1',
  DOTNET_GENERATE_ASPNET_CERTIFICATE='false',DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE='true',RUSTUP_AUTO_INSTALL='0')
 result['tools']={}
 for tool,args in [('dotnet',['--info']),('rustc',['-vV']),('cargo',['-V'])]:
  path=shutil.which(tool);entry={'path':path}
  if path:
   resolved=Path(path).resolve();entry.update(resolved_path=str(resolved),sha256=sha(resolved))
   try:
    run=subprocess.run([path]+args,cwd=work,env=env,stdin=subprocess.DEVNULL,capture_output=True,text=True,timeout=30)
    entry['query']={'exit_code':run.returncode,'stdout':run.stdout,'stderr':run.stderr}
   except (OSError,subprocess.SubprocessError) as error:entry['error']=str(error)
  result['tools'][tool]=entry
 result['library_cache']=query(['ldconfig','-p'])
 result['boot_after']=read('/proc/sys/kernel/random/boot_id')
 result['boot_unchanged']=boot==result['boot_after'] and 'value' in boot
 result['limitations']=['No display connection or backend execution proof','Owner must correlate VMConnect Basic Session and intended VM',
  'Session/seat observations require independent review; env alone is not identity',
  'Process metadata is a snapshot, not cleanup qualification','No existing headless CI repeated']
 (report/'session.json').write_text(json.dumps(result,indent=2)+'\n')
 shutil.copy2(Path(__file__),report/'collector.py')
 hashes=[{'path':p.name,'sha256':sha(p)} for p in sorted(report.iterdir()) if p.is_file()]
 (report/'origin-hashes.json').write_text(json.dumps(hashes,indent=2)+'\n')
 print('Native original: '+str(report),flush=True)
 destination=Path('/mnt/j/coding/pivot/artifacts/ubuntu-gui-preparation/results/Pivot-Ubuntu-x64')/work.name
 try:
  if not Path('/mnt/j/coding/pivot/scripts/qualification').is_dir():raise ValueError('Expected exchange share unavailable')
  destination.parent.mkdir(parents=True,exist_ok=True);shutil.copytree(report,destination)
  for row in hashes:
   if sha(destination/row['path'])!=row['sha256']:raise ValueError('Returned hash mismatch')
  digest=sha(report/'origin-hashes.json')
  if sha(destination/'origin-hashes.json')!=digest:raise ValueError('Manifest mismatch')
  (destination/'transfer-receipt.json').write_text(json.dumps({'status':'Verified','native_manifest_sha256':digest,'returned_manifest_sha256':digest},indent=2)+'\n')
 except (OSError,ValueError) as error:
  print('Transfer failed; retain native original; do not repeat collection: '+str(error));return 2
 print('Returned: '+str(destination));print('Metadata collected; session/seat review pending. No GUI approval.')
 return 0

if __name__=='__main__':
 try:raise SystemExit(main())
 except (OSError,ValueError,KeyError) as error:
  print('Blocked: '+str(error),file=sys.stderr);raise SystemExit(2)
