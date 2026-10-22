# Historical workbook rejection fixtures

`wb1.hex` is the saved UI workbook from accepted Slice B native APFS validation
`macos-validation-20261020T111827Z`. It is retained only to prove the explicit
WB1 unsupported-version error after the accepted WB2 format cut. There is no WB1
reader or migration path. Original binary SHA-256: `08190ff216aeb17fcacc4ac85858cc4d307e9ff73f711dbcf8760731cda013c7`.

`wb2.hex` freezes `artifacts/rectangular-selection-windows/results.pivot` from the
accepted rectangular-selection implementation before ADR-0017. It proves explicit
WB2 rejection after the WB3 cut; there is no compatibility reader or migration.


`legacy-wb4-aggregates.hex` (hex encoding of the original .pivot bytes) is a genuine WB4 document written by the preserved
accepted ADR-0020 pivot-session.exe (SHA256
f462a385605a9c1c59d9eab0e04058d208dd8a6765a1f4f3c725089fae86f228).
Fixture SHA256: 8c407e9a8cf2bfecf57996d4dc296b5890dd40e934e5ea55bf20c2e3461630dc.
A1=2, A2=4, A3 literal Label, A4 explicitly cleared; B1=SUM(A1:A2),
B2=SUM(A1:A3), C1=A4, C2 rejected `=(`, D1 literal 001.20, D2 Invoice.
It validates bounded WB4 migration, strict SUM, blank-versus-zero and WB5 resave.
Do not regenerate it using the new writer or just replace a WB5 magic header.
