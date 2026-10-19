# Portable storage records and restoration — review handoff

Date: October 18, 2026. First bounded implementation under ADR-0008.
No SQLite-backed service or durability is enabled.

## Architecture refinements

ADR-0008 is now Accepted after incorporating the required review points:

- Complete immutable working-revision copies are the initial strategy, not a
  permanent scale requirement; future sharing/chunking must preserve logical semantics.
- SQLite lifetime ownership/WAL/checkpoint/backup feasibility on supported
  platforms is an early gate before full SQL schema/coordinator integration.
- Durable protocol negotiation remains separately reviewed and accepted before
  any external durable authoring; current v1.1 meanings/reserved fields are unchanged.
- Versioned storage-codec compatibility fixtures precede migrations.
- Receipts, revisions, identity/tombstone retention and publications have separate
  proposed count/byte accounting, reservations and authorized actionable status.
- Any unreconstructable authoritative object blocks Ready for the whole catalog.
- External authorization consistency limits and actual filesystem/VFS durability
  assumptions remain explicit. No external identity system is claimed atomic with SQL.

## Implementation

`pivot-storage` now owns portable versioned graph/publication records, exact decimal
data, deterministic bounded CBOR encoding/decoding and validated restoration.
It has no database driver/dependency. Storage records contain UUIDs and logical
metadata, never raw NodeIds/PortIds or serialized Rust layouts.

The restricted codec uses shortest unsigned encodings and definite arrays/strings,
with fixed arities and explicit discriminants. It rejects maps/tags/floats,
indefinite/nonminimal encodings, trailing data, malformed flags/tokens, unknown
versions/operators and invalid cross-model references. The encoder streams with
budgets checked before append; the decoder bounds bytes/depth/items before
constructing owned data. Graph counts, source size, identity counts and validation
work also have independent limits. Full layout and budgets are documented.

Formula source and bound tokens are separate. Bound, parse-failed and bind-failed
states survive; restore never reparses or rebinds. The core doc-hidden cross-crate
transfer bridge validates temporary slot-based images before installing private
fields and allocates fresh model/node/port handles. Retired identities remain in
the allocation ledger. Derived dependencies and structural diagnostics are rebuilt.
The external portable model wrapper keeps runtime handles private.

The wrapper's bounded edit helpers delegate to the existing core and demonstrate
draft editability/repair; they are not new REPL commands or a durable coordinator.
Owned records retain exact coefficients/sign/scale, including signed zero and
trailing scale, independently of backend normalization. Unrelated edits preserve
those representations. Exporting a bare runtime value cannot recover precision
metadata already normalized before export; future durable management must retain
the portable authority alongside its runtime projection.

Publications include complete interface maps, lineage/revision/snapshot IDs and
logical origin. Restored publication records require globally valid snapshots,
reject edits and use the existing engine for required/default public invocation.
Immutable snapshot capture ignores subsequent source-model edits.

`restore_all` is bounded and all-or-error, validating store/identity ownership,
publication/snapshot uniqueness, lineage and stable public-port mappings. Historical
originating epochs can differ; active replay epoch/catalog completeness is a future
coordinator responsibility. No partial collection escapes a failed import.

## Compatibility and verification

There are 11 frozen v1 corpus cases assembled with an independent CBOR encoder,
including exact valid bytes, all definition/operator states, signed zero,
u64::MAX sequence, unknown versions/operators, cross-model references, malformed
decimal flags, nonminimal/trailing encodings and invalid published structure.
Tests read the fixtures and never regenerate them. Decoder and restore rejection
phases are distinguished.

Final Windows full CI passed: [summary](../../artifacts/ci/20261019T010013741677Z/summary.json).
153 Rust tests pass: 17 storage tests plus one core import-boundary test are new.
The full pipeline also passes .NET Release/warnings-as-errors, 10,099 decimal vectors,
58 protocol decimal layouts, 118 authoring fixtures, frozen v1.0 compatibility and
the actual daemon/CLI/REPL suite.

WSL Linux formatting, Clippy, all 153 Rust tests and actual IPC/REPL checks pass:
[Rust log](../../artifacts/storage-foundation-linux-tests.log),
[IPC results](../../artifacts/storage-foundation-linux-ipc/results.txt).
macOS verification of this foundation and preceding authoring slices remains pending.
No SQLite lifecycle/crash/flush test is claimed by these results.

Coverage includes rename without retargeting; delete/recreate without identity
reuse; unresolved names remaining unresolved after later creation; empty/incomplete
formula repair; dangling/computed public-input draft repair; retired ports;
174 decimal representation combinations across values/defaults/literals;
all bound operators; immutable published defaults/results; corrupt collections;
fresh independent runtime handles; source-model independence; every truncated
fixture prefix; malformed/noncanonical cases and bounded arbitrary-byte inputs.

## Review files and integration limits

- [Implementation notes, layout and limits](../architecture/portable-storage-foundation.md)
- [Storage domain types](../../crates/pivot-storage/src/lib.rs)
- [Codec](../../crates/pivot-storage/src/codec.rs)
- [Restoration/export bridge](../../crates/pivot-storage/src/runtime.rs)
- [Core validated transfer](../../crates/pivot-core/src/model/transfer.rs)
- [Semantic/compatibility tests](../../crates/pivot-storage/tests/foundation.rs)
- [Frozen corpus](../../crates/pivot-storage/tests/fixtures/v1/README.md)

Future registry startup must compare portable public-port identities across
independently restored snapshots, rather than assuming equal core PortIds.
The live ModelHost/management service is not connected to this crate. Current
record types cover graphs/publications; receipt codecs, durable quota accounting,
complete database catalog verification and memory installation remain later work.

No database tables/migrations, SQLite feasibility prototype, durable coordinator,
daemon storage flags, Protobuf fields, cross-restart replay, server database provider,
REPL enhancement, workbook or UI feature is implemented. MemoryOnly behavior and
public protocols remain unchanged. Stop here for review; the proposed next slice
is SQLite owner-lock/WAL/backup feasibility and provider foundation, not automatic
full persistence implementation.
