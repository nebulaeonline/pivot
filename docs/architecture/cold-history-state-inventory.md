# Cold-history state inventory and identity boundary

Date: 2026-10-23. **Stage-one inventory/probes accepted. Original inventory evidence below
does not prove cold restoration.**
[Subsequent feature-only cold adapter and executed Stage 1 proofs](../handoffs/2026-10-23-cold-history-restoration-stage1.md)
are separate evidence pending review; the original manifests remain unchanged.
[Parent inventory acceptance and next gate](../handoffs/2026-10-23-cold-history-inventory-review.md).
[Authorized plan](disk-history-feasibility-plan.md),
[architecture approval](../handoffs/2026-10-23-tiered-history-design-review.md),
[ADR-0023](../adr/0023-document-transaction-history.md).
[Parent identity clarification — restart remapping approved](../handoffs/2026-10-23-cold-history-identity-review.md).

## Resolved identity boundary

The core declares NodeId a process-local identifier, not a persistence or wire
identifier (`crates/pivot-core/src/model.rs`). Its model component is allocated from
a process-local AtomicU64 starting at one. Separate processes can legitimately issue
the same numeric pair for unrelated nodes. Cold storage cannot treat that pair alone
as globally durable authority.

Workbook Binding already separates portable GraphNodeId (UUID) from runtime NodeId.
CellId, worksheet/axis/graph UUIDs and range SelectionIds provide other durable logical
identities. Portable import issues a new process model namespace and constructs a
runtime mapping. Historical Expected/Revision values embedded in formula provenance
remain data, never a live request capability.

The parent resolves the inventory's identity question with this contract:

- Preserve durable logical cell/node/range/graph/axis identities, ContentStateIds,
  TransactionIds and original bound-target relationships across cold restore and restart.
- In one live session, cold restoration maps into that session's existing canonical
  model and preserves its runtime node ordinals/identity, including absent historical
  slots and the current committed allocation floor. Existing captures remain isolated.
- On restart, issue fresh runtime/session authority and map durable IDs into a fresh
  process namespace. Persisted runtime pairs may be historical provenance or mapping
  evidence, but cannot grant authority or collide with unrelated open models.
  All historical states loaded in one reopened session must share a coherent mapping;
  repeated cold loads must not independently assign conflicting runtime identities.
- Preserve committed lifetime allocation counts/floors independently of this mapping;
  a fresh runtime namespace must not recover allocation quota for an attached history.
  Keep durable identity reservations for objects on discarded/unreachable branches.
- Open independent copies with isolated runtime namespaces and authority, even when
  their historical logical UUIDs overlap. Compare durable identity/binding equivalence
  across restart, not process-local NodeId bits.

The contract is approved; a cold adapter/remapper is not implemented. Do not change
NodeId representation or introduce persistent model-number ownership in this
experiment. Do not reset/import NEXT_MODEL, manufacture raw NodeIds or infer ownership
from matching numeric bits. Future durable runtime identity design is separate work
only if a concrete requirement demonstrates value beyond durable logical UUIDs.

Making the existing bits persistent is not principally a storage-code change. The
isolated-case implementation could persist the current pair and reserve its model
number on reopen. The product contract must additionally handle unrelated models
allocated before reopen, copied documents with the same model number, multiple open
instances and independently diverging branches. Today NodeId, Revision and PortId
share the process-local model number. A durable identity design must separate logical
identity from live instance/request authority without letting cross-instance handles
bind accidentally. Replacing the model component with a durable UUID is another
option, with effects on those types, provenance and their consumers. Neither requires
rewriting decimal arithmetic or parsing, but both are cross-cutting identity work,
not simply a new history field. These alternatives explain the earlier interlude;
neither is selected for this experiment. No schedule estimate is claimed.

## Authoritative and derived state inventory

The source locations below describe the existing implementation; they are not a
selected cold schema. All counts, lengths and cross-references need checked bounds
before private construction and atomic publication.

| Existing owner/field | Cold representation requirement | Reconstruction/validation requirement |
| --- | --- | --- |
| Workbook worksheet, graph_id, rows/columns namespace and length | Preserve logical UUIDs/axis bounds | Reject nil/duplicate identities and invalid coordinates |
| Workbook history_content | Preserve ContentStateId | Do not mint a replacement on restore |
| Workbook session/state; core Revision/Version | New live authority, with old values only as provenance | Exact old Expected rejects after activation/restart |
| Data records keyed by CellId | Preserve positioned, blank, text, decimal, formula and retired records | Position/content consistency, uniqueness and existing byte/live/allocated limits |
| DecimalInput parts | Preserve coefficient/sign/scale exactly | Existing Decimal96 part validation; no text/float round trip |
| Formula/TextFormula tokens | Preserve already-bound cell/range IDs and operators | Postfix/target validity, limits; never resolve names or rebind source |
| TextFormula references/source/diagnostic | Preserve original spans, submitted coordinates, bound targets and diagnostic | UTF-8/span/length checks; do not replace diagnostic with reparsed result |
| TextFormula transaction_base/edit_ordinal/resolved_graph | Preserve historical provenance | Never interpret as request or mutation authority |
| Data bindings | Preserve portable cell-to-node UUID mapping plus scoped runtime mapping information | Bijection and coverage of live/retired records; distinguish namespace from ordinal |
| Data positions/nodes | Derived indexes | Rebuild from validated records/bindings with duplicate checks |
| Data tokens/source/metadata/live/history_payload counters | Derived resource measures | Recompute from bounded decoded records; never trust serialized accounting |
| Data directory/adapter/annotations | Structured/projection proof state | Reject these modes in ordinary-history experiment; do not snapshot external source contexts |
| Core records: slot, live, name, definition | Preserve live/retired canonical definitions and node correspondence | Preserve retirement/missing-reference meaning; no dummy numeric replacement |
| Core Compiled::Input and values | Preserve authoritative input decimal | No persisted evaluation cache |
| Core Blank/NonNumeric | Preserve distinction | Do not turn Blank into zero or text into a numeric graph input |
| Core Compiled::Formula source/binding | Preserve tokens, scalar deps, aggregate selection IDs and binding error | Verify data/reference consistency, no authoring replay |
| Core names/reverse indexes | Derived lookup/invalidation structures | Rebuild from records and bindings, without text name resolution |
| Core invalid index/error/proof/import_invalid/local_validation | Separate historical diagnostic data from derived validation state | Preserve accepted invalid/cyclic semantics and error ordering; untrusted serialized proof bits cannot certify a graph |
| Core selection descriptors, including retired IDs | Preserve IDs and rectangles/retirement | Unique descriptors, current selection/identity limits |
| Core selection placements | Preserve space/coordinates and logical node correspondence | Rebuild rows/locations/coverage/consumers; validate point/consumer limits |
| Core selection_invalidated | Invalidation bookkeeping, not source publication state | Specify safe reconstruction before using a restored root for later edits; do not silently lose pending invalidation semantics |
| Core next/next_port and history.selections | Current committed lifetime ledger outside undoable content | Preserve floors even when historical records are absent or a branch was discarded |
| Core bytes/live/max_name/history accounting | Derived metadata | Recompute/check against actual bounded reconstruction |
| Core model_name/ports/captured_scalar | Name preserved; interfaces/captured-source modes excluded initially | Ordinary-workbook mode check; no foreign runtime/source attachments |
| Runtime Memory/Shared/Pages/Index/Owned/Ticket | No pointers/layout/charges in durable image | Private admitted construction, exact distinct-allocation charge and bounded cleanup |
| Captures/results | Independent owners, not serialized history | Old capture of B remains valid after restoring A and branching C |
| Saved marker/prepared bytes/checkpoint/ticket/path/fingerprint | Outer lifecycle authority, separate from undoable content | Exact captured Save identity; native installation/uncertainty contract unchanged |
| Cursor/TransactionId/HistoryInstanceId/commit decision | Future history-store metadata, absent from current Workbook | Durable head/cursor/ledger agreement is later-stage work, not proven by these probes |

Sources: `pivot-workbook/src/{lib.rs,persistent.rs,transfer.rs,history_experiment.rs}`;
`pivot-core/src/model/{structural.rs,selection.rs,transfer.rs,integration.rs}` and
`model/structural/{history.rs,materialization.rs}`.

## What the executable probes establish

`crates/pivot-workbook/tests/history_cold_inventory.rs` uses existing APIs only:

1. A scalar binding to a moved cell, aggregate range, literal text, invalid draft
   and cyclic formula preserve exact tested CellViews through portable transfer.
   That transfer deliberately creates a new runtime namespace/session and new
   ContentStateId. It is therefore not a history identity capsule.
2. A has one allocated cell; B adds another; accepted activation returns to A with
   B's floor of two. Creating another cell remains rejected under an allocated limit
   of two. Portable export of this root rejects because the supplied cell mapping
   no longer spans the full canonical allocation ledger. It does not silently lower
   the floor; the existing transfer is safely unsuitable for this history state.
3. Two real child processes independently allocate `NodeId { model: 1, node: 0 }`.
   This is valid process-local reuse, not a current core collision or a failed ABA
   test. It shows why raw persisted bits cannot prove cross-process identity.

There are three probes plus one child-test entry point. They do not serialize or
cold-restore history, prove a disk transaction, or demonstrate restart recovery.
Portable import is used only to inspect its existing boundary, not as the proposed
history mechanism. The accepted hot-root ABA tests remain unchanged.
Windows and Linux each passed all four entries (including the helper). Windows also
passed 99 workbook all-feature tests and strict workbook/all-target/all-feature
Clippy; formatting passed. Linux used native source under
`/var/tmp/pivot-history-feasibility` and the existing target cache on the Windows
mount. No native macOS, filesystem durability or full workspace CI claim is made.

## Separate transaction/identity scaling measurement

`history_identity_inventory` uses the existing feature-only root activation. It is
not a durable history log and does not claim to retain 10,000 Undo actions.

- Existing-cell replacement: 10,000 changed commits, one allocated identity, no
  rejection, zero final charge.
- Create/restore-A/branch: one initial identity plus 8,191 changed commits, then
  `Limit` at floor 8,192 with one active cell. The rejected edit preserves authority
  and content. Zero final charge.

This isolates the finite identity ceiling from live occupancy and transaction count.
Disk eviction alone cannot change that ceiling. This is a real product scaling
constraint to return for review; no quota or identity guarantee was changed.
The parent accepts this finding and classifies identity-space scalability as a
separate architectural prerequisite for removing experimental workbook limits.
No quota increase, recycling or rebasing is part of the current experiment.
Machine-readable platform results and test/lint logs are in
`artifacts/history-cold-inventory/`. Timing is incidental, not a disk latency claim.
Windows elapsed times were 425 ms for replacement and 421 ms for branching; Linux
1,632 ms and 1,357 ms respectively. These include per-iteration cleanup flushes and
no disk-history writes; they are not comparable to durable transaction latency.

## Gate status

Inventory and boundary probes are accepted; the identity contract is resolved.
The inventory itself did not implement restoration. The subsequent adapter now
passes the separate same-session exact-runtime-ID, fresh-process logical-equivalence
and simultaneous-copy tests under the approved mapping contract. See the linked
Stage 1 packet for actual implementation/coverage and limitations; acceptance remains
pending. The stop condition for broad redesign or weakened invariants is unchanged.

No checkpoint/delta/shared-object comparison, commit journal, recovery/decline,
store association or corruption/disk-full experiment is claimed by the inventory.
The later adapter adds no production coordinator or WB5/protocol/desktop/Optimize change. The
remaining authorized stages remain gated on exact cold reconstruction and the
identity-scaling result above; do not skip them to produce storage-size benchmarks.
