"""One fixed Windows guest non-GUI build with private opt-out/process observations.

No GUI runner, telemetry payload inspection, process killing or global settings.
"""
import argparse
import datetime
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time
import uuid

import windows_attended_gui as wrapper
from guest_preflight import windows_architectures
from run_headless_ci import read_json, sha, unpack_verified, verify_files, same_source, write_json

FROZEN_RECEIPT = '905350c373552132becc0547ef467e368d690033f0bab416465e9909752702e0'
OBSERVE_SECONDS = 15
EVIDENCE_TARGETS = '''<Project xmlns="http://schemas.microsoft.com/developer/msbuild/2003">
<Target Name="PivotObserveAvaloniaBefore" BeforeTargets="AvaloniaStats">
<PropertyGroup><_PivotLicenseCount>@(AvaloniaUILicenseKey->Count())</_PivotLicenseCount></PropertyGroup>
<WriteLinesToFile File="$(PivotCollectorEvidenceRoot)/$(MSBuildProjectName)-$(TargetFramework)-before.txt"
 Lines="phase=before|project=$(MSBuildProjectName)|license_items=$(_PivotLicenseCount)|optout=$([System.Environment]::GetEnvironmentVariable('AVALONIA_TELEMETRY_OPTOUT'))" Overwrite="true" />
</Target>
<Target Name="PivotObserveAvaloniaAfter" AfterTargets="AvaloniaStats">
<PropertyGroup><_PivotLicenseCount>@(AvaloniaUILicenseKey->Count())</_PivotLicenseCount></PropertyGroup>
<WriteLinesToFile File="$(PivotCollectorEvidenceRoot)/$(MSBuildProjectName)-$(TargetFramework)-after.txt"
 Lines="phase=after|project=$(MSBuildProjectName)|license_items=$(_PivotLicenseCount)|optout=$([System.Environment]::GetEnvironmentVariable('AVALONIA_TELEMETRY_OPTOUT'))" Overwrite="true" />
</Target></Project>'''


def process_summary(records, work):
    def key(row): return (int(row['pid']), row['start_ticks'])
    if not records or not all(type(row.get('pid')) is int and row['pid']>0 and
                             isinstance(row.get('start_ticks'),str) and row['start_ticks'].isdigit()
                             for record in records for row in record['processes']):
        raise ValueError('Missing baseline/process-start identity')
    initial = {key(row) for row in records[0]['processes']}
    unique = {}; current = set(); creation_events = []
    immediate = set(); final = set()
    for record in records:
        for row in record['processes']:
            identity = key(row); unique[identity] = row
            if str(work).lower() in row['command'].lower():
                current.add(identity)
                if record['kind'] == 'creation-observed': creation_events.append(identity)
                if record['phase'] == 'immediate-after-build': immediate.add(identity)
    # Final snapshot is fixed-budget diagnostic evidence, not a changed CI timeout.
    for row in records[-1]['processes']:
        if str(work).lower() in row['command'].lower(): final.add(key(row))
    return {'preexisting_collectors': [unique[k] for k in sorted(initial)],
            'current_build_collectors_observed': [unique[k] for k in sorted(current)],
            'current_build_creation_events': [list(k) for k in creation_events],
            'current_build_immediate_presence': [list(k) for k in sorted(immediate)],
            'current_build_final_presence': [list(k) for k in sorted(final)],
            'observation_limit': 'WMI creation polling 1s plus snapshots ~0.5s; sub-poll transient starts may be missed'}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle-dir', type=Path, required=True)
    parser.add_argument('--return-root', type=Path, required=True)
    args=parser.parse_args()
    if sys.platform!='win32' or windows_architectures()!=('x64','x64') or platform.node().lower()!='windev2407eval':
        raise SystemExit('Qualified native Windows x64 guest required; no host fallback')
    parent=wrapper.local_tree(Path('C:/PivotWork'))
    bundle=args.bundle_dir.resolve(strict=True)
    if sha(bundle/'bundle-receipt.json')!=FROZEN_RECEIPT: raise SystemExit('Frozen source receipt mismatch')
    verify_files(bundle,read_json(bundle/'bundle-receipt.json')['files'])
    work=parent/('collector-build-'+uuid.uuid4().hex);work.mkdir()
    report=work/'report';report.mkdir();source=work/'source';source.mkdir()
    print('Native original: '+str(report),flush=True)
    rows=read_json(bundle/'source-manifest.json');inputs=read_json(bundle/'build-input-manifest.json')
    unpack_verified(bundle/'source.zip',source,rows);verify_files(source,inputs)
    env=wrapper.clean_environment(work)
    env['AVALONIA_TELEMETRY_OPTOUT']='1'  # Experiment only, not a wrapper/global policy change.
    env.update(PIVOT_COLLECTOR_REPORT=str(report),PIVOT_COLLECTOR_WORK=str(work))
    task_evidence=report/'task-evidence';task_evidence.mkdir()
    targets=work/'observe.targets';targets.write_text(EVIDENCE_TARGETS)
    command=['dotnet','build','clients/dotnet/Pivot.Desktop','-c','Release','--disable-build-servers','-warnaserror',
             '-v:normal','-p:CustomAfterMicrosoftCommonTargets='+str(targets),
             '-p:PivotCollectorEvidenceRoot='+str(task_evidence)]
    result={'status':'Blocked','GUI_executed':False,'source_receipt_sha256':FROZEN_RECEIPT,
            'command':command,'optout':'private environment AVALONIA_TELEMETRY_OPTOUT=1',
            'observation_seconds':OBSERVE_SECONDS,'parent_optout_present':'AVALONIA_TELEMETRY_OPTOUT' in os.environ,
            'parent_ncrunch_present':'NCrunch' in os.environ,'G3':'held'}
    helper=Path(__file__).parent
    result['helper_hashes']={name:sha(helper/name) for name in ['collector_build_probe.py','collector_monitor.ps1']}
    monitor=None
    try:
        with (report/'monitor-console.log').open('w') as monitor_log:
            monitor=subprocess.Popen(['powershell.exe','-NoProfile','-NonInteractive','-Command',
                                      (helper/'collector_monitor.ps1').read_text()],env=env,
                                     stdin=subprocess.DEVNULL,stdout=monitor_log,stderr=subprocess.STDOUT)
            deadline=time.monotonic()+20
            while not (report/'ready.json').exists() and monitor.poll() is None and time.monotonic()<deadline:time.sleep(.1)
            if not (report/'ready.json').exists() or read_json(report/'ready.json')['status']!='Ready':
                raise ValueError('Scoped process observer unavailable; build not started')
            (report/'phase.txt').write_text('build')
            result['build_started_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat()
            with (report/'build.log').open('w',encoding='utf-8') as log:
                result['build_exit_code']=subprocess.run(command,cwd=source,env=env,stdin=subprocess.DEVNULL,
                                                       stdout=log,stderr=subprocess.STDOUT,timeout=900).returncode
            result['build_finished_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat()
            (report/'phase.txt').write_text('immediate-after-build')
            deadline=time.monotonic()+10
            while time.monotonic()<deadline:
                if (report/'snapshot-phase.txt').exists() and (report/'snapshot-phase.txt').read_text().strip()=='immediate-after-build':break
                if monitor.poll() is not None:break
                time.sleep(.1)
            else:raise ValueError('Immediate post-build snapshot not acknowledged')
            if monitor.poll() is not None:raise ValueError('Observer exited before post-build observation')
            (report/'phase.txt').write_text('bounded-observation');time.sleep(OBSERVE_SECONDS)
            (report/'stop').write_text('stop')
            monitor.wait(timeout=20)
        if read_json(report/'monitor-result.json')['status']!='Completed':raise ValueError('Observer incomplete')
        records=[json.loads(line) for line in (report/'processes.ndjson').read_text(encoding='utf-8-sig').splitlines()]
        result['process_observations']=process_summary(records,work)
        result['source_after']=same_source(source,rows)
        evidence=[dict(part.split('=',1) for part in p.read_text(encoding='utf-8-sig').strip().split('|'))
                  for p in sorted(task_evidence.glob('*.txt'))]
        result['task_evidence']=evidence
        output=(report/'build.log').read_text(encoding='utf-8',errors='replace')
        result['override_branch_messages_present']=any(x in output for x in [
            'Avalonia Accelerate Community requires telemetry','Avalonia Accelerate Trial requires telemetry'])
        result['licensing_limit']='MSBuild item counts omit cached tickets; no license contents inspected. Branch inference uses actual task output plus observed behavior.'
        result['status']='EvidenceCollected' if result['build_exit_code']==0 and not result['source_after'] else 'BuildFailed' if result['build_exit_code'] else 'Blocked'
        if not evidence or not all(e.get('optout')=='1' for e in evidence):result['status']='Blocked'
        dependency=work/'nuget/avalonia.buildservices/11.3.2'
        if dependency.is_dir():
            result['dependency_hashes']={str(p.relative_to(dependency)):sha(p) for p in dependency.rglob('*')
                                         if p.is_file() and (p.suffix in ['.dll','.targets','.nuspec'] or p.name=='README.md')}
            dep_report=report/'dependency-evidence';dep_report.mkdir()
            for name in ['README.md','avalonia.buildservices.nuspec']:
                if (dependency/name).is_file():shutil.copy2(dependency/name,dep_report/name)
    except (OSError,ValueError,KeyError,TypeError,subprocess.SubprocessError) as error:result['error']=str(error)
    finally:
        (report/'stop').write_text('stop')
        if monitor is not None and monitor.poll() is None:
            try:monitor.wait(timeout=15)
            except subprocess.TimeoutExpired:result['observer_cleanup']='Blocked; own observer still running; no process killed'
    write_json(report/'result.json',result)
    for name in ['source-manifest.json','build-input-manifest.json','bundle-receipt.json']:shutil.copy2(bundle/name,report/name)
    hashes=[{'path':p.relative_to(report).as_posix(),'sha256':sha(p)} for p in sorted(report.rglob('*')) if p.is_file()]
    write_json(report/'origin-hashes.json',hashes)
    destination=args.return_root/work.name
    try:
        destination.parent.mkdir(parents=True,exist_ok=True);shutil.copytree(report,destination);verify_files(destination,hashes)
        write_json(destination/'transfer-receipt.json',{'status':'Verified','native_manifest_sha256':sha(report/'origin-hashes.json'),
                   'returned_manifest_sha256':sha(destination/'origin-hashes.json')})
        print('Returned: '+str(destination))
    except (OSError,ValueError) as error:
        print('Transfer failed; retain native original; do not repeat build: '+str(error));return 2
    print('Result: '+result['status']+'; collector/branch observations require review, not GUI approval.')
    return 0 if result['status']=='EvidenceCollected' else 2


if __name__=='__main__':raise SystemExit(main())
