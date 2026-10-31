"""Real C# to Rust owned-process session integration and bounded latency observations."""
import argparse
import json
from pathlib import Path
import os
import subprocess

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
subprocess.run(['dotnet','tests/application-session/bin/Release/net10.0/ApplicationSession.dll',str(binary),str(args.output_dir/'results.json')], cwd=ROOT, check=True, timeout=180)
subprocess.run(['cargo','build','--locked','--release','-p','pivot-workbook-document','--example','capacity','--example','capacity_graphs','--example','regions','--example','structure'],cwd=ROOT,check=True)
examples=Path(metadata['target_directory'])/'release/examples'
suffix='.exe' if os.name=='nt' else ''
commands=[('identity-capacity',[str(examples/('capacity'+suffix)),'identities',str(args.output_dir/'identity-capacity.pivot')])]
commands += [('graph-capacity-'+str(n),[str(examples/('capacity_graphs'+suffix)),str(n)]) for n in [1024,10000,65536]]
commands += [('typed-capacity-'+kind+'-'+str(n),[str(examples/('capacity'+suffix)),'typed',str(n),kind,str(args.output_dir/('typed-'+kind+'-'+str(n)+'.pivot'))]) for n in [10000,65536] for kind in ['text','boolean']]
commands += [('format-regions-'+str(n),[str(examples/('regions'+suffix)),str(n),str(args.output_dir/('regions-'+str(n)+'.pivot'))]) for n in [1024,10000,65536]]
commands += [('structure-'+str(n)+('-styles' if styles else ''),[str(examples/('structure'+suffix)),str(n),str(args.output_dir/('structure-'+str(n)+str(styles)+'.pivot'))]+(['styles'] if styles else [])) for n in [1024,10000,65536] for styles in [False,True]]
for name,command in commands:
    with (args.output_dir/(name+'.log')).open('w',encoding='utf-8') as log:
        subprocess.run(command,cwd=ROOT,check=True,stdout=log,stderr=subprocess.STDOUT,timeout=180)
