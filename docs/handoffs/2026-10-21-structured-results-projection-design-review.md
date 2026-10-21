# Structured-result/projection design — review decision

Date received: 2026-10-21. Parent review supplied by the user.
**Disposition: Approved with clarifications. Bounded experimental proof authorized;
implementation not started. Permanent product integration is not approved.**
[ADR-0019](../adr/0019-structured-results-and-projection-identities.md),
[design packet and acceptance matrix](2026-10-21-structured-results-projection-design.md).
The worksheet-text implementation remains accepted and closed.

## Accepted architecture and clarifications

Keep producer, immutable result-generation, schema/field, producer-scoped LineKey,
projection and presentation revision identities distinct. Typed UUIDs are acceptable;
no distributed identity service, general schema registry or competing dependency
graph. One fixed document-owned InvoiceLines descriptor is enough. Its relationship
to a future graph-backed producer must be explicit, without speculative integration.

Unique LineKeys identify invoice lines; product IDs and visible ordinals do not.
Reject absent/duplicate keys atomically. MissingRecord applies when a formerly
present key is absent from a valid new generation. A returning key may resolve again
only under the producer's same-logical-record contract. Old captures never change.

Publication and capture use **one document-owned coordination boundary** to pin
worksheet/graph roots and the result-generation map coherently. Do not independently
read roots and label them one capture. Detached evaluation needs no global lock.
Keep worksheet revision, result epoch and presentation revision separate. Failed
publication retains the last successful generation; a valid empty result is distinct
from failure/unavailability. Recalculation does not publish or refresh.

TotalAmount uses checked Decimal96 addition in canonical record order. Visual sorting
must not change that order. Supply the captured total to an existing numeric graph
invocation through temporary public inputs. Errors, missing data or schema/type
failures must not become zero or a last-good total. No hidden worksheet inputs,
parser tokens or collection/string runtime.

## Presentation and lifecycle

Use a read-only full-width flow band after a stable ordinary row-axis identity,
with at most two projections sharing one immutable result owner. Rendering,
hit-testing, navigation and selection use tagged presentation locations. Growing
three records to five leaves A9, its CellId, scalar bindings and SUM geometry intact.
Distinct numbering/background, a labeled read-only boundary and a non-color source
cue identify projected ownership.

Selection follows a stable key; removal falls back to the projection header with an
explicit missing-record state. **Force the L2 race in regression tests:** begin an
interaction with L2, remove/reorder it before completion, then prove the delayed
pointer/read cannot select a different record now occupying visible #2. Ordinary
logical selection, navigation and clipboard exclude projected records.

The opt-in in-memory document must clearly advertise that it cannot be saved.
Reject Save/Save As without writing partial WB4 output or silently losing objects.
Close/discard must be deterministic, with explicit experimental loss/discard wording
where confirmation is needed and no misleading ordinary Save choice/prompt.
Ordinary WB4 document Save/Open and close behavior remain unchanged. Include these
lifecycle behaviors in the automated proof, not just backend save refusal.

## Admission and verification

Retain one producer, two projections, eight fields, 64 records, 4,096-byte text,
128-byte keys, 256 KiB complete results, 1 MiB total structured ownership and two
retained captures. Charge shared payload once and separately owned indexes/maps/
roots appropriately. Exhaustion rejects publication without truncation, eviction
of live captures or mutation of the current result. Existing worksheet, viewport,
private-frame and runtime-domain limits remain.

Test zero/three/five rows; repeated products with distinct keys; missing/duplicate/
disappearing/returning keys; failed publications; overflow and canonical-order totals;
G1 captures during G2 publication; A1/CellId/SUM invariance; two-view ownership;
read-only actions; logical selection across bands; stale interactions and removed
selection; explicit experimental save refusal; ordinary WB4 persistence and cleanup.
Run actual desktop and the established managed/Native AOT gates on Windows x64,
Linux/WSLg x64 and native macOS ARM64/APFS, with normal zero-charge shutdown.

## Implementation boundary and stop

Implement only the bounded experimental proof, provide its implementation/validation
handoff, then stop for parent review. This is design acceptance, not evidence that
the proof is implemented or accepted.

No SQL/connectors, refresh scheduler, computational collections/strings, per-record
transformation graphs, persistent projections, WB5, permanent `#` syntax, projected
clipboard editing, write-through, materialization into CellIds, arbitrary widgets,
printing, formatting or capacity expansion. Success is one captured result displayed
twice and consumed numerically through refresh/reflow without destabilizing ordinary
worksheet identities or captures; a complete invoice application is not required.
