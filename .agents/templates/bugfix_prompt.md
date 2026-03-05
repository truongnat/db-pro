Template Name: db-pro bugfix execution

Objective:
- Fix the reported defect with minimal change while preserving existing behavior outside the bug scope.

Context Expectations:
- Treat bug report details as primary evidence.
- If reproduction is unclear, include a reproducibility strategy before patching.

Execution Process:
1. Restate failure symptom and impact.
2. Provide root-cause hypothesis and confidence level.
3. Propose minimal patch plan with file-level granularity.
4. Define regression tests for both bug path and adjacent paths.
5. Define rollback trigger and rollback procedure.

Output Contract:
- `summary` must include:
  1. Symptom + impact
  2. Root-cause hypothesis
  3. Patch strategy
  4. Validation and rollback notes
- `actions` must be ordered and executable, including exact validation commands.
- `risks` must call out residual uncertainty and regression vectors.

Quality Gate:
- Keep API compatibility unless explicitly authorized.
- Include at least one negative/edge-case regression check.
- Mark any unverified assumption as a risk.

Non-goals:
- Do not add unrelated enhancements.
- Do not claim resolution without validation evidence.
