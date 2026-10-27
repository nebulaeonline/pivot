# G3 page layouts and authenticated growth

2026-10-27. Research only. Existing binary records, zlib level 6 and atomic
receipt/content/lifetime semantics remain unchanged. No production schema, index,
codec, retention policy or migration is selected.

## Physical comparisons without changing logical data

`research_history_layout.py` compares:

- Existing WITHOUT ROWID payloads keyed by stable content hash.
- Ordinary rowid payloads with an explicitly NOT NULL indexed hash identity.
- Integer-local payload primary keys plus the same unique external hash.
- That local-key layout with a unique partial edit-location index omitting NULL
  control entries, preserving the old non-NULL uniqueness/query behavior.

The payload primary key's NOT NULL requirement is preserved explicitly. Local
rowids never replace authoritative hash references or stable workbook identities.
Receipt semantic columns, immutable intents, checkpoints and lifetime fields are
not removed. Required lookup plans are checked rather than allowing an index drop
to hide a scan.

New research databases are populated from a native read-only source snapshot with
the same canonical insertion order. SHA-256 manifests compare all logical payload,
receipt, bucket and publication bytes, excluding only local physical IDs/schema.
Each copy passes verified opening and full audit. These are offline layout samples,
not an application schema-migration feature or representative incremental-write
timings. Separate 1K/10K incremental replays compare old/new layouts and growth with
the same independent G2 oracle.

The observed row-size/overflow tradeoff is consistent with SQLite's official
[WITHOUT ROWID guidance](https://sqlite.org/withoutrowid.html): the optimization is
not automatically beneficial for large BLOB rows. No undocumented page threshold
is used as an integrity assumption.

## Extensible prefix directory within SQLite

`research_history_growth.py` starts with sixteen authenticated hash-prefix regions.
Each current leaf has an integer prefix locator and a bucket commitment. Receipt
entries retain their full 256-bit scoped key hash and 256-bit row digest. A lookup
routes by increasing prefix length, validates the selected bucket against the
directory commitment, and validates the complete immutable receipt row.

Directory decoding checks canonical order, uniqueness, prefix non-overlap and
complete keyspace coverage. Coverage is a structural partition check, not a receipt
count or high-water substitute for authenticated absence. Missing bucket bytes
with a nonempty commitment fail; they never become an empty region.

When an insertion exceeds the bucket entry budget, it partitions that bucket by
the next hash bit. In the same SQLite transaction it writes the children, retires
the parent representation, encodes the next directory generation and publishes
receipt/content/head/cursor/Redo/latest lifetime authority. No receipt or historical
content is deleted or reclassified. Hash references and external identities remain
unchanged. Receipt/payload manifests match the fixed-directory replays; only index
and current publication representation differ.

There is one application-level current directory. Older SQLite read transactions
retain the old directory **and** retired bucket rows in their consistent WAL view.
Tests force a split of a bucket being read, verify the old proof remains valid and
refuse stale-horizon terminal reconciliation after the snapshot ends. No separate
application generation log or G5 reader-retention policy is introduced.

## Explicit work and admission bounds

Current research defaults are 64 entries/bucket, 1,024 directory leaves, 24 prefix
bits and at most eight splits per publication; directory decoding is capped at
64 KiB. These settings are encoded and checked on reopen. They are **temporary
research resource bounds**, not product capacity limits or a promise of arbitrary
history growth.

An ordinary proof uses at most 25 prefix-routing probes plus a bounded bucket
decode. Each directory validation is bounded by leaves times prefix depth. A new
insert stages at most one previously admitted bucket plus one entry, so split work
cannot silently expand into a full-history repartition. Reaching any resource
bound rolls back/refuses the operation; receipts are never pruned to make space.
SQLite B-tree physical page/cache work remains a separate unqualified cost.

Splits, generations and publication changes share one commit. Tests inject errors
or kill the actual process before a split, after its partial plan, after children,
before COMMIT and after COMMIT. Reopen must observe the complete old or new
generation, without rebuilding from all historical receipts. Incomplete coverage,
missing children and discovered integrity faults block admission. Complete audit
enumerates current bucket entries and retained receipts, verifies membership and
checks for orphan current bucket rows; this scan is not normal opening.

## Limits of this proposal

The directory remains flat. Its descriptor grows with leaf count and is validated
and rewritten during publication. Splitting beyond 256 leaves is demonstrated, but
that does not establish an unlimited production structure. At the admitted leaf
limit this prototype blocks rather than changing policy or rebuilding blindly.

A possible next experiment is a page-bounded hierarchical directory inside SQLite,
updating only affected authenticated pages and a small root in the same transaction.
That is a proposal, not implemented authority or an authorization to create another
storage engine. The same all-receipt proof invariant and old-reader snapshots would
have to survive; a return to permanent per-publication path copies is not presumed.

The experiment retains the accepted correct-writer/SQLite-atomicity/collision/
current-publication trust boundary. Coherent entire-store rollback/forgery, durable
cross-process corruption coordination, general epoch/branch/retention authority and
production memory admission remain outside G3. Known integrity faults still
quarantine a handle until successful full audit.

## Evidence boundary

Use native Windows x64/NTFS, Ubuntu x64/ext4 and Mac ARM64/APFS. Keep exact logical
manifests, native allocation, dbstat overflow/unused/index pages where supported,
freelist/WAL, proof/split/publication/COMMIT latency and process memory distinct.
Compressed retained payload remains the primary metadata denominator. No engine
qualification, sync syscall or physical-device-write claim is inferred from these
figures; old Windows/Linux WAL-reset caveats remain.

The primary evidence is 1K/10K real G2 data. A 16,384-control **index-only** probe
exercises growth past 256 leaves; it is not a 100K G2 run or production scalability
qualification. G1/G2, ordinary WB5, six-RID execution validation and clipboard
ISSUE-0001 remain unchanged. Dev Drive stays shelved; dotZstd/Rust zstd stay deferred.

[Results and source/evidence details](../handoffs/2026-10-27-g3-layout-growth-results.md).
