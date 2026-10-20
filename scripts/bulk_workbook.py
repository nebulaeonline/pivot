"""Guarded matched bulk core and opt-in workbook Release experiments."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import signal
import subprocess
import time
from scaling import rss
ROOT=Path(__file__).resolve().parents[1]
def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--example',choices=['bulk','readiness'],default='bulk')
    p.add_argument('--index-policy',choices=['private','immutable'],default='private')
    p.add_argument('--profile',action='store_true')
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
        subprocess.run(['cargo','build','--locked','--release','-p','pivot-workbook','--features','persistent-core,scaling-experiment','--example',args.example],cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,check=True)
    meta=json.loads(subprocess.check_output(['cargo','metadata','--no-deps','--format-version','1'],cwd=ROOT,text=True));binary=Path(meta['target_directory'])/'release/examples'/(args.example+('.exe' if os.name=='nt' else ''))
    child_env=os.environ.copy()
    child_env.pop('PIVOT_IMMUTABLE_INDEX',None)
    child_env.pop('PIVOT_CONSTRUCTION_PROFILE',None)
    if args.index_policy=='immutable':child_env['PIVOT_IMMUTABLE_INDEX']='1'
    if args.profile:child_env['PIVOT_CONSTRUCTION_PROFILE']='1'
    summary={'status':'running','runs':[]}
    for size in args.sizes.split(','):
        for shape in args.shapes.split(','):
            for layer in args.layers.split(','):
                implementations=args.implementations.split(',') if args.implementations else (['reference','persistent'] if layer=='workbook' else ['reference','individual','batch'])
                for backend in implementations:
                    file=f'{size}-{shape}-{layer}-{backend}.csv';command=[str(binary),size,backend,shape,layer,str(args.samples)];start=time.monotonic();peak=0;stop=None;usage=None;print(file,flush=True)
                    with (out/file).open('w',encoding='utf-8') as log:
                        child=subprocess.Popen(command,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,env=child_env,start_new_session=(os.name!='nt'))
                        while child.returncode is None:
                            if hasattr(os,'wait4'):
                                done,status,observed=os.wait4(child.pid,os.WNOHANG)
                                if done:
                                    child.returncode=os.waitstatus_to_exitcode(status);usage=observed;break
                            elif child.poll() is not None:break
                            peak=max(peak,rss(child.pid) or 0)
                            if peak>args.max_rss_mib*1024*1024:stop='RSS budget exceeded'
                            if time.monotonic()-start>args.timeout_seconds:stop='time budget exceeded'
                            if stop:
                                if os.name=='nt':child.kill()
                                else:os.killpg(child.pid,signal.SIGKILL)
                                break
                            time.sleep(.05)
                        if hasattr(os,'wait4') and child.returncode is None:
                            _,status,usage=os.wait4(child.pid,0);child.returncode=os.waitstatus_to_exitcode(status)
                        code=child.wait()
                    cpu=None if usage is None else usage.ru_utime+usage.ru_stime
                    os_peak=None if usage is None else int(usage.ru_maxrss)*(1 if platform.system()=='Darwin' else 1024)
                    summary['runs'].append({'file':file,'command':command,'exit_code':code,'seconds':round(time.monotonic()-start,3),'peak_rss_bytes':peak,'os_child_peak_rss_bytes':os_peak,'child_cpu_seconds':cpu,'rss_method':'wait4 child high-water plus live guard where supported' if usage is not None else 'Windows process peak working set; CPU unavailable','budget_stop':stop})
                    (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n',encoding='utf-8')
    summary['status']='passed' if all(r['exit_code']==0 for r in summary['runs']) else 'incomplete'
    (out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n',encoding='utf-8');print(str(out),flush=True)
    return 0 if summary['status']=='passed' else 1
if __name__=='__main__':raise SystemExit(main())
