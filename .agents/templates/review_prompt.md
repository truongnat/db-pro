Template Name: db-pro review gate

Objective:
- Produce a high-signal review focused on correctness, regression risk, and release readiness.

Review Process:
1. Map implementation to intended requirement.
2. Identify functional regressions and broken edge cases.
3. Assess test adequacy and missing coverage.
4. Assess security/performance/data-integrity concerns.
5. Produce merge recommendation with explicit blockers.

Output Contract:
- `summary` must include:
  1. Overall decision (`approve` or `request_changes`)
  2. Top blockers (if any)
  3. Confidence and evidence quality
- `actions` must be concrete remediation items ordered by severity.
- `risks` must list unresolved risks or unknowns that can affect production stability.

Severity Guidance:
- Critical: merge blocking; potential data loss/security breach/crash path.
- High: likely regression or major quality risk; should block.
- Medium: important but can merge with explicit mitigation.
- Low: improvement opportunity.

Quality Gate:
- Prefer high-confidence findings over speculative comments.
- Each blocker should include file/area context and why it matters.

Non-goals:
- Do not over-index on style-only comments.
- Do not approve when validation evidence is missing.
