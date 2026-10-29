"""Run the actual Avalonia window self-check; requires a working desktop/display."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'scripts'))
from desktop_test_lock import gui_suite_lock
from execution_policy import Blocked, blocked_exit, declare_execution, desktop_environment, desktop_session

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output-dir', type=Path, default=ROOT/'artifacts/application-ui')
args = parser.parse_args()
declare_execution(__file__)
try:
    with desktop_session("managed desktop self-check"):
        output = args.output_dir.resolve()
        output.mkdir(parents=True, exist_ok=True)
        subprocess.run(['cargo','build','--locked','--release','-p','pivot-session'],cwd=ROOT,check=True)
        subprocess.run(['dotnet','build','clients/dotnet/Pivot.Desktop','-c','Release','--disable-build-servers','-warnaserror'],cwd=ROOT,check=True)
        metadata = json.loads(subprocess.check_output(['cargo','metadata','--no-deps','--format-version','1'],cwd=ROOT,text=True))
        env = desktop_environment()
        env['PIVOT_SESSION_PATH'] = str(Path(metadata['target_directory'])/'release'/('pivot-session.exe' if os.name=='nt' else 'pivot-session'))
        env['PIVOT_UI_SMOKE_REPORT'] = str(output/'results.json')
        with gui_suite_lock() as lock:
            (output/'clipboard-lock.json').write_text(json.dumps(lock, indent=2)+'\n', encoding='utf-8')
            subprocess.run(['dotnet','clients/dotnet/Pivot.Desktop/bin/Release/net10.0/Pivot.Desktop.dll'],cwd=ROOT,env=env,check=True,timeout=180)
        report = json.loads((output/'results.json').read_text(encoding='utf-8'))
        if report.get('execution_mode') != 'managed': raise SystemExit('Ordinary desktop gate did not run the managed application.')
        if not report.get('worksheet_text', '').startswith('passed:'): raise SystemExit('Worksheet text acceptance missing')
        if report['status'] != 'passed' or not report.get('clipboard_checklist', '').startswith('passed:') or not report.get('arrow_edit', '').startswith('passed:'): raise SystemExit(json.dumps(report,indent=2))
        print(json.dumps(report,indent=2))

        for mode in ['clipboard-save', 'discard', 'clean']:
            peer = json.loads((output/f'results.{mode}.json').read_text(encoding='utf-8'))
            if peer['status'] != 'passed' or peer['remaining_charge'] != 0:
                raise SystemExit(json.dumps(peer, indent=2))
            if mode == 'clipboard-save' and not (peer['native_received'] and peer['cancel_checked']):
                raise SystemExit('Cross-process clipboard / close-cancel evidence missing')

        if not report.get("structured_projection", "").startswith("passed:"): raise SystemExit("Structured projection acceptance missing")

        if not report.get("worksheet_binding", "").startswith("passed:"): raise SystemExit("Worksheet binding acceptance missing")

        if not report.get("spreadsheet_aggregates", "").startswith("passed:"): raise SystemExit("Spreadsheet aggregate acceptance missing")

        if not report.get("basic_formatting", "").startswith("passed:"): raise SystemExit("Basic formatting acceptance missing")

        if not report.get("typed_values", "").startswith("passed:"): raise SystemExit("Typed value acceptance missing")

        if not report.get("multiple_worksheets", "").startswith("passed:"): raise SystemExit("Multiple worksheet acceptance missing")

        if not report.get("rectangular_fill", "").startswith("passed:"): raise SystemExit("Rectangular Fill acceptance missing")
except Blocked as error:
    raise SystemExit(blocked_exit(error))
