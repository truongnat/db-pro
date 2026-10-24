---
schema_version: 1
id: "MEM-PLAN-agent-replay-step-budget-1f0ac3c04ac0"
title: "Plan archive: Agent Replay Step Budget"
type: workflow
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, workflow, agent-replay-step-budget]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "1f0ac3c04ac0f8c8939fa60dc3a2b309faee9bd9c547218bbdf06dbe9aae7ebc"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/agent-replay-step-budget/PLAN.md"
  - "docs/plans/active/agent-replay-step-budget/FINDINGS.md"
  - "docs/plans/active/agent-replay-step-budget/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/agent-replay-step-budget/`. Loại suy luận: `workflow`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Agent replay step budget — #448

Lifecycle: REVIEW. Task state: Review. Implementation and local runtime acceptance are satisfied; workspace Clippy remains failed in unchanged app sources. Not ready to merge; no independent approval.
Baseline: `b52a5088f7872f973bf5c31877c9b3698d306e8c`.
Issue: https://github.com/truongnat/db-pro/issues/448

Goal: stop repeated cached agent tool calls from bypassing the existing ten-step limit while preserving idempotency and confirmation safety.

Scope/acceptance: [requirements.md](requirements.md), R1–R4.
Architecture, alternatives, impact and verification design: [context.md](context.md).
Owner approved requirements/context on 2026-10-16. Completion conditions: [dod.md](dod.md). Execution: [tasks.md](tasks.md).

Non-goals: provider transcript/API changes, new retry or timeout mechanisms, UI changes, database mutations, unrelated tickets.

Provider matrix: PostgreSQL n/a; SQLite n/a independently — this fix changes runtime orchestration only. Native UI is unchanged. A local scripted provider proves the orchestrator bound, not a live remote-provider integration.

Evidence: [FINDINGS.md](FINDINGS.md), [VERIFICATION.md](VERIFICATION.md).


## Source: `FINDINGS.md`

# Agent evidence — #448

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | omp coding assistant — issue triage/implementation |
| Issue | https://github.com/truongnat/db-pro/issues/448 |
| Task state | Review — local implementation/verification complete; integration gate blocked |
| Baseline SHA | b52a5088f7872f973bf5c31877c9b3698d306e8c |
| Branch / PR | main / no PR |
| Scope interpretation | Bound completed-call cache replay using the existing tool-step budget. |
| Out of scope | Other tickets, transcript redesign, new knobs, UI/database changes. |

## 2. Progress checkpoint

- Source finding P1 at baseline `b52a5088f7872f973bf5c31877c9b3698d306e8c`: `crates/runtime/src/agent_orchestrator.rs:340-355` returns cached events before step accounting; `drive:285-336` continues provider calls after replay.
- Acceptance rows R1–R4: implemented and locally verified. Workspace fmt/check/test pass; workspace Clippy fails on unchanged app sources; runtime Clippy passes.
- Runtime reproduction and command results: VERIFICATION.md.
- Runtime observation: `cargo run --offline --manifest-path /tmp/db-pro-448-probe/Cargo.toml` exited 101 on the expected guard assertion: limit 10, provider calls 13, tool executions 1, no max-steps failure, completed true. This reproduces the inherited P1, not an introduced regression.
- Approval blocker cleared: owner approved requirements/context on 2026-10-16. Focused suite passes (18/0/0) and public runtime smoke bounds requests at eleven with one actual execution.

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | Baseline `b52a5088f7872f973bf5c31877c9b3698d306e8c` plus uncommitted working-tree candidate; no resulting commit SHA exists. Source fingerprints in VERIFICATION.md. |
| Commit list | none — owner did not request a commit |
| File / surface inventory | `crates/runtime/src/agent_orchestrator.rs`: move existing step guard before completed-cache replay; `crates/runtime/src/tests/agent_orchestrator_tests.rs`: four budget/outcome/boundary regressions; `CHANGELOG.md`: user-facing fix; `docs/plans/STATUS.md`: lifecycle row; this plan's eight Markdown files: requirements/context/DoD/tasks/plan/checklist/findings/verification. |
| Acceptance mapping | R1 → cached-success bound test + public smoke; R2 → cached-failure/rejection tests plus existing collision/mutation idempotency suite; R3 → exact-limit test + confirmation continuation test + fresh-run smoke; R4 → RED/GREEN commands and standalone runtime smoke in VERIFICATION.md. |
| Commands and counts | RED 15 passed / 3 failed / 0 ignored, exit 101; GREEN 18 passed / 0 failed / 0 ignored, exit 0; smoke exit 0; workspace tests 1,964 passed / 0 failed / 42 ignored, exit 0; workspace fmt/check PASS; runtime Clippy PASS; workspace Clippy FAIL (5 diagnostics in unchanged app sources, exit 101); clean-code scripts exit 0. Exact commands in VERIFICATION.md. |
| CI run IDs / status | not run — no commit/push/PR |
| Known limitations | Scripted local provider, not live remote-provider/UI verification; independent review not performed. |
| Migrations / config implications | none; no persistence, dependency, or public API change |
| Out-of-scope changes | none |

## 4. Review outcome

Self-check only, not independent approval. **Integration verdict: BLOCK** because workspace Clippy fails on unchanged app sources; no merge-ready claim. In this narrow change: introduced open P0/P1/P2 = 0/0/0; encountered inherited open P0/P1/P2 = 0/0/1 (grouped workspace lint gate, five diagnostics). The inherited #448 P1 is reproduced before and bounded after; this is not a whole-repo severity audit.

The working-tree candidate based on `b52a5088f7872f973bf5c31877c9b3698d306e8c` reuses the existing ten-step limit/error/cache and adds no new allocation/copy/dependency. Pending-confirmation replay stays before accounting. At exhaustion, max-steps failure takes precedence before dispatch/cache lookup; tool/input collision detection remains on accepted attempts. Focused regressions, standalone runtime smoke, workspace formatting/check/tests and runtime Clippy pass. Existing source-size/clone scan warnings are inherited. Baseline app comparison and exact gate failures are recorded in VERIFICATION.md. Independent review remains n/a.

## 5. Research / audit handoff

- Source date: 2026-10-16.
- Sources: GitHub open issue/PR listings, #448, #388, #461, #458, #457; docs/plans/STATUS.md; docs/plans/FEATURE_LIFECYCLE.md; docs/architecture/system-overview.md; source/test paths in context.md at baseline `b52a5088f7872f973bf5c31877c9b3698d306e8c`.
- Verified: 156 open issues; 13 open draft PRs. #448 has no assignee/comments and no PR found through issue-number/all-PR search plus matching open-PR title/body search.
- Verified: #388's direct-index panic has a guard at `crates/app/src/result_grid/result_grid_edit.rs:302-305` and regression coverage in `crates/app/src/result_grid/tests/result_grid_record_tests.rs:196-226` at the baseline SHA. These source observations do not close that issue or prove a live UI result.
- Inference: #448 is a suitable next focused P1 because its source path is present and provider-independent runtime reproduction is feasible without paid API calls.
- Recommendation: charge cached attempts to the existing budget; see context.md for alternatives and scope.
- Unresolved questions: none on the approved implementation scope.
- Downstream task activated: #448 is in REVIEW; resolving inherited workspace lint failures and obtaining independent review are required before integration. No other ticket was implemented.

## 6. Tổng kết

Đã sửa #448, giữ idempotency và confirmation; thêm bốn regression test. Suite liên quan 18/18 PASS, smoke chặn ở lượt provider thứ 11 và run mới hoàn tất được ở đúng limit. Workspace fmt/check/test PASS (1.964 pass, 42 ignored), Clippy runtime PASS; Clippy workspace FAIL với 5 diagnostic trong app không thay đổi. Trạng thái REVIEW, chưa sẵn sàng merge. Chưa commit/push/tạo PR/đóng issue; self-check không thay thế independent approval.


## Source: `VERIFICATION.md`

# Verification — #448

Baseline: `b52a5088f7872f973bf5c31877c9b3698d306e8c`.
Scope: owner-approved implementation and local runtime verification; working-tree candidate based on the baseline SHA, not a new commit.

## Executed investigation

- `git status --short --branch`: initial checkout was clean, main tracking origin/main.
- `git rev-parse HEAD`: baseline above.
- `gh issue list --state open --limit 250 --json number --jq 'length'`: 156.
- Open PR listing: 13, all Draft. #448 overlap searches returned no linked/matching PR; not proof against unlinked work.
- Source reads: budget, cache replay, drive loop, existing idempotency/confirmation tests.

## Runtime probe

Command: `cargo run --offline --manifest-path /tmp/db-pro-448-probe/Cargo.toml`.
Probe: real public orchestrator, local finite repeating provider, counting tool runner, no remote API calls, no database writes.

Observed baseline output:

```text
limit=10 provider_calls=13 tool_executions=1 max_steps_failure=false completed=true
BUG #448: repeated cached calls bypassed the tool-step limit
```

Exit 101: the probe assertion requiring max-steps failure failed, reproducing #448. The final completion is supplied deliberately after twelve repeated calls to keep reproduction finite. This proves the existing ten-step budget was exceeded; an actually infinite or paid-provider run was not attempted. Execution stayed idempotent (one tool execution).

Raw command output: `artifact://8` (session artifact). Probe source/manifest are temporary and are removed after recording this result.

## Fixed-path evidence

- RED: `cargo test --locked -p db-pro-runtime --lib agent_orchestrator::tests` → 15 passed / 3 failed / 0 ignored / 24 filtered out, exit 101 (`artifact://11`). New cached-success, cached-failure, and cached-rejection budget regressions failed before the production edit. The exact-limit completion test passed.
- GREEN: same focused command after moving the existing budget check → 18 passed / 0 failed / 0 ignored / 24 filtered out, exit 0 (`artifact://15`).
- Public runtime smoke: `cargo run --offline --manifest-path /tmp/db-pro-448-smoke/Cargo.toml` → exit 0 (`artifact://15`).

```text
attempts=12 provider_calls=11 tool_executions=1 tool_events=10 max_steps_failure=true completed=false
attempts=10 provider_calls=11 tool_executions=1 tool_events=10 max_steps_failure=false completed=true
PASS: over-budget replay stops; fresh run can complete at exact limit; one execution per run
```

The smoke uses the real public orchestrator and a local scripted provider, not a live remote AI service. Its second newly constructed run verifies a fresh allowance after the first exceeded its budget. Temporary reproduction/smoke projects were removed after recording their outputs.

## Quality gates

- Initial `cargo +nightly fmt --all -- --check` failed on formatting in the four new tests only (`artifact://14`). Ran `rustfmt +nightly --edition 2021 crates/runtime/src/tests/agent_orchestrator_tests.rs`.
- Final `cargo +nightly fmt --all -- --check` → PASS, exit 0 (chained gate output `artifact://19`).
- `cargo check --workspace` → PASS, exit 0 (`artifact://19`).
- `cargo clippy --workspace --all-targets -- -D warnings` → FAIL, exit 101 (`artifact://19`): two `unnecessary_to_owned` diagnostics in `db-pro-app` production code (compiler output omitted source spans), plus three `field_reassign_with_default` diagnostics in `crates/app/src/schema/tests/schema_workbench_tests.rs:79,114,133`. No suppressions or unrelated edits were made.
- Baseline classification: `git diff b52a5088f7872f973bf5c31877c9b3698d306e8c -- crates/app` returned empty, exit 0. The failing app sources are unchanged against the exact baseline; this is not a claim that a separate baseline Clippy run was executed.
- `cargo clippy -p db-pro-runtime --all-targets -- -D warnings` → PASS, exit 0.
- The chained `cargo test --workspace` was not reached after the workspace Clippy failure. Independent `cargo test --workspace` → PASS, exit 0: 1,964 passed / 0 failed / 42 ignored across 24 suites (`artifact://23`). Ignored cases are not counted as passing or live-provider evidence.
- `bash .skills/clean-code/scripts/clean-code-scan.sh rust --diff --ratchet --ci` → exit 0: 14 pass / 2 warnings / 0 failures. Both warnings are inherited in the unchanged `drive` loop (53 lines) and existing 54 clone sites; this fix adds neither clones nor a new loop.
- `python3 tools/cc-scan.py crates/runtime/src/agent_orchestrator.rs crates/runtime/src/tests/agent_orchestrator_tests.rs` → exit 0: 100/100, 0 errors / 0 warnings / 0 info.

Candidate SHA-256 (uncommitted files on baseline `b52a5088f7872f973bf5c31877c9b3698d306e8c`):

```text
6c87f6b2948c74e051a9e304f42bfd44242d9562ad3cf062fb7262ff4c5ac45f  crates/runtime/src/agent_orchestrator.rs
c463829707ac0354117ad8f24ec35f4f8e9d712f911ca27c9cb35251124cb8b8  crates/runtime/src/tests/agent_orchestrator_tests.rs
```


## Tool limitations

- Descriptive repository search failed because every judge request was rejected; literal searches and direct issue-named source reads were used instead. Tool issue reported.
- rust-analyzer references request failed: server exited unexpectedly (code 0). No symbol refactor or public API change was performed.

## Not executed / not claimed

No native UI screenshots, real remote AI-provider requests, or database-provider runs. PostgreSQL n/a and SQLite n/a independently for this runtime-only change. No CI run, commit, PR, or independent review. Workspace Rust gate results are recorded above, not inferred from focused tests.

