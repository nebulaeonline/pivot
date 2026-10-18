using System.Globalization;

namespace Pivot.Client;

/// <summary>Pivot arithmetic returns canonical decimals, without trailing fractional zeros.
/// Use DecimalRounding separately for explicit quantization. Intermediate precision reduction
/// retains System.Decimal behavior.</summary>
public static class DecimalArithmetic
{
    public static decimal Add(decimal left, decimal right) => Normalize(left + right);
    public static decimal Subtract(decimal left, decimal right) => Normalize(left - right);
    public static decimal Multiply(decimal left, decimal right) => Normalize(left * right);
    public static decimal Divide(decimal left, decimal right) => Normalize(left / right);

    public static decimal Normalize(decimal value) => value == 0 ? 0 : decimal.Parse(
        value.ToString("G29", CultureInfo.InvariantCulture),
        NumberStyles.Float, CultureInfo.InvariantCulture);
}
