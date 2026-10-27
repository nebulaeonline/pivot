"""Run the first G3 research slice on a designated native-local target; headless."""
import argparse
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import time

from validate import ROOT, copy_source, native_root, revision


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=Path)
    parser.add_argument('--transactions',type=int,default=1000)
    args=parser.parse_args()
    output=args.output.resolve()
    output.mkdir(parents=True,exist_ok=False)
    report={'status':'running','revision':revision(ROOT),'platform':platform.platform(),
            'machine':platform.machine(),'filesystem':native_root(output),'steps':[]}
    source=output/'source'
    copy_source(ROOT,source)
    env=os.environ.copy()
    env['PATH']=str(Path.home()/'.cargo/bin')+os.pathsep+env.get('PATH','')
    env['CARGO_TARGET_DIR']=str(output/'target')
    def step(name,command):
        started=time.perf_counter()
        result=subprocess.run(list(map(str,command)),cwd=source,env=env,capture_output=True,text=True)
        (output/(name+'.log')).write_text(result.stdout+result.stderr,encoding='utf-8')
        report['steps'].append({'name':name,'exit_code':result.returncode,'seconds':time.perf_counter()-started})
        print(name,result.returncode,flush=True)
        if result.returncode:raise RuntimeError(name+' failed; see original log')
    try:
        step('rust-version',['rustc','-vV'])
        step('format',['cargo','fmt','--all','--','--check'])
        step('clippy',['cargo','clippy','--locked','-p','pivot-workbook','--features','history-experiment','--example','history_index_fixture','--','-D','warnings'])
        step('g2-regression',['cargo','test','--locked','-p','pivot-workbook','--features','history-experiment','--test','history_mutation_records'])
        step('store-tests',[sys.executable,'tests/history-index/test_index_store.py'])
        step('fixture-build',['cargo','build','--locked','--release','-p','pivot-workbook','--features','history-experiment','--example','history_index_fixture'])
        binary=output/'target/release/examples'/('history_index_fixture.exe' if os.name=='nt' else 'history_index_fixture')
        fixture=output/'fixture.jsonl'
        step('fixture',[binary,fixture,args.transactions])
        step('sqlite',[sys.executable,'scripts/research_history_index.py',fixture,output/'sqlite'])
        report['status']='passed'
    except Exception as error:
        report.update(status='failed',error=str(error))
    (output/'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    return 0 if report['status']=='passed' else 1


if __name__=='__main__':raise SystemExit(main())
