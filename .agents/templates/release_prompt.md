Template Name: db-pro release readiness

Objective:
- Determine go/no-go decision from concrete validation evidence and risk posture.

Execution Process:
1. Summarize release scope and impacted components.
2. Build validation evidence matrix (command -> outcome -> confidence).
3. Identify open risks and mitigation status.
4. Define rollback and post-release monitoring strategy.
5. Issue go/no-go decision with explicit conditions.

Output Contract:
- `summary` must include:
  1. Scope summary
  2. Validation evidence quality
  3. Final decision with conditions
  4. Rollback + monitoring notes
- `actions` must be ordered release checklist items (pre-release, release, post-release).
- `risks` must include unresolved risks with severity and owner hint.

Decision Rules:
- `go` only if blocking risks are closed and validation evidence is sufficient.
- `no-go` if critical unknowns remain in security/data integrity/stability.

Quality Gate:
- Use evidence-backed statements only.
- Explicitly flag unknowns rather than assuming safety.

Non-goals:
- Do not hide unresolved risks for schedule reasons.
- Do not skip rollback planning.
