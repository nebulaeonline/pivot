"""Guest-local evidence bootstrap. Managed execution remains review-gated."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys
import traceback
import uuid


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class Tee:
    def __init__(self, terminal, log):
        self.terminal, self.log = terminal, log
    def write(self, value):
        self.log.write(value); self.log.flush()
        return self.terminal.write(value)
    def flush(self):
        self.log.flush(); self.terminal.flush()
    def isatty(self):
        return self.terminal.isatty()


def collect_native_report(text, code, parent, report, worker):
    marker = 'Native preflight report: '
    matches = [line.split(marker, 1)[1] for line in text.splitlines() if marker in line]
    errors = []
    if len(matches) == 1:
        try:
            native = Path(matches[0])
            if native.parent.parent != parent or native.name != 'report':
                raise ValueError('Unexpected native report path')
            worker.local_tree(native)
            worker.verify_files(native, worker.read_json(native/'origin-hashes.json'))
            shutil.copytree(native, report/'native-report')
        except (OSError, ValueError) as error:
            errors.append(str(error))
    elif code == 0:
        errors.append('Successful wrapper did not identify exactly one native report')
    return errors


def capture_status(run, code, errors):
    status = run['status'] if run else 'PreflightPassed' if code == 0 else 'Blocked'
    return 'Blocked' if errors and status != 'Failed' else status


def main():
    if sys.platform != 'win32':
        raise SystemExit('Windows guest only')
    bundle = Path(__file__).resolve().parent
    if len(sys.argv) not in (2, 3) or (len(sys.argv) == 3 and sys.argv[2] != '--run-managed'):
        raise SystemExit('Usage: launch_windows_managed.py RECEIPT_DIGEST [--run-managed]')
    expected = sys.argv[1]
    managed = len(sys.argv) == 3
    if sha(bundle/'bundle-receipt.json') != expected:
        raise SystemExit('Packet receipt mismatch; no helper imported')
    receipt = json.loads((bundle/'bundle-receipt.json').read_text())
    for row in receipt['files']:
        if Path(row['path']).name != row['path'] or sha(bundle/row['path']) != row['sha256']:
            raise SystemExit('Packet file mismatch; no helper imported')
    sys.path.insert(0, str(bundle))
    import windows_attended_gui as worker
    # This check precedes report acquisition as well as the wrapper's work.
    parent = worker.local_tree(Path('C:/PivotWork'))
    report = parent/('preflight-capture-'+uuid.uuid4().hex)
    report.mkdir()
    original_out, original_err = sys.stdout, sys.stderr
    code = 2
    with (report/'console.log').open('w', encoding='utf-8') as log:
        sys.stdout, sys.stderr = Tee(original_out, log), Tee(original_err, log)
        try:
            sys.argv = [str(bundle/'windows_attended_gui.py'), '--bundle-dir', str(bundle),
                        '--receipt-sha256', expected, '--work-root', str(parent)]
            if managed:
                sys.argv.append('--run-managed')
            code = worker.main()
        except (Exception, KeyboardInterrupt):
            traceback.print_exc()
        finally:
            sys.stdout, sys.stderr = original_out, original_err
    # Locate only the exact report path emitted by this invocation, not a latest
    # directory guess that could accidentally export another invocation's result.
    text = (report/'console.log').read_text(encoding='utf-8')
    # input() does not echo the typed answer into our log. Its prompt and the
    # next print can therefore share a line even when the terminal displayed Enter.
    export_errors = collect_native_report(text, code, parent, report, worker)
    run_path = report/'native-report/managed-run.json'
    run = worker.read_json(run_path) if run_path.exists() else None
    # Keep an executed failure even if collection also has a problem.
    status = capture_status(run, code, export_errors)
    for name in ['bundle-receipt.json', 'source-manifest.json', 'build-input-manifest.json']:
        shutil.copyfile(bundle/name, report/name)
    worker.write_json(report/'capture.json', {'status': status,
                      'exit_code': code, 'GUI_executed': None if run else False,
                      'GUI_evidence': 'Inspect actual application reports; runner invocation alone is not proof of GUI execution',
                      'packet_receipt_sha256': expected, 'export_errors': export_errors,
                      'native_capture': str(report), 'mode': 'owner-attended fixed managed' if managed else 'metadata only'})
    rows = [{'path': p.relative_to(report).as_posix(), 'sha256': sha(p)}
            for p in sorted(report.rglob('*')) if p.is_file()]
    worker.write_json(report/'origin-hashes.json', rows)
    destination = Path('J:/coding/pivot/artifacts/attended-guest-preflight/results/Pivot-Win11-x64')/report.name
    try:
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copytree(report, destination)
        worker.verify_files(destination, rows)
        worker.write_json(destination/'transfer-receipt.json', {
            'status': 'Verified', 'files_verified': len(rows),
            'native_manifest_sha256': sha(report/'origin-hashes.json'),
            'returned_manifest_sha256': sha(destination/'origin-hashes.json')})
        print('Returned report: '+str(destination))
    except (OSError, ValueError) as error:
        print('Transfer failed; do not rerun preflight. Native original: '+str(report))
        print(str(error))
        return 2
    return 2 if export_errors and code == 0 else code


if __name__ == '__main__':
    raise SystemExit(main())
