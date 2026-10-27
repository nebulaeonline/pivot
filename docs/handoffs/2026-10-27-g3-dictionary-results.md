# G3 trained-dictionary zstd investigation

2026-10-27. Owner-requested research complete on Windows NTFS, Ubuntu ext4 and native
Mac APFS. **Trained dictionaries improve payload compression but do not solve the
metadata/page-efficiency problem in this corpus.** The unchanged zlib hierarchy
qualification is [reported separately](2026-10-27-g3-hierarchy-normalization-results.md).

## Controlled experiment

Use dotZstd's existing native libraries, hashed per target, reporting libzstd 1.6.0.
No wrapper source or production dependency changed. Train one fixed set on the Mac's
1K history and apply exactly those dictionary bytes to all native 10K histories.
The 2,023 deduplicated binary samples include deltas, checkpoints, lifetime records
and normalized receipts; zero evaluation samples match training sample hashes.
Training takes about 0.111 seconds for the 8 KiB shared dictionary, 0.103 seconds
for 32 KiB, 0.076 seconds for the 8 KiB delta dictionary and 0.002 seconds for the
512-byte receipt dictionary. These are single Mac training samples, not target rankings.

Candidate physical databases are fresh ordered copies, including a zlib control;
the published incremental zlib baselines remain intact. All file, dictionary,
catalog and framing bytes are charged. Dictionaries are external files, one copy
charged per candidate even if a future deployment could share them.
[Corpus, encoding, integrity and measurement contract](../architecture/history-dictionary-experiment.md).

## Retained file size

Database plus external dictionary file lengths, decimal MB, after complete validation
and checkpoint. These are density/decode experiments, not incremental write results.

| Candidate | Windows | Ubuntu | Mac |
| --- | ---: | ---: | ---: |
| Fresh zlib-6 copy | 12.554 | 12.521 | 12.526 |
| Plain zstd-3 | 12.558 | 12.526 | 12.530 |
| Plain zstd-9 | 12.427 | 12.394 | 12.399 |
| Zstd-3, shared 8 KiB | 12.567 | 12.534 | 12.538 |
| Zstd-3, shared 32 KiB | 12.591 | 12.558 | 12.562 |
| Zstd-9, shared 8 KiB | 12.435 | 12.403 | 12.407 |
| Zstd-3, family dictionaries (8 KiB + 512 bytes) | 12.567 | 12.534 | 12.538 |

The best total size is **plain level 9**, saving 126,976 bytes (about 1.0%) per target.
The strongest dictionary payload result is level 9 with 8 KiB: about **6.93 MB** of
stored payload versus **7.93 MB** for zlib, a 12.5–12.6% reduction. At the same level,
adding the dictionary saves another roughly 0.21 MB of logical payload versus plain
zstd-9, but no further SQLite pages in this layout, and adds the 8 KiB dictionary.

At level 3, family dictionaries reduce payload to 7.08–7.11 MB, versus plain
7.45–7.46 MB. That is a useful logical saving, but does not shrink the allocated
database pages. On Mac, payload pages stay at 8,994,816 for the level-3 candidates;
unused payload-page bytes grow from 550,034 (zlib control) to 1,372,687 (family).
This is measured page slack, not missing compressed data or a claim that dictionaries
can never save physical pages under another layout/workload.

Every normalized receipt body chooses explicit raw encoding because its zstd frame
would be larger, including with the trained receipt dictionary. Stored receipt-body
bytes rise from 620,669 to **640,735**, exactly two header bytes per receipt. Outer
transaction IDs, intents and canonical receipt hashes remain uncompressed and indexed.
The remaining receipt bodies already contain substantial identity/oracle material;
this experiment does not show useful dictionary savings for that normalized family.

The total-space-beyond-compressed-payload ratio becomes roughly **67.9–79.4%**, compared
with 57.9–58.3% for the fresh zlib control. The 20% target remains unmet: better payload
compression shrinks the denominator while much of the metadata/page allocation remains.
No raw-record denominator is substituted to declare a pass.

Native allocation is separately retained in each report. For example, the Windows
family candidate is 12,567,040 file bytes but 12,570,624 allocated including dictionaries;
Ubuntu family is 12,534,272 file bytes and 12,541,952 allocated. Small dictionary files
pay native allocation granularity. No external-dictionary size is omitted as a shared
application asset.

## Runtime costs and integrity

The shared 8 KiB dictionary prepares about 219,712 CDict bytes plus 35,544 DDict bytes;
32 KiB prepares about 440,896 plus 60,120 bytes. Family dictionaries prepare about
259,712 plus 63,408 bytes. Serialized bytes, compression/decompression contexts and
Python allocations are additional; these figures are resident-object snapshots, not
peak training/process memory. Mac dictionary loading samples are about 0.042 ms for
shared 8 KiB at level 3 and 0.070 ms for shared 32 KiB; level-9 preparation differs.

Per-block encoding p95 for plain level 3 is approximately 32.8/33.3/21.1 µs
(Windows/Ubuntu/Mac); shared-8-KiB level 9 is 281.7/240.2/170.0 µs. Corresponding
decode p95 samples are about 15.7/13.6/10.0 and 19.5/14.3/10.3 µs. These distributions
mix payload and receipt blocks and include ctypes/buffer overhead. They are not
publication latency or .NET/Rust API measurements. Verified open is measured after
contexts/dictionaries are prepared; its wall time must not be reported as including
initial dictionary loading. No new production latency budget is proposed.

All **21 candidate databases** passed verified open, complete historical audit,
canonical receipt-manifest equality, typed-hash validation of every decoded payload
and independent cold reconstruction. The same external dictionary bytes and canonical
training manifest match across all three targets; raw JSON manifest-file hashes differ
on Windows due to line endings and are not used to claim dictionary inequality.
New native codec tests: **18 passes**, covering dictionary/plain round trips,
missing-dictionary refusal without fallback, explicit raw blocks, truncated/trailing
frames, invalid tags, canonical/limited lengths and training admission/memory reporting.

The external catalog's full dictionary digests are bound into publication authority;
short zstd IDs are not trusted as cryptographic identifiers. Missing or mismatched
dictionaries do not trigger a legacy/plain fallback or retraining. Complete historical
payload/receipt commitments remain unchanged after decoding. This is an offline reader
candidate with no mutation API, not a codec-integrated transactional production store.

## Evidence and recommendation

`artifacts/g3-dictionary/g3-dictionary-20261027-a/<rid>/` contains commands, native
filesystem/runtime/library hashes, tested codec input hashes, logs, the frozen trained
dictionary set, per-candidate reports and dictionary files. `comparison.json` and
`dictionary-equivalence.json` reconcile results and source/dictionary identity.
Native physical databases remain under each target's
`~/PivotValidation/g3-dictionary-20261027-a/<rid>/comparison/`.

The first Mac launcher failed before execution because a template substitution also
changed an environment-variable name. Its refused script and console are preserved as
`worker-syntax-refusal.py` and `console-syntax-refusal.log`. The corrected file-based
launch passed; no failed comparison was relabelled successful.

**Recommendation:** retain trained dictionaries as a supported research option for
small repetitive records, but do not select a production codec or claim the metadata
problem solved. The measured immediate obstacle is physical page/metadata overhead;
a broader representative corpus or a different record/page organization could change
the result and requires its own evidence. Further tuning/retraining is not justified
as a substitute for understanding that overhead in this bounded fixture.

No production format migration, network-message work, full 100K G2 run, witness,
retention/GC, coordinator, Undo/Redo, identity recycling or product-limit change.
The prior G3 zlib results remain valid for their frozen implementation inputs;
G1/G2 and six-RID acceptance remain closed, and G4–G7 remain gated.
