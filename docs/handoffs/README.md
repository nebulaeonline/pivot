# Development handoffs and reviews

This directory records implementation handoffs, design reviews, experiments and
platform validation. These are development records; the product overview is in the
[project README](../../README.md).

Start with [implementation status](../STATUS.md) to find the current workstream,
latest review and remaining boundaries. See [architecture decisions](../adr/README.md)
for accepted design choices and [testing](../testing.md) for contributor instructions.

Dated records describe their own source, scope and evidence. Older proposals,
failures and superseded results remain here for context; they do not establish
current behavior or broader acceptance. Follow explicit acceptance and supersession
links rather than treating the newest filename as authoritative.

Validation logs and measurements referenced by these records live under
[artifacts/](../../artifacts/), when available in the checkout. They are separate
from product documentation and are often excluded from version control.

## Milestone review practice

Use one concise handoff per feature milestone: user-visible behavior, meaningful
validation and material remaining limitations. Keep raw automated evidence under
artifacts and focus follow-up review on actual failures. Do not create a chain of
acceptance/reconciliation packets or reopen closed decisions for routine progress.
Do not repeat platform qualification for unchanged behavior; choose regression
coverage appropriate to the actual change, including platform-sensitive changes.

As of 2026-10-28, ADR-0023 is parked at G3. Existing history records remain valid
within their original scope; they do not assign further research or observer fixes.
