"""Independent Fraction/integer allocation oracle, plus .NET BigInteger parity."""
from fractions import Fraction
from pathlib import Path
import hashlib,json,random,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];MAX=(1<<96)-1

def exact(value):
 if not value:return (0,0,0)
 negative=int(value<0);value=abs(value);d=value.denominator;a=b=0
 while d%2==0:d//=2;a+=1
 while d%5==0:d//=5;b+=1
 assert d==1
 scale=max(a,b);c=value.numerator*2**(scale-a)*5**(scale-b)
 if scale>28 or c>MAX:raise OverflowError
 return c,negative,scale

def oracle(total,n,i,p):
 if not 1<=n<1<<32 or not 1<=i<=n or not 0<=p<=28:return ('InvalidArgument',)
 value=Fraction(total);unit=Fraction(1,10**p);units=abs(value)/unit
 if units.denominator!=1:return ('InvalidArgument',)
 q,r=divmod(units.numerator,n)
 try:
  small=exact(q*unit);large=small if not r else exact((q+1)*unit);scale=max(small[2],large[2]);quantum=Fraction(1,10**scale)
  # Exhaust possible subset counts in small groups; large groups use two complements.
  if n<=16:
   proper=[(a*q+b*(q+1))*unit for a in range(n-r+1) for b in range(r+1) if a+b<n]
  else:proper=[abs(value)-q*unit]+([abs(value)-(q+1)*unit] if r else [])
  if any(x/quantum>MAX for x in proper):raise OverflowError
  result=(q+(i<=r))*unit*(-1 if value<0 else 1);c,sign,s=exact(result)
  return ('ok',str(c),str(sign),str(s))
 except OverflowError:return ('Overflow',)

def plain(c,s):
 text=str(abs(c)).rjust(s+1,'0');return ('-' if c<0 else '')+(text[:-s]+'.'+text[-s:] if s else text)

def vectors():
 data=[]
 def add(t,n,i,p):data.append((str(t),n,i,p))
 for c in range(-40,41):
  for s in range(3):
   for n in range(1,10):
    for p in range(4):
     for i in range(1,n+1):add(plain(c,s),n,i,p)
 for c in [MAX,MAX-1,MAX-9,1,10,0]:
  for sign in [1,-1]:
   for s in [0,1,2,27,28]:
    for n in [1,2,3,10,11,16,65536,(1<<32)-1]:
     for p in [0,1,2,27,28]:
      for i in sorted({1,n,max(1,n//2)}):add(plain(c*sign,s),n,i,p)
 rng=random.Random(36)
 for _ in range(2500):
  c=rng.randrange(MAX+1)*rng.choice([-1,1]);s=rng.randrange(29);n=rng.choice([rng.randrange(1,17),rng.randrange(1,1<<32)]);p=rng.randrange(29)
  add(plain(c,s),n,rng.randrange(1,n+1),p)
 for n,i,p in [(0,1,2),(3,0,2),(3,4,2),(1<<32,1,2),(3,1,-1),(3,1,29),(-1,1,2)]:add('0',n,i,p)
 add('-0.00',3,1,2)
 return list(dict.fromkeys(data))

def verify(output,configuration='Release'):
 output=Path(output);output.mkdir(parents=True,exist_ok=True);rows=vectors();path=output/'allocation-vectors.tsv'
 text=''.join(f'a{k:06d}\t{t}\t{n}\t{i}\t{p}\n' for k,(t,n,i,p) in enumerate(rows));path.write_text(text,encoding='utf-8')
 def run(cmd):return subprocess.check_output(cmd,cwd=ROOT,text=True,encoding='utf-8').splitlines()
 subprocess.run(['cargo','build','--locked','--release','-p','pivot-types','--example','allocation_parity'],cwd=ROOT,check=True)
 meta=json.loads(subprocess.check_output(['cargo','metadata','--format-version','1','--no-deps'],cwd=ROOT,text=True))
 binary=Path(meta['target_directory'])/'release/examples'/('allocation_parity.exe' if sys.platform=='win32' else 'allocation_parity')
 rust=run([str(binary),str(path)]);net=run(['dotnet',f'tests/decimal-parity/bin/{configuration}/net10.0/DecimalParity.dll','--allocation',str(path)])
 (output/'allocation-rust.tsv').write_text('\n'.join(rust)+'\n');(output/'allocation-dotnet.tsv').write_text('\n'.join(net)+'\n')
 assert len(rust)==len(net)==len(rows)
 failures=[];expected=[]
 for k,(row,r,c) in enumerate(zip(rows,rust,net)):
  want=(f'a{k:06d}',)+oracle(*row);expected.append('\t'.join(want))
  if tuple(r.split('\t'))!=want or tuple(c.split('\t'))!=want:failures.append(dict(input=row,rust=r,dotnet=c,expected=want))
 (output/'allocation-expected.tsv').write_text('\n'.join(expected)+'\n')
 report=dict(status='passed' if not failures else 'failed',vectors=len(rows),input_sha256=hashlib.sha256(text.encode()).hexdigest(),failures=failures[:30],failure_count=len(failures),oracles='Python Fraction/subset enumeration and .NET BigInteger/multiplicities; production fixed-limb arithmetic independent')
 (output/'allocation-parity.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report),flush=True)
 if failures:raise AssertionError('Allocation parity mismatch')
if __name__=='__main__':verify(sys.argv[1],sys.argv[2] if len(sys.argv)>2 else 'Release')
