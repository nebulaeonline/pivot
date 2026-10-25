"""Reviewed Windows boot continuity policy. Not independent GUI authorization."""
import ctypes
from datetime import datetime
import json
import math
import subprocess
import sys
import time

# Pivot policy budgets, not Windows guarantees.
CIM_VARIATION_SECONDS = 0.1
ELAPSED_DRIFT_SECONDS = 2.0

# Reviewed first guest research observation. Static baseline is evidence only;
# callers must obtain live measurements and separately establish execution context.
REVIEWED_SAMPLE = {
    'before': {'utc_unix': 1792939715.416204, 'uptime_ms': 36653703},
    'after': {'utc_unix': 1792939717.9796224, 'uptime_ms': 36656265},
    'query_exit_code': 0,
    'measurement': {
        'last_boot_utc': '2026-10-25T04:37:42.0932210Z',
        'start_event': {'provider': 'Microsoft-Windows-Kernel-General', 'record_id': 2910,
                        'event_id': 12, 'time_utc': '2026-10-25T04:37:38.8635211Z',
                        'start_time': '2026-10-25T04:37:38.5000000Z', 'channel': 'System'}}}

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


def capture():
    if sys.platform != 'win32': raise ValueError('Windows boot adapter unavailable')
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.GetTickCount64.restype = ctypes.c_ulonglong
    before = {'utc_unix': time.time(), 'uptime_ms': kernel.GetTickCount64()}
    run = subprocess.run(['powershell.exe', '-NoProfile', '-NonInteractive', '-Command', QUERY],
                         capture_output=True, text=True, timeout=30)
    after = {'utc_unix': time.time(), 'uptime_ms': kernel.GetTickCount64()}
    try: measurement = json.loads(run.stdout) if run.returncode == 0 else None
    except ValueError: measurement = None
    return {'before': before, 'after': after, 'query_exit_code': run.returncode,
            'query_stderr': run.stderr, 'measurement': measurement}


def instant(value):
    if not isinstance(value, str): raise ValueError('Missing timestamp')
    parsed = datetime.fromisoformat(value)
    if parsed.tzinfo is None: raise ValueError('Timezone required')
    stamp = parsed.timestamp()
    if not math.isfinite(stamp): raise ValueError('Invalid timestamp')
    return stamp


def validate(sample):
    if not isinstance(sample, dict) or type(sample.get('query_exit_code')) is not int or sample['query_exit_code'] != 0:
        raise ValueError('Measurement query failed')
    measurement = sample['measurement']; event = measurement['start_event']
    if (event['provider'] != 'Microsoft-Windows-Kernel-General' or event['channel'] != 'System'
            or type(event['event_id']) is not int or event['event_id'] != 12
            or type(event['record_id']) is not int or event['record_id'] <= 0):
        raise ValueError('Invalid startup event')
    for key in ['time_utc', 'start_time']: instant(event[key])
    instant(measurement['last_boot_utc'])
    for boundary in ['before', 'after']:
        point = sample[boundary]
        if type(point['uptime_ms']) is not int or point['uptime_ms'] < 0: raise ValueError('Invalid uptime')
        if type(point['utc_unix']) not in (int, float) or not math.isfinite(point['utc_unix']):
            raise ValueError('Invalid wall time')
    if sample['after']['uptime_ms'] < sample['before']['uptime_ms']:
        raise ValueError('Uptime reset within measurement')
    if sample['after']['utc_unix'] < sample['before']['utc_unix']:
        raise ValueError('Clock regressed within measurement')
    return measurement


def compare(baseline, current):
    """D=abs(delta wall-delta uptime), separately per query and across observations."""
    diagnostics = {'policy': {'CIM_variation_seconds': CIM_VARIATION_SECONDS,
                               'elapsed_discrepancy_seconds': ELAPSED_DRIFT_SECONDS}, 'intervals': []}
    try:
        old, new = validate(baseline), validate(current)
        if old['start_event'] != new['start_event']:
            return dict(diagnostics, status='Blocked', classification='anchor-mismatch', reason='Startup-event anchor changed; boot/log continuity not established')
        variation = abs((datetime.fromisoformat(old['last_boot_utc'])-
                         datetime.fromisoformat(new['last_boot_utc'])).total_seconds())
        diagnostics['CIM_variation_seconds'] = variation
        if variation > CIM_VARIATION_SECONDS:
            raise ValueError('CIM variation exceeds policy; continuity uncertain, not proof of reboot')
        # Check each query and the cross-observation interval independently.
        for kind, first, last in [('baseline-query', baseline['before'], baseline['after']),
                                  ('current-query', current['before'], current['after']),
                                  ('baseline-to-current', baseline['before'], current['before'])]:
            elapsed_wall = last['utc_unix']-first['utc_unix']
            elapsed_tick = (last['uptime_ms']-first['uptime_ms'])/1000
            discrepancy = abs(elapsed_wall-elapsed_tick)
            diagnostics['intervals'].append({'kind': kind, 'wall_seconds': elapsed_wall,
                                              'uptime_seconds': elapsed_tick, 'discrepancy_seconds': discrepancy})
            if elapsed_wall < 0 or elapsed_tick < 0 or discrepancy > ELAPSED_DRIFT_SECONDS:
                raise ValueError(kind+': uptime/wall continuity uncertain')
        return dict(diagnostics, status='Verified', classification='corroborated-continuity')
    except (KeyError, TypeError, ValueError, OverflowError) as error:
        return dict(diagnostics, status='Blocked', classification='continuity-uncertain', reason=str(error))
