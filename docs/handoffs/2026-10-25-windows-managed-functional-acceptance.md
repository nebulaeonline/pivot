# First Windows managed run — functional evidence accepted

2026-10-25 parent decision, attachment
`5b664141-bf81-4b74-aa63-5a91e5bdadcf/Pasted text.txt`.
[Original result and provenance](2026-10-25-windows-managed-first-run.md).

| Finding | Decision |
| --- | --- |
| Managed functional assertions | Passed; accepted |
| Actual main and three independent GUI peers | Verified; accepted |
| Main backend | Avalonia.Win32.WindowImpl; verified |
| Peer backend fields | Missing |
| Overall managed acceptance | Blocked/2; not closed |

Accept guest session 1 for all GUI processes, zero charges, unchanged source,
matching preflight/post identity, empty scoped survivor check and reported hash
reconciliation. Preserve original packets/reports, including 27 outer and 21
overlapping nested hash entries. One-run authorization is consumed. Do not relabel
the overall capture Passed or backfill historical peer fields.

## Authorized correction and preparation

Change only backend capture timing in ClipboardPeerAsync: capture the actual
corresponding window PlatformImpl type while alive, retain it, and serialize that
value through the existing Closed report path. Keep the schema, clipboard checks,
close timing, process/session diagnostics, cleanup and resource accounting intact.
Do not infer backend from the main application, platform, environment or a literal.

Add non-GUI lifecycle regression coverage showing the live captured value remains
available after close, and that missing live evidence still cannot pass. Run
appropriate non-GUI checks and a warning-free managed build; retain first failures.
Prepare a new immutable managed-only packet and verified source/input hashes with
the minimal delta from the original enabled execution packet documented. Keep
fixed entry point, native NTFS roots, interactive broker, owner CONFIRM and strict
evidence requirements. No unrelated product/UI changes.

Return correction, tests, build, source diff and frozen packet for parent review.
**No positive diagnostic-confirmation run is authorized yet.** Subsequent explicit
approval may permit one attended run proving main and all three peer backends,
with real windows, unchanged assertions, zero charges, native-origin evidence and
current VM/session checks. Do not reuse the consumed first-run approval.

No repeated manual sentinel absent a specific changed condition or contrary
observation. Native AOT and Ubuntu GUI stay separate. No unattended initiation,
dispatcher, second consent protocol, lifecycle operation or host fallback.
Headless gates/G1/G2 remain closed; G3 held; G4–G7/production Undo/Redo deferred.
No graph/workbook/persistence/protocol/limit changes.
