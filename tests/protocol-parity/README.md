# Protocol interoperability checks

Run `python tests/protocol-parity/verify.py` from the repository root.
Rust and C# types are generated from the same proposed proto3 schema. Binary
fixtures cover requests, values, public calculation errors, and 58 .NET decimal
layouts covering every scale and both signs. Artifacts go under
`artifacts/protocol-parity/` or `--output-dir`.

Rust validation tests run through `cargo test -p pivot-protocol`. The C# harness
tests generated codecs, not a second complete semantic validator. Native AOT
publish/run, daemon, registry, transport, and hosting are not implemented.
