"""Separate native publish-and-execute gate. Requires a native toolchain and desktop."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import time

from desktop_test_lock import clipboard_lock

ROOT = Path(__file__).resolve().parents[1]


def host_rid():
    machine = platform.machine().lower()
    rid = {('win32', 'amd64'): 'win-x64', ('win32', 'x86_64'): 'win-x64',
           ('linux', 'x86_64'): 'linux-x64', ('darwin', 'arm64'): 'osx-arm64'}.get((sys.platform, machine))
    if rid is None:
        raise RuntimeError(f'Unsupported baseline host: {sys.platform}/{machine}')
    return rid


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-dir', type=Path, default=ROOT/'artifacts/native-aot')
    args = parser.parse_args()
    output = args.output_dir.resolve()/datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output.mkdir(parents=True)
    summary = {'status': 'failed', 'platform': platform.platform(), 'machine': platform.machine(),
               'python': sys.version, 'steps': [], 'output': str(output)}
    print(f'Native AOT reports: {output}', flush=True)

    def run(name, command, **options):
        started = time.monotonic()
        with (output/f'{name}.log').open('w', encoding='utf-8') as log:
            result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, **options)
        summary['steps'].append({'name': name, 'command': command, 'exit_code': result.returncode,
                                 'seconds': time.monotonic()-started, 'log': f'{name}.log'})
        if result.returncode:
            raise RuntimeError(f'{name} failed ({result.returncode}); see {output/name}.log')

    try:
        rid = host_rid()
        summary['rid'] = rid
        run('desktop-lock-tests', [sys.executable, 'tests/application-session/test_desktop_lock.py'])
        run('dotnet-info', ['dotnet', '--info'])
        run('rust-version', ['rustc', '--version', '--verbose'])
        if os.name == 'nt':
            vswhere = Path(os.environ.get('ProgramFiles(x86)', r'C:\Program Files (x86)'))/'Microsoft Visual Studio/Installer/vswhere.exe'
            if not vswhere.is_file():
                raise RuntimeError('Windows Native AOT requires Visual Studio C++ build tools; vswhere was not found.')
            run('native-toolchain', [str(vswhere), '-latest', '-products', '*', '-requires', 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64', '-format', 'json'])
        else:
            run('native-toolchain', ['clang', '--version'])
            if sys.platform == 'linux':
                run('native-runtime', ['ldd', '--version'])
            else:
                run('native-sdk', ['xcode-select', '-p'])

        run('rust-build', ['cargo', 'build', '--locked', '--release', '-p', 'pivot-session'])
        metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--no-deps', '--format-version', '1'], cwd=ROOT, text=True))
        publish = output/'publish'
        run('publish', ['dotnet', 'publish', 'clients/dotnet/Pivot.Desktop', '-c', 'Release', '-r', rid,
                        '-p:PublishProfile=NativeAot', '--disable-build-servers', '-warnaserror', '-o', str(publish)])
        log = (output/'publish.log').read_text(encoding='utf-8', errors='replace')
        summary['publish_warnings'] = [line for line in log.splitlines() if re.search(r'\bwarning(?:\s+[A-Z]+\d+)?\s*:', line, re.I)]
        if summary['publish_warnings']:
            raise RuntimeError('Publish emitted warnings; retain and investigate them before accepting the baseline.')
        executable = publish/('Pivot.Desktop.exe' if os.name == 'nt' else 'Pivot.Desktop')
        if not executable.is_file() or (publish/'Pivot.Desktop.dll').exists():
            raise RuntimeError('Publish did not produce the expected native desktop image.')
        child_name = 'pivot-session.exe' if os.name == 'nt' else 'pivot-session'
        shutil.copy2(Path(metadata['target_directory'])/'release'/child_name, publish/child_name)
        summary['published_files'] = [{'path': str(p.relative_to(publish)), 'bytes': p.stat().st_size, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()}
                                      for p in sorted(publish.rglob('*')) if p.is_file()]
        summary['desktop_executable_bytes'] = executable.stat().st_size
        summary['published_total_bytes'] = sum(p['bytes'] for p in summary['published_files'])
        summary['runtime_payload_bytes'] = sum(p['bytes'] for p in summary['published_files'] if not p['path'].lower().endswith(('.pdb', '.dbg')) and '.dsym/' not in p['path'].lower().replace('\\', '/'))
        ui = output/'desktop-ui'
        ui.mkdir()
        env = os.environ.copy()
        # Exercise real adjacent-child discovery, not a developer override.
        env.pop('PIVOT_SESSION_PATH', None)
        env.pop('PIVOT_UI_PEER_REPORT', None)
        env.pop('PIVOT_UI_PEER_MODE', None)
        env['PIVOT_UI_SMOKE_REPORT'] = str(ui/'results.json')
        with clipboard_lock() as lock:
            (output/'clipboard-lock.json').write_text(json.dumps(lock, indent=2)+'\n', encoding='utf-8')
            run('desktop-ui', [str(executable)], env=env, timeout=180)
        report = json.loads((ui/'results.json').read_text(encoding='utf-8'))
        for field in ['spreadsheet_aggregates', 'worksheet_binding', 'structured_projection', 'worksheet_text', 'rectangular_clipboard', 'rectangular_selection', 'direct_paste', 'clipboard_focus', 'in_place_editing', 'clipboard_checklist', 'arrow_edit']:
            if not report.get(field, '').startswith('passed:'):
                raise RuntimeError(f'Desktop acceptance field failed: {field}: {report}')
        if report['status'] != 'passed' or report['remaining_charge'] != 0 or report['execution_mode'] != 'native-aot':
            raise RuntimeError(f'Published desktop did not prove Native AOT execution and cleanup: {report}')
        if Path(report['executable']).resolve() != executable.resolve():
            raise RuntimeError('The reported desktop executable is not the published image.')
        summary['startup_to_ready_ms'] = report['startup_to_ready_ms']
        for mode in ['clipboard-save', 'discard', 'clean']:
            peer = json.loads((ui/f'results.{mode}.json').read_text(encoding='utf-8'))
            if peer['status'] != 'passed' or peer['remaining_charge'] != 0 or peer['execution_mode'] != 'native-aot' or Path(peer['executable']).resolve() != executable.resolve():
                raise RuntimeError(f'Peer did not run the actual native image with zero-charge close: {peer}')
            if mode == 'clipboard-save' and not (peer['native_received'] and peer['cancel_checked']):
                raise RuntimeError('Native cross-process clipboard/Cancel-Save evidence missing')
        fixture = (ui/'results.pivot').read_bytes()
        if fixture[:8] != b'PIVOTWB5' or hashlib.sha256(fixture[48:]).digest() != fixture[16:48]:
            raise RuntimeError('Saved WB5 fixture is invalid')
        summary.update(status='passed', native_desktop_and_all_peers=True, adjacent_rust_child=True,
                       remaining_charge=report['remaining_charge'], publish_warnings=[])
    except (Exception, KeyboardInterrupt) as error:
        summary['error'] = str(error)
        print(f'Native AOT failed: {error}', flush=True)
    finally:
        (output/'summary.json').write_text(json.dumps(summary, indent=2)+'\n', encoding='utf-8')
        print(f"Native AOT {summary['status'].upper()}: {output/'summary.json'}", flush=True)
    return 0 if summary['status'] == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())
