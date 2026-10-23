# Historical workbook rejection fixtures

`wb1.hex` is the saved UI workbook from accepted Slice B native APFS validation
`macos-validation-20261020T111827Z`. It is retained only to prove the explicit
WB1 unsupported-version error after the accepted WB2 format cut. There is no WB1
reader or migration path. Original binary SHA-256: `08190ff216aeb17fcacc4ac85858cc4d307e9ff73f711dbcf8760731cda013c7`.

`wb2.hex` freezes `artifacts/rectangular-selection-windows/results.pivot` from the
accepted rectangular-selection implementation before ADR-0017. It proves explicit
WB2 rejection after the WB3 cut; there is no compatibility reader or migration.


WB4 migration is no longer supported. The previous genuine fixture and its
provenance remain in historical aggregate-validation artifacts. Current rejection
tests use the minimal `PIVOTWB4` header, and current invariants use WB5 documents.
