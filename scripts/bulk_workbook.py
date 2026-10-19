"""Guarded matched bulk core and opt-in workbook Release experiments."""
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
ROOT=Path(__file__).resolve().parents[1]
def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--sizes',default='1000,10000,100000')
    p.add_argument('--shapes',default='independent,mixed,chain,fanout')
    p.add_argument('--layers',default='new,existing,workbook')
    p.add_argument('--implementations',default='')
    p.add_argument('--samples',type=int,default=3)
    p.add_argument('--timeout-seconds',type=int,default=600)
    p.add_argument('--max-rss-mib',type=int,default=8192)
    p.add_argument('--output-dir',type=Path,default=ROOT/'artifacts/bulk-workbook')
    args=p.parse_args();out=args.output_dir.resolve()/datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ');out.mkdir(parents=True)
    manifest={str(f.relative_to(ROOT)):hashlib.sha256(f.read_bytes()).hexdigest() for folder in ['crates/pivot-core','crates/pivot-workbook','scripts'] for f in (ROOT/folder).rglob('*') if f.is_file() and f.suffix in ['.rs','.toml','.py']}
    (out/'source-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    env={'platform':platform.platform(),'cpu':platform.processor(),'logical_cpus':os.cpu_count(),'python':platform.python_version(),'rustc':subprocess.check_output(['rustc','-Vv'],text=True),'cargo_target_dir':os.environ.get('CARGO_TARGET_DIR'),'arguments':vars(args)|{'output_dir':str(args.output_dir)},'method':'ordinary Release; counting allocator active; construction samples separate from 31-sample edit rows; reference worksheet experiment checks final budget once for monotonic fixtures'}
    (out/'environment.json').write_text(json.dumps(env,indent=2)+'\n',encoding='utf-8')
    with (out/'build.log').open('w',encoding='utf-8') as log:
        subprocess.run(['cargo','build','--locked','--release','-p','pivot-workbook','--features','persistent-core,scaling-experiment','--example','bulk'],cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,check=True)
    meta=json.loads(subprocess.check_output(['cargo','metadata','--no-deps','--format-version','1'],cwd=ROOT,text=True));binary=Path(meta['target_directory'])/'release/examples'/('bulk.exe' if os.name=='nt' else 'bulk')
    summary={'status':'running','runs':[]}
    for size in args.sizes.split(','):
        for shape in args.shapes.split(','):
            for layer in args.layers.split(','):
                implementations=args.implementations.split(',') if args.implementations else (['reference','persistent'] if layer=='workbook' else ['reference','individual','batch'])
                for backend in implementations:
                    file=f'{size}-{shape}-{layer}-{backend}.csv';command=[str(binary),size,backend,shape,layer,str(args.samples)];start=time.monotonic();peak=0;stop=None;print(file,flush=True)
                    with (out/file).open('w',encoding='utf-8') as log:
                        child=subprocess.Popen(command,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT)
                        while child.poll() is None:
                            peak=max(peak,rss(child.pid) or 0)
                            if peak>args.max_rss_mib*1024*1024:stop='RSS budget exceeded'
                            if time.monotonic()-start>args.timeout_seconds:stop='time budget exceeded'
                            if stop:child.kill();break
                            time.sleep(.05)
                        code=child.wait()
                    summary['runs'].append({'file':file,'command':command,'exit_code':code,'seconds':round(time.monotonic()-start,3),'peak_rss_bytes':peak,'budget_stop':stop})
                    (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n',encoding='utf-8')
    summary['status']='passed' if all(r['exit_code']==0 for r in summary['runs']) else 'incomplete'
    (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n',encoding='utf-8');print(str(out),flush=True)
    return 0 if summary['status']=='passed' else 1
if __name__=='__main__':raise SystemExit(main())
