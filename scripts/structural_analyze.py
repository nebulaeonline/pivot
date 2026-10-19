"""Normalize structural experiment CSVs; keep latency and allocation runs distinct."""
import argparse
import csv
import json
from pathlib import Path


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory',type=Path)
    args=parser.parse_args()
    summary=json.loads((args.directory/'summary.json').read_text(encoding='utf-8'))
    rows=[]
    for run in summary['runs']:
        mode='profile' if '--profile' in run['command'] else 'latency'
        for row in csv.reader((args.directory/run['file']).read_text(encoding='utf-8').splitlines()):
            if not row or row[0]!='ROW':continue
            item=dict(zip(['implementation','cells','shape','operation','samples','p50_ns','p95_ns','worst_ns','allocated_mean','calls_mean','logical_copy_mean','retained_delta','peak_extra'],row[1:]))
            for key in set(item)-{'implementation','shape','operation'}:item[key]=int(item[key])
            item.update(mode=mode,file=run['file'],process_peak_rss=run['peak_rss_bytes'],exit_code=run['exit_code'])
            if mode=='latency':
                for key in ['allocated_mean','calls_mean','logical_copy_mean','retained_delta','peak_extra']:item[key]=None
            rows.append(item)
    (args.directory/'metrics.json').write_text(json.dumps(rows,indent=2)+'\n',encoding='utf-8')
    lines=['# Structural experiment results','','All latency times are milliseconds. Construction/retention/cleanup and any --samples 1 run are single diagnostics, not distribution evidence. Process RSS includes construction/import and retention.','','| Cells | Shape | Operation | Implementation | Samples | p50 | p95 | Worst |','| --- | --- | --- | --- | --- | --- | --- | --- |']
    for r in rows:
        if r['mode']=='latency':lines.append(f"| {r['cells']} | {r['shape']} | {r['operation']} | {r['implementation']} | {r['samples']} | {r['p50_ns']/1e6:.6f} | {r['p95_ns']/1e6:.6f} | {r['worst_ns']/1e6:.6f} |")
    lines+=['','## Separate process peaks','','| Process | Exit | Peak RSS MiB | Guard stop |','| --- | --- | --- | --- |']
    for r in summary['runs']:lines.append(f"| {r['file']} | {r['exit_code']} | {r['peak_rss_bytes']/1048576:.2f} | {r['budget_stop'] or ''} |")
    (args.directory/'analysis.md').write_text('\n'.join(lines)+'\n',encoding='utf-8',newline='\n')


if __name__=='__main__':main()
