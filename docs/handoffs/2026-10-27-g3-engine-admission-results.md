# G3 fixed-engine/VFS and small-bound admission qualification

2026-10-27. **Research qualification complete, ready for parent review.** Fixed
SQLite source, real connection telemetry, staged pre-write index planning and failure
isolation passed on Windows NTFS, Ubuntu ext4 and native Mac APFS. The accepted
**1,024-leaf** runtime profile and zlib 6 remain unchanged. No 32K/65K or full 100K run.

[Assignment](2026-10-27-g3-accounting-acceptance-engine-qualification.md) ·
[Implementation, instrumentation and limit definitions](../architecture/history-engine-qualification.md).

## Fixed research engine and VFS

All targets compiled the same official **SQLite 3.51.3** amalgamation. The pinned
archive SHA-256 and `sqlite3.c` SHA3-256 are verified before compilation. Actual
loaded source ID is:

`2026-03-13 10:38:09 737ae4a34738ffa0c3ff7f9bb18df914dd1cad163f28fd6b6e114a344fe6d618`.

The [official release record](https://www.sqlite.org/releaselog/3_51_3.html) explicitly
identifies the WAL-reset correction and the source SHA3. This resolves the known
fix question for these private builds, without inferring vendor backports. Earlier
Windows 3.50.4, Ubuntu 3.46.1 and Apple 3.54.0 evidence remains under those identities;
installed runtimes are not upgraded or requalified by this result.

| Target | Compiler reported by actual SQLite | Selected VFS | Delegate/file-control identity |
| --- | --- | --- | --- |
| Windows x64 NTFS | MSVC 1944 | pivot-observe | win32 |
| Ubuntu x64 ext4 | GCC 15.2.0 | pivot-observe | unix |
| Mac ARM64 APFS | Clang 21.0.0 | pivot-observe | unix |

The amalgamation is compiled directly into the shim with static SQLite symbols.
The store and schema models use that connection factory; native status queries are
not sent to a separately loaded system library. Full compile options, source/build
commands, bridge/library hashes and per-connection identities are retained.

The profile is WAL, synchronous FULL, 4 KiB pages, configured cache -2048 KiB,
mmap off and automatic checkpoints off. Mac measurement runs explicitly enable
both fullfsync settings. No production SQLite binding/package/VFS decision follows
from selecting this research build.

## Genuine workload and identity qualification

Both **1K and 10K genuine G2 fixtures** completed on all three targets, including
independent reconstruction, complete audit, exact replay/negative/cursor probes and
reopen. Canonical payload/receipt manifests and cold-image bytes exactly match each
target's accepted normalized baseline. No formula replay, identity substitution,
receipt pruning, codec change or hidden logical-format migration was used.

| Checkpointed after-ingest database bytes | Windows | Ubuntu | Mac |
| --- | ---: | ---: | ---: |
| 1K | 1,277,952 | 1,290,240 | 1,282,048 |
| 10K | 12,546,048 | 12,517,376 | 12,517,376 |

These are the same retained-size outcomes as the accepted incremental representation.
The 20% aspiration remains retired as a mandatory gate; no storage optimization ran.

## Native memory, pager, WAL and synchronization

The following counters isolate the first 10,000 real publications. Later open/query/
audit probes and SQLite temporary sorting are separate; final process/library totals
must not be substituted for this phase.

| Measured 10K phase | Windows | Ubuntu | Mac |
| --- | ---: | ---: | ---: |
| SQLite-managed heap high-water, bytes | 2,266,336 | 2,266,232 | 2,634,960 |
| Maximum sampled connection cache bytes | 2,154,936 | 2,155,016 | 2,155,264 |
| Maximum dirty pages sampled after SQL steps | 26 | 24 | 25 |
| Pager cache-write count | 106,760 | 106,810 | 106,981 |
| Cache spills reported | 0 | 0 | 0 |
| WAL frame-header write events | 106,760 | 106,810 | 106,981 |
| WAL xWrite bytes | 439,851,392 | 440,057,392 | 440,761,912 |
| Main DB xWrite bytes | 15,822,848 | 15,790,080 | 15,855,616 |
| WAL xSync calls | 10,012 | 10,012 | 10,012 |
| Main DB xSync calls | 6 | 6 | 6 |
| Checkpoint starts/completions | 6 / 6 | 6 / 6 | 6 / 6 |
| Largest committed WAL prefix, bytes | 67,139,552 | 67,139,552 | 67,147,792 |

Pager-write counts and observed complete frame-header writes agree in these runs.
The observed per-publication frame events are median 10, p95 15, maximum 24–26.
Those are measurements, not universal page bounds. The raw index plan still peaks
at ten logical node writes / 4,426 staged node bytes in this fixture and retains the
reviewed worst-case model of 33 writes / 5,537 index bytes. The two are different
layers: whole-publication WAL traffic includes payload, receipt and SQL index pages,
not just application authentication bytes.

The WAL prefix exceeds the **64 MiB trigger by 30,688 / 30,688 / 38,928 bytes**.
This directly confirms that the trigger is not a hard cap. These are committed-prefix
lengths, not a measured peak of allocated extents or all uncommitted suffixes. Endpoint
WAL EOF/allocation and checkpoint results are retained in each raw report. Physical
allocation can differ from file length, especially on APFS.

The cache figures are sampled current cache use, including overhead, not an invented
native cache high-water. They exceed the nominal 2 MiB setting slightly. SQLite heap
high-water is continuous within the private library, not total process memory. The
10K ingest RSS samples are about 29–37 MB and final process peaks about 30–38 MB,
including Python, ctypes and measurement overhead. No root/ticket/OS-cache budget
has been inferred from them.

Windows/Linux use normal VFS sync flags under FULL synchronous mode; Mac measurement
calls use full VFS sync flags. WAL xSync delegate time totals about 15.415 / 33.174 /
46.505 seconds over the 10K phase. This is neither OS syscall count nor proof of
physical media durability: one delegate call may perform multiple file/directory
operations, and host/hypervisor/device behavior remains outside this observation.
Linux own-process write counters also include non-SQLite harness output and are
retained separately; they are not used as physical-device-write amplification.

| Instrumented wall time | Windows | Ubuntu | Mac |
| --- | ---: | ---: | ---: |
| Operation p95 excluding telemetry snapshots/sink, ms | 4.362 | 5.636 | 10.049 |
| Generic backend p95 including telemetry/sink, ms | 4.583 | 5.827 | 10.573 |
| COMMIT p95, ms | 2.293 | 4.317 | 5.982 |
| Verified activation median, ms | 11.581 | 9.071 | 6.581 |
| Existing-receipt lookup median, µs | 1,344 | 856 | 812 |

COMMIT values are recorded directly in raw summaries; all timings include relevant
adapter/instrumentation work. The bridge does per-step dirty-list sampling and ctypes
column dispatch, so these are not Rust/.NET performance predictions. Different hosts,
engine builds versus older vendor baselines and sync paths prohibit cross-host ranking
or attributing previous-to-current timing changes solely to admission.

## Pre-write admission and failure isolation

The reviewed planner is wired into the actual research store. Under the verified
writer horizon it stages bounded leaf bytes, branch dependencies and the receipt-digest
placeholder **before payload/receipt DML**. Bootstrap now stages its seed plan before
schema and payload creation as well. Committing fills the digest and applies that
already admitted plan; counters assert agreement with the independent node/byte model.

Implemented experimental controls are:

- Actual Python index-staging buffer up to **1 MiB**, with smaller limits for refusal tests.
- Pre-write **2 MiB native SQLite headroom check**, under an enforced **64 MiB private-library heap cap**.
- Fresh DB **16,384-page / 64 MiB logical page cap**, verified and tested with a smaller quota.
- Existing **64 MiB WAL checkpoint trigger**, **128 MiB recovery preflight** and pinned-reader backpressure.
- No larger leaf profile, mmap allocation or automatic-checkpoint change.

These do not promise that an admitted operation cannot later fail. Python object/hash
temporaries are outside the encoded staging buffer; SQLite may need more than the
headroom estimate; page balancing/spills are not prepaid frames; DB page caps do not
reserve filesystem blocks. Actual OOM/FULL/I/O failures must remain atomic and explicit.

**150 native tests passed:** 16 actual pinned-engine tests per platform (48), plus
34 original hierarchy regressions on the ordinary/default connection path per platform
(102). The latter preserve original assertions; they are not relabelled pinned-engine
tests or vendor-runtime fix qualification. No new skip or assertion suppression.

The actual-engine cases cover:

- Exact source/VFS/native memory observation, canonical baseline/oracle equality,
  identical retry and conflicting intent.
- Scratch refusal before any data/schema mutation, including bootstrap; native
  headroom refusal before DML; exact old payload/receipt/index/authority preservation.
- Persistent injected allocation failure after payload writes, returning native
  SQLITE_NOMEM and rolling back; a real 512 KiB heap cap refusing 2 MiB deferred
  BLOB materialization without substituting empty data.
- SQLITE_FULL from a deliberately constrained page quota, partial planned-index
  errors, invalid budgets/configuration and rejection of 4,096 leaves.
- Reader-pinned checkpoint/backpressure, old snapshot consistency and stale writer refusal.
- Corrupt authenticated pages, VFS write/sync errors and process death at split,
  partial-node and before/after-COMMIT boundaries.

Injected VFS errors occur at the delegate boundary, not as simulated torn sectors or
power loss. A sync error can leave a committed WAL frame: tests reopen and require a
whole old/new state with complete audit and reconciliation, never assume definitely
absent from an error alone. Tests use a portable FULL profile; Mac's stronger measured
fullfsync settings are separately recorded and do not establish a hardware guarantee.

## Resource decision and separate scaling authorization

**Engine/VFS identity and small-bound failure qualification are established for this
research profile. Complete physical resource reservation is not.** The native heap
cap is real and failure isolation works; the headroom check is not an irrevocable
reservation. No hard total-Python, physical WAL-allocation, filesystem-space or
media-durability guarantee is claimed. Valid recovery near 128 MiB and sustained
multithreaded instrumentation/concurrent checkpoint stress are not qualified here.

Recommendation for parent review:

1. A separately authorized **32,768-control pilot** is reasonable as bounded research
   with the fixed build, current 1,024 leaves, explicit disk provisions and abort on
   any refusal/resource/verification anomaly. It is not permission to raise limits
   after a failed attempt or to claim production admission guarantees.
2. **65,536 controls cannot fit the current profile even ideally**: 1,024 × 60 is
   only 61,440 entries, including seed. That point needs separate authorization to
   implement/validate the reviewed larger profile and its resource guards, informed
   by the 32K evidence. Do not enable 4,096 leaves or run 65K under this assignment.
3. Full G2 scaling, production backend/configuration selection, witness and G4–G7
   remain separate. The index-only pilot cannot certify broad Fill/Paste, content
   reconstruction memory or arbitrary large payload transactions.

No expanded command/runner was added. This packet stops for review at the requested
engine/VFS and small-bound admission gate. Retain the current disk/free-space and
failure-stop requirements; use measured frames/allocations rather than logical index
counts as a physical estimate. Codec and further storage tuning remain parked.

## Evidence and reproduction

Final source: `artifacts/g3-engine/g3-engine-20261027-c/<rid>/` contains native logs,
build commands/options/hashes, source-input hashes, tests, raw whole-operation native
JSONL and 1K/10K summaries/cold images. `comparison.json` isolates the genuine ingest
phase. Its source hashes match the working implementation and all native snapshots.
Native DBs, vendor source and compiled libraries remain under each target's
`~/PivotValidation/g3-engine-20261027-c/<rid>/`. SQLite source ZIP is retained under
shared `artifacts/g3-engine/vendor/` with its tracked pin.

Earlier build smoke evidence (`g3-engine-build-20261027-a`) and preliminary runs
(`g3-engine-20261027-a`, `-b`) remain intact for their tested inputs. Final qualification
adds explicit column-allocation handling and bootstrap pre-write planning/refusal
coverage. Earlier positive results are not substituted for final-source evidence.

Reproduce only the authorized sizes with `research_sqlite_build.py`, an explicit
`PIVOT_RESEARCH_SQLITE` pointing to that private build, `test_engine_store.py`, and
`research_history_engine_measure.py <existing-1K-or-10K-fixture> <new-output>`. The
selected source/checksums must match the pin. This is not a production dependency
installation or permission for an expanded workload.

No new codec, format migration, production integration, witness, retention/GC,
Undo/Redo, protocol changes, identity recycling, product-limit or VM/Defender changes.
G1/G2 and six-RID acceptance remain closed; clipboard ISSUE-0001 remains independent.
