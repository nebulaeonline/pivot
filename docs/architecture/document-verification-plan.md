# Document verification and compatibility plan

Status: Proposed ADR-0009 future verification. No document tests are implemented or
claimed to pass. Existing storage fixtures and 153 reported Rust tests cover only
the previously implemented foundation.

## Acceptance scenarios

| Area | Required evidence before claiming support |
| --- | --- |
| Engine independence | Standalone Rust core builds and evaluates with no document, daemon, file or database dependency |
| Model-only workflow | New Revenue/Costs graph with Net and Margin, required/default ports; Save, close, open, calculate, edit and Save As with no worksheet, publication or store configuration |
| Interfaces | Named direct invocation works before publication and after reopen; overrides remain isolated and captured defaults remain unchanged |
| Stable bindings | Rename/name reuse, deletion/recreation and retired ports round-trip; old references never retarget; reopening allocates fresh runtime handles |
| Invalid drafts | Empty/incomplete/unresolved source, cycles and dangling retired references remain editable; never-allocated IDs or malformed tokens reject |
| Exact values | Sign/scale/coefficient, signed zero, trailing zeros, defaults and literals survive reopen and unrelated edits; arithmetic parity remains unchanged |
| Session identity | Two copied files sharing IDs have isolated runtime maps; stale results/commands from close or Revert cannot apply |
| Revision separation | Formatting-only edits dirty the file without changing graph revision; scenario calculation does not dirty definitions; saved version is not a graph counter |
| Concurrent edits/save | Save S while editing T, then acknowledge S: disk contains S and T stays dirty; repeated acknowledgment has no extra effect |
| File lifecycle | New/Save As/Close/Cancel/Revert behavior, failed save and failed revert preserve state; Save As changes association only after verified success |
| External edits | Replacement/deletion, same-size/time changes, aliases, two applications and uncooperative writer race limits are exercised and disclosed |
| Broker boundary | C# executable client transfers real content to Rust; bounded export/import, ownership checks and disconnect reconciliation; no path authority or evaluator in frontend |
| In-process use | Same document semantics work in Rust without IPC or hosted registry |
| Compatibility | Frozen document fixtures per accepted version, independent fixture generation, unknown mandatory features, unsupported numeric/operator versions and migration failure |
| File faults | Fault injection at each write/flush/replace/verify/ack boundary; old/new complete state or explicit uncertain outcome under documented profile |
| Recovery | Optional checkpoint never clears main-file dirty state; restore against changed original opens an unsaved copy |
| Resource safety | Truncation, decompression bombs, offset overflow, duplicate sections/IDs, invalid UTF-8, hostile counts/depth and bounded arbitrary bytes |

## Compatibility corpus and test layering

Keep hosted-v1 frozen fixtures byte-for-byte. Add graph adapter semantic tests before
any document byte format is frozen. After container selection, create independent
document fixtures and old-version readers; tests must not regenerate expected bytes
using the implementation under test. Include rejected cases with explicit failure
phase: envelope, payload, graph integrity or calculation diagnostics. Numeric and
expression semantic versions must not silently change when a container changes.

Use unit/semantic tests for session state, integration tests for the real broker,
and executable IPC tests for the separate-process boundary. Test C# as a real caller
without recreating Rust semantics. Future generated contracts must preserve existing
1.0/1.1 negotiation, framing and authoring behavior; any document capability requires
separate protocol review. Current frame limits cannot simply be ignored for files.

Run native Windows, Linux and macOS file tests on supported local filesystems.
WSL checks are useful Linux evidence, but a mounted Windows/shared directory is not
a substitute for native filesystem durability validation. Record OS, filesystem,
library/driver versions and actual flush/replacement guarantees. Process-kill tests
alone do not prove power-loss survival; stronger claims need appropriate fault
testing and a stated storage profile. Network/cloud filesystem support is separate.

## Review and implementation gates

1. Review ADR-0009 boundaries, identity/lifecycle semantics and outstanding decisions.
2. Authorize a narrow payload-adapter/session proof, preserving all existing fixtures
   and tests. Do not conflate it with choosing or implementing a production file format.
3. Separately authorize comparable container prototypes and select a physical format
   from requirements, measurements and safety evidence.
4. Authorize a model-only file workflow and prove its broker/failure behavior before
   adding worksheets. Evaluate a thin headless proof first; UI is not required.
5. Design workbook projection, range/bulk APIs, incremental calculation and UI in
   separate slices. Deferred hosted database work remains independent.

These gates are proposed sequencing for review, not authorization to continue past
the current design slice. The design packet stops here.
