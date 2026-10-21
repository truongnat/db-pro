---
name: logic-concurrency-tester
description: "State, timing and invariant testing. Use this skill to plan, verify, or review logic concurrency tester in any software project."
---

# logic-concurrency-tester

**Purpose:** State, timing and invariant testing. Technology agnostic: web, native, mobile, API, CLI, infrastructure and data workloads are in scope only when applicable.

## Mandatory testing contract
1. Read project-level instructions, architecture and requirements before testing. Identify technology, supported platforms, release scope and critical risks.
2. Choose relevant cases by likelihood × impact. Record exclusions and N/A with reasons. Never claim that every catalog entry applies.
3. For each chosen case produce **precondition, fixture, action/steps, expected independent oracle, cleanup, priority, and evidence**.
4. Cover applicable variants: happy path, invalid inputs, min/max boundaries, repeated operations, interrupted/cancelled operations, concurrent access, persistence/restart, compatibility and accessibility.
5. Run only authorized non-destructive checks using available tools. Tests that could affect production, external targets or secrets must be blocked pending authorization.
6. Record actual execution, version, command, environment, logs, captures and test data. Static code inspection does not count as runtime visual/functional PASS.
7. Report findings with reproducibility, severity, confidence and a candidate regression test. Return NOT TESTED/BLOCKED when evidence is missing.
8. Follow the bundled reference templates and avoid invented screenshots, defects, CVEs or measurements.

## Output
- Risk and scope inventory with applicable standards
- Test design matrix: ID | technique | preconditions | steps | independent expected result | status | evidence
- Confirmed findings (actual/expected, severity, reproduction) separate from hypothetical risks
- Coverage gaps and blocked checks
- Prioritized regression backlog

## Scenario catalog

Each row is a scenario-design prompt, **not** an executed or runnable automated test. Expand to atomic cases and provide a real oracle before reporting PASS.

| Case | Scenario | Coverage variants | Evidence to capture |
|---|---|---|---|
| LOGI-001 | Invalid state transition | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-002 | Repeated event | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-003 | Stale asynchronous response | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-004 | Cancel versus complete race | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-005 | Concurrent writes | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-006 | Partial transaction failure | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-007 | Lost update | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-008 | Optimistic lock conflict | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-009 | TOCTOU | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-010 | Resource lifecycle race | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-011 | Out of order events | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-012 | Replay processing | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-013 | Retry amplification | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-014 | Queue saturation | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-015 | Deadlock | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-016 | Starvation | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-017 | Cache invalidation | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-018 | Integer overflow | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-019 | Decimal rounding | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-020 | Null semantics | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-021 | Clock skew | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-022 | Restart recovery | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-023 | Mid-operation feature flag | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| LOGI-024 | Exactly-once cleanup | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |

## Verification checklist
- [ ] Scope, roles, tech stack and test basis verified
- [ ] Risk-based scenario selection and N/A reasons documented
- [ ] Expected and actual results independently evaluated
- [ ] Real execution versus static inspection distinguished
- [ ] Confirmed findings backed by repeatable evidence
- [ ] Blockers and coverage gaps preserved
- [ ] Regression and release impact recommended
