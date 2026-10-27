# ISSUE-0001: Intermittent Windows clipboard paste failure

Status: Open. Recorded 2026-10-27. Separate from accepted six-RID execution validation.

The historical Windows managed checklist failed at its first translated native
paste destination with a null cell. Windows ARM64 Native AOT later reported a
NullReferenceException in `ClipboardChecklistAsync`; a diagnostic reproduction
identified missing F5 after step `03-paste` at the same check location.

A subsequent handler-instrumented ARM64 AOT run passed. Its trace recorded native
bytes available, engine paste completion and grid refresh. No behavioral or timing
correction was made. Neither a shared cause nor an ARM64-specific cause is proven;
instrumentation may affect scheduling.

`ClipboardAsync` can complete after declining a stale/unavailable read or catching
an error for display. Task completion therefore does not acknowledge successful
paste. A future focused reliability investigation should examine that completion
contract and distinguish unsuccessful/discarded operations from acknowledged edits.
This observation is not a demonstrated root cause.

Evidence under `artifacts/validation-matrix/`:

- Original ARM64 failure: `20261027T025246Z-b82c5b95/`.
- Missing-F5 diagnostic failure: `20261027T144402Z-a3ccd622/`.
- Accepted instrumented pass: `20261027T144914Z-cab4acb5/`.

[Historical Windows evidence](../handoffs/2026-10-22-worksheet-structured-binding-implementation-review.md#open-reliability-issue--windows-clipboard-checklist) ·
[Diagnostic results](../handoffs/2026-10-27-arm64-aot-diagnostic-results.md).

Preserve original failures and metadata-only checkpoints; never log clipboard
contents. Keep explicit missing-cell failures and all original assertions. Do not
add speculative sleeps, internal retries, exception suppression or success defaults.
The ordinary validator must report Failed whenever the assertion fails. Closure
requires an established cause and justified correction with focused regression
evidence; another unexplained green run does not close this issue.
