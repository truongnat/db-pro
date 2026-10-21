---
name: ui-visual-tester
description: "Native and web visual integrity. Use this skill to plan, verify, or review ui visual tester in any software project."
---

# ui-visual-tester

**Purpose:** Native and web visual integrity. Technology agnostic: web, native, mobile, API, CLI, infrastructure and data workloads are in scope only when applicable.

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
| UI-001 | Visual hierarchy | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-002 | Spacing alignment | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-003 | Typography legibility | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-004 | Text wrapping | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-005 | Truncation policy | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-006 | Color contrast | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-007 | Light theme | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-008 | Dark theme | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-009 | High contrast | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-010 | Zoom and text scaling | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-011 | Minimum viewport | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-012 | Maximum viewport | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-013 | Narrow height | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-014 | Long translations | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-015 | RTL rendering | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-016 | Empty state | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-017 | Loading state | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-018 | Error state | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-019 | Selected and disabled state | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-020 | Modal positioning | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-021 | Overlay clipping | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-022 | Table overflow | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-023 | Pixel density | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| UI-024 | Visual regression baseline | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |

## Verification checklist
- [ ] Scope, roles, tech stack and test basis verified
- [ ] Risk-based scenario selection and N/A reasons documented
- [ ] Expected and actual results independently evaluated
- [ ] Real execution versus static inspection distinguished
- [ ] Confirmed findings backed by repeatable evidence
- [ ] Blockers and coverage gaps preserved
- [ ] Regression and release impact recommended
