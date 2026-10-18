# Decimal compatibility checks

From the repository root, run:

```text
python tests/decimal-parity/verify.py
cargo test --workspace
```

Requires Python 3.8+, .NET 10 SDK, and the Rust toolchain. The script builds the
Rust comparison example and C# console harness; no additional Python packages or
.NET test framework packages are required. Generated TSV inputs use decimal text
and fixed-seed randomness. Failures return a nonzero exit status.

The 10,099 vectors exercise seven rounding modes, the default, invalid scale,
normalization, boundaries, signed arithmetic, recurring division, overflow,
division by zero, underflow, and precision loss. Python Decimal at precision 100
provides independent expectations for explicit rounding. Arithmetic compares
Rust against actual .NET execution, not Python arithmetic.

`results.txt` records Pivot value/error and scale agreement. The suite fails on
either kind of mismatch. `backend-baseline.txt` separately records unnormalized
native backend representation differences; these do not imply value differences.
Zero sign is deliberately excluded from the Pivot compatibility contract.

Coverage is not exhaustive. The suite checks individual operations, not all
possible arithmetic sequences, parsing parity, wire encodings, or AOT support.
Strict Rust literal parsing remains separate from permissive .NET decimal parsing;
all generated inputs are valid, representable Pivot decimal literals.
