"""Run prepared GUI suites from the target's desktop using existing live consent."""
import argparse
import datetime
import json
from pathlib import Path
import platform
import shutil
import subprocess
import sys

from execution_policy import Blocked, blocked_exit, declare_execution, desktop_session
from gui_session import inspect_session
from validate import RIDS, actual_rid

ROOT=Path(__file__).resolve().parents[1]


def save_block(args,session,error):
    name=platform.node()+'-'+datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')+'.json'
    path=Path.home()/'PivotValidation/activation-reports'/name
    data={'status':'Blocked','suite':args.suite,'requested_rid':args.rid,'hostname':platform.node(),
          'graphical_session':session,'error':str(error),'this_launcher_started_tests':False}
    try:
        path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(data,indent=2)+'\n',encoding='utf-8')
        shared=ROOT/'artifacts/validation-activation'/name;shared.parent.mkdir(parents=True,exist_ok=True)
        shutil.copy2(path,shared);print('Activation diagnostic:',shared)
    except OSError as export_error: print('Activation diagnostic export:',export_error,'native path:',path,file=sys.stderr)


def latest_plan(root,rid,suite):
    for path in sorted((root/'artifacts/validation-matrix').glob('*/matrix.json'),reverse=True):
        try: matrix=json.loads(path.read_text(encoding='utf-8'))
        except (OSError,ValueError): continue
        if matrix.get('suite')!=suite: continue
        result=next((r for r in matrix['targets'] if r['rid']==rid and r['status']!='NotSelected'),None)
        if result is None: continue
        if result['status']=='Passed': return None
        plan=result.get('activation')
        if not plan: raise ValueError('Latest '+rid+' '+suite+' result needs investigation; prepare a fresh plan after resolving its error')
        if not all(Path(p).is_file() for p in plan['argv'][:2]):
            raise ValueError('Prepared interpreter/source is unavailable on this machine; use the designated target desktop')
        started=Path.home()/'PivotValidation'/plan['run_id']/rid
        if started.exists():
            raise ValueError('This attended run already started. Collect its reports before preparing another run: '+str(started))
        return plan
    raise ValueError('No prepared '+rid+' '+suite+' plan; prepare it with scripts/validate.py first')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('suite',choices=['managed','aot','both'])
    parser.add_argument('rid',nargs='?',choices=RIDS+['mac'])
    args=parser.parse_args()
    declare_execution(__file__)
    session={}
    try:
        session=inspect_session()
        if not session['ready_for_consent']: raise Blocked(session.get('reason','Graphical session unavailable'))
        controller,identity=actual_rid()
        native=controller.rsplit('-',1)[0]+'-'+identity['native_architecture']
        rids=['osx-arm64','osx-x64'] if args.rid=='mac' and sys.platform=='darwin' else [args.rid or native]
        if any(r not in RIDS or r.rsplit('-',1)[0]!=native.rsplit('-',1)[0] for r in rids):
            raise ValueError('Activation targets must run on this operating system')
        suites=['managed','aot'] if args.suite=='both' else [args.suite]
        plans=[]
        for rid in rids:
            for suite in suites:
                plan=latest_plan(ROOT,rid,suite)
                if plan: plans.append((rid,suite,plan))
                else: print(rid,suite,'already passed; not repeated')
        if not plans: return 0
        # Reuse the established broker once for this explicitly selected suite.
        # Workers and original runners inherit its live authority; no saved token.
        with desktop_session('attended '+', '.join(r+' '+s for r,s,_ in plans)):
            for rid,suite,plan in plans:
                print('Running',rid,suite,flush=True)
                code=subprocess.call(plan['argv'])
                if code: return code
        return 0
    except Blocked as error:
        save_block(args,session,error);return blocked_exit(error)
    except (OSError,ValueError) as error:
        save_block(args,session,error)
        print(str(error),file=sys.stderr);return 2


if __name__=='__main__': raise SystemExit(main())
