# Build output and validation artifact retention

Adopted 2026-10-29 by owner direction and accepted by parent review as the ongoing
housekeeping standard, without another acceptance process. Applies to this workspace and native validation
work directories on Windows, Linux and macOS. This is housekeeping, not a new
validation milestone. Moving an old report into an archive does not require rerunning
its tests or changing its recorded outcome.

## Retention policy

| Material | Keep expanded | Historical retention |
| --- | --- | --- |
| Project source, Git history, docs, live validation/SSH configuration and user workbooks | Normal working locations | Never prune as build output |
| Current developer build cache (`target`, project `bin`/`obj`) | At most one working cache per active checkout/RID | Disposable; do not archive compiler caches |
| Per-run Cargo/.NET/package caches and duplicate build trees | Only while a run or its immediate investigation needs them | Delete after results are collected; retain reports first |
| Published executables, libraries and symbols | At most **two most recent completed runs per RID/suite**, counting success and failure; one physical copy per logical run | Prune older product sets, retaining published-file inventory/hashes, compiler information and reports; pin a required baseline explicitly |
| Test receipts, original logs, failure reports, screenshots, metrics, source-input hashes and artifact inventories | Latest **two completed logical runs per RID/suite**, plus active/recent feature context | Archive older closed runs; retain successes and failures equally |
| Test workbooks, SQLite/research fixtures and dirty-source snapshots needed to understand/reproduce evidence | Active investigation/current feature | Include in historical archives; redundant identical source copies may be consolidated after hash verification |
| Releases or an explicitly pinned investigation | As required | Exempt from automatic pruning until owner releases the pin |

Two runs is a maximum for build products, not a requirement to keep reconstructible
managed binaries or headless test executables. Those may all be discarded once
reports are collected. The ordinary developer cache is a convenience, not evidence.
Do not keep a separate cache for every feature, retry, preflight or report collection.
A collection copy is not another logical execution.

A pin is a `.retain` file in the run directory, with its reason and owner. Preserve
that run's payloads/configuration until the pin is removed. Keep active source trees,
uncommitted changes and current validation configuration outside archive selection.
If a frozen source differs from Git HEAD, retain the snapshot or an equivalent
complete source bundle with its input manifest; a commit ID alone is insufficient.

Archives are retained indefinitely for now: no age-based deletion of historical
report archives is authorized. Their small compressed size is preferable to losing
review evidence. Revisit archive lifetime if it becomes a measured storage problem.

## Cadence and space limits

- **After a validation batch:** collect results, then discard superseded run caches
  and build products outside the two-run window. Failed runs retain their diagnostics.
- **After each feature milestone:** archive closed evidence outside the hot window,
  remove duplicate source/build copies, and record the space recovered. Preserve the
  active feature and pins. Use the existing feature handoff; no cleanup acceptance packet.
- **Weekly during active development:** check this workspace and each used native
  validation volume. Apply the same pruning/archival policy; do not accumulate copies
  just because another platform has more disk space.
- **Before a full validation batch:** check the native target's free space. Aim for
  at least 15 GiB or twice the last batch's additional footprint, whichever is larger;
  reclaim disposable caches or expand storage before a batch that would not fit.
- Review the shared workspace if it exceeds **15 GiB**, or a developer cache exceeds
  **8 GiB**. These are housekeeping triggers, not permission to discard retained
  evidence. Clean a large cache when idle; keep archives and active configuration.

The person or agent finishing the batch/milestone performs this maintenance. The
weekly check is the fallback. There is no requirement to run tests again after
removing only generated outputs or moving byte-verified evidence into an archive.

## Archive layout and verification

Use self-contained ZIP/ZIP64 archives with ordinary Deflate compression so Windows,
macOS and Linux can restore them without an external dictionary or special service.
Each closed run becomes `artifacts/archives/<original-relative-path>.zip`; legacy
feature groups may be archived together with their internal paths unchanged.
For example, `artifacts/validation-matrix/RUN/` becomes
`artifacts/archives/validation-matrix/RUN.zip`.

Archive members retain paths relative to `artifacts/`. Include a manifest listing
original paths, sizes and SHA-256 hashes. Decompress/read every member and compare
its size/hash **before removing the expanded directory**. Leave the original intact
on any verification error. Write the archive's own hash, original path, file count
and compressed/uncompressed sizes to `artifacts/archives/index.jsonl`.
Pruned build products are recorded separately in `pruned-build-products.json`;
report inventories may still name those discarded binaries. That is historical
metadata, not a claim that the binaries remain available locally.

A report remains exactly as written, including Failed/Stopped/qualified outcomes.
Do not rewrite original result files or old handoffs merely because their payloads
were archived. Use the archive index to find an old path. Preserve old filenames
inside the ZIP so restoring recovers the path referenced by the handoff.

From the repository root, restore a run on Windows with:

```powershell
Expand-Archive -LiteralPath 'artifacts/archives/validation-matrix/RUN.zip' -DestinationPath 'artifacts'
```

Or on any supported platform:

```sh
python -m zipfile -e artifacts/archives/validation-matrix/RUN.zip artifacts
```

Restore only when the destination run is absent, or to a separate temporary directory
to avoid overwriting a live run. Verify the extracted files against
`_pivot_archive_manifest.json` before relying on them. Restoring reports needs no
native filesystem qualification; rebuilding/testing still uses native local storage.
After investigation, return the run to the ordinary archive/hot-window policy.

## Cleanup boundaries

Resolve every recursive deletion target and verify it remains inside the intended
workspace/native validation root. Do not follow linked directories or delete a
live worktree, active run, fixture still being used, configuration or secret as a
cache. Remove ordinary Cargo outputs with `cargo clean` when appropriate; inspect
custom historical target directories separately. .NET `bin`/`obj` directories are
regenerable, including those inside archived source snapshots.

Release a disposable process holding a generated build file only after confirming
that it owns files in this workspace. Preserve the editor and unrelated processes.
Record what was pruned/archived and the before/after sizes. No VM provisioning,
remote orchestration or unrelated infrastructure change is part of this policy.


## Initial application

On 2026-10-29 the shared tree was reduced from **105.0 GiB to 1.85 GiB**.
Historical Cargo outputs accounted for 91.5 GiB. Generated .NET output, obsolete
per-run caches and superseded published products were pruned. **361 ZIPs** preserve
105,075 historical files; 2.339 GiB of expanded evidence occupies 0.633 GiB compressed.
A restore check matched all 344 extracted files from a sample archive. All 368
rounding implementation inputs still match their successful validation hashes.
Source, live configuration and current evidence remain intact. Detailed operational
receipts are in `artifacts/workspace-cleanup-summary.json`, the archive index and
`pruned-build-products.json`; the policy, not this one-off tally, controls future work.
