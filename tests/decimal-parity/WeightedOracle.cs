using System.Globalization;
using System.Numerics;
internal static class WeightedOracle
{
    private static readonly BigInteger Maximum=(BigInteger.One<<96)-1;
    private static (BigInteger Coefficient,int Scale) Parts(decimal value)
    {
        int[] b=decimal.GetBits(value);
        BigInteger c=(BigInteger)(uint)b[0]+((BigInteger)(uint)b[1]<<32)+((BigInteger)(uint)b[2]<<64);
        return (b[3]<0?-c:c,(b[3]>>16)&255);
    }
    private static (BigInteger Coefficient,int Scale) Canonical(BigInteger value,int scale)
    {
        if(value.IsZero)return(BigInteger.Zero,0);
        while(scale>0&&value%10==0){value/=10;scale--;}
        if(value>Maximum)throw new OverflowException();
        return(value,scale);
    }
    public static void Run(string path)
    {
        foreach(string line in File.ReadLines(path))
        {
            string[] f=line.Split('\t');
            try
            {
                decimal total=decimal.Parse(f[1],CultureInfo.InvariantCulture);
                decimal index=decimal.Parse(f[2],CultureInfo.InvariantCulture);
                decimal precision=decimal.Parse(f[3],CultureInfo.InvariantCulture);
                decimal[] weights=f[4].Length==0?[]:f[4].Split(',').Select(s=>decimal.Parse(s,CultureInfo.InvariantCulture)).ToArray();
                if(weights.Length is <1 or >64 || index<1 || index>weights.Length || decimal.Truncate(index)!=index || precision<0 || precision>28 || decimal.Truncate(precision)!=precision || weights.Any(w=>w<0))throw new ArgumentException();
                var tp=Parts(total);int p=(int)precision;
                BigInteger units=BigInteger.DivRem(BigInteger.Abs(tp.Coefficient)*BigInteger.Pow(10,p),BigInteger.Pow(10,tp.Scale),out var fractional);
                if(!fractional.IsZero)throw new ArgumentException();
                var wp=weights.Select(Parts).ToArray();int commonWeight=wp.Max(w=>w.Scale);
                BigInteger[] numerators=wp.Select(w=>w.Coefficient*BigInteger.Pow(10,commonWeight-w.Scale)).ToArray();
                BigInteger denominator=numerators.Aggregate(BigInteger.Zero,(a,b)=>a+b);
                if(denominator.IsZero)throw new ArgumentException();
                BigInteger[] floors=new BigInteger[weights.Length],remainders=new BigInteger[weights.Length];
                for(int j=0;j<weights.Length;j++)floors[j]=BigInteger.DivRem(units*numerators[j],denominator,out remainders[j]);
                int residual=(int)(units-floors.Aggregate(BigInteger.Zero,(a,b)=>a+b));
                foreach(int j in Enumerable.Range(0,weights.Length).Where(j=>weights[j]>0).OrderByDescending(j=>remainders[j]).ThenBy(j=>j).Take(residual))floors[j]++;
                var settled=floors.Select(v=>Canonical(v,p)).ToArray();int scale=settled.Max(v=>v.Scale);
                BigInteger[] coefficients=settled.Select(v=>v.Coefficient*BigInteger.Pow(10,scale-v.Scale)).ToArray();
                var positive=coefficients.Where(v=>v>0).ToArray();
                if(positive.Length>0&&coefficients.Aggregate(BigInteger.Zero,(a,b)=>a+b)-positive.Min()>Maximum)throw new OverflowException();
                var answer=settled[(int)index-1];
                Console.WriteLine($"{f[0]}\tok\t{answer.Coefficient}\t{(total<0&&!answer.Coefficient.IsZero?1:0)}\t{answer.Scale}");
            }
            catch(OverflowException){Console.WriteLine(f[0]+"\tOverflow");}
            catch(ArgumentException){Console.WriteLine(f[0]+"\tInvalidArgument");}
        }
    }
}
