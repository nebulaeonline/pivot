"""Virtual-window controls with the headless Avalonia platform; no native GUI or clipboard."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from execution_policy import declare_execution,headless_environment

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-dir',type=Path,default=ROOT/'artifacts/geometry-ui')
    parser.add_argument('--session',type=Path)
    args=parser.parse_args()
    declare_execution(__file__)
    output=args.output_dir.resolve();output.mkdir(parents=True,exist_ok=True)
    env=headless_environment();env['AVALONIA_TELEMETRY_OPTOUT']='1'
    env['DOTNET_CLI_TELEMETRY_OPTOUT']='1'
    if args.session is None:
        subprocess.run(['cargo','build','--locked','--release','-p','pivot-session'],cwd=ROOT,env=env,check=True)
        metadata=json.loads(subprocess.check_output(['cargo','metadata','--format-version','1','--no-deps'],cwd=ROOT,env=env,text=True))
        session=Path(metadata['target_directory'])/'release'/('pivot-session.exe' if os.name=='nt' else 'pivot-session')
    else:
        session=args.session.resolve()
    subprocess.run(['dotnet','build','tests/geometry-ui/GeometryUi.csproj','-c','Release','--disable-build-servers','-m:1','-warnaserror'],cwd=ROOT,env=env,check=True)
    with (output/'virtual-window.log').open('w',encoding='utf-8') as log:
        result=subprocess.run(['dotnet','tests/geometry-ui/bin/Release/net10.0/GeometryUi.dll',str(session),str(output)],cwd=ROOT,env=env,timeout=180,stdout=log,stderr=subprocess.STDOUT)
    print((output/'virtual-window.log').read_text(encoding='utf-8'),end='')
    (output/'summary.json').write_text(json.dumps({'status':'passed' if result.returncode==0 else 'failed','execution_class':'headless','native_windows':False,'system_clipboard':False,'exit_code':result.returncode},indent=2)+'\n',encoding='utf-8')
    result.check_returncode()

if __name__=='__main__':main()
