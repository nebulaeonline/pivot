"""Fixed Windows Native AOT preparation. Positive publish-and-execute is disabled."""
import argparse
import datetime
import io
import json
import os
from pathlib import Path
import runpy
import shutil
import subprocess
import sys
import time
import traceback

import windows_attended_gui as common
from guest_preflight import image_arch
from launch_windows_managed import Tee
from run_headless_ci import read_json,sha,write_json,same_source

ENTRY='scripts/verify_native_aot.py'
AOT_EXECUTION_APPROVED=False


def owned_processes(work):
    """AOT-specific read-only scope: existing actors plus native compiler tools."""
    env=dict(os.environ,PIVOT_AOT_PROCESS_ROOT=str(work))
    script=r"""$ErrorActionPreference='Stop'
$root=$env:PIVOT_AOT_PROCESS_ROOT
@(Get-CimInstance Win32_Process -ErrorAction Stop | Where-Object {
 ($_.ExecutablePath -and $_.ExecutablePath.StartsWith($root+'\',[StringComparison]::OrdinalIgnoreCase)) -or
 ($_.Name -match '^(dotnet|cargo|rustc|python[0-9.]*(w)?|MSBuild|VBCSCompiler|cl|link|rc|ilc)\.exe$' -and
 $_.CommandLine -and $_.CommandLine.IndexOf($root,[StringComparison]::OrdinalIgnoreCase) -ge 0)
} | ForEach-Object {
 [ordered]@{ProcessId=$_.ProcessId;Name=$_.Name;CommandLine=$_.CommandLine;ExecutablePath=$_.ExecutablePath;
 StartTicks=if($_.CreationDate){$_.CreationDate.ToUniversalTime().Ticks.ToString()}else{$null}}
}) | ConvertTo-Json -Depth 4 -Compress
"""
    try:
        run=subprocess.run(['powershell.exe','-NoProfile','-NonInteractive','-Command',script],env=env,
                           stdin=subprocess.DEVNULL,capture_output=True,text=True,timeout=20)
        if run.returncode:return {'status':'Blocked','reason':run.stderr}
        rows=json.loads(run.stdout.strip() or '[]')
        if isinstance(rows,dict):rows=[rows]
        if not isinstance(rows,list) or any(not isinstance(row,dict) or not row.get('ProcessId') or not row.get('StartTicks') for row in rows):
            return {'status':'Blocked','reason':'Scoped process-start identity unavailable','matches':rows}
        return {'status':'Verified','matches':rows,'observed_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()}
    except (OSError,ValueError,subprocess.SubprocessError) as error:return {'status':'Blocked','reason':str(error)}


def prerequisites(source,env):
    """Read installed tools/libraries only. No restore, compile, publish or install."""
    result={'status':'Blocked','rid':'win-x64','queries':[],'tools':{},'libraries':{}}
    try:
        if sys.platform!='win32':raise ValueError('Windows prerequisite adapter only')
        settings={k.upper():v for k,v in env.items()}
        def query(command):
            run=subprocess.run(command,cwd=source,env=env,stdin=subprocess.DEVNULL,
                               capture_output=True,text=True,timeout=30)
            result['queries'].append({'command':command,'exit_code':run.returncode,
                                       'stdout':run.stdout,'stderr':run.stderr})
            if run.returncode:raise ValueError('Tool query failed: '+command[0])
            return run.stdout.strip()
        for tool in ['dotnet','rustc','cargo','cl','link','rc']:
            path=shutil.which(tool,path=settings.get('PATH'))
            if not path:raise ValueError('Required installed tool missing: '+tool)
            path=Path(path);common.local_tree(path.parent)
            if image_arch(path)!='x64':raise ValueError('Native x64 tool required: '+tool)
            result['tools'][tool]={'path':str(path),'sha256':sha(path)}
        compiler=Path(result['tools']['cl']['path'])
        linker=Path(result['tools']['link']['path'])
        if compiler.parent!=linker.parent or [p.lower() for p in compiler.parts[-3:-1]]!=['hostx64','x64']:
            raise ValueError('Prepared Hostx64/x64 MSVC compiler/linker required')
        vswhere=Path(settings.get('PROGRAMFILES(X86)',r'C:\Program Files (x86)'))/'Microsoft Visual Studio/Installer/vswhere.exe'
        if not vswhere.is_file():raise ValueError('Installed vswhere unavailable')
        instances=json.loads(query([str(vswhere),'-latest','-products','*','-requires',
                                    'Microsoft.VisualStudio.Component.VC.Tools.x86.x64','-format','json']))
        if not isinstance(instances,list) or not instances or any(not isinstance(item,dict) for item in instances):
            raise ValueError('Visual Studio C++ component not found')
        result['visual_studio']=[{'installationPath':i.get('installationPath'),'installationVersion':i.get('installationVersion')} for i in instances]
        version=query(['dotnet','--version']);result['dotnet_sdk']=version
        if not version.startswith('10.') or '-' in version:raise ValueError('Stable .NET SDK 10 required')
        info=query(['dotnet','--info'])
        if 'win-x64' not in info:raise ValueError('Native win-x64 dotnet runtime not established')
        rust=query(['rustc','--version','--verbose'])
        if 'host: x86_64-pc-windows-msvc' not in rust:raise ValueError('Native Rust MSVC host required')
        sdk=settings.get('WINDOWSSDKDIR');sdk_version=settings.get('WINDOWSSDKVERSION','').strip('\\/')
        vc=settings.get('VCTOOLSINSTALLDIR')
        if not sdk or not sdk_version or not vc:raise ValueError('Prepared Windows SDK/MSVC library environment missing')
        libraries={'kernel32':Path(sdk)/'Lib'/sdk_version/'um/x64/kernel32.lib',
                   'ucrt':Path(sdk)/'Lib'/sdk_version/'ucrt/x64/ucrt.lib',
                   'libcmt':Path(vc)/'lib/x64/libcmt.lib'}
        for name,path in libraries.items():
            common.local_tree(path.parent)
            if not path.is_file():raise ValueError('Native library missing: '+str(path))
            result['libraries'][name]={'path':str(path),'sha256':sha(path)}
        result['status']='Verified'
    except (OSError,ValueError,TypeError,subprocess.SubprocessError) as error:result['reason']=str(error)
    return result


def invoke_aot(source,report,env):
    if not AOT_EXECUTION_APPROVED:raise ValueError('Positive Native AOT gate disabled pending parent review')
    previous=(Path.cwd(),dict(os.environ),sys.argv[:],sys.path[:])
    try:
        os.chdir(source);os.environ.clear();os.environ.update(env)
        sys.path.insert(0,str(source/'scripts'))  # Same import root as direct script execution.
        sys.argv=[ENTRY,'--output-dir',str(report/'native-aot')]
        runpy.run_path(str(source/ENTRY),run_name='__main__')
        return 0
    except SystemExit as error:
        if isinstance(error.code,int):return error.code
        if error.code is not None:print(str(error.code),file=sys.stderr);return 1
        return 0
    except Exception:traceback.print_exc();return 1
    finally:
        os.chdir(previous[0]);os.environ.clear();os.environ.update(previous[1]);sys.argv=previous[2];sys.path[:]=previous[3]


def aot_evidence(report):
    summaries=list((report/'native-aot').glob('*/summary.json'))
    if len(summaries)!=1:raise ValueError('Exactly one original AOT summary required')
    summary=read_json(summaries[0]);output=summaries[0].parent;publish=output/'publish'
    if (summary.get('status')!='passed' or summary.get('rid')!='win-x64' or summary.get('publish_warnings')!=[]
            or not summary.get('native_desktop_and_all_peers') or not summary.get('adjacent_rust_child')):
        raise ValueError('Original Native AOT acceptance incomplete')
    executable=publish/'Pivot.Desktop.exe';child=publish/'pivot-session.exe'
    common.local_tree(publish)
    if image_arch(executable)!='x64' or image_arch(child)!='x64' or (publish/'Pivot.Desktop.dll').exists():
        raise ValueError('Published native Windows x64 artifact identity invalid')
    recorded={item['path'] for item in summary['published_files']}
    actual={str(p.relative_to(publish)) for p in publish.rglob('*') if p.is_file()}
    if recorded!=actual or len(recorded)!=len(summary['published_files']):raise ValueError('Published file inventory mismatch')
    for item in summary['published_files']:
        path=Path(item['path'])
        if path.anchor or '..' in path.parts or ':' in item['path'] or sha(publish/path)!=item['sha256']:
            raise ValueError('Published artifact hash mismatch/unsafe path')
    records=[read_json(output/'desktop-ui/results.json')]+[
        read_json(output/f'desktop-ui/results.{mode}.json') for mode in ['clipboard-save','discard','clean']]
    expected_session=common.desktop_identity()['session_id']
    for record in records:
        if (record.get('status')!='passed' or record.get('remaining_charge')!=0
                or record.get('execution_mode')!='native-aot' or not record.get('graphical_backend')
                or record.get('process_session_id')!=expected_session
                or Path(record.get('executable','')).resolve()!=executable.resolve()):
            raise ValueError('Actual native application/peer backend/session/charge/image evidence missing')
    if len({record.get('process_id') for record in records})!=4 or any(not record.get('process_id') for record in records):
        raise ValueError('Four independent native GUI processes required')
    return {'summary':str(summaries[0]),'process_ids':[r['process_id'] for r in records],
            'graphical_backends':[r['graphical_backend'] for r in records],'zero_charge_fields':4,
            'published_artifacts_hashed':len(summary['published_files'])}


def run_fixed_aot(source,report,env,identity,rows):
    if not AOT_EXECUTION_APPROVED:raise ValueError('Positive Native AOT gate disabled pending review')
    before=common.guest_identity()
    if common.identity_continuity(identity,before)['status']!='Verified' or common.reviewed_identity(before)['status']!='Verified':
        raise ValueError('Current Native AOT context/continuity not established')
    start=time.time();code=invoke_aot(source,report,env)
    result={'status':'Passed' if code==0 else 'Blocked' if code==2 else 'Failed',
            'runner_exit_code':code,'runner_invoked':True,'entry_point':ENTRY,'started_unix':start,
            'elapsed_seconds':time.time()-start,'evidence_errors':[]}
    for name,action in [('source_after',lambda:same_source(source,rows)),
                        ('survivors',lambda:owned_processes(source.parent)),('identity_after',common.guest_identity)]:
        try:result[name]=action()
        except (OSError,ValueError,subprocess.SubprocessError) as error:result['evidence_errors'].append(name+': '+str(error))
    if code==0:
        try:result['application_evidence']=aot_evidence(report)
        except (OSError,ValueError,KeyError,TypeError) as error:result['evidence_errors'].append(str(error))
        result['continuity_after']=common.identity_continuity(identity,result.get('identity_after'))
        result['reviewed_after']=common.reviewed_identity(result.get('identity_after'))
        if (result.get('source_after') or result['continuity_after']['status']!='Verified'
                or result['reviewed_after']['status']!='Verified' or result.get('survivors',{}).get('status')!='Verified'
                or result.get('survivors',{}).get('matches') or result['evidence_errors']):result['status']='Blocked'
    write_json(report/'aot-run.json',result)
    return 0 if result['status']=='Passed' else 1 if result['status']=='Failed' else 2


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle-dir',type=Path,required=True)
    parser.add_argument('--receipt-sha256',required=True)
    parser.add_argument('--work-root',type=Path,required=True)
    parser.add_argument('--run-aot',action='store_true',help='Disabled; requires separate parent approval')
    args=parser.parse_args();native=None
    try:
        if args.run_aot and not AOT_EXECUTION_APPROVED:raise ValueError('Positive Native AOT execution not approved')
        # Reuse exact accepted preflight, never its managed execution option.
        buffer=io.StringIO();previous_args=sys.argv[:];previous_out=sys.stdout
        try:
            sys.stdout=Tee(previous_out,buffer)
            sys.argv=['windows_attended_gui.py','--bundle-dir',str(args.bundle_dir),
                      '--receipt-sha256',args.receipt_sha256,'--work-root',str(args.work_root)]
            code=common.main()
        finally:sys.argv=previous_args;sys.stdout=previous_out
        marker='Native preflight report: '
        paths=[line.split(marker,1)[1] for line in buffer.getvalue().splitlines() if marker in line]
        if len(paths)==1:
            native=Path(paths[0]);parent=common.local_tree(args.work_root)
            if native.name!='report' or native.parent.parent!=parent:raise ValueError('Unexpected native preflight report path')
        if code:return code
        if native is None:raise ValueError('Native preflight report identity missing')
        source=native.parent/'source';env=common.clean_environment(native.parent)
        inputs=read_json(args.bundle_dir/'build-input-manifest.json')
        if ENTRY not in {item['path'] for item in inputs}:raise ValueError('Fixed Native AOT entry absent from input manifest')
        for filename in ['windows_attended_aot.py']:
            receipt=read_json(args.bundle_dir/'bundle-receipt.json')
            if not any(row['path']==filename for row in receipt['files']):raise ValueError('AOT helper absent from receipt')
            if sha(Path(__file__).parent/filename)!=sha(args.bundle_dir/filename):raise ValueError('AOT helper differs from frozen packet')
        tools=prerequisites(source,env);write_json(native/'aot-prerequisites.json',tools)
        if tools['status']!='Verified':raise ValueError('Native AOT prerequisites Blocked: '+str(tools.get('reason')))
        write_json(native/'aot-preparation.json',{'status':'PreflightPassed','GUI_executed':False,'entry_point':ENTRY,
                   'positive_path_enabled':AOT_EXECUTION_APPROVED,'separate_GUI_consent_required':True})
        if not args.run_aot:return 0
        return run_fixed_aot(source,native,env,read_json(native/'preflight.json')['guest'],
                             read_json(args.bundle_dir/'source-manifest.json'))
    except (OSError,ValueError,KeyError,TypeError,subprocess.SubprocessError) as error:
        result={'status':'Blocked','reason':str(error),'GUI_executed':False if not args.run_aot else None}
        if native is not None:write_json(native/'aot-blocked.json',result)
        print(json.dumps(result),file=sys.stderr);return 2
    finally:
        if native is not None and native.is_dir():
            write_json(native/'origin-hashes.json',[{'path':p.relative_to(native).as_posix(),'sha256':sha(p)}
                for p in sorted(native.rglob('*')) if p.is_file() and p.name!='origin-hashes.json'])


if __name__=='__main__':raise SystemExit(main())
