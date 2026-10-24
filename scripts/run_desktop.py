"""Build and run the real Pivot desktop with its owned Rust session."""
import argparse
import os
from pathlib import Path
import subprocess
from execution_policy import declare_execution, headless_environment

declare_execution(__file__)

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument("--projections",action="store_true",help="Open the non-savable, in-memory structured-result proof")
parser.add_argument("--bindings",action="store_true",help="Open the opt-in non-savable worksheet-output binding proof")
args=parser.parse_args()

ROOT = Path(__file__).resolve().parents[1]
subprocess.run(['cargo', 'build', '--locked', '--release', '-p', 'pivot-session'], cwd=ROOT, check=True)
meta = __import__('json').loads(subprocess.check_output(['cargo', 'metadata', '--format-version', '1', '--no-deps'], cwd=ROOT, text=True))
env = headless_environment()  # Manual launcher must not inherit automated activation.
if args.projections: env["PIVOT_PROJECTION_PROOF"]="1"
if args.bindings: env["PIVOT_BINDING_PROOF"]="1"
env['PIVOT_SESSION_PATH'] = str(Path(meta['target_directory']) / 'release' / ('pivot-session.exe' if os.name == 'nt' else 'pivot-session'))
raise SystemExit(subprocess.call(['dotnet', 'run', '--project', 'clients/dotnet/Pivot.Desktop', '-c', 'Release', '--disable-build-servers'], cwd=ROOT, env=env))
