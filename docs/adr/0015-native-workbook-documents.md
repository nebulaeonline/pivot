# ADR-0015: Native workbook documents and local application lifecycle

Status: Implemented for Slice B review, 2026-10-20. Not yet accepted.

## Decision

A local `.pivot` file represents one coherent workbook generation. Rust owns
encoding, validation, restoration and filesystem operations. Avalonia owns file
selection, discard confirmation, presentation and asynchronous client calls.
The canonical graph and workbook remain unchanged in ownership and capacity.

`pivot-workbook` stays independent of storage and transport. Its owned logical
transfer bridge exports a graph image and worksheet projection together.
`pivot-workbook-document` supplies the byte envelope and validation, reusing
`pivot-storage` portable graph CBOR and the canonical core transfer importer.
There is no second evaluator, graph backend, hosted registry or SQL provider.

## Initial format

The envelope is `PIVOTWB1` (eight bytes), an eight-byte little-endian payload
length, a 32-byte SHA-256 payload digest and the payload. Total size is at most
4 MiB. The digest detects corruption; it is not authentication. Unsupported
magic/version, lengths, trailing bytes and invalid metadata are rejected.

Payload scalar integers and collection lengths are unsigned 64-bit little-endian;
UUIDs are 16 raw bytes; strings are length-prefixed UTF-8. Fields are ordered:
worksheet UUID, row namespace/length, column namespace/length, worksheet state
counter, length-prefixed portable graph document CBOR, allocated cell count,
then cell records in portable GraphNodeId order. Each record contains CellId,
GraphNodeId and a live/retired tag. Live records include position and content.
Content tags distinguish blank, exact decimal, typed bound formula and text formula.
Decimals retain lo/mid/hi coefficient, sign and scale; graph numeric values use
canonical runtime normalization. Formula tokens retain stable cell references.
Text formulas additionally retain original source, submitted reference spans and
coordinates, target CellIds/GraphNodeIds, draft status, historical transaction
session/state/revision provenance, edit ordinal and resolved-revision provenance.
The codec source defines the fixed field order for each tagged record.

The portable graph section uses existing graph encoding, including complete node
identity ledgers and definitions. Its document/model UUID is the stable workbook
identity; its state/graph-revision UUID is the saved checkpoint. It is logical CBOR,
not a nested experimental model-file container. No calculation caches, runtime
handles or Arc/radix/AVL layouts are stored. There are no earlier deployed workbook
formats to migrate. New versions require explicit support.

## Restoration

Decode into a bounded candidate, cross-check unique identities, live/retired graph
membership, positions, axes, resource limits and projection/graph definitions.
Check text syntax and provenance spans against the original source, comparing
stored bindings by identity without looking historical coordinates up in the grid.
Invalid drafts regenerate their diagnostic from source/bounds and remain invalid.
Their core representation uses the existing non-executable transfer error code;
worksheet diagnostics remain richer. Import through the canonical graph importer
on the application's existing runtime, then install the complete candidate.
Failed Open leaves the current workbook intact. Reopening preserves saved logical
identity/state but creates a fresh session and current core revision; historical
revision components are provenance and grant no new session mutation authority.

## Save and file lifecycle

PREPARE_SAVE validates an explicit expected token and retains one bounded owned
export plus an observed destination fingerprint. It returns an opaque ticket.
SAVE validates the ticket plus its captured revision (or the current revision),
then consumes the ticket and stages/installs those exact bytes; edits accepted
between prepare and commit do not alter the saved generation. A successful receipt
acknowledges the captured Expected/checkpoint only; a newer generation stays dirty.
The client convenience Save wraps both operations. There is at most one prepared
save, separately from the existing four calculation captures. Explicit abandonment,
New/Open/Close and orderly process shutdown release it.

The shared file broker creates a private same-directory staging file, writes and
syncs it, verifies bytes, rechecks the baseline and installs atomically. New names
use non-clobbering creation; replacement never truncates the old file in place.
Fingerprints include file identity, metadata and SHA-256 content. A cooperative
per-directory lock serializes Pivot writers. External modification causes conflict;
Save As to the current associated path retains the original baseline. As with the
existing broker, this is not an atomic compare-and-swap against arbitrary hostile
noncooperating writers between the final check and replacement. Symlink/hardlink,
file ownership and filesystem limitations follow the existing broker.

Failures before installation preserve the prior valid file. Failures once
installation is attempted are conservatively uncertain and do not acknowledge a
checkpoint. Reopen explicitly validates the installed file; Save As to a new file
can preserve current work before resolving uncertainty. There is no automatic
mutation replay or claim of unsaved-work recovery after child death. Process death
may leave a private orphan staging file; it is never treated as the document.
Directory-sync support is platform-dependent; no power-loss durability guarantee
is made. The reservation file intentionally remains after unlocking.

New/Open/Close-document refuse dirty replacement without explicit discard. The UI
prompts Save/Discard/Cancel and shows document name, dirty status and errors. Window
close prompts before disposing the owned child. The lower-level terminal CLOSE
continues to be unconditional process cleanup, including after failure. File work
runs behind awaited client requests and does not block the UI thread. The child
still serializes requests: editing can advance between prepare and commit, while
an individual disk operation occupies its one request slot.

Cancellation before dispatch prevents execution; after dispatch the response is
drained, authoritative document/checkpoint state is updated and the caller receives
cancellation. Abandoned prepare replies release their tickets. Ambiguous transport
failure remains terminal, with no retry/replay and no graceful cleanup claim.

## Scope and evidence

See the [Slice B handoff](../handoffs/2026-10-20-workbook-persistence-slice-b.md).
Keep existing frame/admission/capture/viewport/domain and production workbook caps.
No SQLite/hosted persistence, ranges/SUM, copy/fill, formatting expansion, Excel
compatibility, alternative engine, million-cell increase, AOT or installer is added.
The unresolved WSLg native startup abort and historical Linux viewport miss remain
on record. Windows, isolated Linux and fresh native macOS/APFS full CI, persistence
and actual-window checks pass. The [native report](../../artifacts/macos-validation-20261020T111827Z/REPORT.md)
records matching source and no fixes/reruns. OS picker and discard-dialog interaction
remain outside automated coverage. Implementation is ready for parent review; this
validation does not change the ADR acceptance status.
