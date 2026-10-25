"""Manually invoked guest-local evidence wrapper around the existing headless CI.

No VM discovery/dispatch, installation, GUI permit, product test changes or retries.
Copy this file and guest_preflight.py from the frozen packet into guest-local storage.
"""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import uuid
import zipfile

from guest_preflight import filesystem, image_arch, kernel_arch, windows_architectures


def sha(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024*1024), b''): digest.update(block)
    return digest.hexdigest()


def read_json(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def verify_files(root, rows):
    seen = set()
    for row in rows:
        path = Path(row['path'])
        if path.anchor or '..' in path.parts or ':' in row['path'] or '\\' in row['path']:
            raise ValueError('Unsafe manifest path')
        if row['path'] in seen: raise ValueError('Duplicate manifest path')
        seen.add(row['path'])
        if sha(root/path) != row['sha256']: raise ValueError('Hash mismatch: '+row['path'])


def unpack_verified(archive, destination, rows):
    expected = {row['path'] for row in rows}
    if len(expected) != len(rows): raise ValueError('Duplicate manifest entries')
    with zipfile.ZipFile(archive) as packed:
        names = packed.namelist()
        if len(names) != len(set(names)) or set(names) != expected:
            raise ValueError('Archive inventory differs from manifest')
        # Validate the entire inventory before writing any entry.
        for name in names:
            path = Path(name)
            if path.anchor or '..' in path.parts or ':' in name or '\\' in name:
                raise ValueError('Unsafe archive path')
        for name in names:
            path = destination/name; path.parent.mkdir(parents=True, exist_ok=True)
            with packed.open(name) as source, path.open('xb') as target:
                shutil.copyfileobj(source,target)
    verify_files(destination,rows)


def same_source(root, rows):
    changes = []
    for row in rows:
        path = root/row['path']
        if not path.is_file() or sha(path) != row['sha256']: changes.append(row['path'])
    expected={row['path'] for row in rows}
    for directory, folders, files in os.walk(root):
        folders[:]=[f for f in folders if f not in ['.git','.vs','.vscode','artifacts','target','bin','obj','__pycache__']]
        for filename in files:
            relative=(Path(directory)/filename).relative_to(root).as_posix()
            if relative not in expected:changes.append('ADDED: '+relative)
    return changes


def owned_processes(work):
    """Read-only scoped snapshot, never kill. Not a whole-machine sandbox."""
    if os.name == 'nt':
        env = dict(os.environ, PIVOT_CI_PROCESS_ROOT=str(work))
        command = ['powershell','-NoProfile','-NonInteractive','-Command',
            "@(Get-CimInstance Win32_Process -ErrorAction Stop | Where-Object { ($_.ExecutablePath -and $_.ExecutablePath.IndexOf($env:PIVOT_CI_PROCESS_ROOT,[StringComparison]::OrdinalIgnoreCase) -ge 0) -or ($_.Name -match '^(dotnet|cargo|rustc|python[0-9.]*(w)?|MSBuild|VBCSCompiler)\\.exe$' -and $_.CommandLine -and $_.CommandLine.IndexOf($env:PIVOT_CI_PROCESS_ROOT,[StringComparison]::OrdinalIgnoreCase) -ge 0) } | Select-Object ProcessId,Name,CommandLine) | ConvertTo-Json -Depth 3"]
        result = subprocess.run(command,env=env,capture_output=True,text=True,timeout=20)
        if result.returncode: return {'status':'Blocked','reason':result.stderr}
        matches=json.loads(result.stdout.strip() or '[]')
        if isinstance(matches,dict):matches=[matches]
        return {'status':'Verified','matches':matches or []}
    matches = []; unreadable = []
    for entry in Path('/proc').iterdir():
        if not entry.name.isdigit() or int(entry.name) == os.getpid(): continue
        try:
            if entry.stat().st_uid != os.getuid(): continue
            raw=(entry/'cmdline').read_bytes()
            executable=raw.split(b'\0')[0].decode(errors='replace')
            name=Path(executable).name
            is_tool=name in ['cargo','rustc','dotnet'] or name.startswith('python')
            if not executable.startswith(str(work)+os.sep) and not is_tool:continue
            command = raw.replace(b'\0',b' ').decode(errors='replace')
            cwd=os.readlink(entry/'cwd')
            if str(work) in command or cwd==str(work) or cwd.startswith(str(work)+os.sep):
                matches.append({'pid':int(entry.name),'command':command,'cwd':cwd})
        except FileNotFoundError: pass
        except PermissionError: unreadable.append(int(entry.name))
    return {'status':'Verified' if not unreadable else 'Blocked','matches':matches,
            'unreadable_processes':unreadable,'method':'current guest user; unique native work-root in command line/cwd'}


def write_json(path, data):
    path.write_text(json.dumps(data,indent=2)+'\n',encoding='utf-8')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle-dir',type=Path,default=Path(__file__).resolve().parent)
    parser.add_argument('--work-root',type=Path,required=True,help='Existing guest-local NTFS/ext4 parent')
    parser.add_argument('--expected-hostname',required=True,help='Preflight identity check, not remote target selection')
    parser.add_argument('--return-root',type=Path,required=True,help='Explicit evidence-exchange destination; no test files run here')
    args = parser.parse_args()
    parent = args.work_root.resolve(strict=True); fs = filesystem(parent)
    native_arch = kernel_arch()
    process_arch = windows_architectures()[1] if os.name == 'nt' else image_arch(Path(sys.executable).resolve())
    if (not fs['native_work_root'] or native_arch != 'x64' or process_arch != 'x64'
            or platform.node().lower() != args.expected_hostname.lower()):
        print(json.dumps({'status':'Blocked','reason':'Expected qualified x64 guest identity/native root not proven','filesystem':fs}))
        return 2
    bundle = args.bundle_dir.resolve(strict=True)
    receipt = read_json(bundle/'bundle-receipt.json')
    verify_files(bundle,receipt['files'])
    if sha(Path(__file__)) != sha(bundle/'run_headless_ci.py'):
        raise ValueError('Executed helper differs from frozen helper')
    rows = read_json(bundle/'source-manifest.json')
    inputs = read_json(bundle/'build-input-manifest.json')
    work = parent/('ci-'+datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ')+'-'+uuid.uuid4().hex[:8])
    work.mkdir(); source = work/'source'; source.mkdir(); report = work/'report';report.mkdir()
    unpack_verified(bundle/'source.zip',source,rows)
    verify_files(source,inputs)
    temp = work/'tmp';temp.mkdir(); target=work/'target';target.mkdir()
    env = {k:v for k,v in os.environ.items() if k != 'PIVOT_DESKTOP_PERMIT' and not k.startswith('PIVOT_UI_')}
    env.update({'CARGO_TARGET_DIR':str(target),'PIVOT_FILE_TEST_ROOT':str(temp),
        'TMPDIR':str(temp),'TEMP':str(temp),'TMP':str(temp),'DOTNET_CLI_HOME':str(work/'dotnet-home'),
        'DOTNET_CLI_TELEMETRY_OPTOUT':'1','DOTNET_GENERATE_ASPNET_CERTIFICATE':'false',
        'DOTNET_CLI_WORKLOAD_UPDATE_NOTIFY_DISABLE':'true','RUSTUP_AUTO_INSTALL':'0'})
    # Caches are also private native paths; ordinary project dependency restore is
    # part of CI. No OS package/SDK/toolchain installer is invoked.
    env['NUGET_PACKAGES']=str(work/'nuget');env['CARGO_HOME']=str(work/'cargo-home')
    metadata={'class':'headless','hostname':platform.node(),'native_architecture':native_arch,
        'platform':platform.platform(),'uname':platform.uname()._asdict(),
        'python_process_architecture':process_arch,'filesystem':fs,'source':str(source),'target':str(target),
        'temp':str(temp),'dotnet_intermediates':'source/**/obj and bin on same native volume',
        'bundle':str(bundle),'source_files':len(rows),'build_inputs':len(inputs),
        'helper_sha256':sha(Path(__file__)),'bundle_receipt_sha256':sha(bundle/'bundle-receipt.json'),
        'GUI_isolation':'not approved','G3':'held; not started'}
    for name in ['source-manifest.json','build-input-manifest.json','bundle-receipt.json']:
        shutil.copy2(bundle/name,report/name)
    write_json(report/'environment.json',metadata)
    write_json(report/'source-before.json',{'status':'Verified','files':len(rows),'inputs':len(inputs)})
    command=[sys.executable,'scripts/ci.py','--output-dir',str(report/'ci')]
    print('Native work/report: '+str(work),flush=True)
    print('Running existing headless CI once; no desktop/AOT gate.',flush=True)
    code=None;failure=None
    try:
        with (report/'console.log').open('w',encoding='utf-8') as log:
            code=subprocess.run(command,cwd=source,env=env,stdin=subprocess.DEVNULL,
                                stdout=log,stderr=subprocess.STDOUT,timeout=3600).returncode
    except (OSError,KeyboardInterrupt,subprocess.TimeoutExpired) as error:failure=str(error)
    changes=same_source(source,rows)
    write_json(report/'source-after.json',{'status':'Verified' if not changes else 'Failed','changed_or_missing':changes})
    verify_files(bundle,receipt['files'])
    try:survivors=owned_processes(work)
    except (OSError,ValueError,subprocess.TimeoutExpired) as error:
        survivors={'status':'Blocked','reason':str(error)}
    write_json(report/'owned-processes.json',survivors)
    summaries=list((report/'ci').glob('*/summary.json'))
    ci=read_json(summaries[0]) if len(summaries)==1 else {}
    valid_ci=(ci.get('status')=='passed' and ci.get('desktop') is False
              and len(ci.get('steps',[]))==13 and all(s.get('exit_code')==0 for s in ci['steps']))
    status='Passed' if code==0 and not changes and valid_ci else 'Failed'
    if ci.get('missing_tools') or failure:status='Blocked'
    if (survivors['status']!='Verified' or survivors.get('matches')) and status!='Failed':status='Blocked'
    result={'status':status,'command':command,'ci_exit_code':code,'wrapper_error':failure,
        'CI_summary':str(summaries[0]) if summaries else None,'ci_reported_status':ci.get('status'),'steps':ci.get('steps',[]),
        'source_integrity':not changes,'complete_headless_summary':valid_ci,
        'survivor_check':survivors['status'],'native_report':str(report),
        'GUI_AOT_acceptance':'not executed','formal_acceptance':'pending parent review'}
    write_json(report/'result.json',result)
    hashes=[{'path':p.relative_to(report).as_posix(),'sha256':sha(p)} for p in sorted(report.rglob('*')) if p.is_file()]
    write_json(report/'origin-hashes.json',hashes)
    # Explicit source/evidence exchange only. On copy failure, keep originals and
    # copy them manually later; never rerun tests to retry a transfer.
    try:
        destination=args.return_root/work.name
        destination.parent.mkdir(parents=True,exist_ok=True)
        shutil.copytree(report,destination)
        verify_files(destination,hashes)
        write_json(destination/'transfer-receipt.json',{'status':'Verified','native_report':str(report),
            'files_verified':len(hashes),'origin_manifest_sha256':sha(report/'origin-hashes.json'),
            'copied_manifest_sha256':sha(destination/'origin-hashes.json')})
        print('Returned report: '+str(destination),flush=True)
    except (OSError,ValueError) as error:
        print('Transfer Blocked; retain native report and copy manually: '+str(error),flush=True)
        return 2
    print('CI result: '+status,flush=True)
    return 0 if status=='Passed' else 1 if status=='Failed' else 2


if __name__=='__main__':raise SystemExit(main())
