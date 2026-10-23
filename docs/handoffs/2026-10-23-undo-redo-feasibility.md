# ADR-0023 root activation and ownership feasibility

Date: 2026-10-23. **Bounded experiment implemented; submitted for feasibility review.
Full Undo/Redo remains deferred.**
[Authorization and stop point](2026-10-23-undo-redo-design-review.md),
[ADR-0023](../adr/0023-document-transaction-history.md).

## Finding and decision requested

Retained canonical/workbook roots can be activated with fresh mutation authority
and monotonic lifetime identity floors, without codec restore or graph redesign.
The executable ownership cases support proceeding to a separately authorized
history coordinator. The proposed 16 MiB conservative full-state weight is practical
for the sampled edits, but does **not** guarantee 32 Undo transitions: the measured
1,024-live-cell case retained ten transitions and the retired-identity case three.
Review that retention tradeoff before production integration. No limits were raised.

All new APIs, counters, tests and the measurement executable require Cargo feature
`history-experiment`. Default builds have no history controls or changed history
semantics. No general history stack, protocol operations, session/save integration,
desktop controls, persisted history or proof-mode barriers were implemented.
WB5, CB2, private v4 and frozen hosted v1 are unchanged. ADR-0022 remains closed.

## Executable foundation

`crates/pivot-workbook/tests/history_foundation.rs` proves A → B → restore A →
branch C while retaining B's calculation capture. Exact historical cell content,
formula bindings, portable/runtime identities, invalid drafts and cyclic drafts
return under fresh Expected authority. A's old authority rejects. C's new IDs do
not alias B; after releasing B's root, another branch allocates beyond C. B's capture
continues evaluating B. Activation does not parse, validate, replay or deserialize.

Activation retains current lifetime node/selection allocation floors separately
from historical root occupancy; restoring A cannot recover B's allocation quota.
The existing runtime cleanup mechanism owns released roots. Cancellation, foreign
roots, failed staged edits and real allocation-domain exhaustion preserve the
current root/content identity. A deliberately undersized mandatory-pair budget
exercises candidate rejection before installation in the test harness. This is
not a production history-admission coordinator or a proof of its concurrency policy.

`crates/pivot-workbook-document/tests/history_save.rs` exports B, restores A,
releases B's root, then installs the prepared B bytes through the existing native
file adapter. The file contains B; live content remains A under fresh guards.
The captured content identity/checkpoint and file association are separate from
activation. Reopening is used only to verify installed bytes, never for activation.
Production save-ticket validation, dirty-state integration and uncertain-installation
fault handling remain unimplemented here and must be tested in their later owner.

## Weight and ownership model

Each pin has a fixed conservative full-state weight. Mutation-local counters track
definition payload, names, dependencies, records, physical extent and lifetime
selection allocations. Existing persistent-index lengths and selection counters
bound structural headers; sparse page bounds use a fixed 16-level calculation.
Workbook record payload is maintained on replacement. Opening an existing document
initializes counters inside its existing import traversal. Query/activation never
walk the complete root. `weight_scan_visits = 0` describes that implementation;
it is not independent dynamic instrumentation. Replacing equal-sized content does
not accumulate allocation traffic into the logical weight.

The bound includes allocation headers, a wrapper margin, names and formula/error
payloads. Invalid-error bounds can overcount substantially; the measurements do not
establish tightness for every possible invalid graph. Single-root measured allocated
bytes are asserted below the bound in each sample. This is a structural bound plus
sampled checks, not a universal measured heap census.

Logical weights deliberately charge shared content once per retained state. Domain
allocated bytes count distinct allocations; domain charged bytes also include
reservations/runtime ownership. Deferred cleanup and calculation captures remain
separate owners. Therefore neither logical pruning nor its byte sum predicts actual
memory reclaimed. Pin wrappers are explicitly charged to the existing domain.

## Windows release measurements

The bounded measurement window samples 32 changed candidate transactions, alternating
input replacement and Fill Down (up to 256 cells), except the retired-identity case
which replaces its one live input. It stages candidates before pruning/installing,
retains at most 33 states/16 MiB, then retains a B capture across activation/pruning.
This harness has no history cursor, general stack or production request surface.

| Case | Initial weight / allocated bytes above runtime baseline | Retained states | Logical window bytes | Domain allocated / charged bytes |
| --- | ---: | ---: | ---: | ---: |
| 16 live cells | 38,547 / 30,103 | 33 | 1,272,051 | 181,735 / 2,035,823 |
| 256 live cells | 375,683 / 365,151 | 33 | 12,397,539 | 687,823 / 2,541,911 |
| 1,024 live cells | 1,459,715 / 1,443,087 | 11 | 16,056,865 | 1,525,927 / 3,326,335 |
| 256 cells, mostly formulas | 560,067 / 549,535 | 29 | 16,241,943 | 757,295 / 2,601,623 |
| 4,095 lifetime nodes, one live cell | 3,549,075 / 3,494,658 | 4 | 14,196,300 | 3,508,314 / 5,291,642 |

All sampled history candidates were admitted; pruning removed 0/0/22/4/29 states
respectively. No ordinary first before/after pair rejected the 16 MiB budget.
The retired-identity setup reached the existing workbook byte limit before the
8,192 allocation limit: nine rejected setup attempts, shrinking batches down to
one, stopped at 4,095 lifetime nodes. These are existing mutation-admission
rejections, not history rejections. An initial setup that assumed all 8,192 slots
would fit failed; the harness now measures actual capacity instead of raising it.

Windows weight queries took 39–44 ns and root activation 654–782 ns per sampled
call; candidate p95 was 32–644 µs. These local release microbenchmarks are descriptive,
not CI thresholds. Peak domain charge was at most 12,540,991 bytes across the cases.
After pruning, releasing the retained B capture lowered charge by 6,568–6,952 bytes;
its evaluation remained correct before release. Every run finished at zero charge.

Machine-readable Windows/Linux measurements and logs are under
`artifacts/history-feasibility/`. Linux source/tests/temp files run from native
`/var/tmp/pivot-history-feasibility`; the existing Rust target cache on the Windows
mount is reused. No .NET validation or native macOS run is claimed for this phase.
Linux reproduced the five initial weights, retention counts and zero-charge exits.
Its domain allocation varies slightly with cleanup timing; the maximum observed
peak was 12,543,295 bytes. Queries took 36–40 ns, activation 898–2,054 ns and candidate
p95 121–499 µs. No cross-platform performance conclusion is drawn from these runs.
The source manifest records 428 repository files; all 151 Rust build-input/fixture
hashes match the isolated Linux copy. The receipt hashes the evidence separately.

## Validation and remaining boundary

Windows: formatting, strict workspace/all-target/all-feature Clippy, full workspace
default (346 tests) and all-feature (367 tests) suites passed. The five feature-only
tests comprise four root/accounting cases and one native-file ownership case.
Linux default (347 tests) and all-feature (368 tests) suites passed. The save test was rerun on both platforms after
removing a redundant local assertion; that change does not add coverage claims.

Development fixes included Shared payload dereference accounting, imported-root
support, separating physical root extent from lifetime floors, and canonical Windows
test-file paths. Initial Clippy failure was item placement; final strict lint passed.
Sandbox native-file/WSL access failures were rerun with approval. An initial Linux
copy named nonexistent optional paths; workspace members were then copied. A shell
variable lost through WSL invocation prevented initial measurement launch; explicit
binary paths corrected it. None of those attempts is counted as passing evidence.

Stop here for parent feasibility review. Later authorization must cover actual
atomic history admission/branching/pruning, Redo/no-op/failure behavior, saved markers
and real in-flight tickets, proof barriers, document lifecycle, protocol compatibility
and desktop automation. Their normal three-platform managed/Native AOT gates remain
future requirements. Existing clipboard, WSLg startup, mounted-filesystem Save
uncertainty and Linux timing reliability issues remain open.
