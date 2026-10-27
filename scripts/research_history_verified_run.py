"""Native-local G3 verified-open research worker; no desktop or production changes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

from validate import ROOT,copy_source,native_root,revision


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=Path)
    parser.add_argument('--fixture-binary',type=Path,required=True,help='Unchanged accepted Rust fixture executable on native storage')
    parser.add_argument('--fixture-source',type=Path,required=True,help='Source compiled into that fixture for comparison')
    args=parser.parse_args()
    output=args.output.resolve();output.mkdir(parents=True,exist_ok=False)
    report={'status':'running','revision':revision(ROOT),'filesystem':native_root(output),'steps':[]}
    source=output/'source';copy_source(ROOT,source)
    old=args.fixture_source.read_bytes()
    new=(source/'crates/pivot-workbook/examples/history_index_fixture.rs').read_bytes()
    # Line endings can differ between native source copies; code must be unchanged.
    if old.replace(b'\r\n',b'\n')!=new.replace(b'\r\n',b'\n'):
        raise RuntimeError('fixture source changed; build and validate it before reuse')
    report['fixture_executable_sha256']=hashlib.sha256(args.fixture_binary.read_bytes()).hexdigest()
    env=os.environ.copy();temp=output/'temp';temp.mkdir()
    for key in ['TMPDIR','TEMP','TMP']:env[key]=str(temp)
    def step(name,command):
        start=time.perf_counter()
        run=subprocess.run(list(map(str,command)),cwd=source,env=env,capture_output=True,text=True)
        (output/(name+'.log')).write_text(run.stdout+run.stderr,encoding='utf-8')
        report['steps'].append({'name':name,'command':list(map(str,command)),
                                'returncode':run.returncode,'seconds':time.perf_counter()-start})
        print(name,run.returncode,flush=True)
        if run.returncode:raise RuntimeError(name+' failed; see original log')
    try:
        step('verified-store-tests',[sys.executable,'tests/history-index/test_verified_store.py','-v'])
        for n in [1000,10000]:
            fixture=output/('fixture-'+str(n)+'.jsonl')
            step('producer-'+str(n),[args.fixture_binary,fixture,n])
            command=[sys.executable,'scripts/research_history_verified_measure.py',fixture,output/str(n)]
            if sys.platform=='darwin':command.append('--fullfsync')
            step('verified-'+str(n),command)
            result=json.loads((output/str(n)/'summary.json').read_text())
            if result['status']!='passed' or any(row['historical_decision_scan'] for row in result['normal_open_samples']):
                raise RuntimeError('bounded-open gate failed; larger scale refused')
            if n==1000 and sys.platform=='darwin':
                step('mac-sync-off-1000',[sys.executable,'scripts/research_history_verified_measure.py',fixture,output/'1000-sync-off'])
        report['status']='passed'
    except Exception as error:
        report.update(status='failed',error=str(error))
    (output/'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    return 0 if report['status']=='passed' else 1


if __name__=='__main__':raise SystemExit(main())
