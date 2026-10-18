# Initial decimal and expression interfaces

## Decimal boundary

`PivotDecimal` owns a private `rust_decimal::Decimal`. The dependency is used
with default features disabled and only `std` enabled. No serialization format,
float conversion, or backend types enter the public interface.

Text syntax is `[+-]?[0-9]+(\.[0-9]+)?`. Whitespace, exponent notation, separators,
and missing digits around a decimal point are rejected. Parsing uses the backend's
exact parser so excess precision is rejected rather than rounded. Values use a
96-bit significand and scale 0–28. Formatting preserves backend scale; equality
compares numeric value, so `1.0` equals `1`. Sign of zero is not a promised API.

Checked operations return `Result`; division by any numeric zero returns
`DivisionByZero`, and nonzero arithmetic that exceeds range returns `Overflow`.
Parsing errors are `InvalidLiteral`, including values outside representable range.

Arithmetic results are now canonicalized on both Rust and C#: trailing fractional
zeros are removed and zero has scale zero. Parsed literals keep their scale;
call `normalize()` explicitly to canonicalize them. Native backend scale differences
are characterized in `tests/decimal-parity/backend-baseline.txt`.

Arithmetic precision reduction and underflow currently inherit the locked backend
implementations. For example, `1 / 3` produces
28 fractional digits and `1e-28 * 0.1` underflows to zero (the exponent here is
descriptive notation, not supported input syntax). These are provisional backend
behaviors, not a final arbitrary-precision Pivot policy. Checked arithmetic does
not mean exact arithmetic or an error on every loss of precision. The shared
10,099-case suite currently agrees on values, errors, and normalized scales; this
is bounded evidence, not a proof of universal .NET parity.

## Explicit rounding

Rust exposes `round(places)` and `round_with_mode(places, mode)`; C# exposes
`DecimalRounding.Round(value, places, mode)`. Decimal places must be 0–28.
The default is nearest, midpoint away from zero: `2.499999 -> 2`, `2.5 -> 3`,
and `-2.5 -> -3`. This differs from the default of `decimal.Round` in .NET.

| Mode | Behavior | 2.5 | -2.5 |
| --- | --- | --- | --- |
| ToEven | Nearest; midpoint to even | 2 | -2 |
| MidpointAwayFromZero | Nearest; midpoint away from zero (default) | 3 | -3 |
| MidpointTowardZero | Nearest; midpoint toward zero | 2 | -2 |
| ToZero | Truncate | 2 | -2 |
| AwayFromZero | Directed away from zero for any discarded fraction | 3 | -3 |
| ToNegativeInfinity | Floor | 2 | -3 |
| ToPositiveInfinity | Ceiling | 3 | -2 |

Rounding reduces scale when necessary; it does not pad zeros to a requested scale.
Modes apply to explicit quantization, not the internal precision reduction of
arithmetic. Repeated rounding can introduce double rounding; round deliberately
at the required business boundary. Display formatting remains separate.

C# arithmetic uses `DecimalArithmetic.Add/Subtract/Multiply/Divide/Normalize`.
Direct C# decimal operators still have native .NET scale behavior. The C# helper
is a reference/interop utility; authoritative expression evaluation remains Rust.
Its normalization currently uses invariant decimal text, avoiding float conversion;
it has not been optimized.

Reference semantics: [Microsoft MidpointRounding](https://learn.microsoft.com/en-us/dotnet/api/system.midpointrounding?view=net-10.0)
and [rust_decimal rounding strategies](https://docs.rs/rust_decimal/1.43.0/rust_decimal/enum.RoundingStrategy.html).

## Expression grammar

```text
expression = sum
sum        = product (("+" | "-") product)*
product    = unary (("*" | "/") unary)*
unary      = ("+" | "-") unary | primary
primary    = decimal | identifier | "(" expression ")"
decimal    = digit+ ("." digit+)?
identifier = (ASCII-letter | "_") (ASCII-letter | digit | "_")*
```

ASCII whitespace is allowed between tokens. Unary signs bind most tightly.
All numeric literals become `PivotDecimal`, without float intermediates.
Parse errors include a byte offset. A complete input must be consumed.
Inputs are limited to 4096 UTF-8 bytes and 64 recursive expression levels.

This is an initial grammar, not the final formula or REPL language. No leading
equals sign, assignment, calls, strings, booleans, cell/range syntax, or name
resolution is implemented. Identifiers such as `A1` are ordinary names here.
Parsing `Net / Revenue` does not evaluate it or check whether the names exist.

## Open decisions

- Expand division precision and underflow vectors before freezing internal
  precision-reduction policy. Explicit rounding default is owner-requested.
- Extend values and expression syntax only with a specified use case.
- Define source spans and configurable limits if later editing/diagnostics need them.
