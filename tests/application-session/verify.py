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
