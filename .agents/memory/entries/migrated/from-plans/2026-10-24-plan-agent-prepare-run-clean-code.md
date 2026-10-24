---
schema_version: 1
id: "MEM-PLAN-agent-prepare-run-clean-code-b6c75b86aad8"
title: "Plan archive: Agent Prepare Run Clean Code"
type: decision
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, decision, agent-prepare-run-clean-code]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "b6c75b86aad8fa677a3cd70ccc8dd44ca802cc2384e7d03b380c81c2284c8723"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/completed/agent-prepare-run-clean-code/PLAN.md"
  - "docs/plans/completed/agent-prepare-run-clean-code/FINDINGS.md"
  - "docs/plans/completed/agent-prepare-run-clean-code/VERIFICATION.md"
  - "docs/plans/completed/agent-prepare-run-clean-code/AGENT_EVIDENCE.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/completed/agent-prepare-run-clean-code/`. Loại suy luận: `decision`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Plan: Fix AgentState::prepare_run Parameter Count (Clippy too_many_arguments)

## Feature Slug
`agent-prepare-run-clean-code`

## Lifecycle State
`COMPLETED`

## Problem Statement
Commit `90b72fa` introduced an 8-parameter method `AgentState::prepare_run` in `crates/ui/src/agent_state.rs`. This violates Clippy's `clippy::too_many_arguments` rule (max 7 parameters including `self`) and fails the Rust quality gate (`cargo clippy -p db-pro-core -p db-pro-ui --all-targets -- -D warnings`).

## Severity
P1 — Quality Gate failure / Clippy blocking error under `-D warnings`.

## Evidence
Running `cargo clippy -p db-pro-core -p db-pro-ui --all-targets -- -D warnings` produces:
```
error: this function has too many arguments (8/7)
  --> crates/ui/src/agent_state.rs:53:5
```

## Proposed Fix
1. Define a parameter struct `PrepareAgentRunOptions` in `crates/ui/src/agent_state.rs` to encapsulate the input options.
2. Refactor `AgentState::prepare_run` signature to take `&mut self` and `options: PrepareAgentRunOptions`.
3. Update `submit_typed_agent_prompt` and unit tests in `crates/ui/src/agent_state.rs`.
4. Run `cargo clippy` and `cargo test`.

## Merge resolution (2026-09-23)
`main` independently landed the same clippy fix as `AgentRunPreparation` (same fields as `PrepareAgentRunOptions`), plus `prepare_continuation` and the `AgentState` / `DbProApp` split into `agent_actions.rs`. Conflict resolution keeps `main`'s DTO and call sites and drops the duplicate `PrepareAgentRunOptions` name.


## Source: `FINDINGS.md`

# Findings: Fix AgentState::prepare_run Parameter Count

## Analysis & Evidence

In commit `90b72fa`, `AgentState::prepare_run` in `crates/ui/src/agent_state.rs` was refactored with the following signature:

```rust
pub(super) fn prepare_run(
    &mut self,
    request_id: crate::RequestId,
    prompt: String,
    document: db_pro_core::domain::agent::AgentDocumentSnapshot,
    connection_id: Option<String>,
    schema: Option<String>,
    mode: db_pro_core::domain::agent::AgentMode,
    context: db_pro_core::domain::agent_context::AgentContext,
) -> Result<PreparedAgentRun, AgentRunPreparationError>
```

Counting parameters: `&mut self` (1) plus 7 arguments = 8 total function parameters.

Clippy's `clippy::too_many_arguments` triggers on functions with more than 7 parameters. Under `-D warnings`, this halts compilation.

## Solution

Group parameters into a struct `PrepareAgentRunOptions`:

```rust
pub(super) struct PrepareAgentRunOptions {
    pub(super) request_id: crate::RequestId,
    pub(super) prompt: String,
    pub(super) document: db_pro_core::domain::agent::AgentDocumentSnapshot,
    pub(super) connection_id: Option<String>,
    pub(super) schema: Option<String>,
    pub(super) mode: db_pro_core::domain::agent::AgentMode,
    pub(super) context: db_pro_core::domain::agent_context::AgentContext,
}
```

This reduces `prepare_run` to 2 arguments (`&mut self`, `options: PrepareAgentRunOptions`), fully satisfying Clippy and Clean Code function design guidelines.

## Merge resolution (2026-09-23)

`main` already contains the equivalent DTO as `AgentRunPreparation` in `crates/ui/src/agent_state.rs`, used by `agent_actions.rs`. Re-introducing `PrepareAgentRunOptions` would duplicate the type and fight the post-split `DbProApp` adapters. The merge keeps `AgentRunPreparation` and does not restore the pre-split `impl DbProApp` block from this branch.


## Source: `VERIFICATION.md`

# Verification: Fix AgentState::prepare_run Parameter Count

## Verification Strategy
1. Automated clippy check: `cargo clippy -p db-pro-core -p db-pro-ui --all-targets -- -D warnings`
2. Automated unit tests: `cargo test -p db-pro-core -p db-pro-ui`
3. Formatting check: `cargo fmt --all -- --check`

## Execution Results
- `cargo fmt --all -- --check`: PASS (0 formatting issues)
- `cargo check -p db-pro-core -p db-pro-ui`: PASS (0 errors)
- `cargo clippy -p db-pro-core -p db-pro-ui --all-targets -- -D warnings`: PASS (0 errors/warnings)
- `cargo test -p db-pro-core -p db-pro-ui`: PASS (404 core tests + 639 UI tests = 1043 tests passed, 0 failed)

## Post-merge re-run (2026-09-23, SHA `5fe9428fc95694d2428328664bf3d5bba0ba8e94`)
- `cargo fmt --all -- --check`: PASS
- `cargo check -p db-pro-core -p db-pro-ui`: PASS
- `cargo clippy -p db-pro-core -p db-pro-ui --all-targets -- -D warnings`: PASS
- `cargo test -p db-pro-core -p db-pro-ui`: PASS (407 core + 673 UI = 1080 passed, 0 failed, 0 ignored)

PostgreSQL: n/a (no provider SQL path changed).
SQLite: n/a (no provider SQL path changed).
Native UI runtime screenshots: skipped — merge-only resolution, no visual surface change.


## Source: `AGENT_EVIDENCE.md`

# Agent evidence — PR #304 merge conflict resolution

## 1. Claim

| Field | Value |
|---|---|
| Agent identity | cursor-grok-4.6 · merge-conflict lane |
| Issue(s) | PR `#304` |
| Task state | In Progress |
| Baseline SHA | `6fc3e6626fb89a05b9deb35927ec764095e9eb55` — PR head before merge |
| Branch / PR | `fix/agent-prepare-run-clean-code-14904060059921777994` / PR `#304` (`Ready`) |
| Scope interpretation | Resolve merge conflicts with `main`. Keep one `prepare_run` DTO and preserve `main`'s post-split Agent adapters. |
| Out of scope | renaming `AgentRunPreparation` to `PrepareAgentRunOptions`; new Agent behavior; native UI screenshots |

## 2. Progress checkpoint

- Current HEAD at gate run: `5fe9428fc95694d2428328664bf3d5bba0ba8e94`
- Completed acceptance rows: `[x]` merge conflict resolved, `[x]` no leftover `PrepareAgentRunOptions`, `[x]` `prepare_continuation` retained, `[x]` fmt/check/clippy/test re-run
- Remaining acceptance rows: none for this conflict-resolution task
- Findings / risks: none introduced. The original clippy `too_many_arguments` defect is already fixed on `main` via `AgentRunPreparation`.
- Tests already run: see §3
- Dependency / blocker changes: PR was `mergeable_state: dirty`; conflict was only `crates/ui/src/agent_state.rs`

## 3. Implementation handoff / review request

| Field | Value |
|---|---|
| Exact SHA | `5fe9428fc95694d2428328664bf3d5bba0ba8e94` (merge commit; docs-only follow-up may sit on top) |
| Commit list | `6fc3e662` fix(ui): encapsulate prepare_run parameters in PrepareAgentRunOptions; `5fe9428f` merge: resolve main conflicts for prepare_run options |
| File / surface inventory | `crates/ui/src/agent_state.rs` — take `main`'s `AgentRunPreparation` + `prepare_continuation`; drop duplicate `PrepareAgentRunOptions` and the pre-split `impl DbProApp`. Plan docs under `docs/plans/active/agent-prepare-run-clean-code/` record the resolution. |
| Acceptance mapping | dirty PR → mergeable tree; Clippy 8-arg `prepare_run` remains encapsulated as `AgentRunPreparation` |
| Commands and counts | `cargo fmt --all -- --check` exit 0; `cargo check -p db-pro-core -p db-pro-ui` exit 0; `cargo clippy -p db-pro-core -p db-pro-ui --all-targets -- -D warnings` exit 0; `cargo test -p db-pro-core -p db-pro-ui` → 407 + 673 = 1080 passed / 0 failed / 0 ignored, exit 0 |
| CI run IDs / status | not waited in this session |
| Known limitations | this branch no longer carries a unique code change versus `main` for `prepare_run`; remaining unique files are plan docs |
| Migrations / config implications | none |
| Out-of-scope changes | none |

## 4. Review outcome

| Field | Value |
|---|---|
| Reviewed SHA | n/a — implementer self-check only |
| Verdict | n/a |
| P0 / P1 / P2 counts | introduced 0 / 0 / 0; inherited none for this path |
| Findings | none |
| CI disposition | local gates only at `5fe9428f` |
| Next task(s) unblocked | independent review of PR `#304` |

## 5. Research / audit handoff

n/a

## 6. Tổng kết

Đã merge `main` vào PR `#304`. Conflict duy nhất ở `crates/ui/src/agent_state.rs`: `main` đã có DTO tương đương `AgentRunPreparation` cùng `prepare_continuation` và tách `DbProApp` sang `agent_actions.rs`. Giữ bản `main`, bỏ `PrepareAgentRunOptions` trùng. Gate local xanh (fmt/check/clippy; 1080 test core+ui). Không đổi hành vi Agent.

