# Milestone code-size reports

At each completed milestone, run from the canonical checkout on ROBPC:

```powershell
python scripts/measure_loc.py --milestone ADR-0033
```

Use the current milestone identifier. `cloc` must be on PATH. The same source on
the Mac or VMs does not need another count. This is optional reporting tooling,
not a contributor build or CI prerequisite.

The report groups physical source by Rust crate and .NET project and shows code,
comment and blank lines. Separately identifiable test/example files are a subset
of each project's code total; inline tests remain in their containing source files.
Standalone test and automation scripts have their own groups. Language totals and
documentation/configuration/fixture counts are kept separately in JSON.

Inputs come from Git's tracked and nonignored untracked files, so owner-authorized
work is counted before commit. Generated sources, build outputs, dependency/vendor
trees and artifacts are excluded. Linked source files count at their physical owner
once; identical files at different physical paths still count individually.

Timestamped raw cloc output, input list, source digest, tool version and project
summary live under `artifacts/loc/<milestone>/`. `latest.md` and `latest.json` provide
stable handoff links. Later milestones show per-project changes against the previous
recorded milestone. ADR-0033 starts the recorded baseline. Include the project table
or its link in each implementation handoff and preserve the reports under the normal
artifact-retention policy.

Line counts are diagnostic evidence, not a quality or reduction target. Preserve
understandable code rather than optimizing the project for fewer physical lines.

ADR-0036: **89,459 code lines across 391 files**, +859 lines versus ADR-0035.
See the [per-project report](../artifacts/loc/ADR-0036/20261031T233804215663Z/report.md)
and [implementation handoff](handoffs/2026-10-31-exact-allocation.md).

ADR-0037: **90,859 code lines across 400 files**, +1,400 lines versus ADR-0036.
The final [per-project report](../artifacts/loc/ADR-0037/20261101T113816456043Z/report.md)
includes the test-fixture correction; the earlier 90,855-line count remains historical.
See the [implementation review](handoffs/2026-11-01-weighted-allocation.md).

ADR-0038 implementation measurement: **92,351 code lines across 406 files**, +1,492
lines versus ADR-0037. See the [per-project report](../artifacts/loc/ADR-0038/20261101T143646793437Z/report.md)
and [implementation handoff](handoffs/2026-11-01-range-based-weighted-allocation.md).

ADR-0040 implementation measurement: **94,406 code lines across 413 files**, +2,055
lines and seven files versus ADR-0038. See the
[per-project report](../artifacts/loc/ADR-0040/20261101T230102093541Z/report.md).
See the [implementation handoff](handoffs/2026-11-01-unbound-grid-windows.md).
ADR-0040 is accepted and closed with the unresolved Windows ARM64 cold-run qualification.
Line counts remain diagnostic evidence rather than an acceptance target.
