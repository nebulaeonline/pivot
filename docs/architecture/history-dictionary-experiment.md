# Trained-dictionary compression investigation

2026-10-27. The owner explicitly authorized zstd dictionary research while the
hierarchy/normalization qualification was finishing. The accepted assignment's zlib
baseline remains unchanged and separately evidenced. This adds an offline codec
comparison, not production codec selection, migration or protocol work.

## Corpus and implementation

`research_history_dictionary.py` uses the native libzstd binaries supplied by
`J:\coding\dotZstd\dotZstd\runtimes\<rid>\native`. Each native copy is hashed and
reports its actual runtime (1.6.0 in these binaries). This Python harness calls the
stable C ABI directly; it does not qualify the .NET wrapper API, adopt a Rust crate
or add a product dependency. The source wrapper remains unmodified.

Train once on the Mac's genuine 1K normalized history, then freeze and use identical
dictionary bytes on all three native 10K histories. Training families are raw binary
deltas, checkpoints, complete lifetime records and physical normalized receipt bodies.
Samples are deduplicated. The 2,023 unique samples contain 4,208,728 delta bytes,
287,418 checkpoint bytes, 9,697 lifetime bytes and 61,997 receipt bytes.

Compare zlib 6; plain zstd levels 3/9; shared 8/32 KiB dictionaries at level 3;
8 KiB at level 9; and family-specific level-3 dictionaries. Training uses
`ZDICT_trainFromBuffer`, at least 64 unique samples and at least 100 times the
dictionary capacity in sample bytes. Family training admits an 8 KiB delta dictionary
and 512-byte receipt dictionary. Only 16 checkpoint and four lifetime samples exist,
so no family dictionary is trained for them; their explicitly selected codec is plain
zstd, not an error fallback. The global dictionaries include all available families.

No 10K evaluation data is used for training or parameter search. Exact sample-hash
overlap is zero on all three targets. These histories exercise the same controlled
42-cell workload; this is held-out identity/data evidence, not a broad production
corpus of every future message type. Future network/protocol messages are outside
this investigation.

See upstream [dictionary-training guidance](https://github.com/facebook/zstd/blob/dev/lib/zdict.h)
and [stable compression/dictionary APIs](https://github.com/facebook/zstd/blob/dev/lib/zstd.h).

## Encoding and external dictionary authority

Each research block stores an explicit raw/plain-zstd/dictionary-zstd tag and a
canonical bounded raw-length varint. Raw storage is selected by the writer if a
compressed frame would be larger. The decoder never tries raw/plain/another
dictionary after a malformed or unsupported native frame. It rejects unknown tags,
noncanonical/oversized lengths, truncated or concatenated frames and missing dictionary
IDs, and checks exact output length. The admitted decoded block bound remains 32 MiB.

An external dictionary catalog binds each nonzero zstd dictionary ID to its full
SHA-256 digest and size. Its digest is included in the current experimental publication
format identifier (`G3Z1` plus catalog digest), so changing the catalog cannot preserve
the same publication authority. Dictionary-ID collisions are refused. Caller-supplied
loaded bytes must match the catalog before verified open. A short zstd dictionary ID
alone is not treated as a cryptographic identity.

Decoded payloads retain their original typed content hash; decoded receipts restore
the original normalized body and then the exact canonical receipt before existing
hash/proof checks. Full audit reconstructs the history with its independent oracles
and complete lifetime authority. The codec candidate exposes no publication, cursor
movement or reconciliation API: it is an offline density/decode experiment.

Each candidate conservatively pays for one external dictionary copy, its native file
allocation, codec catalog/pages, block headers and raw blocks. No free amortization
across installations/documents is assumed. An external dictionary is a required
durable decode dependency; a production design would have to decide how portable
documents retain or resolve its exact bytes. Retraining cannot substitute for the
recorded dictionary. No new product compatibility commitment is made here.

## Measurement boundary

The original history databases are not rewritten. Fresh native candidate databases
copy the same logical tables/identities in the same order, preserving explicit local
payload keys. A fresh zlib copy controls for copy/packing effects. No VACUUM is used.
These are offline physical-density measurements, not incremental publication or
production migration performance. Checkpointed database and dictionary file lengths
and native allocations are separate from compressed logical bytes and page slack.

All candidates undergo verified open, complete audit, canonical receipt-manifest
comparison, typed-hash checks for every decoded payload and independent current-state
reconstruction. Microtimings include Python/ctypes dispatch and buffer allocation;
they are not .NET/Rust application benchmarks. Native CDict/DDict sizes and loading
times are reported, plus context-size snapshots after the workload. Context snapshots
are not native peak-memory measurements; training peak memory is not instrumented.

The ordinary compressed-authoritative-payload denominator remains primary. Dictionary
and other metadata do not become free when payload compression improves. Report both
total retained bytes and the resulting ratio. No codec or compression level is selected
for production from this corpus alone.
