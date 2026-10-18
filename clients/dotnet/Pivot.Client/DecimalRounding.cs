namespace Pivot.Client;

/// <summary>Pivot rounding modes. AwayFromZero is directed; MidpointAwayFromZero rounds to nearest.</summary>
public enum RoundingMode
{
    MidpointAwayFromZero = 0,
    ToEven,
    MidpointTowardZero,
    ToZero,
    AwayFromZero,
    ToNegativeInfinity,
    ToPositiveInfinity
}

/// <summary>Explicit decimal rounding shared with the Rust numeric API.</summary>
public static class DecimalRounding
{
    public static decimal Round(decimal value, int places = 0,
        RoundingMode mode = RoundingMode.MidpointAwayFromZero)
    {
        ArgumentOutOfRangeException.ThrowIfNegative(places);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(places, 28);
        return mode switch
        {
            RoundingMode.MidpointAwayFromZero => decimal.Round(value, places, MidpointRounding.AwayFromZero),
            RoundingMode.ToEven => decimal.Round(value, places, MidpointRounding.ToEven),
            RoundingMode.ToZero => decimal.Round(value, places, MidpointRounding.ToZero),
            RoundingMode.ToNegativeInfinity => decimal.Round(value, places, MidpointRounding.ToNegativeInfinity),
            RoundingMode.ToPositiveInfinity => decimal.Round(value, places, MidpointRounding.ToPositiveInfinity),
            RoundingMode.AwayFromZero => decimal.Round(value, places,
                value < 0 ? MidpointRounding.ToNegativeInfinity : MidpointRounding.ToPositiveInfinity),
            RoundingMode.MidpointTowardZero => RoundHalfTowardZero(value, places),
            _ => throw new ArgumentOutOfRangeException(nameof(mode))
        };
    }

    private static decimal RoundHalfTowardZero(decimal value, int places)
    {
        var truncated = decimal.Round(value, places, MidpointRounding.ToZero);
        decimal unit = 1;
        for (var i = 0; i < places; i++) unit /= 10;
        // Compare twice the remainder: half a unit at scale 28 is not representable.
        return decimal.Abs(value - truncated) * 2 <= unit
            ? truncated
            : decimal.Round(value, places, MidpointRounding.AwayFromZero);
    }
}
