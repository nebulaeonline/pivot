"""Exact Fraction oracle with bounded subset checks, independent of production limbs."""
from fractions import Fraction
from itertools import product
from pathlib import Path
import hashlib,json,random,subprocess,sys
from allocation_reference import exact,plain,MAX
ROOT=Path(__file__).resolve().parents[2]
def oracle(total,index,p,weights):
 try:
  index=Fraction(index);p=Fraction(p);weights=list(map(Fraction,weights));n=len(weights)
  if not 1<=n<=64 or index.denominator!=1 or not 1<=index<=n or p.denominator!=1 or not 0<=p<=28 or any(w<0 for w in weights) or sum(weights)==0:return ('InvalidArgument',)
  p=int(p);index=int(index);total=Fraction(total);u=abs(total)*10**p
  if u.denominator!=1:return ('InvalidArgument',)
  quota=[u*w/sum(weights) for w in weights];shares=[q.numerator//q.denominator for q in quota]
  residual=u.numerator-sum(shares)
  ranked=sorted(range(n),key=lambda j:(-(quota[j]-shares[j]),j))
  for j in ranked[:residual]:shares[j]+=1
  settled=[Fraction(v,10**p) for v in shares];canonical=[exact(v) for v in settled];scale=max(v[2] for v in canonical)
  if n<=8:
   partials=[sum(v for j,v in enumerate(settled) if mask>>j&1) for mask in range(1<<n)]
   proper=[v for v in partials if v<abs(total)]
  else:proper=[abs(total)-v for v in settled if v>0]
  if any(v*10**scale>MAX for v in proper):raise OverflowError
  c,sign,s=exact(settled[index-1]*(-1 if total<0 else 1));return ('ok',str(c),str(sign),str(s))
 except OverflowError:return ('Overflow',)
def vectors():
 rows=[]
 def add(t,ws,p):
  for i in range(1,len(ws)+1):rows.append((str(t),str(i),str(p),tuple(map(str,ws))))
 for n in range(1,5):
  for weights in product(range(3),repeat=n):
   for units in range(-4,5):
    for p in [0,2]:add(plain(units,p),weights,p)
 for total in [str(MAX),str(MAX-9),'10','-10','8','0','0.0000000000000000000000000001']:
  for ws in [[1],[0,1,0],[1,1,1,0],[1,2,3],[str(MAX),'0.0000000000000000000000000001'],[str(MAX)]*64,[1]*64]:
   for p in [0,1,2,28]:add(total,ws,p)
 for ws in [[1,2,3],['0.1','0.2','0.3'],[10,20,30]]:
  for total in ['10','-10','0.05']:
   for p in [0,2,28]:add(total,ws,p)
 for total in ['4','5']:add(total,[5,3,1],0)
 rng=random.Random(37)
 for _ in range(500):
  n=rng.choice([1,2,3,8,16,64]);ws=[plain(rng.randrange(MAX+1),rng.randrange(29)) for _ in range(n)];t=plain(rng.randrange(MAX+1)*rng.choice([-1,1]),rng.randrange(29));p=rng.randrange(29)
  for i in sorted({1,n,rng.randrange(1,n+1)}):rows.append((t,str(i),str(p),tuple(ws)))
 for i,p,ws in [('0','2',('1',)),('2','2',('1',)),('1.5','2',('1',)),('1','-1',('1',)),('1','29',('1',)),('1','2',()),('1','2',('1',)*65),('1','2',('-1','1'))]:rows.append(('0',i,p,ws))
 return list(dict.fromkeys(rows))
def verify(output,configuration='Release'):
 output=Path(output);output.mkdir(parents=True,exist_ok=True);rows=vectors();path=output/'weighted-vectors.tsv'
 text=''.join(f'w{k:06d}\t{t}\t{i}\t{p}\t{",".join(ws)}\n' for k,(t,i,p,ws) in enumerate(rows));path.write_text(text,encoding='utf-8')
 subprocess.run(['cargo','build','--locked','--release','-p','pivot-core','--example','weighted_parity'],cwd=ROOT,check=True)
 meta=json.loads(subprocess.check_output(['cargo','metadata','--format-version','1','--no-deps'],cwd=ROOT,text=True));binary=Path(meta['target_directory'])/'release/examples'/('weighted_parity.exe' if sys.platform=='win32' else 'weighted_parity')
 def run(command):return subprocess.check_output(command,cwd=ROOT,text=True,encoding='utf-8').splitlines()
 rust=run([str(binary),str(path)]);net=run(['dotnet',f'tests/decimal-parity/bin/{configuration}/net10.0/DecimalParity.dll','--weighted',str(path)])
 assert len(rust)==len(net)==len(rows);failures=[];expected=[]
 for k,(row,r,c) in enumerate(zip(rows,rust,net)):
  want=(f'w{k:06d}',)+oracle(*row);expected.append('\t'.join(want))
  if tuple(r.split('\t'))!=want or tuple(c.split('\t'))!=want:failures.append(dict(input=row,rust=r,dotnet=c,expected=want))
 for name,lines in [('rust',rust),('dotnet',net),('expected',expected)]: (output/('weighted-'+name+'.tsv')).write_text('\n'.join(lines)+'\n',encoding='utf-8')
 report=dict(status='passed' if not failures else 'failed',vectors=len(rows),input_sha256=hashlib.sha256(text.encode()).hexdigest(),failure_count=len(failures),failures=failures[:20],oracles='Python Fraction/subsets and .NET BigInteger/LINQ ranking; independent of production fixed limbs')
 (output/'weighted-parity.json').write_text(json.dumps(report,indent=2));print(json.dumps(report),flush=True)
 if failures:raise AssertionError('Weighted parity mismatch')
if __name__=='__main__':verify(sys.argv[1],sys.argv[2] if len(sys.argv)>2 else 'Release')
