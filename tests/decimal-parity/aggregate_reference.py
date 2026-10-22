"""Independent Fraction reference corpus; no production reducer code is used."""
from fractions import Fraction
from pathlib import Path
import random,sys
MAX=(1<<96)-1
ROOT=Path(__file__).resolve().parents[2]
def text(c,s):
 sign='-' if c<0 else '';v=str(abs(c)).zfill(s+1)
 return sign+(v[:-s]+'.'+v[-s:] if s else v)
def average(values):
 total=sum(map(Fraction,values));count=len(values)
 for scale in range(28,-1,-1):
  rational=abs(total)*10**scale/count
  quotient,remainder=divmod(rational.numerator,rational.denominator)
  quotient+=int(2*remainder>=rational.denominator)
  if quotient<=MAX:
   result=text(-quotient if total<0 else quotient,scale)
   return result.rstrip('0').rstrip('.') if '.' in result else result
 raise AssertionError('finite average outside Decimal96')
def corpus():
 rng=random.Random(210021)
 cases=[['1','2','2'],[str(MAX),str(MAX)],[str(-MAX),str(-MAX)],[str(MAX),str(-MAX),'1'],['0',text(1,28)],['0',text(-1,28)]]
 for _ in range(240):
  n=rng.randint(1,20);values=[text(rng.randint(-MAX,MAX),rng.randint(0,28)) for _ in range(n)]
  cases.extend([values,list(reversed(values))])
 for scale in range(29):
  for sign in [-1,1]:cases.append([text(sign*MAX,scale),text(sign*(MAX-1),scale)])
 return ''.join(','.join(v)+'\t'+average(v)+'\n' for v in cases)
if __name__=='__main__':
 path=ROOT/'crates/pivot-core/tests/fixtures/average-rational.tsv';expected=corpus()
 if '--check' in sys.argv:assert path.read_text()==expected,'rational reference corpus differs'
 else:path.parent.mkdir(parents=True,exist_ok=True);path.write_text(expected,encoding='utf-8',newline='\n')
 print('Independent rational cases:',len(expected.splitlines()))
