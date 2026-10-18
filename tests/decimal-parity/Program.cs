using System.Globalization;
using Pivot.Client;

CultureInfo.CurrentCulture = CultureInfo.InvariantCulture;
// API validation that cannot be expressed by unsigned Rust scale arguments.
try { DecimalRounding.Round(1, -1); throw new Exception("negative scale accepted"); }
catch (ArgumentOutOfRangeException) { }
try { DecimalRounding.Round(1, 0, (RoundingMode)999); throw new Exception("invalid mode accepted"); }
catch (ArgumentOutOfRangeException) { }
var native = args.Contains("--native");
foreach (var line in File.ReadLines(args[0]))
{
    if (line.StartsWith('#') || string.IsNullOrWhiteSpace(line)) continue;
    var fields = line.Split('\t');
    var id = fields[0];
    try
    {
        var a = decimal.Parse(fields[2], CultureInfo.InvariantCulture);
        var b = fields[3] == "-" ? 0 : decimal.Parse(fields[3], CultureInfo.InvariantCulture);
        var value = fields[1] switch
        {
            "add" => native ? a + b : DecimalArithmetic.Add(a, b),
            "sub" => native ? a - b : DecimalArithmetic.Subtract(a, b),
            "mul" => native ? a * b : DecimalArithmetic.Multiply(a, b),
            "div" => native ? a / b : DecimalArithmetic.Divide(a, b),
            "round" => DecimalRounding.Round(a, int.Parse(fields[3]), Enum.Parse<RoundingMode>(fields[4])),
            "default" => DecimalRounding.Round(a, int.Parse(fields[3])),
            "normalize" => DecimalArithmetic.Normalize(a),
            _ => a
        };
        // Numeric and representation comparisons are separate in the runner.
        var scale = (decimal.GetBits(value)[3] >> 16) & 255;
        Console.WriteLine($"{id}\tok\t{value}\t{scale}");
    }
    catch (DivideByZeroException) { Console.WriteLine($"{id}\tDivisionByZero"); }
    catch (OverflowException) { Console.WriteLine($"{id}\tOverflow"); }
    catch (ArgumentOutOfRangeException) { Console.WriteLine($"{id}\tInvalidScale"); }
}

