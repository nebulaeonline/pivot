# G3 physical storage accounting

2026-10-27. Read-only assessment of the accepted separate-table, normalized-receipt,
rowid-payload, zlib-6 baseline. No schema, codec, retention or admission change.

## Measurement and model

`research_history_accounting.py` reads the existing native 10K databases using
SQLite `mode=ro` and `query_only`. The measured phase is after the standard probes:
10,000 authoring edits plus seed and 32 controls, or 10,033 retained receipts.
Compressed payload is unchanged by those probes. Database/WAL fingerprints match
before and after collection; canonical receipt manifests match accepted evidence.

The model calculates SQLite record bytes for each table and required index from
actual column values, including serial types, integer widths, record headers and
NULL storage for INTEGER PRIMARY KEY aliases. Index records include their row locator.
These rules follow SQLite's [file format](https://www.sqlite.org/fileformat2.html).
On Ubuntu and Mac, the calculated record payload matches **every one of the nine
dbstat objects exactly**. [DBSTAT](https://www.sqlite.org/dbstat.html) independently
reports used payload, unused space and page sizes. Windows lacks that extension:
its record lengths are modeled, with page/cell versus unused-space attribution unknown.

| Bytes, final post-probe phase | Windows NTFS | Ubuntu ext4 | Mac APFS |
| --- | ---: | ---: | ---: |
| Compressed authoritative payload P | 7,931,306 | 7,931,326 | 7,932,104 |
| Database file | 12,554,240 | 12,521,472 | 12,525,568 |
| Column content, all tables/indexes | 10,945,626 | 10,945,448 | 10,946,424 |
| SQLite record headers | 254,167 | 254,143 | 254,167 |
| Total serialized record payload | 11,199,793 | 11,199,591 | 11,200,591 |
| Unused bytes within allocated pages | Unknown | 1,059,588 | 1,062,627 |
| Page/cell/rowid/pointer overhead | Unknown | 262,293 | 262,350 |
| Free-list pages | 0 | 0 | 0 |

On Mac, `11,200,591 + 1,062,627 + 262,350 = 12,525,568` exactly. The equivalent
Ubuntu identity also closes. There are no unexplained pages or overflow pages in
these two snapshots. Record headers are counted inside record payload, not again
as page overhead. Unused space is not all recoverable: packing constraints and
future insertion room prevent assuming perfect occupancy.

File length is separate from native allocation. Accepted native allocation evidence
records Mac 12,648,448 allocated bytes for this 12,525,568-byte file; Windows and
Ubuntu allocation equals their EOF in the corresponding samples. These are separate
native observations, not a claim that a file hash determines allocated extents.
This partition concerns the main database; the collected WAL is zero bytes. Runtime
SHM, temporary files, logs, OS cache and filesystem metadata are not declared zero or
folded into compressed payload. They remain separate resource/accounting obligations
in the admission proposal. SQLite object page bytes are not per-object filesystem
extent measurements.

## What is being paid for

The Mac record-payload partition makes category boundaries explicit:

| Record content and encoding | Bytes | Meaning |
| --- | ---: | --- |
| Compressed G2 payload blobs | 7,932,104 | Deltas, checkpoints and complete lifetime blobs |
| Payload-table remainder | 416,762 | Stored content hashes, kind/length integers and record headers |
| Receipt table | 1,533,478 | Exact IDs/intents, normalized body, canonical digest, kind/edit and encoding |
| Payload hash index | 375,829 | Hash lookup plus local row locators and encoding |
| Transaction-ID index | 210,563 | Exact transaction lookup plus row locators and encoding |
| Edit-location index | 69,907 | Direct checkpoint/state lookup, including NULL-control entries |
| Authenticated leaves | 643,387 | Scoped key/digest entries, node tags and record headers |
| Authenticated branches | 17,526 | Child commitments, node tags and record headers |
| Publication/schema records | 1,035 | Current authority and required schema definitions |

These sum to 11,200,591. SQL indexes support required indexed queries; an entire
index is not deemed unnecessary merely because some key bytes duplicate table values.
Similarly, authentication digests are not all redundant just because receipt hashes
also exist in SQL rows. A proposal removing a copy must preserve how it is found and
independently validated, with its extra lookup/recovery work measured.

Receipt columns contain 160,528 ID bytes, 321,056 intent bytes, 620,669 normalized-body
bytes and 321,056 canonical-hash bytes. Their SQL record headers cost 90,265 bytes;
other integer content costs 19,904 bytes. The body inventory is exact:

| Encoded body component | Bytes |
| --- | ---: |
| `N1` marker with framing | 40,132 |
| Lifetime references | 20,066 |
| Content identities with framing | 180,018 |
| Delta references | 29,880 |
| Nullable checkpoint references | 10,313 |
| Independent reconstruction oracles with framing | 340,034 |
| Control head/cursor/Redo fields | 226 |

There are 10,001 distinct content IDs and oracle digests, 10,004 distinct intent
digests, 10,033 exact transaction IDs/receipt commitments and 10,161 payload identities.
Storing one fixed-width copy of these particular values totals **1,606,912 bytes**
(20.26% of P). This is a conditional field budget, not an information-theoretic bound:
some values can potentially be derived from other authenticated data at additional
work, and compressed payload may already encode related information. It does not
prove that every possible representation must exceed 20%.

## Conditional minimum and the 20% aspiration

**With the current schema, values, encodings and index keys held fixed, 20% is not
achievable through better SQLite page packing.** The serialized records alone occupy
about 11.20 MB, leaving **41.21% beyond P** even if every page/cell overhead byte and
all unused space were free. Dropping all SQLite record headers as well still leaves
about **38.00%**. Neither idealization is a realizable SQLite layout; each is an
optimistic lower bound for the specified representation.

The 20% aspiration would require a total near 9.518 MB, already about 1.68 MB below
the existing serialized records before any page structure. The actual Mac difference
between file size and record payload is only 1,324,977 bytes. Removing all of it
cannot bridge that gap. Reaching 20% would require changing record/key representation
or derivation/work contracts, not merely reclaiming slack or choosing a codec.

This does not prove that such changes are impossible, desirable or free. The parent
has made 20% an unproven aspiration rather than a mandatory engineering gate.

## Reducibility without pretending guarantees are free

| Cost | Theoretical opportunity | Required tradeoff/review |
| --- | --- | --- |
| Payload hash stored in table and index | One repeated 325,152-byte copy could potentially be avoided by a different hash-to-local layout | Keep bounded indexed lookup, hash validation and local-reference integrity; schema/write costs change |
| Receipt hash in SQL and authenticated leaf | One repeated 321,056-byte copy could potentially be derived from the authenticated index | Prior-hash restoration and audit gain proof lookups; cannot simply drop the commitment |
| Scoped leaf keys | 321,056 stored bytes are derivable from exact IDs; another representation may use IDs/prefix encoding | Preserve unambiguous routing, negative proof and complete coverage; measure proof/decode costs |
| ID in receipt and ID index | Repeated 160,528-byte key copy | A covering/different layout changes receipt body lookup and page behavior; the lookup itself remains required |
| Body marker/type/length framing | Fixed schema could reduce framing | Current strict format/validator must change explicitly; most of the 620,669-byte body is not framing |
| Repeated intents | Only 928 repeated digest bytes in this fixture | A dictionary/reference index is unlikely to repay its cost here; no entropy claim for arbitrary workloads |
| Page slack | About 1.06 MB in the Mac/Ubuntu snapshots | Perfect recovery is impossible; reorganization needs transient space/lifecycle rules, not an inferred GC mandate |
| Content IDs/oracles also inferable from payload | Some semantic information may be recoverable by decoding state | Changes bounded metadata lookup and independent verification work; no subtraction without a reviewed design |

Opportunities overlap, and some move bytes between indexes rather than remove them.
Do not sum them into a promised saving or call all cryptographic material indispensable
in its current physical location. Conversely, omitting required identity/intent/receipt
or oracle semantics would not be storage optimization.

The observed cost is about 1.25 kB per retained decision for this controlled fixture,
with roughly 0.79 kB of compressed authoritative payload per authoring edit. This is
no longer the original 196 MB amplification. It supports further SQLite feasibility
work, not extrapolation to broad Fill/Paste or larger histories. No additional tiny
packing/codec round is recommended solely to chase 20%.

Evidence: `artifacts/g3-accounting/g3-accounting-20261027-b/<rid>/evidence/`.
The [admission proposal](history-scalable-admission-proposal.md) and
[readiness assessment](../handoffs/2026-10-27-g3-storage-admission-assessment.md)
define the review boundary before further execution.
