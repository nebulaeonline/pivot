"""Real C# to Rust owned-process session integration and bounded latency observations."""
import argparse
import json
from pathlib import Path
import os
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output-dir', type=Path, default=ROOT/'artifacts/application-session')
args = parser.parse_args()
args.output_dir = args.output_dir.resolve()
args.output_dir.mkdir(parents=True, exist_ok=True)
subprocess.run(['cargo','build','--locked','--release','-p','pivot-session'], cwd=ROOT, check=True)
metadata = json.loads(subprocess.check_output(['cargo','metadata','--format-version','1','--no-deps'], cwd=ROOT, text=True))
binary = Path(metadata['target_directory'])/'release'/('pivot-session.exe' if os.name == 'nt' else 'pivot-session')
subprocess.run(['dotnet','build','tests/application-session/ApplicationSession.csproj','-c','Release','--disable-build-servers','-warnaserror'], cwd=ROOT, check=True)
# A controlled startup failure must be a failed console result, never an unhandled
# test-runner exception that can invoke the operating system's crash dialog.
missing=args.output_dir/'intentionally-missing-session-executable'
if missing.exists():
    raise RuntimeError('Failure fixture must not exist')
failure=subprocess.run(['dotnet','tests/application-session/bin/Release/net10.0/ApplicationSession.dll','--weighted-checks',str(missing),str(args.output_dir/'startup-refusal')],cwd=ROOT,text=True,encoding='utf-8',stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30)
(args.output_dir/'headless-startup-refusal.log').write_text(failure.stdout+failure.stderr,encoding='utf-8')
if failure.returncode!=1 or not failure.stderr.strip() or 'Unhandled exception' in failure.stderr or 'PASS' in failure.stdout:
    raise AssertionError('Headless failure was not reported through the controlled console boundary')
subprocess.run(['dotnet','tests/application-session/bin/Release/net10.0/ApplicationSession.dll',str(binary),str(args.output_dir/'results.json')], cwd=ROOT, check=True, timeout=180)
subprocess.run([sys.executable,str(ROOT/'tests/geometry-ui/verify.py'),'--session',str(binary),'--output-dir',str(args.output_dir/'geometry-ui')],cwd=ROOT,check=True,timeout=180)
subprocess.run(['cargo','build','--locked','--release','-p','pivot-workbook-document','--example','capacity','--example','capacity_graphs','--example','regions','--example','structure','--example','sorting','--example','filtering','--example','visible_clipboard','--example','visible_edit','--example','allocation','--example','weighted','--example','weighted_range','--example','windows','--example','bound'],cwd=ROOT,check=True)
examples=Path(metadata['target_directory'])/'release/examples'
suffix='.exe' if os.name=='nt' else ''
commands=[('identity-capacity',[str(examples/('capacity'+suffix)),'identities',str(args.output_dir/'identity-capacity.pivot')])]
commands += [('graph-capacity-'+str(n),[str(examples/('capacity_graphs'+suffix)),str(n)]) for n in [1024,10000,65536]]
commands += [('typed-capacity-'+kind+'-'+str(n),[str(examples/('capacity'+suffix)),'typed',str(n),kind,str(args.output_dir/('typed-'+kind+'-'+str(n)+'.pivot'))]) for n in [10000,65536] for kind in ['text','boolean']]
commands += [('format-regions-'+str(n),[str(examples/('regions'+suffix)),str(n),str(args.output_dir/('regions-'+str(n)+'.pivot'))]) for n in [1024,10000,65536]]
commands += [('structure-'+str(n)+('-styles' if styles else ''),[str(examples/('structure'+suffix)),str(n),str(args.output_dir/('structure-'+str(n)+str(styles)+'.pivot'))]+(['styles'] if styles else [])) for n in [1024,10000,65536] for styles in [False,True]]
commands += [('sorting-'+str(n)+'-'+kind,[str(examples/('sorting'+suffix)),str(n),str(args.output_dir/('sorting-'+str(n)+'-'+kind+'.pivot')),kind]) for n in [1024,10000,65536] for kind in ['numeric','styles']]
commands += [('sorting-'+str(n)+'-'+kind,[str(examples/('sorting'+suffix)),str(n),str(args.output_dir/('sorting-'+str(n)+'-'+kind+'.pivot')),kind]) for n,kind in [(10000,'ten'),(10000,'sparse'),(65536,'sparse')]]
commands += [('filtering-'+str(n)+'-'+kind,[str(examples/('filtering'+suffix)),str(n),str(args.output_dir/('filtering-'+str(n)+'-'+kind+'.pivot')),kind]) for n in [1024,10000,65536] for kind in ['numeric','text']]
commands += [('filtering-'+str(n)+'-'+kind,[str(examples/('filtering'+suffix)),str(n),str(args.output_dir/('filtering-'+str(n)+'-'+kind+'.pivot')),kind]) for n,kind in [(10000,'ten'),(65536,'sparse'),(65536,'blank')]]
commands += [('visible-clipboard-'+kind,[str(examples/('visible_clipboard'+suffix)),kind]) for kind in ['values','formulas','styles']]
commands += [('visible-edit-'+kind,[str(examples/('visible_edit'+suffix)),kind]) for kind in ['clear-contents','clear-format','clear-both','fill-down','fill-up','fill-left','fill-right']]
commands += [('allocation-'+kind,[str(examples/('allocation'+suffix)),kind]) for kind in ['cents','tiny','wide','large-count','overflow']]
commands += [('weighted-'+kind,[str(examples/('weighted'+suffix)),kind]) for kind in ['mixed','mixed-scale','zeros-ties','wide','overflow']]
commands += [('weighted-range-'+kind,[str(examples/('weighted_range'+suffix)),kind]) for kind in ['mixed','mixed-scale','zeros-ties','wide','overflow']]
commands += [('windows-'+str(n),[str(examples/('windows'+suffix)),str(n)]) for n in [1024,10000,65536]]
commands += [('bound-'+str(n),[str(examples/('bound'+suffix)),str(n)]) for n in [1024,10000,65536]]
for name,command in commands:
    with (args.output_dir/(name+'.log')).open('w',encoding='utf-8') as log:
        subprocess.run(command,cwd=ROOT,check=True,stdout=log,stderr=subprocess.STDOUT,timeout=180)
