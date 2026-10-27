# G3 hierarchical authentication and receipt normalization

2026-10-27. Research-only follow-up to the accepted
[layout/growth assignment](../handoffs/2026-10-27-g3-layout-growth-acceptance.md).
No production backend, codec or format migration is selected.

## Authenticated logical pages inside SQLite

`research_history_hierarchy.py` stores a binary radix directory in the existing
rowid `bucket` table. Each row is a bounded logical authentication node, not a claim
of one-to-one correspondence with a physical SQLite page. Integer addresses encode
prefixes: root 1; left/right children `2*n` and `2*n+1`. A node digest commits to its
address and exact bytes. A branch contains two 32-byte child digests; a leaf contains
sorted full 32-byte scoped transaction keys and 32-byte canonical receipt digests.

An authenticated path proves which leaf covers a key. Both children of every branch
are mandatory, and prefix routing divides its entire interval into two disjoint
halves. Leaf keys must match their interval. A selected-path failure cannot become
authenticated absence. Full audit traverses every reachable node, checks complete
coverage, leaf/receipt counts, every canonical receipt and the absence of orphan nodes.
Untouched historical subtrees are validated on access or full audit, rather than
scanned during routine publication. The accepted current-root/SQLite atomicity trust
boundary still applies; this does not detect coherent rollback without a witness.

Publication overwrites the affected leaf and its ancestor rows in the same SQLite
transaction as the receipt and authority. Splitting replaces a leaf at its existing
address with a branch and inserts its children. No superseded application-level
paths or root generations are retained. SQLite MVCC owns older versions for existing
reader snapshots. A later writer using an older verified root is refused.

The publication record carries a constant-size root envelope: format, admitted
resource configuration, generation, current leaf count and root digest. The full
directory is no longer decoded or rewritten by every publication. An ordinary
update writes at most depth+1 logical nodes. Each split adds at most two more nodes
relative to a leaf write; the conservative tested bound is depth+1+3*split-budget.
Actual SQLite B-tree/WAL writes are a separate measurement.

Research bounds: 60 entries/leaf (3,841 bytes including tag), 65 bytes/branch,
depth 24, 1,024 current leaves and eight splits/publication. A path reads at most
25 logical nodes and 5,401 node bytes. These are admitted research resource bounds,
not product history limits or proof of unlimited growth. Refusal rolls back the
whole transaction without pruning receipts. Root envelopes encode the configuration
and reopening with a different configuration is refused. Changing these bounds or
authorizing full 100K G2 work requires further review.

## Canonical receipt restoration

The unnormalized hierarchy retains the accepted ordinary rowid payload schema.
The normalized candidate uses the same ordinary rowid organization with an explicit
`INTEGER PRIMARY KEY` alias and indexed, UNIQUE, NOT NULL authoritative hashes.
The explicit alias makes local payload references stable through SQLite VACUUM;
these integers never replace external stable identities or canonical hash commitments.
Its extra column/index footprint is included in measurements.
SQLite documents the distinction in its [rowid table reference](https://www.sqlite.org/rowidtable.html)
and [VACUUM reference](https://www.sqlite.org/lang_vacuum.html).

All outer receipt fields remain: sequence, exact transaction ID, exact intent digest,
decision kind, edit locator and canonical receipt hash. Only the physical body changes.
The `N1` body restores the original eleven fields as follows:

| Canonical body field | Physical normalized representation |
| --- | --- |
| Previous receipt digest | Prior sequence row's digest; seed uses the original zero digest |
| Head, cursor, Redo | Edit receipts derive edit/edit/null; control receipts retain all three |
| Complete lifetime payload hash | Stable local payload reference, resolved back to full hash |
| Parent edit | Edit minus one; seed and controls retain canonical null semantics |
| Checkpoint base | Edit rounded down to the fixed research cadence of 64; controls null |
| Content identity | Original 16 bytes retained |
| Delta/checkpoint payload hashes | Nullable local payload references resolved to full hashes |
| Independent reconstruction oracle | Original 32 bytes retained |

Before insertion, the candidate restores the canonical body and rejects any mismatch
with the input body. Thus this encoding cannot silently accept a different checkpoint
cadence or other non-derivable values. Reads restore the exact canonical row before
existing receipt-hash binding and authenticated proof checks. Missing/changed references
or predecessors therefore fail closed. Historical audit still reconstructs every
state and checks its independent oracle, complete lifetime binding, receipt chain,
terminal negatives, replay identity and cursor/Redo relationships.

The two candidates have distinct current experimental format identifiers (`G3H1`
and `G3N1`). No migration reader or new compatibility promise is introduced. Lossless
restoration of this research contract is not a claim that these derivations cover
future G4/G5 production receipt variants.

An additional normalized `PackedStore` (`G3P1`) keeps branches in a separate rowid
table and leaves in `bucket`. The logical tree, node commitments, canonical receipts
and bounds are unchanged. Splitting atomically moves the old leaf address into the
branch table; ordinary branch updates overwrite existing rows. Lookup considers both
tables and refuses missing or duplicate physical addresses. Full audit checks that
their union exactly matches the reachable tree. This compares physical organization
without adding retained generations or a different authentication algorithm. The
mixed-table candidate remains in the evidence, including its page-density regression.

## Measurement boundaries

`research_history_hierarchy_measure.py` compares a fresh flat-directory baseline,
unnormalized hierarchy, normalized hierarchy and separate-table normalized hierarchy
against the same genuine 1K/10K
G2 fixtures. It compares canonical manifests and reconstructed cold-image hashes,
reports every physical receipt field, payload bytes, database allocation and dbstat
objects where supported, plus verified startup, replay, negative decisions, cold
reconstruction, splits, publication and complete audit. Zlib 6 remains fixed.

Per-publication JSONL records logical node reads/writes and logical authentication
bytes read/rewritten. Path-depth/proof-byte peaks and activation memory are measured.
The proof-byte counter is the local index-path read footprint: full branch and leaf
node bytes, excluding the separately fetched authority and canonical receipt. It is
not a proposed wire-proof encoding or the total reconciliation response size.
These node reads describe logical cache demand, not distinct physical SQLite pages
or actual cache misses. Python's SQLite binding does not expose native cache status;
the configured 2 MiB cache is not a measured ownership/high-water result. RSS also
includes the harness. WAL size/checkpoints, native allocation and Linux own-process
write counters remain separate; none establishes physical device-write amplification.

Separate 16,384-terminal-negative-control probes exercise directory growth; they
are not G2 edits or full 100K qualification. Tests inject split faults and process
termination before/after COMMIT, retain old snapshots, reject stale writes, detect
missing/corrupt pages and invalid coverage, and enforce admission and audit agreement.
Production integration, retention/GC, witness, coordinator, identity recycling,
protocol changes and G4–G7 remain excluded.
