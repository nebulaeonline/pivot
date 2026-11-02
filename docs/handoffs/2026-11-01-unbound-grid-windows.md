# ADR-0040 — unbound Grid Windows implementation

Accepted and closed by parent implementation review, 2026-11-01, with the unresolved
Windows ARM64 cold-run qualification retained. The parent accepts all three mandatory
design clarifications, functionality, WB14/private v14 cuts, clipboard refusals,
six-target evidence, resource results, source qualification, code-size reporting and
verified archival. No additional matrix, AOT rerun or acceptance packet is required.
The review assessed supplied implementation records, without independent execution.

Grid Windows now have distinct tagged identity, independent named coordinate spaces
and a separate Desktop list/editor. `WindowName|A1` references share the canonical
workbook calculation graph with worksheet references. Each private request carries
its owner and expected state; selected scope remains client-local. Scope captures pin
the same calculation generation and its roots, without a second evaluator or cache.

Compact axis runs preserve surviving logical identities through structural changes.
Empty maximum-dimension Windows do not allocate every axis position. Allocated runs,
retired scopes/cells and old captures participate in aggregate admission. Tests exercise
320 independent structural oracle steps on both axes, non-reuse, capture retention,
budget exhaustion and zero final charges. Deletion publishes once atomically; exact
inbound-reference review uses a state-qualified one-use ticket. Reusing a deleted
name does not repair old references.

The complete candidate graph is cycle-checked when live Windows exist, including
structural changes and native restore; unrelated invalid drafts cannot mask cycles.
Qualified bounds use the target scope. Legacy worksheet-only editable-cycle and empty
structural no-op behavior remain unchanged. Trusted same-document copy creates
destination definitions, translates relative references and preserves explicit
qualifiers. Cross-scope cut/move is unavailable; filtered cross-scope copy refuses
rather than bypassing visibility confirmation.

WB14 is the sole native workbook format; WB13 and older are refused without migration.
Private session v14 carries tagged owners. Hosted v1 and CB5/CB6 remain unchanged.
Native clipboard operations involving Window authority refuse before changing the
OS clipboard. Explicit Copy Values and Paste Text provide the textual routes;
leading-zero String content remains preserved. Native preferred paste never silently
falls back to text. See [implementation semantics](../architecture/unbound-grid-windows.md).

## Verification and qualification

ROBPC CI passed 13/13; all-feature workspace tests passed 535; strict all-feature
Clippy passed. Session, native corruption, stable-axis, retained-capture, cycle,
clipboard and in-memory UI regressions passed. All six targets passed managed and
Native AOT GUI suites: 12 suites, 48 zero-charge reports, with 92 selected source
hashes matching. macOS x64 ran under Rosetta.

Five targets passed their original headless CI. Windows ARM64's original cold run
failed an existing worksheet timing objective; a separate full retained-cache CI
passed all 13 gates. Both outcomes remain unchanged. The cause is unresolved and
the passing run does not prove cold-run reliability. See the
[resource results and exact evidence identifiers](../performance/unbound-grid-windows.md).

The additional Grid Window resource corpus covers all six targets at 1,024, 10,000
and 65,536 total cells with 64 Windows, deletion/recreation, old captures and WB14
round trips. Those declared checkpoints satisfy existing timing and charged-memory
objectives. These are observed samples, not general latency guarantees or RSS data.

The canonical cloc report records **94,406 code lines across 413 files**, +2,055 lines
and seven files versus ADR-0038. The
[per-project breakdown](../../artifacts/loc/ADR-0040/20261101T230102093541Z/report.md)
includes raw inputs/tool metadata in its containing directory.

## Retention and cleanup

Every remote evidence archive was read back and checked by member size and SHA-256
before cache removal; original reports and current AOT publications were verified
unchanged afterward. Five native hosts cover all six RIDs. Their removed caches total
81.35 GiB of logical file sizes, not physical space reclaimed: macOS hosts the ARM
VMs, so host and guest free-space changes must not be added. ROBPC's idle Cargo
cache separately removed 19.02 GiB. Sources, fixtures, configuration and failure
evidence remain retained under [the retention policy](../artifact-retention.md).

The cleanup ledger is `artifacts/grid-final-native-cleanup-summary.json` with
`artifacts/grid-local-cleanup-summary.json`. Verified ZIPs and their manifests are
indexed in `artifacts/archives/index.jsonl`; the final source/receipt bundle is
`artifacts/archives/grid-final-source-and-receipts-20261101.zip`. Local evidence is
also preserved in `artifacts/archives/grid-final-local-evidence-20261101.zip`.

## Boundaries

Producer bindings, external refresh, Text, section geometry, document capture,
layout/PDF and history integration remain outside this implementation. Experimental
paths refuse unsupported ownership. No ADR-0041 feature or design work is included.
Documentation closeout is complete. Stop for the owner's commit/push and explicit
authorization before starting the next design. ADR-0041, Worksheet-Range Producers
and Bound Grid Windows, is assigned next; its design and implementation have not
started. ADR-0023 remains parked at G3.
