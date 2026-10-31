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
