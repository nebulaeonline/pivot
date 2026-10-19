"""Isolated-process compatibility foundation experiment; standard library only."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import time
from scaling import rss

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sizes', default='1000,10000,100000')
    parser.add_argument('--degrees', default='1,medium,high')
    parser.add_argument('--implementations', default='baseline,paged')
    parser.add_argument('--modes', default='latency,profile')
    parser.add_argument('--samples', type=int, default=31)
    parser.add_argument('--max-rss-mib', type=int, default=8192)
    parser.add_argument('--timeout-seconds', type=int, default=600)
    parser.add_argument('--output-dir', type=Path, default=ROOT/'artifacts/foundation')
    args = parser.parse_args()
    output = args.output_dir.resolve()/datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ')
    output.mkdir(parents=True)
    manifest = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for folder in ['crates/pivot-core','crates/pivot-workbook','scripts'] for p in (ROOT/folder).rglob('*') if p.is_file() and p.suffix in ['.rs','.toml','.py']}
    (output/'source-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    environment = {'platform': platform.platform(), 'cpu': platform.processor(), 'logical_cpus': os.cpu_count(), 'python': platform.python_version(), 'cargo_target_dir': os.environ.get('CARGO_TARGET_DIR'), 'arguments': vars(args)|{'output_dir': str(args.output_dir)}, 'rustc': subprocess.check_output(['rustc','-Vv'],text=True), 'release': 'default opt-level=3; no custom LTO/target CPU; separate profiling run'}
    (output/'environment.json').write_text(json.dumps(environment,indent=2)+'\n',encoding='utf-8')
    with (output/'build.log').open('w',encoding='utf-8') as log:
        subprocess.run(['cargo','build','--locked','--release','-p','pivot-core','--features','scaling-experiment','--example','foundation'],cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,check=True)
    metadata = json.loads(subprocess.check_output(['cargo','metadata','--no-deps','--format-version','1'],cwd=ROOT,text=True))
    binary = Path(metadata['target_directory'])/'release/examples'/('foundation.exe' if os.name=='nt' else 'foundation')
    summary = {'status':'running','runs':[]}
    for size in args.sizes.split(','):
        for label in args.degrees.split(','):
            degree = {'medium':min(1000,(int(size)-2)//2),'high':int(size)-2}.get(label)
            if degree is None: degree=int(label)
            shape=f'degree{degree}'
            for implementation in args.implementations.split(','):
                for mode in args.modes.split(','):
                    assert mode in ['latency','profile']
                    filename = f'{size}-{shape}-{implementation}-{mode}.csv'
                    command = [str(binary),'--size',size,'--degree',str(degree),'--implementation',implementation,'--samples',str(args.samples if mode=='latency' else min(args.samples,5))]
                    if mode=='profile': command.append('--profile')
                    peak=0
                    stopped=None
                    start=time.monotonic()
                    print(filename,flush=True)
                    with (output/filename).open('w',encoding='utf-8') as log:
                        process=subprocess.Popen(command,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT)
                        while process.poll() is None:
                            peak=max(peak,rss(process.pid) or 0)
                            if peak>args.max_rss_mib*1024*1024: stopped='RSS budget exceeded'
                            if time.monotonic()-start>args.timeout_seconds: stopped='time budget exceeded'
                            if stopped: process.kill();break
                            time.sleep(.05)
                        code=process.wait()
                    summary['runs'].append({'file':filename,'command':command,'exit_code':code,'seconds':round(time.monotonic()-start,3),'peak_rss_bytes':peak,'budget_stop':stopped})
                    (output/'summary.json').write_text(json.dumps(summary,indent=2)+'\n',encoding='utf-8')
    summary['status']='passed' if all(r['exit_code']==0 for r in summary['runs']) else 'incomplete'
    (output/'summary.json').write_text(json.dumps(summary,indent=2)+'\n',encoding='utf-8')
    print(str(output),flush=True)
    return 0 if summary['status']=='passed' else 1


if __name__=='__main__':
    raise SystemExit(main())
