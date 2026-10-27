"""Native worker for byte accounting and controlled 1K/10K compact comparison."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time

from validate import ROOT,copy_source,native_root,revision


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=Path);parser.add_argument('--prior',type=Path,required=True)
    args=parser.parse_args();out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
    result={'status':'running','filesystem':native_root(out),'revision':revision(ROOT),'steps':[]}
    source=out/'source';copy_source(ROOT,source)
    env=os.environ.copy();env['PATH']=str(Path.home()/'.cargo/bin')+os.pathsep+env.get('PATH','')
    env['CARGO_TARGET_DIR']=str(out/'target');temp=out/'temp';temp.mkdir()
    for name in ['TMPDIR','TEMP','TMP']:env[name]=str(temp)
    def step(name,command):
        started=time.perf_counter()
        run=subprocess.run(list(map(str,command)),cwd=source,env=env,capture_output=True,text=True)
        (out/(name+'.log')).write_text(run.stdout+run.stderr,encoding='utf-8')
        result['steps'].append({'name':name,'command':list(map(str,command)),'returncode':run.returncode,'seconds':time.perf_counter()-started})
        print(name,run.returncode,flush=True)
        if run.returncode:raise RuntimeError(name+' failed; see original log')
    try:
        step('compact-tests',[sys.executable,'tests/history-index/test_compact_store.py','-v'])
        step('account-previous',[sys.executable,'scripts/research_history_bytes.py',args.prior/'10000/history.sqlite',
                                  args.prior/'fixture-10000.jsonl',out/'previous-accounting.json'])
        step('format',['cargo','fmt','--all','--','--check'])
        step('clippy',['cargo','clippy','--locked','-p','pivot-workbook-document','--features','history-experiment','--example','history_workbook_baseline','--','-D','warnings'])
        step('wb5-build',['cargo','build','--locked','--release','-p','pivot-workbook-document','--features','history-experiment','--example','history_workbook_baseline'])
        binary=out/'target/release/examples'/('history_workbook_baseline.exe' if os.name=='nt' else 'history_workbook_baseline')
        for n in [1000,10000]:
            fixture=args.prior/('fixture-'+str(n)+'.jsonl')
            for mode in ['plain','buckets']:
                dest=out/(str(n)+'-'+mode)
                command=[sys.executable,'scripts/research_history_storage_measure.py',fixture,dest,'--mode',mode]
                if sys.platform=='darwin':command.append('--fullfsync')
                step(str(n)+'-'+mode,command)
            step(str(n)+'-wb5',[binary,out/(str(n)+'-buckets/current.cold'),out/(str(n)+'-current.pivot')])
        result['status']='passed'
    except Exception as error:result.update(status='failed',error=str(error))
    (out/'result.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    return 0 if result['status']=='passed' else 1


if __name__=='__main__':raise SystemExit(main())
