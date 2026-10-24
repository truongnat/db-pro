---
schema_version: 1
id: "MEM-SRC-442852969b1baf7b"
title: "Source mirror: docs/plans/completed/agent-prepare-run-clean-code/PLAN.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/agent-prepare-run-clean-code/PLAN.md"
source_sha256: "442852969b1baf7b7368113c09b38df996eb4314e52f0b531f74b903b72083b5"
mirror_body_sha256: "442852969b1baf7b7368113c09b38df996eb4314e52f0b531f74b903b72083b5"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
