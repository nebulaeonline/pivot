"""Range adapter parity through real workbook binding; independent numeric oracles."""
from pathlib import Path
import hashlib,json,subprocess,sys
from weighted_reference import vectors,oracle,ROOT

def verify(output,configuration='Release'):
 output=Path(output);all_rows=[r for r in vectors() if 1<=len(r[3])<=64]
 rows=list(dict.fromkeys(all_rows[::23]+all_rows[-1500:]))
 text=''.join(f'r{k:06d}\t{t}\t{i}\t{p}\t{",".join(ws)}\n' for k,(t,i,p,ws) in enumerate(rows))
 path=output/'range-weighted-vectors.tsv';path.write_text(text,encoding='utf-8')
 subprocess.run(['cargo','build','--locked','--release','-p','pivot-workbook','--example','weighted_range_parity'],cwd=ROOT,check=True)
 meta=json.loads(subprocess.check_output(['cargo','metadata','--format-version','1','--no-deps'],cwd=ROOT,text=True))
 binary=Path(meta['target_directory'])/'release/examples'/('weighted_range_parity.exe' if sys.platform=='win32' else 'weighted_range_parity')
 def run(cmd):return subprocess.check_output(cmd,cwd=ROOT,text=True,encoding='utf-8').splitlines()
 rust=run([str(binary),str(path)]);net=run(['dotnet',f'tests/decimal-parity/bin/{configuration}/net10.0/DecimalParity.dll','--weighted',str(path)])
 assert len(rust)==len(net)==len(rows);failures=[];expected=[]
 for k,(row,r,c) in enumerate(zip(rows,rust,net)):
  want=(f'r{k:06d}',)+oracle(*row);expected.append('\t'.join(want))
  if tuple(r.split('\t'))!=want or tuple(c.split('\t'))!=want:failures.append(dict(input=row,rust=r,dotnet=c,expected=want))
 for name,lines in [('rust',rust),('dotnet',net),('expected',expected)]:
  (output/('range-weighted-'+name+'.tsv')).write_text('\n'.join(lines)+'\n',encoding='utf-8')
 report=dict(status='passed' if not failures else 'failed',vectors=len(rows),input_sha256=hashlib.sha256(text.encode()).hexdigest(),failure_count=len(failures),failures=failures[:20],oracles='Python Fraction/subsets and .NET BigInteger; Rust uses literal worksheet ranges')
 (output/'range-weighted-parity.json').write_text(json.dumps(report,indent=2));print(json.dumps(report),flush=True)
 if failures:raise AssertionError('Range weighted parity mismatch')
if __name__=='__main__':verify(sys.argv[1],sys.argv[2] if len(sys.argv)>2 else 'Release')
