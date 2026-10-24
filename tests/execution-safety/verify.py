"""Build and exercise the real desktop startup policy without loading any GUI."""
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
from execution_policy import headless_environment

subprocess.run(['dotnet', 'build', 'tests/execution-safety/PolicyProbe.csproj',
                '-c', 'Release', '--disable-build-servers', '-warnaserror'],
               cwd=ROOT, env=headless_environment(), check=True)
env = headless_environment()
env['PIVOT_POLICY_PROBE'] = str(ROOT / 'tests/execution-safety/bin/Release/net10.0/PolicyProbe.dll')
raise SystemExit(subprocess.call([sys.executable, 'tests/execution-safety/test_policy.py'],
                                cwd=ROOT, env=env))
