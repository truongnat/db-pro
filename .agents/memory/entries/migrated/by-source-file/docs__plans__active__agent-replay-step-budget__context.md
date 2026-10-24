---
schema_version: 1
id: "MEM-SRC-cd2deee82f2b450b"
title: "Source mirror: docs/plans/active/agent-replay-step-budget/context.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/active/agent-replay-step-budget/context.md"
source_sha256: "cd2deee82f2b450bcd04e0930f934b3891f9c74ef6b443b25ab0ebd5a19da1b5"
mirror_body_sha256: "cd2deee82f2b450bcd04e0930f934b3891f9c74ef6b443b25ab0ebd5a19da1b5"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Context and decision — #448

Status: approved by owner on 2026-10-16. Analysis depth: L2 (standard).
Source baseline: `b52a5088f7872f973bf5c31877c9b3698d306e8c`.

## Verified context

- `crates/runtime/src/agent_orchestrator.rs:285-336`: `drive` continues provider calls while tool calls are returned; tool errors terminate the run through `fail_run`.
- `crates/runtime/src/agent_orchestrator.rs:340-355`: pending confirmation and cached execution return before the existing step increment/check.
- `crates/runtime/src/agent_orchestrator.rs:444-491`: cached success/failure/rejection preserves execution idempotency and validates the tool/input fingerprint.
- `crates/core/src/domain/agent.rs:18`: the existing limit is ten tool steps.
- `crates/runtime/src/tests/agent_orchestrator_tests.rs`: existing FakeProvider, CountingToolRunner and confirmation/replay/collision tests are the reuse points; tests remain outside production source.
- `docs/architecture/system-overview.md:72-95`: runtime owns orchestration; no UI or connector contract change is required.
- GitHub fetched 2026-10-16: #448 has no assignee/comments; searching all PRs for `448 in:body` returned none, and open PR bodies/titles contained no matching replay/step fix. This is search evidence, not a guarantee against unlinked concurrent work.

## Gaps, impacts and alternatives

- [BLOCKER, verified] Cached tool outcomes bypass the budget, so identical calls need not reach the existing termination condition.
- [RISK] Moving the guard may affect boundary/error precedence. Mitigation: exact ten/eleven-attempt coverage, existing collision and confirmation regressions.
- [OUT-OF-SCOPE] Appending replay transcript messages alone cannot guarantee termination; it changes provider serialization semantics without fixing the hard bound.
- [OVER-ENGINEERING] A second provider-roundtrip counter introduces a second limit where the existing tool-attempt limit suffices.

Criteria: bounded termination/correctness (weight 5), idempotency/safety (5), maintainability (3). Scores: 1 = fails criterion, 5 = satisfies directly.

| Option | Termination | Idempotency | Maintainability |
|---|---:|---:|---:|
| A: charge cached attempts to existing budget | 5 | 5 | 5 |
| B: add independent provider-roundtrip cap | 5 | 5 | 3 |
| C: only append replay transcript messages | 1 | 5 | 3 |

Decision: A. Preserve the pending-confirmation path; move the existing increment/check before completed-cache replay. Do not add another counter, dependency, or public API. Replays consume the existing allowance rather than receiving an unlimited exemption; this is the intentional behavior change.

Confidence: high from the source path; runtime reproduction is recorded separately in VERIFICATION.md. Reconsider if runtime evidence shows another unbounded continuation path or protocol semantics require additional changes. No assumption that one database provider proves another.

## Verification scope

- Local finite repeating provider calls the real public `AgentRunOrchestrator::run`; stop the probe after a fixed number of responses so the baseline cannot run forever or consume quota.
- Assert terminal max-steps failure, bounded provider calls, and one execution for repeated successful calls. Cover cached failure/rejection, exact limit, fresh run, and confirmation continuation as applicable without duplicating existing same-path tests.
- Run focused runtime tests, workspace Rust gates, and inspect their actual outcomes; do not predeclare passes.
- PostgreSQL: n/a (no connector or SQL change). SQLite: n/a independently for the same reason. Native UI: unchanged; no screenshots claimed. Real remote provider behavior: not verified by a scripted provider.

## Decision log and scope fences

User delegated ticket selection. Agent selected an uncovered P1 ahead of new P2 work; several higher-profile tickets have open PRs or already-present fixes. Owner approved requirements/context on 2026-10-16 before implementation. No commits, push, PR creation, or issue closure are authorized here.

MUST: bounded replay attempts, idempotency, exact budget boundary, before/after evidence.
SHOULD: reuse existing test fixtures and error type.
COULD: replay protocol changes only if required by reproduced evidence.
NOT NOW: other tickets, new provider caps, UI redesign; revisit only under separately approved scope.
Stop/reassess if fixing the bound requires changing provider protocol or database safety policy.
