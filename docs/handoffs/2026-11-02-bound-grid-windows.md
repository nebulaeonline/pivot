# ADR-0041 — bound Grid Windows implementation

Accepted and closed by implementation review, 2026-11-02, with the unresolved
Windows ARM64 cold-run timing qualification retained. The review assessed supplied
documentation and qualification evidence, without independent source execution.
All four mandatory design clarifications, functionality, WB15/private v15, resource
results, source-qualified six-target evidence, code-size reporting and verified
retention are accepted. No additional ADR-0041 validation or housekeeping acceptance
is required.

Named bound Windows now project a worksheet rectangle or an explicitly refreshed
installed fixture. Views are read-only, include physical Blank/hidden positions
and expose source provenance and formatting. They have no manufactured CellIds or
independent editing, filtering, sorting or formatting state. Desktop provides
binding/source controls, explicit fixture refresh/reconnect, producer deletion
review and bounded detach into a new unbound Window. Copy Values preserves exact
typed text; native Window clipboard authority remains refused.

Worksheet scalar bindings alias source CellIds with Window lifetime and membership
guards. Aggregate/allocation ranges retain SelectionId spatial authority and dense
Blank positions. Structural changes transform source/composed descriptors under
existing affinity rules; feedback through producers is rejected against the
candidate graph. Temporary exclusion may resume the same authority, while retired
identities and reused names do not repair it. Whole-source range deletion and
explicit producer retirement are covered. There is no new whole-worksheet deletion
command in this milestone.

Fixture refresh publishes immutable, typed generations. Durable keys can follow
record reorder/absence/reappearance; generation-qualified references never inherit
replacement generations. Decimal consumers share one captured leaf per producer,
key and field. Other field types remain available to presentation and exact values
export. Old captures retain source roots and the canonical calculation context
from their own generation. Failure, disconnect and cancellation retain selected
data independently of attempt status; obsolete completions cannot publish. Staging
permits remain charged until actual task ownership releases.

WB15 stores and validates the complete producer/Window/schema/key/guard/captured
output authority before installation. Digests alone do not authorize a provider
schema or relationship. Reopen is disconnected and never executes a provider.
Private session v15 carries explicit scope and projected-location metadata. Hosted
v1 and CB5/CB6 are unchanged; earlier native formats are refused. Only installed
deterministic fixtures are supported; no SQL/network provider framework or implicit
write-back is introduced.

## Validation

- Local headless CI passed 13/13; final all-features Rust tests, strict Clippy and
  updated execution-safety tests passed.
- Initial headless matrix: Windows/Linux x64, Linux ARM64 and both macOS targets
  passed. macOS x64 is explicitly Rosetta evidence. Windows ARM64 failed the
  existing cold compact structural timing gate, then passed a separate retained-
  cache full CI run. The original failure remains unresolved and preserved.
- All six working source snapshots match the frozen inputs. A second freeze
  changes only the AOT fixture validator, its existing tests and the execution-
  safety entry; product sources are unchanged.
- Windows x64 managed GUI passed. Its Native AOT application and four graphical
  reports passed, followed by an obsolete wrapper WB14 check failure. Corrected
  WB15 validation, publication hashes, graphical records and owned-process cleanup
  passed separate read-only retained-evidence revalidation. Original receipts were
  not edited. A fresh full Windows x64 AOT wrapper retry subsequently passed, including all
  four graphical reports, zero charges and owned-process cleanup.
- Windows ARM64 managed GUI passed. Its AOT attempt failed during NuGet package
  restore because DNS resolution failed, before graphical execution. Clearing the
  DNS cache restored HTTPS access. A headless publish preflight and a fresh full
  AOT graphical retry passed, with four native ARM64 reports, zero charges,
  no owned survivors and matching source hashes. Both Windows targets are complete.
- Linux x64 managed/AOT passed with four graphical reports per suite, zero final
  charges, no owned survivors and matching native source snapshots.
- Linux ARM64 managed/AOT and macOS ARM64 managed also passed, with verified
  source snapshots, zero charges and owned-process cleanup.
- All remaining Mac suites passed: ARM64 managed/AOT and x64 managed/AOT under
  Rosetta. Final qualification covers 12 suites and 48 actual graphical reports,
  each with zero final charge and no surviving owned processes. Actual source
  snapshots match their frozen inputs.

Focused regressions cover scalar/range composition, structural authority, feedback,
old captures, keyed generations, typed presentation, malformed payload rollback,
source/Window/producer retirement, ticket admission/replay, copy/fault recovery,
native authority forgery and zero final runtime charges. The 32-producer/64-Window
coexistence workload and aggregate external-slot boundary pass without raising
limits. Release checkpoints include 1,024/10,000/65,536 total nodes.

See [resource results and qualification details](../performance/bound-grid-windows.md).
`artifacts/bound-final-qualification.json` selects the actual passing receipts,
including fresh Windows AOT retries, and retains the cold headless qualification.
Original Failed receipts were not rewritten.

## Retention

Reports and fixtures were archived and byte-verified before native/local cache
cleanup. Final native audits find no remaining per-run compiler caches. The idle
local Cargo cache was cleared (30,181 files; 24.7 GB by inventory). Duplicate binary
copies in collected/exported reports were checked against their publication
inventories and removed; report files and current canonical native publications
remain intact. Source snapshots, installed toolchains, pins and original failures
are preserved. The old G3 research source remains parked and unchanged.

Exact inventories, verification and non-additive volume/free-space qualifications
are in `artifacts/bound-final-retention-summary.json`; the archive index is
`artifacts/archives/index.jsonl`. The source bundle retains the tested freezes and
final candidate. No test rerun is implied by cache removal.

## Code size

The canonical cloc report is [retained here](../../artifacts/loc/ADR-0041/latest.md).
Generated code, dependencies and build outputs are excluded. Test/example code is
included in project totals; linked source is counted at its physical owner once.

| Project | Code | Comments | Blank | Test/example code* | Languages (code) | Change |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| ApplicationSession | 1,530 | 17 | 34 | 1,530 | C# 1,361, Python 169 | +56 |
| DecimalParity | 419 | 12 | 23 | 419 | C# 113, Python 306 | +0 |
| LegacyProtocol | 161 | 5 | 2 | 161 | C# 43, Protocol Buffers 118 | +0 |
| Pivot.Avalonia | 427 | 3 | 2 | 0 | C# 427 | +0 |
| Pivot.Client | 573 | 25 | 18 | 0 | C# 573 | +32 |
| Pivot.Desktop | 3,256 | 43 | 44 | 1,228 | C# 3,107, XML 149 | +231 |
| PolicyProbe | 1,621 | 54 | 128 | 1,621 | C# 1, PowerShell 38, Python 1,582 | +2 |
| ProtocolParity | 81 | 2 | 8 | 81 | C# 55, Python 26 | +0 |
| pivot-cli | 1,413 | 12 | 3 | 280 | Rust 1,413 | +0 |
| pivot-core | 17,391 | 236 | 174 | 3,192 | Rust 17,391 | +123 |
| pivot-host | 8,431 | 69 | 95 | 4,165 | Rust 8,431 | +0 |
| pivot-protocol | 2,668 | 47 | 29 | 1,023 | Protocol Buffers 597, Rust 2,071 | +22 |
| pivot-session | 4,200 | 18 | 11 | 382 | Rust 4,200 | +503 |
| pivot-storage | 5,134 | 52 | 72 | 1,952 | Rust 5,134 | +0 |
| pivot-transport | 897 | 26 | 13 | 369 | Rust 897 | +0 |
| pivot-types | 1,222 | 26 | 13 | 194 | Rust 1,222 | +0 |
| pivot-workbook | 32,370 | 201 | 148 | 13,720 | Rust 32,370 | +4,732 |
| pivot-workbook-document | 8,466 | 57 | 33 | 5,557 | Rust 8,466 | +473 |
| pivotd | 1,134 | 20 | 7 | 899 | Rust 1,134 | +0 |
| Other source | 214 | 11 | 4 | 0 | Protocol Buffers 214 | +0 |
| Scripts/tooling | 7,673 | 373 | 671 | 0 | Bourne Shell 126, C 166, DOS Batch 12, PowerShell 438, Python 6,931 | +0 |
| Standalone tests | 1,299 | 87 | 191 | 1,299 | Python 1,299 | +0 |

**Total source code: 100,580 lines** across 424 files.

ADR-0041 is accepted and closed. Section geometry remains a proposed next scope;
no later ADR design or feature work has started.
