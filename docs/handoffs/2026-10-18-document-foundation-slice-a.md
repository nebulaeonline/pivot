# Document Foundation — Slice A review handoff

Date: October 18, 2026. ADR-0009 accepted with review refinements.
Disposition: Hosting-independent graph payload and in-memory session implemented;
stop after Slice A for review.

## Result

A standalone ProfitModel can be created with Revenue, Costs, Net and Margin,
evaluated, edited, exported as owned immutable content, restored into another
session and evaluated again without a daemon, database record, publication,
hosted-model registry, receipt or StoreId/RecoveryEpochId.

The first implementation stays inside pivot-storage. No crate, dependency or
core edit was needed. The shared graph runtime delegates staging, formula binding,
validation and calculation to the existing core; it does not reproduce hosted
WorkingModelService attachments, authorization or retained operation machinery.

## Review refinements incorporated

- ADR-0009 is Accepted. ADR-0006/0007/0008 and optional hosted scope remain intact.
- Thin sessions reuse computational semantics rather than building another full
  hosted management service.
- IDs exist only for present invariants: owned document/state/graph/revision,
  fresh session, calculation provenance and private export ordering. There is no
  general identity framework, saved-file version or filesystem fingerprint yet.
- Logical DocumentId means ancestry. Copies can share IDs while sessions/runtime
  handles and editable state remain independent. Foreign session tokens and
  checkpoint acknowledgments are rejected.
- Desktop broker direction remains C# file authority and Rust semantic authority.
  Large transfer needs a separate bounded contract; do not raise the existing
  1 MiB frame limit as its substitute. Slice A introduces no transfer operations.
- The next proposed milestone is the smallest useful experimental headless file
  workflow, before extensive large-container benchmarking. No production format
  is selected or frozen.

## Implementation

| Surface | Implemented behavior |
| --- | --- |
| GraphPayload | Hosted-free versioned definitions, exact Decimal96, bound tokens/source/failure states, stable identities and retired ledgers |
| EditableGraph | Private core/maps, shared validated import, staged edits, diagnostics, selected evaluation and direct named invocation |
| ModelDocument | One graph, logical document/state IDs, minimal version metadata; owned representation rather than native bytes |
| DocumentSession | New/load/load_graph, revision-guarded edits, validate/evaluate/invoke, fresh calculation generations and isolated runtime state |
| ExportCapture | Immutable owned snapshot with private originating session and export order |
| Saved checkpoint | Trusted in-memory acknowledgment of the exact capture; S acknowledgment cannot mark newer T clean; duplicates idempotent, regressing acknowledgments rejected |
| Hosted adapter | Existing Record, publication/external port translation and whole-store checks over shared graph machinery |

New/load sessions start dirty. Loading owned logical content does not attest that
anything was saved to a file. A future broker can acknowledge only after resolving
its actual write. Results and invocation overrides do not mutate definitions or
the saved checkpoint. Actual no-op edits retain state/revision; exact decimal
representation changes count as content changes.

Graph payload validation rejects malformed/foreign/never-allocated identities,
invalid token arity, unsupported semantic versions and invalid decimal flags.
Well-formed invalid drafts remain editable and exportable. Restoration uses tokens
directly, never source reparsing/rebinding. Every load creates fresh core handles.

## Compatibility and bounds

The hosted storage-v1 schema, codec, 11 frozen fixtures and fixture manifest are
unchanged. Hosted records still enforce their existing exact encoded budgets and
publication rules. A new regression test confirms tight encoded-byte budgets do
not accidentally inherit the document payload's separate logical budget accounting.

The new graph payload has bounded logical byte/item/work accounting but no document
encoder. Existing small counts and independent core hard ceilings remain; this is
not large-workbook capacity. Full staging/capture copies remain the correctness
baseline. Error categories are intentionally still coarse for some graph operations;
no client diagnostic protocol was added.

## Tests and actual evidence

Thirteen new document integration tests cover:

- ProfitModel results before/after restoration and independent edits.
- Required/default public interface behavior, explicit overrides and isolation.
- Renames, name reuse, retired node/port identities and explicit repair.
- Unresolved, incomplete and cyclic drafts surviving until explicit repair.
- 174 coefficient/sign/scale combinations for input/default edits, trailing-scale
  formula literals, signed zero and untouched imported exact literal representations.
- Immutable S capture, later T edit and S saved acknowledgment retaining dirty T.
- Idempotent acknowledgment and rejection of checkpoint regression.
- Copied logical IDs with independent sessions and rejection of stale tokens.
- Atomic failure under name, numeric, source, interface and count-limit errors.
- Malformed owned payloads, semantic versions and byte/item/work limits.
- Bound-token restoration even when original source cannot currently parse.
- Core dependency independence from storage, documents and hosted infrastructure.

The existing fresh-runtime-handle unit test now exercises the shared graph path;
the original storage semantic/frozen corpus tests still pass. One additional hosted
budget-compatibility test brings the workspace to **167 Rust tests** (153 before
this slice + 13 document + 1 hosted compatibility test).

Full Windows local CI passed with Rust nightly 1.101.0 and .NET SDK 10.0.402:
formatting, Clippy with warnings denied, all Rust tests, Release solution build,
10,099 decimal vectors, Rust/C# protocol and frozen legacy compatibility, and real
named-pipe daemon/CLI/REPL integration. No warnings or parity failures.

- [Windows CI summary](../../artifacts/ci/20261019T015810797840Z/summary.json)
- [Windows Rust test log](../../artifacts/ci/20261019T015810797840Z/rust-tests.log)
- [Windows IPC results](../../artifacts/ci/20261019T015810797840Z/ipc/results.txt)

The initial sandbox CI run failed at existing named-pipe tests with Access denied;
the permitted run above passed. Its failed report is retained at
`artifacts/ci/20261019T014254291024Z/summary.json`.

Ubuntu WSL Linux formatting, workspace Clippy, all **167 Rust tests** and the real
Unix-socket IPC/REPL harness passed using the separate target/linux-wsl build path:

- [Linux Rust test log](../../artifacts/document-foundation-linux-tests.log)
- [Linux IPC results](../../artifacts/document-foundation-linux-ipc/results.txt)

Linux .NET was not rerun for this Rust-only change. Native macOS verification of
recent work remains pending. None of these checks prove native document-file or
database crash/power-loss durability. Reports under artifacts are local ignored
evidence, not tracked fixtures.

## Review files

- [Accepted ADR-0009](../adr/0009-portable-documents-and-local-lifecycle.md)
- [Actual implementation and limits](../architecture/document-foundation.md)
- [Ownership and lifecycle](../architecture/document-lifecycle.md)
- [Portable representation](../architecture/portable-document-representation.md)
- [Storage reuse](../architecture/document-storage-reuse.md)
- [Verification direction](../architecture/document-verification-plan.md)
- [Container evaluation and sequencing](../architecture/document-container-analysis.md)
- [Graph runtime/payload](../../crates/pivot-storage/src/graph.rs)
- [Document session](../../crates/pivot-storage/src/document.rs)
- [Hosted adapter](../../crates/pivot-storage/src/runtime.rs)
- [Document tests](../../crates/pivot-storage/tests/documents.rs)
- [Hosted compatibility tests](../../crates/pivot-storage/tests/foundation.rs)
- [Frozen fixtures](../../crates/pivot-storage/tests/fixtures/v1/README.md)

## Exclusions and stop boundary

No native file format/extension, ZIP/SQLite/binary container, file broker,
New/Open/Save/Save As filesystem workflow, replacement/recovery I/O, daemon document
RPC, Protobuf changes, C# enhancements, worksheet/range objects, incremental
calculation, UI, hosted durability or durable receipts were implemented.

After this review, separately authorize the smallest experimental headless model-only
file lifecycle with safe replacement. Do not infer a production .pivot format or
start broad million-cell container research from Slice A acceptance. Optional hosted
persistence remains deferred. Stop here for review.
