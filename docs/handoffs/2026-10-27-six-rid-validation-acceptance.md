# Six-RID validation accepted and closed

Parent disposition, 2026-10-27: accept and close headless CI, managed GUI and
actual Native AOT execution across all six designated RIDs. The infrastructure
detour is complete. No additional confirmation run is required.

The accepted Windows ARM64 AOT result is
`artifacts/validation-matrix/20261027T144914Z-cab4acb5/`. It published and executed
the real native main and three peers with ARM64 architecture, Win32 backends,
four zero final Rust charges and no reported owned survivors. The five other
AOT passes remain accepted in `20261027T025246Z-b82c5b95/`; managed and headless
remain accepted in `20261027T025246Z-deb1324f/` and `20261026T002242Z-f76230c2/`.
All paths above are beneath `artifacts/validation-matrix/`.

The original ARM64 failure and diagnostic failure in
`20261027T144402Z-a3ccd622/` remain Failed. The instrumented pass establishes
execution capability, not a clipboard correction or repeatability guarantee.
[ISSUE-0001](../issues/0001-windows-clipboard-paste-reliability.md) tracks that
reliability defect separately. Retain useful metadata-only diagnostics and fail
actual assertions normally; no suppression, automatic retries or weakened checks.

## Ordinary development validation

Use `scripts/validate.py` from controller-only ROBPC and actual designated guest/
Mac execution environments. Keep SSH deployment, native-local builds, separate
mode results, live graphical consent, original assertions, zero-charge checks,
owned-process cleanup and native Mac ARM64/Rosetta exclusion. ROBPC/WSL diagnostics
are not substitutes for matrix targets. Regressions are actionable failures, not
grounds to reopen infrastructure architecture or introduce forensic observers,
immutable acceptance packets or per-execution parent review rituals.

## Return to G3

Release the infrastructure scheduling hold on bounded ADR-0023 indexed-store
feasibility. Begin with the [previously accepted scope](2026-10-24-g2-acceptance-g3-held.md)
and existing design constraints. Preserve G1 ordinary Save and G2 mutation-local
authority: stable identity, provenance, atomicity, lifetime reservations, capture
isolation and original-state coalescing.

G3 is feasibility, not production database adoption, a persistence-format migration
or production Undo/Redo authorization. G4–G7 remain gated. Select validation gates
appropriate to each change rather than automatically running all eighteen mode/
platform combinations. Return the first substantive feasibility result for parent
architectural review. This reconciliation changes documentation only and does not
start G3 implementation or rerun completed gates.
