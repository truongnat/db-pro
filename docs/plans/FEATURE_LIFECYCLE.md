# DB Pro — Feature Lifecycle Standard

This file is the canonical lifecycle for every non-trivial feature, hardening wave, and runtime-verification task.

## State machine

```text
BACKLOG
  → PLANNING
  → IMPLEMENTING
  → REVIEW
  → RUNTIME_VERIFY
  → COMPLETED
```

`BLOCKED` may be used from any non-completed state when an external dependency prevents progress.

## State definitions

| State | Meaning | Exit criteria |
|---|---|---|
| BACKLOG | Prioritized but not started | Scope selected and plan created |
| PLANNING | Evidence, scope and acceptance criteria are being established | PLAN/CHECKLIST/FINDINGS/VERIFICATION exist and scope is coherent |
| IMPLEMENTING | Code/tests are being changed | Implementation is complete enough for self-review |
| REVIEW | Architecture/correctness/test review is active | P0=0, P1=0, accepted findings resolved |
| RUNTIME_VERIFY | Source and automated tests are acceptable; runtime evidence is still incomplete | Required provider/runtime matrix is satisfied |
| COMPLETED | Feature is proven within its declared support matrix | All completion gates below are satisfied |
| BLOCKED | Progress is prevented by an external dependency | Blocker removed or scope explicitly changed |

## Required plan files

Every active feature owns exactly one directory under `docs/plans/active/<feature-slug>/` containing:

- `PLAN.md` — goal, scope, non-goals, architecture and acceptance criteria
- `CHECKLIST.md` — executable work items and gates
- `FINDINGS.md` — evidence-backed P0/P1/P2 findings and decisions
- `VERIFICATION.md` — commands actually executed and runtime evidence actually observed

When and only when the feature reaches `COMPLETED`, move the directory to `docs/plans/completed/<feature-slug>/` and update `docs/plans/STATUS.md` in the same PR.

## Evidence levels

Do not mix evidence levels.

1. **Source evidence** — code inspection proves an implementation path exists.
2. **Automated evidence** — unit/integration/E2E command executed and result recorded.
3. **Provider runtime evidence** — behavior observed against the actual PostgreSQL or SQLite provider.
4. **UI runtime evidence** — user-facing lifecycle observed end-to-end.

Source evidence must never be written as runtime evidence.

## Database provider matrix

Every database-facing feature must explicitly record:

| Provider | Supported operation | Automated evidence | Live/runtime evidence | Capability gate |
|---|---|---|---|---|
| PostgreSQL | yes/no/partial | PASS/PENDING/N/A | PASS/PENDING/N/A | reason if unsupported |
| SQLite | yes/no/partial | PASS/PENDING/N/A | PASS/PENDING/N/A | reason if unsupported |

One provider's test never proves another provider.

If an operation is intentionally unsupported, completion requires:

- capability detection is deterministic,
- the UI/API does not emit unsupported SQL,
- a clear reason is surfaced,
- tests cover the capability boundary.

## Completion gates

A feature may be marked `COMPLETED` only when all applicable gates are true:

- [ ] PLAN scope and non-goals are explicit
- [ ] CHECKLIST reflects actual work, not assumptions
- [ ] P0 = 0
- [ ] P1 = 0
- [ ] required quality gates were actually executed
- [ ] regression tests prove important invariants
- [ ] PostgreSQL and SQLite are independently accounted for
- [ ] unsupported provider operations are capability-gated
- [ ] UI-facing database features verify `UI → command → backend → database → introspection → refreshed UI`
- [ ] VERIFICATION contains evidence, not phrases such as "works perfectly" or "fully verified" without proof
- [ ] STATUS.md matches the plan folder and feature state

## Review and automation ownership

- Jules/implementer: analyze, plan, implement, test, publish PR; never self-declare unsupported runtime evidence.
- Kilo VPS reviewer: independent read-only adversarial review.
- Human/arbiter: accept, reject or downgrade findings and make the final merge decision.

Review infrastructure is intentional repository infrastructure. Do not delete `REVIEW.md`, `.kilo/`, `.github/review-prompts/`, or `.github/workflows/vps-pr-review.yml` as cleanup unless the task explicitly replaces that system.
