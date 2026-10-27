# G3 hierarchy and dictionary research accepted; feasibility assessment next

Parent disposition, 2026-10-27: accept the hierarchical authenticated directory and
lossless receipt normalization within their measured scope. Separately accept the
owner-authorized trained-dictionary investigation. **G3 remains active.** No production
backend, codec or resource policy is selected.

## Accepted baseline and findings

Continue research with normalized canonical receipts, separate branch/leaf tables,
ordinary SQLite rowid payload storage and zlib level 6. Preserve exact transaction
identities/intents/replay, authenticated presence/absence, complete lifetime authority,
atomic publication, old-reader isolation and independent reconstruction oracles.

The qualified incremental baseline occupies 12.52–12.55 MB at 10K, with 249 native
test passes and three explicit skips. Path-local updates avoid permanent superseded
paths; receipt bodies shrink from 1,687,396 to 620,669 bytes. Logical index writes
fall from about 71.5 to 32.8 MB. The 16,385-receipt control probe is accepted within
its bounds, not as full G2 scaling qualification.

Archive the dictionary comparison as accepted research: 18 additional native codec
tests and 21 validated offline candidate databases. Its plain zstd-9 density result
is not a replacement for the qualified incremental zlib baseline. No further codec
optimization round is the next priority. Preserve all evidence and original refusals.

[Accepted hierarchy/normalization results](2026-10-27-g3-hierarchy-normalization-results.md) ·
[Accepted dictionary investigation](2026-10-27-g3-dictionary-results.md).

## Revised storage objective

Broader overhead remains 57.8–58.2% of compressed payload. The parent now treats 20%
as an **unproven research aspiration, not a mandatory engineering acceptance target**.
It is neither achieved nor silently replaced by a favorable denominator. Establish
whether the cost is reasonable for 10K individually addressable, authenticated,
durable transactions and whether the original percentage is physically achievable.

## Next assignment: scalability and storage-efficiency feasibility gate

**A — Physical accounting.** Build a defensible minimum-storage model for the 10K
fixture. Separate indispensable authoritative bytes, cryptographic authentication,
duplicated representations, required indexes, SQLite row/page overhead, unused space
and filesystem allocation. Identify theoretically reducible costs without losing
lookup, authentication or recovery semantics. Keep compressed authoritative payload
as the primary denominator and report evidence rather than forcing a percentage.

**B — Extensible admission proposal.** Analyze the 1,024-leaf, depth-24 and
per-publication split bounds. Document a larger configuration, worst-case costs and
explicit refusal behavior before any expanded execution. Distinguish structural
capacity from memory, scratch, transaction and physical storage limits. Preserve
logical identities and all receipts; no permanent per-publication path copies or
silent increases to existing research bounds.

**C — Conditional scaling.** After safe admission/growth is demonstrated, controlled
index-only experiments may investigate beyond the current probe. Retain native
NTFS/ext4/APFS evidence and measure physical database/WAL allocation, logical writes,
indexed lookup, verified startup, cold reconstruction, publication/COMMIT and process
memory. A bounded full-G2 workload requires a separate proposal and parent authorization,
including disk reservations and stop conditions. Synthetic controls remain distinct
from genuine G2 transactions.

**D — Readiness comparison.** Compare evidence directly with original G3 requirements.
List unresolved SQLite engine/VFS qualification, physical sync, WAL/read-lease bounds,
memory admission, storage scaling and recovery questions. Recommend backend/configuration
selection readiness or one narrowly scoped further investigation.

## Immediate stopping boundary

The parent explicitly directs: “Stop and return for parent review when the
physical-storage assessment and scalable admission proposal are complete, before
any full 100K G2 workload.” The immediate deliverable is that assessment/proposal
packet with the readiness comparison. Preserve the conditional scaling sequence;
do not use it to bypass this review boundary or treat it as full-G2 authorization.
This reconciliation records the assignment; new feasibility work has not started.

No production integration, codec adoption, Undo/Redo, witness, G4–G7 implementation,
protocol/workbook-format changes, GC, retention policy, identity recycling or product
limit changes. G1/G2 and six-RID execution acceptance remain closed. Clipboard
ISSUE-0001 remains separate; Dev Drive work stays shelved.
