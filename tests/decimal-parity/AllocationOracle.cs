using System.Globalization;
using System.Numerics;
internal static class AllocationOracle {
 static readonly BigInteger Maximum=(BigInteger.One<<96)-1;
 static (BigInteger C,int Scale) Canonical(BigInteger units,int scale){if(units.IsZero)return(BigInteger.Zero,0);while(scale>0&&units%10==0){units/=10;scale--;}if(units>Maximum)throw new OverflowException();return(units,scale);}
 public static void Run(string file){
  foreach(string line in File.ReadLines(file)){
   var f=line.Split('\t');string id=f[0];
   try{
    decimal total=decimal.Parse(f[1],CultureInfo.InvariantCulture);var bits=decimal.GetBits(total);int scale=(bits[3]>>16)&255;bool negative=bits[3]<0;
    var coefficient=(BigInteger)(uint)bits[0]+((BigInteger)(uint)bits[1]<<32)+((BigInteger)(uint)bits[2]<<64);
    var count=BigInteger.Parse(f[2],CultureInfo.InvariantCulture);var index=BigInteger.Parse(f[3],CultureInfo.InvariantCulture);var precision=BigInteger.Parse(f[4],CultureInfo.InvariantCulture);
    if(count<1||count>uint.MaxValue||index<1||index>count||precision<0||precision>28)throw new ArgumentException();
    int p=(int)precision;BigInteger numerator=coefficient*BigInteger.Pow(10,p),denominator=BigInteger.Pow(10,scale);var units=BigInteger.DivRem(numerator,denominator,out var fractional);if(!fractional.IsZero)throw new ArgumentException();
    var lower=BigInteger.DivRem(units,count,out var remaining);var low=Canonical(lower,p);var high=remaining.IsZero?low:Canonical(lower+1,p);int common=Math.Max(low.Scale,high.Scale);
    var a=low.C*BigInteger.Pow(10,common-low.Scale);var b=high.C*BigInteger.Pow(10,common-high.Scale);
    var whole=a*(count-remaining)+b*remaining;
    // Independently construct possible all-but-one subsets from multiplicities.
    var largest=whole-a;if(remaining>0)largest=BigInteger.Max(largest,whole-b);if(largest>Maximum)throw new OverflowException();
    var answer=index<=remaining?high:low;Console.WriteLine($"{id}\tok\t{answer.C}\t{(negative&&!answer.C.IsZero?1:0)}\t{answer.Scale}");
   }catch(OverflowException){Console.WriteLine(id+"\tOverflow");}catch(ArgumentException){Console.WriteLine(id+"\tInvalidArgument");}
  }
 }
}
