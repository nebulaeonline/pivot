# Native AOT baseline — implementation acceptance

Date received: 2026-10-20. Parent review supplied by the user.
**Disposition: Accepted. Native AOT Publish-and-Execute Validation is closed.**
[Implementation and cross-platform evidence](2026-10-20-native-aot-baseline.md).

The parent accepts actual native publication and execution on Windows x64, Linux
x64 and native macOS ARM64, including the separate Rust child, clipboard, WB3
Save/Open, rejected drafts, dirty-close choices and zero-charge shutdown. The
shared Windows/WSLg clipboard-suite serialization follow-up is also accepted as
complete. Ordinary managed development and CI remain available alongside
`python scripts/verify_native_aot.py`.

Retain named JSON DTOs and source-generated metadata, compiled XAML/bindings,
execution-aware helper launching and adjacent Rust discovery. Source generation
is the preferred strategy for future frontend JSON. Retain AOT as a regression
gate at milestone boundaries and when adding dependencies, serializers or
reflection-sensitive frontend features. Do not weaken trim/AOT diagnostics or
suppress warnings merely to pass. Future components need their own evidence.

Preserve all platform reports, source hashes, exact-image checks, inventories and
warning-free publish logs. Executable/payload sizes and single Main-to-ready
observations are baseline measurements, not controlled comparisons or deployment
guarantees. Windows/Linux all-feature counts are the retained source-equivalent
baseline; the current milestone reran their ordinary CI and native gate. Mac also
ran fresh strict all-feature lint/tests.

Clean-machine deployment, minimum OS/glibc support, installers, physical keyboard,
IME/accessibility/native-picker coverage under AOT, future dependencies and
power-loss durability remain unproven. Historical WSLg allocator abort,
Windows-mounted-path installation uncertainty and Linux viewport performance
exception remain recorded separately.

## Next proposed milestone: ordinary worksheet text

Prepare a focused semantic design for document-owned text cells, distinct from
computational string values. The intended bounded workflow is text entry, editing,
copy/paste and save/reopen, preserving exact-decimal, numeric formula and SUM
semantics unless separately reviewed. Native AOT compatibility remains required.

Stop for review before committing new worksheet-value semantics. This acceptance
does not approve a particular text representation, coercion rule or file-format
change. Structured-result projections, database connectors and permanent `#`
reference grammar are outside this milestone. Structured-result/projection design
with fixed typed records remains a subsequent direction.
