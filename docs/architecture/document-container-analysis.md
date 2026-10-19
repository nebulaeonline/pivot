# Native document container alternatives

Status: Accepted ADR-0009 evaluation direction. No physical container selected or implemented.
The `.pivot` name is illustrative; logical content and lifecycle do not depend on
the final extension or container signature.

## Requirements before selection

A saved artifact must be portable and self-contained for embedded content, preserve
invalid drafts and exact typed data, expose explicit compatibility versions and
support bounded validation. Save must capture one coherent state without retaining
every edit. Support a model without worksheets now and leave room for sparse sheets,
presentation, resources and multiple embedded models later. Evaluate file size,
open/validation latency, selected-range access, peak memory, save latency, bytes
rewritten, recovery and compatibility tooling independently.

| Candidate | Benefits to evaluate | Costs and unresolved questions |
| --- | --- | --- |
| ZIP with versioned entries | Familiar inspection/extraction tools; separate manifest, graph and presentation resources; compression | Updating a large entry may require rewriting it or the archive; lazy graph access needs chunked entries/indexing; define duplicate-name, decompression and path policies |
| SQLite application file | Indexed sparse records and selective reads; transactional changes; standard inspection tools | Needs a document schema, defensive untrusted-file handling and explicit-save staging; active journal/WAL handling and portable transfer must be specified; database overhead must be measured |
| Versioned binary container | Direct control of section directory, streaming, chunking and exact typed records | We own indexing, integrity, compatibility and tooling; incremental updates can become a bespoke storage engine |

SQLite is a valid document-container candidate, independently of hosted persistence.
Its application-format documentation describes local application files and indexed
updates. Selecting it would not select a daemon registry or ADR-0008 receipt scheme.
See [SQLite application file formats](https://www.sqlite.org/appfileformat.html).
ZIP entry-layout, compression and rewrite statements above are evaluation concerns;
the actual profile and supported library must be checked against the
[PKWARE specification](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT)
before implementation. No archive library or feature profile is chosen here.

## Common safety and compatibility concerns

Use explicit section identities, lengths, integrity checks and versioned semantics.
Checksums detect damage; they do not authenticate an untrusted author. Cap compressed
and expanded bytes, entry counts, nesting and work; reject overlapping sections,
duplicate identities and ambiguous entries. ZIP readers must not extract arbitrary
paths or accept traversal/symlink entries. SQLite readers must validate the expected
schema and disable unnecessary executable/extension behavior before consuming
untrusted content. Binary readers must validate all offsets and arithmetic before
allocation. A sectioned container does not make a monolithic graph payload lazy.

## Proof first; comparison before a production format freeze

Following the ADR-0009 review, the next goal after Slice A is the smallest useful
headless model-file proof, using a clearly experimental container. That proof needs
a bounded format and safe write behavior, not an extensive benchmarking project.
The workloads below inform later production selection and must not block proving
New/Open/Save/Save As. [Slice B](experimental-document-format.md) now implements a
minimal experimental binary envelope, without selecting the production container.

## Later comparison before production selection

Use the same representative logical data for each prototype: a small standalone
model, a medium formula graph, and sparse workbook-shaped data at 100,000 and
1,000,000 populated-cell targets. These are proposed measurement workloads, not
supported capacities. Include long source, invalid drafts, retired identities,
presentation-only changes, numeric inputs and binary resources. Storage-shaped
synthetic data can compare containers before the evaluator supports those scales.

Measure cold open plus full validation separately from metadata browsing and
viewport reads. Measure full Save and small-change Save, total bytes written, peak
resident memory and temporary disk usage. Include poorly compressible content and
heavily fragmented/edited documents. Test failure injection on Windows, Linux and
macOS. If a prototype bypasses current graph limits, label it container-only rather
than claiming end-to-end spreadsheet performance.

Review complexity and maintenance alongside throughput: migration tools, independent
inspection, read-only compatibility, repair diagnostics, licensing/deployment,
cross-language boundaries and supported filesystems. Do not choose from an isolated
microbenchmark or from the availability of the current CBOR encoder. Prototypes
and production container selection require separate authorization; none are part
of Slice A. Do not freeze production .pivot bytes from the experimental proof.
