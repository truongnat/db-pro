---
name: release-readiness-auditor
description: "Evidence-backed ship/no-ship. Use this skill to plan, verify, or review release readiness auditor in any software project."
---

# release-readiness-auditor

**Purpose:** Evidence-backed ship/no-ship. Technology agnostic: web, native, mobile, API, CLI, infrastructure and data workloads are in scope only when applicable.

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
| RELE-001 | Artifact identity | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-002 | Reproducible build | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-003 | Package launch | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-004 | Install and uninstall | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-005 | Upgrade smoke test | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-006 | Critical journey evidence | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-007 | Outstanding critical defects | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-008 | Dependency security checks | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-009 | Privacy checks | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-010 | Accessibility target | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-011 | Performance budget | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-012 | Backup restore readiness | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-013 | Monitoring readiness | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-014 | Incident response ownership | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-015 | Release notes accuracy | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-016 | Feature flag rollback | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-017 | Data migration readiness | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-018 | Skipped tests reviewed | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-019 | Cross-platform evidence | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-020 | Formal exception sign-off | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-021 | Rollout monitoring | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-022 | Rollback thresholds | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-023 | Code signing | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |
| RELE-024 | Post-release verification | Test valid/invalid, boundary, repeat and interruption variants as applicable. | Logs, observed state, fixture and reproduction |

## Verification checklist
- [ ] Scope, roles, tech stack and test basis verified
- [ ] Risk-based scenario selection and N/A reasons documented
- [ ] Expected and actual results independently evaluated
- [ ] Real execution versus static inspection distinguished
- [ ] Confirmed findings backed by repeatable evidence
- [ ] Blockers and coverage gaps preserved
- [ ] Regression and release impact recommended
