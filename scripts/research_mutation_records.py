"""Fresh-process G2 producer/oracle measurements; preserve all stdout and exact failures."""
import hashlib,json,statistics,subprocess,sys
from pathlib import Path
platform,binary,report=sys.argv[1:]
binary=Path(binary).resolve();report=Path(report).resolve();report.mkdir(parents=True,exist_ok=False)
rows=[]
for case in ('input','formula','range-input','range-formula','allocate'):
 for size in (32,256,768):
  for sample in range(3):
   result=subprocess.run([str(binary),str(size),case],capture_output=True,text=True,timeout=120)
   (report/f'{case}-{size}-{sample}.log').write_text(result.stdout+result.stderr,encoding='utf-8')
   assert result.returncode==0 and result.stdout.strip().endswith('FINISH 0'),(case,size,sample,result.returncode,result.stdout,result.stderr)
   data=json.loads(next(s for s in result.stdout.splitlines() if s.startswith('{')))
   rows.append(dict(data,sample=sample))
  print(platform,case,size,'passed',flush=True)
summary=[]
for case in ('input','formula','range-input','range-formula','allocate'):
 for size in (32,256,768):
  group=[r for r in rows if r['case']==case and r['size']==size]
  summary.append(dict(case=case,size=size,**{k:statistics.median(r[k] for r in group) for k in group[0] if k not in ('case','size','sample')}))
(report/'samples.json').write_text(json.dumps(rows,indent=2)+'\n')
(report/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
(report/'worker.json').write_text(json.dumps({'platform':platform,'binary':str(binary),'sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'processes':len(rows),'scope':'isolated fresh process; independent calculation capture held; complete oracle measured outside producer; requested domain charge, not RSS'},indent=2)+'\n')
print(len(rows),'fresh-process zero-charge samples')
