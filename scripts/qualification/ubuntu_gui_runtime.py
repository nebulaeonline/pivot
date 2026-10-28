"""Fixed Linux attended wrapper. Positive gate remains disabled in contract helper."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import socket
import subprocess
import sys
import threading
import time
import traceback
import uuid

import ubuntu_attended_gui as contract
from ubuntu_gui_preparation import query,read,socket_metadata,PROPERTIES,own_graphical
from guest_preflight import filesystem,image_arch
from run_headless_ci import read_json,write_json,sha,verify_files,unpack_verified,same_source

DISPLAY_KEYS=['DISPLAY','WAYLAND_DISPLAY','XDG_RUNTIME_DIR','XDG_SESSION_TYPE','XDG_CURRENT_DESKTOP']
ACTORS={'dotnet','cargo','rustc','rustup','MSBuild','VBCSCompiler','Pivot.Desktop','pivot-session',
        'python','python3','cc','gcc','clang','ld','collect2'}

def local_tree(path):
 path=Path(os.path.abspath(path))
 for parent in [path,*path.parents]:
  if parent.is_symlink():raise ValueError('Symlink in native path: '+str(parent))
 existing=path
 while not existing.exists():existing=existing.parent
 fs=filesystem(existing)
 if not fs.get('native_work_root') or fs.get('type')!='ext4':raise ValueError('Guest-local ext4 required: '+str(path))
 return path

def live_context():
 if sys.platform!='linux' or platform.machine()!='x86_64':raise ValueError('Native Linux x64 required')
 boot=read('/proc/sys/kernel/random/boot_id')
 uid=os.getuid();user=query(['id','-un'])
 userq=query(['loginctl','show-user',str(uid),'-p','Display','-p','Sessions','-p','State'])
 props=contract.properties(userq);ids=props.get('Sessions','').split()
 sessions={sid:query(['loginctl','show-session',sid]+sum([['-p',p] for p in PROPERTIES],[])) for sid in ids}
 candidates=own_graphical([contract.properties(q) for q in sessions.values()],uid)
 if len(candidates)!=1:raise ValueError('Exactly one active local graphical session required: '+json.dumps(sessions))
 seats={s['Seat']:query(['loginctl','show-seat',s['Seat'],'-p','ActiveSession','-p','CanGraphical']) for s in candidates if s.get('Seat')}
 data={'hostname':socket.gethostname(),'uid':uid,'user':user,'platform':platform.platform(),
  'hypervisor':query(['systemd-detect-virt']),'boot_before':boot,
  'logind':{'queries':{'user':userq,'self':query(['loginctl','show-session','self','-p','Id'])},'own_session_queries':sessions},
  'seats':seats,'environment':{k:os.environ.get(k) for k in DISPLAY_KEYS},
  'display_sockets':[socket_metadata(Path(p)) for p in ['/run/user/1000/wayland-0','/tmp/.X11-unix/X0']],
  'caller_cgroup':read('/proc/self/cgroup')}
 data['boot_after']=read('/proc/sys/kernel/random/boot_id');data['boot_unchanged']=boot==data['boot_after'] and 'value' in boot
 data['contract']=contract.session_contract(data)
 return data

def compare_context(baseline,current):
 a=contract.session_contract(baseline);b=contract.session_contract(current)
 for key in ['session','seat','boot']:
  if a[key]!=b[key]:raise ValueError('Changed Linux '+key)
 for key in ['hostname','uid']:
  if baseline[key]!=current[key]:raise ValueError('Changed '+key)
 before={r['path']:r for r in baseline['display_sockets']};after={r['path']:r for r in current['display_sockets']}
 if before.keys()!=after.keys():raise ValueError('Changed display socket inventory')
 for path in before:
  for key in ['socket','uid','device','inode']:
   if key not in before[path] or before[path][key]!=after[path].get(key):raise ValueError('Changed socket '+path+' '+key)
 if 'value' not in current.get('caller_cgroup',{}) or 'user-1000.slice' not in current['caller_cgroup']['value']:
  raise ValueError('Caller user cgroup unavailable/mismatched')
 return {'status':'Verified','session':b['session'],'seat':b['seat'],'terminal_logind_membership':'not required or inferred'}

def private_environment(work):
 env={k:v for k,v in os.environ.items() if not k.upper().startswith('PIVOT_')}
 roots={'CARGO_HOME':'cargo-home','CARGO_TARGET_DIR':'target','NUGET_PACKAGES':'nuget','DOTNET_CLI_HOME':'dotnet-home',
  'DOTNET_BUNDLE_EXTRACT_BASE_DIR':'bundle','TEMP':'tmp','TMP':'tmp','TMPDIR':'tmp','PIVOT_FILE_TEST_ROOT':'fixtures'}
 for key,folder in roots.items():
  path=local_tree(work/folder);path.mkdir(exist_ok=True);env[key]=str(path)
 env.update(AVALONIA_TELEMETRY_OPTOUT='1',DOTNET_CLI_TELEMETRY_OPTOUT='1',DOTNET_GENERATE_ASPNET_CERTIFICATE='false',
  DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE='true',RUSTUP_AUTO_INSTALL='0',DOTNET_SKIP_FIRST_TIME_EXPERIENCE='1')
 return env

def tools(source,env):
 result={}
 for name,args in [('dotnet',['--info']),('rustc',['-vV']),('cargo',['-V'])]:
  path=shutil.which(name,path=env.get('PATH'))
  if not path:raise ValueError('Missing tool: '+name)
  resolved=Path(path).resolve();local_tree(resolved.parent)
  if image_arch(resolved)!='x64':raise ValueError('Non-x64 installed tool: '+name)
  run=subprocess.run([path]+args,cwd=source,env=env,stdin=subprocess.DEVNULL,capture_output=True,text=True,timeout=30)
  result[name]={'path':path,'resolved_path':str(resolved),'sha256':sha(resolved),'exit_code':run.returncode,'stdout':run.stdout,'stderr':run.stderr}
  if run.returncode:raise ValueError('Tool query failed: '+json.dumps(result[name]))
 sdk=subprocess.run(['dotnet','--version'],cwd=source,env=env,stdin=subprocess.DEVNULL,capture_output=True,text=True,timeout=30)
 result['selected_sdk']={'exit_code':sdk.returncode,'stdout':sdk.stdout,'stderr':sdk.stderr}
 if sdk.returncode or not re.fullmatch(r'10\.\d+\.\d+',sdk.stdout.strip()):raise ValueError('Stable SDK10 selection required: '+json.dumps(result['selected_sdk']))
 if '10.0.' not in result['dotnet']['stdout'] or 'Architecture: x64' not in result['dotnet']['stdout']:
  # Whitespace in dotnet metadata is variable.
  if '10.0.' not in result['dotnet']['stdout'] or not any('Architecture:' in l and l.split()[-1]=='x64' for l in result['dotnet']['stdout'].splitlines()):raise ValueError('SDK10/x64 host not proven')
 if 'host: x86_64-unknown-linux-gnu' not in result['rustc']['stdout']:raise ValueError('Wrong Rust host')
 cache=query(['ldconfig','-p']);result['native_libraries']=cache
 for name in ['libX11.so.6','libXrandr.so.2','libXi.so.6','libXcursor.so.1','libXext.so.6','libSM.so.6','libICE.so.6','libfontconfig.so.1']:
  lines=[line for line in cache.get('stdout','').splitlines() if line.strip().startswith(name+' ') and 'x86-64' in line]
  if not lines:raise ValueError('Required native library unavailable: '+name)
  path=Path(lines[0].split('=>',1)[1].strip()).resolve();local_tree(path.parent)
  if image_arch(path)!='x64':raise ValueError('Wrong native library architecture: '+name)
  result[name]={'path':str(path),'sha256':sha(path)}
 return result

def proc_identity(pid,proc=Path('/proc'),step=None):
 def at(name,fn):return step('identity.'+name,fn) if step else fn()
 folder=proc/str(pid);text=at('stat',lambda:(folder/'stat').read_text());end=text.rfind(')')
 fields=text[end+2:].split()
 if end<0 or len(fields)<20 or not fields[19].isdigit() or int(fields[19])<=0:raise ValueError('Missing process start ticks')
 info=at('uid_directory_identity',lambda:folder.stat())
 return {'pid':int(pid),'uid':info.st_uid,'proc_device':info.st_dev,'proc_inode':info.st_ino,'start_ticks':fields[19],'ppid':int(fields[1]),
  'executable':at('executable',lambda:str((folder/'exe').resolve(strict=True))),
  'command':at('command',lambda:(folder/'cmdline').read_bytes().replace(b'\0',b' ').decode(errors='replace')),
  'cgroup':at('cgroup',lambda:(folder/'cgroup').read_text().strip())}

def preexisting_nonactor(name,command,start_ticks,observer_start,work):
 """Bounded cooperative scope, never an exemption for a runtime/build actor."""
 return (name not in ACTORS and str(work) not in command and
         str(start_ticks).isdigit() and str(observer_start).isdigit() and
         0<int(start_ticks)<int(observer_start))

def process_ancestry(row,proc):
 # A checker can itself be selected by its native work-path command line.
 # Its parent is outside this invocation; do not walk beyond the wrapper root.
 ancestry=[];parent=row['ppid'];visited=set()
 if row['pid']==os.getpid():return [{'pid':row['pid'],'start_ticks':row['start_ticks'],'uid':row['uid']}]
 while parent>1 and parent not in visited and len(ancestry)<32:
  visited.add(parent);ancestor=proc_identity(parent,proc)
  ancestry.append({'pid':parent,'start_ticks':ancestor['start_ticks'],'uid':ancestor['uid']})
  if parent==os.getpid():break
  parent=ancestor['ppid']
 return ancestry

def process_arguments(folder):
 return [s.decode('utf-8',errors='strict') for s in (folder/'cmdline').read_bytes().split(b'\0') if s]

def cwd_identity(folder):
 path=(folder/'cwd').resolve(strict=True);info=(folder/'cwd').stat()
 return {'path':str(path),'device':info.st_dev,'inode':info.st_ino}

def canonical_managed(argv,cwd,source):
 """Resolve only the fixed two-argument managed invocation against target cwd."""
 if len(argv)!=2 or Path(argv[0]).name!='dotnet' or not argv[1].endswith('.dll'):return None
 cwd=local_tree(cwd)
 if not cwd.is_dir():raise ValueError('Target cwd unavailable')
 argument=Path(argv[1]);candidate=argument if argument.is_absolute() else cwd/argument
 # Reject symlinked ancestors before canonicalization, including source boundaries.
 local_tree(candidate.parent)
 resolved=candidate.resolve(strict=True)
 expected=source/'clients/dotnet/Pivot.Desktop/bin/Release/net10.0/Pivot.Desktop.dll'
 if resolved!=expected.resolve(strict=True):raise ValueError('Managed DLL outside fixed frozen target')
 if candidate.is_symlink():raise ValueError('Managed DLL symlink')
 return str(resolved)

def stable_identity(before,after):
 keys=['pid','uid','proc_device','proc_inode','start_ticks','ppid','executable','command','cgroup']
 if any(before.get(k)!=after.get(k) for k in keys):raise ValueError('Process identity changed during observation (reuse/exec/context)')

def exited_original(folder,before,pidfd):
 """Only a pinned pidfd exit plus fresh /proc evidence can classify an exit."""
 import select
 poll=select.poll();poll.register(pidfd,select.POLLIN)
 events=poll.poll(0) # observation only; no cleanup wait
 if not any(flags & select.POLLIN for _,flags in events):return False
 try:
  text=(folder/'stat').read_text();fields=text[text.rfind(')')+2:].split()
  if fields[19]!=before['start_ticks']:raise ValueError('PID reused after pinned process exit')
  if before.get('proc_inode') is not None:
   info=folder.stat()
   if (info.st_dev,info.st_ino)!=(before['proc_device'],before['proc_inode']):raise ValueError('Proc directory identity changed after pinned exit')
  return fields[0] in ['Z','X']
 except FileNotFoundError:
  # pidfd establishes original process exit; a second fresh lookup must still be absent.
  if folder.exists():raise ValueError('PID directory reappeared; exit identity ambiguous')
  return True

def capture_scoped(folder,work,managed,step=None):
 def at(name,fn):return step(name,fn) if step else fn()
 def identity(label):
  nested=(lambda name,fn:step(label+'.'+name,fn)) if step else None
  return proc_identity(folder.name,folder.parent,step=nested)
 before=at('capture.initial_identity',lambda:identity('capture.initial'))
 if not callable(getattr(os,'pidfd_open',None)):raise ValueError('Linux pidfd_open unavailable; cannot pin scoped identity')
 fd=at('capture.pidfd_open',lambda:os.pidfd_open(before['pid'],0))
 try:
  initial_after=at('capture.initial_recheck',lambda:identity('capture.initial_recheck'))
  at('capture.validate_initial_identity',lambda:stable_identity(before,initial_after))
  try:
   row=dict(before);row['argv']=at('capture.argv',lambda:process_arguments(folder))
   row['cwd_identity']=at('capture.cwd',lambda:cwd_identity(folder));row['cwd']=row['cwd_identity']['path']
   row['ancestry']=at('capture.ancestry',lambda:process_ancestry(row,folder.parent));row['observer_pid']=os.getpid()
   if row['uid']!=1000 or not row['cgroup']:raise ValueError('Ambiguous scoped process ownership')
   if managed:
    row['managed_dll']=at('capture.canonical_dll',lambda:canonical_managed(row['argv'],Path(row['cwd']),work/'source'))
    if not row['managed_dll']:raise ValueError('Managed command changed')
    raw=at('capture.display_environment',lambda:(folder/'environ').read_bytes())
    environment=dict(part.split(b'=',1) for part in raw.split(b'\0') if b'=' in part)
    row['display']={k:environment.get(k.encode(),b'').decode(errors='replace') for k in DISPLAY_KEYS}
    row['display_evidence']='direct managed-process environment'
   else:
    # Build/runtime children do not execute the graphical backend. Context is
    # independently bound by UID/cgroup/ancestry and the live wrapper context.
    if not any(a['pid']==os.getpid() and a['uid']==1000 for a in row['ancestry']):
     raise ValueError('Scoped actor ancestry does not reach wrapper')
    row['display_context']={k:os.environ.get(k) for k in DISPLAY_KEYS}
    row['display_evidence']='non-GUI actor: wrapper context plus direct UID/cgroup/ancestry; environment not read'
   after=at('capture.post_identity',lambda:identity('capture.post'))
   at('capture.validate_post_identity',lambda:stable_identity(before,after))
   if row['argv']!=at('capture.post_argv',lambda:process_arguments(folder)) or row['cwd_identity']!=at('capture.post_cwd',lambda:cwd_identity(folder)):
    raise ValueError('Target argv/cwd changed during observation')
   return row,None
  except OSError as error:
   if exited_original(folder,before,fd):
    return None,{'pid':before['pid'],'start_ticks':before['start_ticks'],'status':'Pinned original exited during metadata read',
                 'proc_device':before.get('proc_device'),'proc_inode':before.get('proc_inode'),
                 'error':str(error),'pidfd_exit':True,'substitute_for_runtime_identity':False}
   raise
 finally:os.close(fd)

def process_token(folder):
 """Minimal untrusted identity, bracketed across early pin acquisition."""
 text=(folder/'stat').read_text();end=text.rfind(')');fields=text[end+2:].split()
 if end<0 or len(fields)<20 or not fields[19].isdigit() or int(fields[19])<=0:
  raise ValueError('Missing original start ticks')
 info=folder.stat()
 return {'pid':int(folder.name),'uid':info.st_uid,'start_ticks':fields[19],
         'proc_device':info.st_dev,'proc_inode':info.st_ino}

def observe_candidate(folder,work,observer_start):
 """Pin before selection. No vulnerable selected read lies outside this lifecycle."""
 state={'pid':int(folder.name),'start_ticks':None,'scope_reason':'not yet determined',
        'pidfd_acquired':False,'original_identity_bound':False,'exit_corroborated':False}
 fd=None;original=None
 def at(operation,fn):
  state['operation']=operation
  state['stage']=operation.split('.',1)[0]
  return fn()
 try:
  original=at('pin.preliminary_stat_uid',lambda:process_token(folder));state['start_ticks']=original['start_ticks']
  if original['uid']!=os.getuid():return {'kind':'outside','reason':'different UID','pid':original['pid']}
  if not callable(getattr(os,'pidfd_open',None)):raise ValueError('pidfd_open unavailable')
  fd=at('pin.pidfd_open',lambda:os.pidfd_open(original['pid'],0));state['pidfd_acquired']=True
  confirmed=at('pin.original_token_recheck',lambda:process_token(folder))
  if confirmed!=original:raise ValueError('PID/start/UID changed across pin acquisition')
  state['original_identity_bound']=True
  name=at('selection.comm',lambda:(folder/'comm').read_text().strip())
  cmd=at('selection.command',lambda:(folder/'cmdline').read_bytes().replace(b'\0',b' ').decode(errors='replace'))
  try:executable=at('selection.executable',lambda:str((folder/'exe').resolve(strict=True)))
  except PermissionError as error:
   if not preexisting_nonactor(name,cmd,original['start_ticks'],observer_start,work):raise
   if at('selection.outside_token_recheck',lambda:process_token(folder))!=original:raise ValueError('Outside-scope identity changed')
   return {'kind':'outside','pid':original['pid'],'name':name,'start_ticks':original['start_ticks'],
    'observer_start_ticks':observer_start,'reason':'Preexisting nonactor; no invocation-root command reference',
    'error':str(error),'error_type':type(error).__name__,'errno':error.errno,'original_identity_bound':True}
  selected=executable.startswith(str(work)+'/') or (name in ACTORS and str(work) in cmd)
  state['scope_reason']='private executable or root-referenced actor' if selected else 'selection pending'
  managed=False
  if Path(executable).name=='dotnet':
   argv=at('selection.argv',lambda:process_arguments(folder))
   if len(argv)==2 and argv[1].endswith('Pivot.Desktop.dll'):
    cwd=at('selection.cwd',lambda:cwd_identity(folder))
    managed=bool(at('selection.canonical_dll',lambda:canonical_managed(argv,Path(cwd['path']),work/'source')))
    selected=selected or managed
    if managed:state['scope_reason']='fixed canonical managed DLL from actual target cwd'
  if not selected:
   if at('selection.outside_token_recheck',lambda:process_token(folder))!=original:raise ValueError('Unselected identity changed')
   return {'kind':'outside','pid':original['pid'],'start_ticks':original['start_ticks'],'reason':'not in invocation scope','original_identity_bound':True}
  row,event=capture_scoped(folder,work,managed,step=at)
  if row:
   if row['start_ticks']!=original['start_ticks'] or row['uid']!=original['uid']:raise ValueError('Capture differs from early pinned original')
   if at('capture.final_token_recheck',lambda:process_token(folder))!=original:raise ValueError('Original identity changed after capture')
   if row['executable']!=executable or row['command']!=cmd:raise ValueError('Selection executable/command changed before full capture')
   row['observation_lifecycle']=dict(state,stage='complete',operation='verified selection and capture')
   return {'kind':'observation','record':row}
  if any(event.get(k)!=original.get(k) for k in ['start_ticks','proc_device','proc_inode']):raise ValueError('Exit event differs from early pinned original')
  return {'kind':'exit','record':dict(state,**event,exit_corroborated=True)}
 except (OSError,ValueError,IndexError) as error:
  diagnostic=dict(state,error=str(error),error_type=type(error).__name__,errno=getattr(error,'errno',None))
  if fd is not None and state['original_identity_bound'] and isinstance(error,OSError):
   try:
    if exited_original(folder,original,fd):
     diagnostic.update(status='Pinned original exited during observation',pidfd_exit=True,
                       exit_corroborated=True,substitute_for_runtime_identity=False)
     return {'kind':'exit','record':diagnostic}
   except (OSError,ValueError,IndexError) as corroboration:
    diagnostic['corroboration_error']={'error':str(corroboration),'error_type':type(corroboration).__name__,
                                      'errno':getattr(corroboration,'errno',None)}
  return {'kind':'unresolved','record':diagnostic}
 finally:
  if fd is not None:os.close(fd)

def scoped_snapshot(work,proc=Path('/proc')):
 rows=[];errors=[];outside=[];exits=[];prefix=str(work)+'/'
 try:observer_start=proc_identity(os.getpid(),proc)['start_ticks']
 except (OSError,ValueError,IndexError) as error:
  return {'status':'Blocked','matches':[],'errors':[{'observer_identity':str(error)}],
          'outside_invocation_permission_denials':[],'observed_unix':time.time()}
 for folder in proc.iterdir():
  if not folder.name.isdigit():continue
  result=observe_candidate(folder,work,observer_start)
  if result['kind']=='observation':rows.append(result['record'])
  elif result['kind']=='exit':exits.append(result['record'])
  elif result['kind']=='unresolved':errors.append(result['record'])
  elif result.get('error'):outside.append(result)
 return {'status':'Blocked' if errors else 'Verified','matches':rows,'errors':errors,
  'outside_invocation_permission_denials':outside,'exited_during_observation':exits,'observed_unix':time.time()}

class Observer:
 def __init__(self,work):self.work=work;self.stop_event=threading.Event();self.records={};self.errors=[];self.exits=[]
 def sample(self):
  snapshot=scoped_snapshot(self.work)
  self.errors.extend(snapshot['errors'])
  self.exits.extend(snapshot.get('exited_during_observation',[]))
  for row in snapshot['matches']:
   key=(row['pid'],row['start_ticks'])
   self.records[key]=row
 def loop(self):
  try:
   while not self.stop_event.is_set():self.sample();self.stop_event.wait(.01)
  except Exception as error:self.errors.append({'observer_error':str(error)})
 def start(self):self.thread=threading.Thread(target=self.loop,daemon=True);self.thread.start()
 def stop(self):self.stop_event.set();self.thread.join();self.sample()

def correlate(records,reports,source,display):
 if len({r.get('process_id') for r in reports})!=4:raise ValueError('Four distinct application PIDs required')
 for report in reports:
  if report.get('status')!='passed' or report.get('execution_mode')!='managed' or report.get('remaining_charge')!=0 or not report.get('graphical_backend'):raise ValueError('Incomplete actual managed backend/charge evidence')
  matches=[r for r in records if r['pid']==report['process_id']]
  if len(matches)!=1:raise ValueError('Missing/reused application PID')
  row=matches[0]
  if not row.get('start_ticks') or row['uid']!=1000 or 'user-1000.slice' not in row.get('cgroup','') or not row.get('ppid'):raise ValueError('Ambiguous runtime identity')
  if not any(a['pid']==row.get('observer_pid') and a.get('start_ticks') and a.get('uid')==1000 for a in row.get('ancestry',[])):
   raise ValueError('Application ancestry does not reach wrapper')
  if Path(row['executable']).name!='dotnet' or row.get('managed_dll')!=str(source/'clients/dotnet/Pivot.Desktop/bin/Release/net10.0/Pivot.Desktop.dll'):raise ValueError('Wrong actual managed executable/canonical DLL')
  if report.get('executable')!=row['executable']:raise ValueError('Report/executable mismatch')
  for key in ['DISPLAY','WAYLAND_DISPLAY','XDG_RUNTIME_DIR']:
   if row['display'].get(key)!=display.get(key):raise ValueError('Runtime display mismatch')
  children=[r for r in records if r.get('ppid')==row['pid'] and r.get('executable')==str(source.parent/'target/release/pivot-session')]
  if len(children)!=1 or children[0].get('uid')!=1000 or not children[0].get('start_ticks'):
   raise ValueError('Actual adjacent execution Rust child identity unavailable/ambiguous')
  if report.get('rust_process_id') is not None and report['rust_process_id']!=children[0]['pid']:
   raise ValueError('Peer Rust child report mismatch')
 return {'status':'Verified','process_ids':[r['process_id'] for r in reports],
  'backends':[r['graphical_backend'] for r in reports],'zero_charge_fields':4,
  'limitation':'10ms cooperative polling may miss a process; missing identity Blocks, never inferred. Linux process_session_id is not logind.'}

def classify(code,errors):return 'Failed' if code not in [0,2] else 'Blocked' if code==2 or errors else 'Passed'

def execute(source,report,env,baseline,rows):
 if not contract.POSITIVE_RUN_APPROVED:raise ValueError('Ubuntu positive managed GUI disabled pending review')
 compare_context(baseline,live_context())
 observer=Observer(source.parent);observer.start();code=2;errors=[]
 try:
  try:contract.invoke_fixed(source,report/'desktop-ui',env);code=0
  except SystemExit as error:code=error.code if isinstance(error.code,int) else 1
  except Exception:traceback.print_exc();code=1
 finally:
  try:observer.stop()
  except Exception as error:errors.append('Observer final snapshot failed: '+str(error))
 errors.extend(observer.errors)
 result={'runner_exit_code':code,'source_after':same_source(source,rows),'process_observations':list(observer.records.values()),'exited_during_observation':observer.exits,'evidence_errors':errors}
 try:
  result['continuity_after']=compare_context(baseline,live_context())
  result['survivors']=scoped_snapshot(source.parent)
  if result['source_after'] or result['survivors']['status']!='Verified' or result['survivors']['matches']:errors.append('Source changed or scoped survivors/incomplete snapshot')
  if code==0:
   reports=[read_json(report/'desktop-ui'/n) for n in ['results.json','results.clipboard-save.json','results.discard.json','results.clean.json']]
   result['runtime_identity']=correlate(result['process_observations'],reports,source,baseline['environment'])
   fixture=(report/'desktop-ui/results.pivot').read_bytes()
   if fixture[:8]!=b'PIVOTWB8' or hashlib.sha256(fixture[48:]).digest()!=fixture[16:48]:raise ValueError('Invalid saved WB7')
 except (OSError,ValueError,KeyError,TypeError) as error:errors.append(str(error))
 result['status']=classify(code,errors);write_json(report/'run.json',result)
 return 0 if result['status']=='Passed' else 1 if result['status']=='Failed' else 2

def export(report,return_root):
 if any(p.is_symlink() for p in report.rglob('*')):raise ValueError('Symlink in report tree')
 rows=[{'path':p.relative_to(report).as_posix(),'sha256':sha(p)} for p in sorted(report.rglob('*')) if p.is_file() and p.name!='origin-hashes.json']
 write_json(report/'origin-hashes.json',rows)
 destination=return_root/report.parent.name
 destination.parent.mkdir(parents=True,exist_ok=True);shutil.copytree(report,destination);verify_files(destination,rows)
 digest=sha(report/'origin-hashes.json')
 if sha(destination/'origin-hashes.json')!=digest:raise ValueError('Returned origin manifest mismatch')
 write_json(destination/'transfer-receipt.json',{'status':'Verified','native_manifest_sha256':digest,'returned_manifest_sha256':digest})
 print('Returned: '+str(destination))

def main():
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--packet-dir',type=Path,required=True);parser.add_argument('--receipt-sha256',required=True)
 parser.add_argument('--run-managed',action='store_true');args=parser.parse_args()
 if args.run_managed and not contract.POSITIVE_RUN_APPROVED:
  print('Blocked: Ubuntu positive GUI disabled.',file=sys.stderr);return 2
 report=None;code=2
 try:
  if not sys.stdin.isatty() or not sys.stdout.isatty():raise ValueError('Interactive owner terminal required; piped consent not accepted')
  if sys.platform!='linux':raise ValueError('Linux only')
  packet=local_tree(args.packet_dir)
  if sha(packet/'bundle-receipt.json')!=args.receipt_sha256:raise ValueError('Pinned packet mismatch')
  verify_files(packet,read_json(packet/'bundle-receipt.json')['files'])
  for name in ['ubuntu_attended_gui.py','ubuntu_gui_runtime.py','ubuntu_gui_preparation.py','guest_preflight.py','run_headless_ci.py']:
   if sha(Path(__file__).parent/name)!=sha(packet/name):raise ValueError('Helper differs from frozen packet: '+name)
  parent=local_tree(Path.home());work=parent/'pivot-preflight'/('ubuntu-attended-'+uuid.uuid4().hex);work.mkdir(parents=True)
  local_tree(work);report=work/'report';report.mkdir();print('Native report: '+str(report),flush=True)
  source=work/'source';source.mkdir();rows=read_json(packet/'source-manifest.json')
  for name in ['bundle-receipt.json','source-manifest.json','build-input-manifest.json']:shutil.copy2(packet/name,report/name)
  unpack_verified(packet/'source.zip',source,rows);verify_files(source,read_json(packet/'build-input-manifest.json'))
  for p in source.rglob('*'):
   if p.is_symlink():raise ValueError('Source symlink')
  baseline=read_json(packet/'session-evidence.json');current=live_context();write_json(report/'context-before.json',current)
  write_json(report/'continuity-before.json',compare_context(baseline,current))
  env=private_environment(work);write_json(report/'tools.json',tools(source,env))
  print('Confirm ROBPC / Pivot-Ubuntu-x64 / VM 9ce1b716-aa20-4b16-a206-25126fab46d5, Basic Session and unchanged resource/isolation settings. This is not GUI consent.')
  if input('Type CONFIRM: ').strip()!='CONFIRM':raise ValueError('Owner conditions not confirmed')
  after=live_context();write_json(report/'context-after-confirm.json',after);compare_context(baseline,after)
  code=execute(source,report,env,current,rows) if args.run_managed else 0
  changed=same_source(source,rows)
  if changed and code==0:code=2
  write_json(report/'preflight.json',{'status':'PreflightPassed' if not changed else 'Blocked','GUI_executed':None if args.run_managed else False,'positive_enabled':contract.POSITIVE_RUN_APPROVED,
   'owner_attested_VM':True,'source_after':same_source(source,rows),'entry':contract.ENTRY,'mode':'managed' if args.run_managed else 'non-GUI preflight'})
 except (OSError,ValueError,KeyError,TypeError,EOFError,subprocess.SubprocessError) as error:
  print('Blocked: '+str(error),file=sys.stderr)
  if report:write_json(report/'blocked.json',{'status':'Blocked','error':str(error)})
  code=2
 finally:
  if report:
   try:export(report,Path('/mnt/j/coding/pivot/artifacts/ubuntu-managed-implementation/results/Pivot-Ubuntu-x64'))
   except (OSError,ValueError) as error:
    print('Export failed; retain native original, copy-only recovery; do not replay: '+str(report)+'; '+str(error),file=sys.stderr)
    if code==0:code=2
 return code

if __name__=='__main__':raise SystemExit(main())
