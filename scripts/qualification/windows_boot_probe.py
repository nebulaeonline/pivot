"""Read-only boot-measurement research. Never grants GUI authority or changes settings."""
import argparse
import ctypes
import datetime
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time
import uuid

from guest_preflight import filesystem

QUERY = r"""
$ErrorActionPreference='Stop'
$os=Get-CimInstance Win32_OperatingSystem
$result=[ordered]@{last_boot_utc=$os.LastBootUpTime.ToUniversalTime().ToString('o')}
try {
 $e=Get-WinEvent -FilterHashtable @{LogName='System';ProviderName='Microsoft-Windows-Kernel-General';Id=12} -MaxEvents 1 -ErrorAction Stop
 [xml]$xml=$e.ToXml()
 $start=$null
 foreach($d in $xml.Event.EventData.Data) { if($d.GetAttribute('Name') -eq 'StartTime') { $start=$d.InnerText } }
 $result.start_event=[ordered]@{provider=$e.ProviderName;record_id=$e.RecordId;event_id=$e.Id;time_utc=$e.TimeCreated.ToUniversalTime().ToString('o');start_time=$start;channel=$e.LogName}
} catch { $result.start_event_error=$_.Exception.Message }
$result | ConvertTo-Json -Depth 4 -Compress
"""


def sample():
    if sys.platform != 'win32': raise ValueError('Windows only')
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.GetTickCount64.restype = ctypes.c_ulonglong
    before = {'utc_unix': time.time(), 'uptime_ms': kernel.GetTickCount64()}
    result = subprocess.run(['powershell.exe', '-NoProfile', '-NonInteractive', '-Command', QUERY],
                            capture_output=True, text=True, timeout=30)
    after = {'utc_unix': time.time(), 'uptime_ms': kernel.GetTickCount64()}
    return {'before': before, 'after': after, 'query_exit_code': result.returncode,
            'query_stderr': result.stderr, 'measurement': json.loads(result.stdout) if result.returncode == 0 else None}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-root', type=Path, required=True)
    parser.add_argument('--return-root', type=Path)
    args = parser.parse_args()
    parent = args.output_root.resolve(strict=True)
    if sys.platform != 'win32' or not filesystem(parent).get('native_work_root'):
        raise SystemExit('Existing Windows native NTFS output directory required')
    report = parent/('boot-probe-'+uuid.uuid4().hex); report.mkdir()
    data = {'hostname': platform.node(), 'captured_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'scope': 'read-only six-sample investigation; no GUI authority', 'samples': [],
            'helper_sha256': {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                              for p in [Path(__file__), Path(__file__).parent/'guest_preflight.py']}}
    for index in range(6):
        try: data['samples'].append(sample())
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            data['samples'].append({'error': str(error)})
        if index != 5: time.sleep(0.2)
    (report/'measurements.json').write_text(json.dumps(data, indent=2)+'\n', encoding='utf-8')
    rows = [{'path': 'measurements.json', 'sha256': hashlib.sha256((report/'measurements.json').read_bytes()).hexdigest()}]
    (report/'origin-hashes.json').write_text(json.dumps(rows, indent=2)+'\n', encoding='utf-8')
    print('Native original: '+str(report))
    if args.return_root:
        destination = args.return_root/report.name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copytree(report, destination)
        assert hashlib.sha256((destination/'measurements.json').read_bytes()).hexdigest() == rows[0]['sha256']
        (destination/'transfer-receipt.json').write_text(json.dumps({
            'status': 'Verified', 'origin_manifest_sha256': hashlib.sha256((report/'origin-hashes.json').read_bytes()).hexdigest(),
            'returned_manifest_sha256': hashlib.sha256((destination/'origin-hashes.json').read_bytes()).hexdigest()}, indent=2)+'\n')
        print('Returned: '+str(destination))
    print('Collection complete; observations require review and grant no boot/GUI approval.')


if __name__ == '__main__': main()
