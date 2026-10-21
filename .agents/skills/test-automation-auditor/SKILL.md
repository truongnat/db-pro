---
name: test-automation-auditor
description: "Reliable testing and CI quality. Use this skill to plan, verify, or review test automation auditor in any software project."
---

# test-automation-auditor

**Purpose:** Reliable testing and CI quality. Technology agnostic: web, native, mobile, API, CLI, infrastructure and data workloads are in scope only when applicable.

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
| TEST-001 | Test portfolio risk fit | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-002 | Unit invariant coverage | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-003 | Integration boundary coverage | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-004 | End to end critical journeys | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-005 | Deterministic random seed | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-006 | Frozen clocks | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-007 | Order independence | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-008 | Fixture cleanup | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-009 | Parallel isolation | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-010 | Behavioral assertions | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-011 | Failure diagnostics | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-012 | Flaky retry policy | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-013 | Quarantine ownership | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-014 | Coverage quality limits | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-015 | Mutation testing | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-016 | Contract drift | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-017 | Snapshot review | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-018 | CI gating | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-019 | Runtime budget | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-020 | Secrets in test data | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-021 | Build reproducibility | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-022 | Captured artifacts | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-023 | Release artifact targeting | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| TEST-024 | Regression for known bugs | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |

## Verification checklist
- [ ] Scope, roles, tech stack and test basis verified
- [ ] Risk-based scenario selection and N/A reasons documented
- [ ] Expected and actual results independently evaluated
- [ ] Real execution versus static inspection distinguished
- [ ] Confirmed findings backed by repeatable evidence
- [ ] Blockers and coverage gaps preserved
- [ ] Regression and release impact recommended
