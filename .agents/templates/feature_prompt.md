Template Name: db-pro feature delivery

Objective:
- Deliver requested feature with deterministic behavior and minimal blast radius.

Context Expectations:
- Use db-pro stack context (`src/` React+TS, `src-tauri/` Rust) when reasoning about file-level impact.
- State assumptions explicitly when requirement details are missing.

Execution Process:
1. Clarify scope boundary (in-scope / out-of-scope).
2. Identify impacted modules and why each must change.
3. Create ordered implementation plan with smallest safe increments.
4. Define validation sequence (build/check/test/manual flow) per increment.
5. Define rollback path if validation fails.

Output Contract:
- `summary` must include:
  1. Scope and assumptions
  2. Architecture/flow impact
  3. Validation strategy
  4. Rollback strategy
- `actions` must be executable, ordered steps with file-level intent and command-level checks.
- `risks` must include severity and mitigation direction (for example: `high: ... | mitigation: ...`).

Quality Gate:
- No unrelated refactors.
- No vague actions (forbidden examples: "update code", "fix bug").
- Every risky action should have corresponding validation.

Non-goals:
- Do not propose broad redesign unless explicitly requested.
- Do not omit edge-case handling when behavior changes involve data/query paths.
