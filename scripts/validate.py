"""Practical six-RID runner. Default: discover available targets; no GUI.

python scripts/validate.py --suite headless
python scripts/validate.py --targets linux-x64 --suite managed --gui
Configure SSH aliases/source roots in a private copy of validation-targets.json.
"""
import argparse
import base64
from concurrent.futures import ThreadPoolExecutor
from contextlib import nullcontext
import datetime
import json
import os
from pathlib import Path
import platform
import plistlib
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import uuid

ROOT=Path(__file__).resolve().parents[1]
RIDS=['win-x64','linux-x64','win-arm64','linux-arm64','osx-arm64','osx-x64']
SUITES=['discover','headless','managed','aot']

def default_config():
 local=ROOT/'artifacts/validation-config'/(platform.node()+'.json')
 return local if local.is_file() else ROOT/'scripts/validation-targets.json'

def write(path,data):
 path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(data,indent=2)+'\n',encoding='utf-8')
def query(command,cwd=None,env=None):
 try:
  r=subprocess.run(command,cwd=cwd,env=env,stdin=subprocess.DEVNULL,capture_output=True,text=True,timeout=30)
  return {'command':command,'exit_code':r.returncode,'stdout':r.stdout,'stderr':r.stderr}
 except (OSError,subprocess.SubprocessError) as e:return {'command':command,'error':str(e),'exit_code':127}
def revision(root):
 deployed=root/'.pivot-validation-source.json'
 if deployed.is_file():return json.loads(deployed.read_text())
 q=query(['git','rev-parse','HEAD'],root)
 return {'commit':q.get('stdout','').strip() or None,'dirty':query(['git','status','--porcelain'],root).get('stdout','').strip()!='','error':q.get('stderr') if q['exit_code'] else None}
def actual_rid():
 machine=platform.machine().lower();arch={'amd64':'x64','x86_64':'x64','arm64':'arm64','aarch64':'arm64'}.get(machine)
 if sys.platform=='win32':
  sys.path.insert(0,str(ROOT/'scripts/qualification'))
  from guest_preflight import windows_architectures
  native,process=windows_architectures()
  arch=process
 else:native=arch
 prefix={'win32':'win','linux':'linux','darwin':'osx'}.get(sys.platform)
 if prefix is None or arch is None:raise ValueError('Unsupported executing OS/process architecture')
 translation=query(['sysctl','-n','sysctl.proc_translated']) if sys.platform=='darwin' else None
 if translation and translation.get('stdout','').strip()=='1':native='arm64'
 return prefix+'-'+arch,{'native_architecture':native,'python_machine':machine,'translation':translation,
  'execution':'emulated' if native!=arch else 'native'}
def validate_tools(rid,probes):
 hosts={'win-x64':'x86_64-pc-windows-msvc','win-arm64':'aarch64-pc-windows-msvc',
  'linux-x64':'x86_64-unknown-linux-gnu','linux-arm64':'aarch64-unknown-linux-gnu',
  'osx-x64':'x86_64-apple-darwin','osx-arm64':'aarch64-apple-darwin'}
 if 'host: '+hosts[rid] not in probes['rustc'].get('stdout',''):raise ValueError('Rust host does not match target execution architecture')
 expected='x64' if rid.endswith('-x64') else 'arm64'
 if not any('Architecture:' in line and line.split()[-1].lower()==expected for line in probes['dotnet'].get('stdout','').splitlines()):
  raise ValueError('.NET host does not match target execution architecture')
 if not probes['sdk'].get('stdout','').strip().startswith('10.'):raise ValueError('Selected .NET SDK10 is required')
def validate_controller(target,controller,identity,allow_emulation):
 if target.rsplit('-',1)[0]!=controller.rsplit('-',1)[0]:raise ValueError('Requested '+target+' cannot execute on controller '+controller)
 if target.rsplit('-',1)[1]!=identity['native_architecture'] and not allow_emulation:
  raise ValueError('Target emulation requires explicit configuration')
def native_root(path):
 path.mkdir(parents=True,exist_ok=True)
 if sys.platform=='darwin':
  disk=query(['/bin/df','-P',str(path)]);mounts=query(['/sbin/mount'])
  if disk['exit_code'] or mounts['exit_code']:raise ValueError('Cannot inspect native Mac mount: '+str(disk))
  device=disk['stdout'].splitlines()[-1].split()[0]
  matches=[line for line in mounts['stdout'].splitlines() if line.startswith(device+' on ') and '(apfs,' in line and 'local' in line]
  if not device.startswith('/dev/') or not matches:raise ValueError('Native local APFS work root required')
  return {'type':'apfs','device':device,'mount':matches[0]}
 sys.path.insert(0,str(ROOT/'scripts/qualification'))
 from guest_preflight import filesystem
 fs=filesystem(path)
 if not fs.get('native_work_root'):raise ValueError('Native local work root required: '+str(fs))
 return fs
def copy_source(source,destination):
 def ignore(folder,names):
  return [n for n in names if n in {'.git','artifacts','target','bin','obj','.vs','.vscode','.codex','.agents','.aws','.venv','__pycache__','node_modules'} or (n.startswith('gpt') and n.endswith('.md'))]
 shutil.copytree(source,destination,ignore=ignore,symlinks=False)
def environment(work,target_rid=None):
 env={k:v for k,v in os.environ.items() if not k.startswith('PIVOT_')}
 installed_rust=Path.home()/'.cargo/bin'
 if installed_rust.is_dir():env['PATH']=str(installed_rust)+os.pathsep+env.get('PATH','')
 if sys.platform=='darwin':
  dotnet=Path('/usr/local/share/dotnet/x64') if target_rid=='osx-x64' else Path('/usr/local/share/dotnet')
  installed=[dotnet,Path.home()/'.dotnet',Path('/usr/local/bin'),Path('/opt/homebrew/bin')]
  env['PATH']=os.pathsep.join(str(p) for p in installed if p.is_dir())+os.pathsep+env.get('PATH','')
  if target_rid=='osx-x64':
   env.update(DOTNET_ROOT=str(dotnet),DOTNET_ROOT_X64=str(dotnet),RUSTUP_TOOLCHAIN='nightly-x86_64-apple-darwin')
   clang=Path('/Library/Developer/CommandLineTools/usr/bin/clang')
   if clang.is_file():
    env.update(CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER=str(clang),CC=str(clang),CXX=str(clang.parent/'clang++'))
    env['PATH']=str(clang.parent)+os.pathsep+env['PATH']
 for key,name in {'CARGO_HOME':'cargo-home','CARGO_TARGET_DIR':'target','NUGET_PACKAGES':'nuget',
  'DOTNET_CLI_HOME':'dotnet-home','DOTNET_BUNDLE_EXTRACT_BASE_DIR':'bundle',
  'TMPDIR':'tmp','TEMP':'tmp','TMP':'tmp','PIVOT_FILE_TEST_ROOT':'fixtures'}.items():
  p=work/name;p.mkdir(exist_ok=True);env[key]=str(p)
 env.update(AVALONIA_TELEMETRY_OPTOUT='1',DOTNET_CLI_TELEMETRY_OPTOUT='1',
  DOTNET_GENERATE_ASPNET_CERTIFICATE='false',DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE='true',RUSTUP_AUTO_INSTALL='0')
 return env
def command_for(suite,output):
 if suite=='headless':return [sys.executable,'scripts/ci.py','--output-dir',str(output/'ci')]
 if suite=='managed':return [sys.executable,'tests/application-session/ui_smoke.py','--output-dir',str(output/'desktop-ui')]
 if suite=='aot':return [sys.executable,'scripts/verify_native_aot.py','--output-dir',str(output/'aot')]
 raise ValueError('Unknown suite')
def owned_alive(pid):
 if os.name=='nt':
  import ctypes
  kernel=ctypes.WinDLL('kernel32',use_last_error=True)
  kernel.OpenProcess.restype=ctypes.c_void_p
  handle=kernel.OpenProcess(0x1000,False,int(pid))
  if not handle:
   if ctypes.get_last_error()==87:return False
   raise OSError('Cannot inspect owned PID '+str(pid))
  try:
   code=ctypes.c_ulong();kernel.GetExitCodeProcess.argtypes=[ctypes.c_void_p,ctypes.POINTER(ctypes.c_ulong)]
   if not kernel.GetExitCodeProcess(handle,ctypes.byref(code)):raise OSError('Cannot query owned PID')
   return code.value==259
  finally:
   kernel.CloseHandle.argtypes=[ctypes.c_void_p];kernel.CloseHandle(handle)
 try:os.kill(int(pid),0);return True
 except ProcessLookupError:return False
def check_gui(output,suite,group):
 if suite=='aot':
  summaries=list((output/'aot').glob('*/summary.json'))
  if len(summaries)!=1:raise ValueError('Native AOT summary missing')
  summary=json.loads(summaries[0].read_text());folder=summaries[0].parent/'desktop-ui'
  if summary['status']!='passed':raise ValueError('Native AOT runner failed')
 else:folder=output/'desktop-ui'
 reports=[json.loads((folder/n).read_text()) for n in ['results.json','results.clipboard-save.json','results.discard.json','results.clean.json']]
 if len({r['process_id'] for r in reports})!=4:raise ValueError('Four actual application reports required')
 for r in reports:
  if r['status']!='passed' or r['execution_mode']!=('managed' if suite=='managed' else 'native-aot') or r['remaining_charge']!=0 or not r.get('graphical_backend'):
   raise ValueError('Application mode/backend/assertion/resource check failed')
 pids={r['process_id'] for r in reports}|{r['rust_process_id'] for r in reports if r.get('rust_process_id')}
 survivors=[p for p in sorted(pids) if owned_alive(p)]
 if group is not None:
  try:os.killpg(group,0);survivors.append('owned-process-group:'+str(group))
  except ProcessLookupError:pass
 if survivors:raise ValueError('Owned processes survived: '+str(survivors))
 return {'processes':[r['process_id'] for r in reports],'backends':[r['graphical_backend'] for r in reports],
  'zero_charges':4,'owned_survivors':[],'cleanup':'reported owned PIDs plus launched process group where available; no unrelated /proc enumeration'}
def worker(args):
 source=ROOT;report_source=source/'artifacts/validation-runs'/args.run_id/args.worker
 work=Path.home()/'PivotValidation'/args.run_id/args.worker
 output=work/'reports';result={'rid':args.worker,'suite':args.suite,'status':'SetupFailed',
  'revision':revision(source),'environment':platform.platform(),'original_reports':str(output),'command':None}
 try:
  actual,identity=actual_rid();result.update(controller_rid=actual,identity=identity)
  validate_controller(args.worker,actual,identity,args.allow_emulation)
  result['filesystem']=native_root(work.parent)
  if work.exists():raise ValueError('Work directory already exists; use a new run ID')
  work.mkdir();output.mkdir();env=environment(work,args.worker)
  probes={n:query([n]+v,cwd=source,env=env) for n,v in [('dotnet',['--info']),('rustc',['-vV']),('cargo',['-V'])]}
  probes['sdk']=query(['dotnet','--version'],cwd=source,env=env)
  result['tools']=probes
  if any(q['exit_code'] for q in probes.values()):raise ValueError('Missing/unusable dotnet or Rust tool; see tools')
  validate_tools(args.worker,probes)
  result['actual_rid']=args.worker
  result['tool_execution']='emulated' if identity['native_architecture']!=args.worker.rsplit('-',1)[1] else 'native'
  if args.suite=='discover':result['status']='Available';result['note']='Metadata only; no build/test execution pass'
  else:
   if args.suite in ['managed','aot'] and not args.gui:
    result['status']='NeedsConsent';result['error']='Add --gui from the target desktop terminal';return finish_worker(result,output,report_source)
   snapshot=work/'source';copy_source(source,snapshot)
   sys.path.insert(0,str(source/'scripts'))
   from execution_policy import desktop_session,desktop_environment,Blocked
   consent=desktop_session('six-RID '+args.suite+' validation') if args.gui and args.suite in ['managed','aot'] else nullcontext()
   with consent:
    if args.gui:
     # Standard existing live broker; no manufactured response or permit.
     from execution_policy import PERMIT
     if PERMIT in os.environ:env[PERMIT]=os.environ[PERMIT]
    command=command_for(args.suite,output);result['command']=command;started=time.monotonic()
    with (output/'console.log').open('w',encoding='utf-8') as log:
     process=subprocess.Popen(command,cwd=snapshot,env=env,stdin=subprocess.DEVNULL,stdout=log,stderr=subprocess.STDOUT,
      start_new_session=(os.name!='nt'))
     try:code=process.wait()
     except BaseException:
      process.terminate();process.wait();raise
    result.update(exit_code=code,seconds=round(time.monotonic()-started,3),status='Passed' if code==0 else 'Blocked' if code==2 else 'Failed')
    if code==0 and sys.platform=='darwin':
     binary=work/'target/release/pivot-session'
     if binary.is_file():
      result['rust_binary']=query(['/usr/bin/file',str(binary)])
      expected='x86_64' if args.worker=='osx-x64' else 'arm64'
      if result['rust_binary']['exit_code'] or expected not in result['rust_binary']['stdout']:
       result.update(status='Failed',error='Built Rust executable architecture does not match target')
    if code==0 and args.suite in ['managed','aot']:
     result['application']=check_gui(output,args.suite,process.pid if os.name!='nt' else None)
   if result['status'] in ['Failed','Blocked'] and not result.get('error'):result['error']='Runner returned '+str(result['exit_code'])+'; see console.log'
 except Exception as error:
  result['error']=str(error)
  if result.get('exit_code')==0:result['status']='Failed'
 return finish_worker(result,output,report_source)
def finish_worker(result,output,destination):
 output.mkdir(parents=True,exist_ok=True);write(output/'result.json',result)
 try:
  destination.parent.mkdir(parents=True,exist_ok=True);shutil.copytree(output,destination)
 except OSError as e:result['export_error']=str(e);result['status']='ExportFailed';write(output/'result.json',result)
 print(json.dumps(result),flush=True)
 return 0 if result['status'] in ['Passed','Available'] else 2 if result['status'] in ['NeedsConsent','Blocked','SetupFailed'] else 1
def remote_command(target,argv):
 if target.get('shell')=='powershell':
  script='& '+' '.join("'"+str(a).replace("'","''")+"'" for a in argv)
  return 'powershell.exe -NoProfile -EncodedCommand '+base64.b64encode(script.encode('utf-16-le')).decode()
 return shlex.join(argv)
def ssh_tool(name):
 if os.name=='nt':
  installed=Path(os.environ.get('SystemRoot','C:/Windows'))/'System32/OpenSSH'/(name+'.exe')
  if installed.is_file():return str(installed)
 return name
def deploy_source(target,run_id,rid):
 """Ordinary source transfer to local guest storage; no source-manifest ceremony."""
 remote=str(target['source_root']).rstrip('/')+'/sources/'+run_id+'/'+rid
 bootstrap=target.get('bootstrap_python',['python3'])
 ssh=[ssh_tool('ssh'),'-o','BatchMode=yes']+target.get('ssh_options',[])+[target['host']]
 mkdir=bootstrap+['-c','from pathlib import Path; Path('+repr(remote)+').mkdir(parents=True,exist_ok=False)']
 subprocess.run(ssh+[remote_command(target,mkdir)],stdin=subprocess.DEVNULL,check=True,capture_output=True,text=True)
 with tempfile.TemporaryDirectory(prefix='pivot-validation-transfer-') as temp:
  snapshot=Path(temp)/'source';copy_source(ROOT,snapshot)
  write(snapshot/'.pivot-validation-source.json',revision(ROOT))
  archive=shutil.make_archive(str(Path(temp)/'source'),'zip',snapshot)
  subprocess.run([ssh_tool('scp')]+target.get('ssh_options',[])+[archive,target['host']+':'+remote+'/source.zip'],
   stdin=subprocess.DEVNULL,check=True,capture_output=True,text=True)
 extract=bootstrap+['-c',"import zipfile; from pathlib import Path; p=Path("+repr(remote)+"); zipfile.ZipFile(p/'source.zip').extractall(p)"]
 subprocess.run(ssh+[remote_command(target,extract)],stdin=subprocess.DEVNULL,check=True,capture_output=True,text=True)
 return remote
def run_target(rid,target,args,run_id,output):
 transport=target['transport'];result={'rid':rid,'suite':args.suite,'label':target.get('label'),
  'status':'NotConfigured','transport':transport}
 root=target.get('source_root') or (str(ROOT) if transport=='local' else None)
 if transport=='ssh' and (not target.get('host') or not root):
  result['error']='Configure SSH host alias and source_root';return result
 if transport=='ssh' and target.get('deploy'):
  try:root=deploy_source(target,run_id,rid)
  except (OSError,subprocess.CalledProcessError) as error:
   result.update(status='SetupFailed',error=str(error),details=getattr(error,'stderr',None));return result
 if transport=='local':command=[sys.executable,str(ROOT/'scripts/validate.py')]
 else:
  command=target.get('python',['python3'])+[str(root).rstrip('/')+'/scripts/validate.py']
 command+=['--worker',rid,'--suite',args.suite,'--run-id',run_id]
 if args.gui:command+=['--gui']
 if target.get('emulation'):command+=['--allow-emulation']
 if transport=='ssh':command=[ssh_tool('ssh'),'-o','BatchMode=yes']+target.get('ssh_options',[])+(['-tt'] if args.gui else [])+[target['host'],remote_command(target,command)]
 elif transport=='wsl':command=['wsl','--exec']+command
 elif transport!='local':result['error']='Unsupported transport: '+transport;return result
 result['dispatch_command']=command
 interactive=args.gui and args.suite in ['managed','aot']
 try:
  if interactive:code=subprocess.call(command)
  else:
   log=output/(rid+'.dispatch.log')
   with log.open('w',encoding='utf-8') as stream:code=subprocess.call(command,stdin=subprocess.DEVNULL,stdout=stream,stderr=subprocess.STDOUT)
  returned=ROOT/'artifacts/validation-runs'/run_id/rid/'result.json'
  if transport=='ssh':
   remote_path=str(root).rstrip('/')+'/artifacts/validation-runs/'+run_id+'/'+rid
   transfer=subprocess.run([ssh_tool('scp')]+target.get('ssh_options',[])+['-r',target['host']+':'+remote_path,str(output/rid)],
    stdin=subprocess.DEVNULL,capture_output=True,text=True)
   copied={'exit_code':transfer.returncode,'stdout':transfer.stdout,'stderr':transfer.stderr}
   returned=output/rid/'result.json'
   if copied['exit_code']:
    result.update(status='SetupFailed' if code else 'ExportFailed',error=copied,dispatch_exit_code=code)
    log=output/(rid+'.dispatch.log')
    if log.exists():result['dispatch_error']=log.read_text(errors='replace')[-4000:]
    return result
  if returned.is_file():
   data=json.loads(returned.read_text());result.update(data);result['label']=target.get('label');result['dispatch_exit_code']=code
  else:result.update(status='SetupFailed',error='No returned report; dispatch exit '+str(code)+'; see dispatch log')
 except OSError as e:result.update(status='SetupFailed',error=str(e))
 return result
def main():
 parser=argparse.ArgumentParser(description=__doc__)
 parser.add_argument('--config',type=Path,default=default_config())
 parser.add_argument('--targets',nargs='+',choices=RIDS,default=RIDS)
 parser.add_argument('--suite',choices=SUITES,default='discover');parser.add_argument('--gui',action='store_true')
 parser.add_argument('--jobs',type=int,default=2,help='Concurrent headless/discovery targets; GUI stays serial')
 parser.add_argument('--worker',choices=RIDS,help=argparse.SUPPRESS);parser.add_argument('--run-id',help=argparse.SUPPRESS)
 parser.add_argument('--allow-emulation',action='store_true',help=argparse.SUPPRESS)
 args=parser.parse_args()
 if args.worker:
  if not args.run_id or any(c not in '0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_' for c in args.run_id):parser.error('Safe run ID required')
  return worker(args)
 run_id=datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ')+'-'+uuid.uuid4().hex[:8]
 output=ROOT/'artifacts/validation-matrix'/run_id;output.mkdir(parents=True)
 config=json.loads(args.config.read_text())['targets'];results=[]
 if args.jobs<1:parser.error('--jobs must be positive')
 def selected_target(rid):
  if rid not in args.targets:return {'rid':rid,'status':'NotSelected','suite':args.suite}
  print('['+rid+'] '+args.suite,flush=True)
  return run_target(rid,config[rid],args,run_id,output)
 with ThreadPoolExecutor(max_workers=1 if args.gui else args.jobs) as pool:results=list(pool.map(selected_target,RIDS))
 selected=[r for r in results if r['status']!='NotSelected']
 bad=any(r['status'] in ['Failed','SetupFailed','ExportFailed'] for r in selected)
 partial=any(r['status'] not in ['Passed','Available'] for r in selected)
 summary={'run_id':run_id,'suite':args.suite,'revision':revision(ROOT),'targets':results,
  'status':'Failed' if bad else 'Partial' if partial else 'DiscoveryComplete' if args.suite=='discover' else 'Passed'}
 write(output/'matrix.json',summary)
 lines=['| Target | Result | Environment / reason |','| --- | --- | --- |']
 for r in results:lines.append('| '+r['rid']+' | '+r['status']+' | '+str(r.get('error') or r.get('label') or '').replace('|','/')+' |')
 (output/'matrix.md').write_text('\n'.join(lines)+'\n');print('\n'.join(lines));print('Reports: '+str(output))
 return 1 if bad else 2 if partial else 0

if __name__=='__main__':raise SystemExit(main())
