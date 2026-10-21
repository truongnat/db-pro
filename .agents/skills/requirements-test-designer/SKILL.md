---
name: requirements-test-designer
description: "Standards-informed test design. Use this skill to plan, verify, or review requirements test designer in any software project."
---

# requirements-test-designer

**Purpose:** Standards-informed test design. Technology agnostic: web, native, mobile, API, CLI, infrastructure and data workloads are in scope only when applicable.

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
| REQU-001 | Ambiguous acceptance criteria | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-002 | Mandatory optional fields | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-003 | Empty null missing values | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-004 | Whitespace and trim policy | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-005 | Input min boundary | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-006 | Input max boundary | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-007 | Off-by-one errors | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-008 | Equivalent valid classes | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-009 | Equivalent invalid classes | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-010 | Decision table interactions | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-011 | State transition validity | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-012 | Role and permission matrix | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-013 | Idempotency expectations | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-014 | Duplicate submission | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-015 | Date and time rules | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-016 | Timezone and DST | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-017 | Decimal and rounding | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-018 | Unicode normalization | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-019 | Feature flag behavior | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-020 | Migration acceptance | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-021 | Partial success semantics | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-022 | Interrupted operations | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-023 | Test oracle independence | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| REQU-024 | Traceability completeness | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |

## Verification checklist
- [ ] Scope, roles, tech stack and test basis verified
- [ ] Risk-based scenario selection and N/A reasons documented
- [ ] Expected and actual results independently evaluated
- [ ] Real execution versus static inspection distinguished
- [ ] Confirmed findings backed by repeatable evidence
- [ ] Blockers and coverage gaps preserved
- [ ] Regression and release impact recommended
