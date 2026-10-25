"""Read-only validation of production boot/context adapters; never invokes GUI runners."""
import argparse
import datetime
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time
import uuid

import windows_attended_gui as wrapper
import windows_boot_continuity as policy
from run_headless_ci import read_json, sha, verify_files, write_json


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-root', type=Path, required=True)
    parser.add_argument('--return-root', type=Path)
    args = parser.parse_args()
    if sys.platform != 'win32': raise SystemExit('Windows only')
    parent = wrapper.local_tree(args.output_root)
    if not parent.is_dir(): raise SystemExit('Existing native output parent required')
    report = parent/('continuity-adapters-'+uuid.uuid4().hex); report.mkdir()
    files = ['windows_continuity_probe.py', 'windows_boot_continuity.py', 'windows_attended_gui.py',
             'guest_preflight.py', 'run_headless_ci.py']
    data = {'hostname': platform.node(), 'captured_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'GUI_executed': False, 'scope': 'production adapter measurement only; not execution approval',
            'helper_hashes': {name: sha(Path(__file__).parent/name) for name in files}, 'samples': []}
    first = None; first_identity = None
    for index in range(6):
        row = {}
        try:
            row['boot'] = policy.capture()
            row['desktop'] = wrapper.desktop_identity()
            if first is None: first = row['boot']
            row['within_collection'] = policy.compare(first, row['boot'])
            row['reviewed_boot'] = policy.compare(policy.REVIEWED_SAMPLE, row['boot'])
            if platform.node().lower() == 'windev2407eval':
                row['guest'] = wrapper.guest_identity()
                if first_identity is None: first_identity = row['guest']
                row['reviewed_guest'] = wrapper.reviewed_identity(row['guest'])
                row['guest_within_collection'] = wrapper.identity_continuity(first_identity, row['guest'])
        except (OSError, ValueError, TypeError, KeyError, subprocess.SubprocessError) as error:
            row['adapter_error'] = str(error)
        data['samples'].append(row)
        if index != 5: time.sleep(.2)
    write_json(report/'adapters.json', data)
    rows = [{'path': 'adapters.json', 'sha256': sha(report/'adapters.json')}]
    write_json(report/'origin-hashes.json', rows)
    print('Native original: '+str(report))
    if args.return_root:
        destination = args.return_root/report.name; destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copytree(report, destination); verify_files(destination, rows)
        write_json(destination/'transfer-receipt.json', {
            'status': 'Verified', 'native_manifest_sha256': sha(report/'origin-hashes.json'),
            'returned_manifest_sha256': sha(destination/'origin-hashes.json')})
        print('Returned: '+str(destination))
    print('Read-only measurements complete; no GUI consent or execution occurred.')


if __name__ == '__main__': main()
