# ADR-0020 design review — direction approved, contracts required

Date received: 2026-10-22. Parent review supplied by the user.
**Disposition: Architectural direction approved with required implementation-contract
revisions. ADR-0020 remains Proposed; implementation is not authorized.**
[ADR-0020](../adr/0020-worksheet-structured-output-bindings.md),
[original design packet](2026-10-22-structured-output-binding-design.md),
[focused contract addendum](2026-10-22-structured-output-binding-contract-addendum.md).
ADR-0019's accepted experimental implementation remains closed.

## Approved direction

- Bind ordinary formulas to document/producer/output/type identity independently of
  names and projections. Explicit authoring resolves names; rename retains identity,
  deletion leaves missing references and same-name recreation does not repair them.
- Share one narrow captured scalar-source leaf per referenced output in the canonical
  graph. The document owns producer semantics and output preparation; the core owns
  numeric dependency propagation, not records or acquisition.
- Distinguish rejected publication attempts from authoritative unavailable source
  heads. Keep attempt diagnostics visible. An unavailable captured head must never
  use cached last-good values.
- Capture worksheet/graph, directory, source heads/results and scalar context coherently.
  Worksheet revision alone cannot identify a source-dependent calculation.
- Same-document copy preserves structured identity and translates only ordinary A1
  operands. Foreign structured bindings reject atomically, without text fallback.

## Required before implementation

The parent requires a concrete scalar leaf identity/lifecycle, immutable exact context
validation, Decimal96/fault representation, deterministic error precedence and capture
accounting. The core should retain a stable source-fault identity while the document
explains MissingProducer, UnavailableResult, SchemaMismatch and other source meanings;
do not add an open-ended hierarchy of source-specific core errors.

Specify the actual experimental token/typed occurrence/spans, Rust correspondence
checks, mixed A1 operands, editor/no-op/resubmission and invalid-draft handling. Client
annotations cannot inject arbitrary NodeIds. Any spelling remains private to the proof.

Specify native clipboard limitations and same-document identity-preserving copy,
including atomic rejection of unsupported/foreign representations. No PIVOTCB2 change
or new version is approved. Provide exact composite expected-state validation and
bounded reservations for registry, slots, annotations, diagnostics, tombstones and
capture contexts without raising ordinary capacities.

The linked addendum supplies proposed contracts, including a private `@{Producer.Output}`
primary, generic fault IDs, server-owned DuplicateBlock while OS Copy of structured
formulas is refused, and one resident source slot including retained retirement.
These concrete choices are **new proposals for parent review**, not approved details.

## Scope and stop

Only focused design/addendum work is authorized. After approval, a possible bounded
implementation would cover one producer/output, one shared slot, ordinary formula
consumers/dependents and SUM, two views and existing record/capture ceilings. Actual
worksheet calculations must be proved, not another standalone numeric invocation.
Keep opt-in non-savability, real boundary tests, automated desktop and Native AOT
validation on Windows/Linux/WSLg/native macOS APFS.

Do not begin core, workbook, protocol or desktop implementation until the concrete
contracts are approved. No SQL/connectors, scheduler, permanent `#` grammar, WB5,
projection persistence, computational records, keyed fields, formatting or capacity
expansion is authorized. No new implementation or validation claim accompanies this
review record.
