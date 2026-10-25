"""One Windows guest Native AOT prerequisite collection; no build/publish/GUI."""
import argparse
import datetime
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import uuid

import windows_attended_aot as aot
from run_headless_ci import read_json,sha,write_json,verify_files,unpack_verified,same_source

DISABLED_PACKET_SHA256='e74a9fee3c44874cf2823becf181b61b37d163b79ca83371f6b2ae946133ad64'
TARGET='x86_64-pc-windows-msvc'


def classify(result):
    if result.get('status')=='Verified':return 'Verified'
    reason=result.get('reason','').lower()
    if any(reason.startswith(prefix) for prefix in [
        'required installed tool missing:','installed vswhere unavailable',
        'visual studio c++ component not found','prepared windows sdk/msvc library environment missing',
        'native library missing:','native rust target library missing']):return 'Missing'
    return 'Blocked'


def tool_version(path,env):
    child=dict(env,PIVOT_AOT_VERSION_PATH=str(path))
    command=['powershell.exe','-NoProfile','-NonInteractive','-Command',
             "$ErrorActionPreference='Stop'; $v=(Get-Item -LiteralPath $env:PIVOT_AOT_VERSION_PATH).VersionInfo; "
             "[ordered]@{file_version=$v.FileVersion;product_version=$v.ProductVersion} | ConvertTo-Json -Compress"]
    run=subprocess.run(command,env=child,stdin=subprocess.DEVNULL,capture_output=True,text=True,timeout=20)
    result={'exit_code':run.returncode,'stderr':run.stderr,'stdout':run.stdout}
    if run.returncode==0:
        try:result['version']=json.loads(run.stdout)
        except ValueError:result['version']=None
    return result


def collect(source,env):
    result=aot.prerequisites(source,env)
    result['qualification_status']=classify(result)
    if result['status']!='Verified':return result
    try:
        for name,tool in result['tools'].items():
            tool['architecture']=aot.image_arch(Path(tool['path']))
            tool['file_version_query']=tool_version(Path(tool['path']),env)
            version=tool['file_version_query']
            if version['exit_code'] or not isinstance(version.get('version'),dict):
                raise ValueError('Version metadata query failed: '+name)
            if name in ['cl','link','rc'] and not version['version'].get('file_version'):
                raise ValueError('Compiler/linker/SDK file version unavailable: '+name)
        command=['rustc','--print','target-libdir','--target',TARGET]
        run=subprocess.run(command,cwd=source,env=env,stdin=subprocess.DEVNULL,
                           capture_output=True,text=True,timeout=30)
        result['rust_target_query']={'command':command,'exit_code':run.returncode,'stdout':run.stdout,'stderr':run.stderr}
        if run.returncode:raise ValueError('Native Rust target query failed')
        folder=Path(run.stdout.strip());aot.common.local_tree(folder)
        libraries=sorted(folder.glob('libstd-*.rlib'))
        if not folder.is_dir() or not libraries:raise ValueError('Native Rust target library missing: '+str(folder))
        result['rust_target']={'target':TARGET,'library_root':str(folder),
                               'standard_library_hashes':[{'path':str(p),'sha256':sha(p)} for p in libraries]}
    except (OSError,ValueError,TypeError,subprocess.SubprocessError) as error:
        result['status']='Blocked';result['reason']=str(error);result['qualification_status']=classify(result)
    return result


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--packet-dir',type=Path,required=True)
    parser.add_argument('--return-root',type=Path,required=True)
    args=parser.parse_args()
    if sys.platform!='win32':raise SystemExit('Windows guest only')
    parent=aot.common.local_tree(Path('C:/PivotWork'))
    if not parent.is_dir():raise SystemExit('Existing native work-root required')
    packet=args.packet_dir.resolve(strict=True)
    if sha(packet/'bundle-receipt.json')!=DISABLED_PACKET_SHA256:raise SystemExit('Accepted disabled source packet mismatch')
    verify_files(packet,read_json(packet/'bundle-receipt.json')['files'])
    work=parent/('aot-prerequisites-'+uuid.uuid4().hex);work.mkdir()
    report=work/'report';report.mkdir();source=work/'source';source.mkdir()
    print('Native original: '+str(report),flush=True)
    result={'status':'Blocked','scope':'read-only guest Native AOT prerequisites; no execution approval',
            'captured_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'source_packet_sha256':DISABLED_PACKET_SHA256,'GUI_executed':False,
            'build_or_publish_executed':False,'G3':'held'}
    try:
        identity=aot.common.guest_identity();result['guest']=identity
        continuity=aot.common.reviewed_identity(identity);result['continuity']=continuity
        if continuity['status']!='Verified':raise ValueError('Reviewed guest/context continuity Blocked')
        rows=read_json(packet/'source-manifest.json');inputs=read_json(packet/'build-input-manifest.json')
        unpack_verified(packet/'source.zip',source,rows);verify_files(source,inputs)
        env=aot.common.clean_environment(work)
        result['prerequisites']=collect(source,env)
        result['status']=result['prerequisites']['qualification_status']
        result['source_after']=same_source(source,rows)
        if result['source_after']:result['status']='Blocked';result['source_error']='source changed'
        after=aot.common.guest_identity();result['identity_after']=after
        result['context_after']=aot.common.identity_continuity(identity,after)
        result['reviewed_after']=aot.common.reviewed_identity(after)
        if result['context_after']['status']!='Verified' or result['reviewed_after']['status']!='Verified':result['status']='Blocked'
    except (OSError,ValueError,KeyError,TypeError,subprocess.SubprocessError) as error:
        result['status']='Blocked';result['error']=str(error)
    helpers=[p for p in Path(__file__).parent.glob('*.py')]
    result['helper_hashes']={p.name:sha(p) for p in helpers}
    write_json(report/'prerequisites.json',result)
    for name in ['source-manifest.json','build-input-manifest.json','bundle-receipt.json']:shutil.copy2(packet/name,report/name)
    hashes=[{'path':p.relative_to(report).as_posix(),'sha256':sha(p)} for p in sorted(report.rglob('*')) if p.is_file()]
    write_json(report/'origin-hashes.json',hashes)
    destination=args.return_root/work.name
    try:
        destination.parent.mkdir(parents=True,exist_ok=True);shutil.copytree(report,destination);verify_files(destination,hashes)
        write_json(destination/'transfer-receipt.json',{'status':'Verified','native_manifest_sha256':sha(report/'origin-hashes.json'),
                   'returned_manifest_sha256':sha(destination/'origin-hashes.json')})
        print('Returned: '+str(destination))
    except (OSError,ValueError) as error:
        print('Transfer failed; retain native original and do not replay: '+str(error));return 2
    print('Prerequisites: '+result['status']+'; no publish/GUI approval.')
    return 0 if result['status']=='Verified' else 2


if __name__=='__main__':raise SystemExit(main())
