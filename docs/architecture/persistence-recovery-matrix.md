# Persistence failure and recovery matrix

## Current scope and priority

This is the accepted ADR-0008 design for **optional durable hosted storage**.
Further hosted implementation, including the SQLite feasibility gate, is deferred
following the portable-foundation review. It is not required for ordinary local
files and does not select their container. See accepted
[ADR-0009](../adr/0009-portable-documents-and-local-lifecycle.md). Requirements below
remain applicable if hosted work resumes; they are not authorization to proceed.

Status: Accepted ADR-0008 design. These are required provider outcomes, not executed database tests.
Ready is the only normal serving state. Storage failures never trigger a silent
MemoryOnly fallback. Provider error text is not exposed verbatim to clients.

| Event | Durable truth and response | Required recovery |
| --- | --- | --- |
| Crash before commit | No terminal receipt or candidate state if rollback is established; no success acknowledgment | Reopen exclusively; load prior committed generation; same-intent explicit retry may be admitted |
| Crash after commit before response | State and terminal receipt both exist, or both do not; client outcome is uncertain | Recover receipt under authorized durable namespace; identical retry returns original result without applying again |
| Commit succeeds before memory installation | Database is authoritative; memory may be old | Block new state-dependent work/delivery, rebuild the committed projection; never return a fabricated rejection |
| Memory allocation/invariant failure after commit | Receipt remains committed | RecoveryRequired; restart if necessary; no in-memory rollback presented as database rollback |
| Lost response followed by restart | Durable-domain receipt survives ordinary restart; legacy instance-scoped replay does not change contract | Fresh service instance, same store/epoch; future durable client queries stable key. Legacy request remains StaleInstance |
| Identical duplicate operation | Same namespace/typed intent | Return original terminal success or rejection before head/quota checks; do not reinstall an old head |
| Different intent under same operation ID | Existing receipt stays unchanged | Operation-ID conflict; no graph/publication change and no overwritten receipt |
| Different principal or replay dialect | No implicit access to another receipt | Independent namespace plus current Recover authorization; no existence/outcome disclosure |
| Concurrent stale expected head | Exactly one writer wins a shared head; loser changes no state | Record admitted conflict rejection atomically; require client to refresh explicitly and choose its next intent |
| Publication snapshot/receipt insertion fails | Neither publication nor success receipt commits | Roll back whole transaction; prior immutable registry remains authoritative |
| Publication commits but registry install fails | Complete immutable snapshot and receipt exist | Freeze registry-dependent admission; rebuild one registry including bootstrap/durable publications |
| Receipt capacity exhausted before admission | No new operation accepted | Refuse new mutations without eviction; allow readable existing receipt lookup/replay |
| Disk full or I/O error | Error alone does not establish commit outcome | Stop mutation admission; inspect engine transaction state and resolve key after exclusive reopen; remain unavailable if truth cannot be read |
| Commit acknowledgment lost | Indeterminate; no new ID or automatic replay | Freeze affected serving, reacquire ownership if needed, query committed receipt/head; absence counts only after authoritative rollback/recovery |
| Owner session/lock lost | Former owner cannot safely commit or serve coherent new state | OwnershipLost; invalidate permits/projections and stop serving. No transparent takeover/reconnect with old fence |
| Second service opens same store | Existing owner remains authoritative | Bounded ownership failure before mutations/migrations/listening; no TTL or PID-file takeover |
| Interrupted migration | Old complete schema or new complete schema; partial state is not Ready | Inspect ledger/engine recovery under exclusive ownership; restore verified backup or require explicit repair if inconsistent |
| Unsupported newer schema/codec | Cannot safely interpret store | Fail before writes; no downgrade, ignored fields or partial model loading |
| Corruption/inconsistent binding record | Not an ordinary formula parse failure | Failed/quarantined store; preserve evidence, restore verified backup administratively; do not rebind source as repair |
| Restore after rename | Source still names old label, token retains original UUID | Allocate new runtime handles and install stored portable binding, not a name lookup |
| Restore after delete/name reuse | Token points to retired A, replacement B has same label | Preserve missing-reference draft; never retarget to B |
| Restore unresolved/parse-failed draft | Original invalid state persists | Restore failure state/source/diagnostics; only a new formula submission can bind anew |
| Backup interrupted | Source commits unaffected; target may be incomplete | No completed manifest; discard incomplete target through explicit maintenance cleanup |
| Rollback restore to an older backup | Some previously committed records are absent | Rotate recovery epoch before serving; old requests fail scope checks, never interpret missing receipts as proof of noncommit |
| Delivery authorization revoked after commit | Mutation remains durable, result may be withheld | Keep receipt and original outcome; later authorized recovery reports truth |
| Calculation disconnected/cancelled | Captured calculation is ephemeral and holds no write transaction | Existing bounded calculation may finish; no new mutation/receipt is synthesized |

## Recovery state machine

Opening acquires ownership and validates compatibility. Recovering loads a single
committed generation and installs it. Ready admits operations. An uncertain commit
or failed installation moves to RecoveryRequired, blocking new state-dependent
requests and successful delivery until truth is established. OwnershipLost or
unreadable/corrupt storage moves to Failed/closed admission, not a degraded writable
mode. Recovery work has explicit byte/time budgets and does not skip failing models.

In particular, one unsupported/corrupt authoritative model, publication or required
receipt prevents Ready for the entire service. Partial startup with only the
remaining valid objects is forbidden. Portable collection import tests cover its
all-or-error portion; database catalog completeness is a later coordinator test.

If a provider proves that a submitted transaction rolled back, an absent receipt
can resolve as NotCommittedInActiveEpoch. If access is unavailable or ownership
cannot be regained, the only truthful outcome is Indeterminate/Unavailable.
Providers must not expose these distinct cases through one generic NotFound.

Publication and edit projection installation share one coordinator generation.
Already captured computations may execute on immutable state, but delivery waits
for readiness and current rights. Database rollback never rewinds a computation's
captured definitions or mutates an already published revision.

## Durability reporting gate

The current protocol only supports MemoryOnly; this design adds no fields. During
implementation, durable mode cannot be externally enabled until separately
reviewed capability/error/recovery metadata can express store/epoch scope, verified
durability and failure state. Never reuse reserved Durability value 2. If this gate
or filesystem verification fails, refuse durable configuration explicitly. Driver
installation and successful database opening are not evidence of durable service.
