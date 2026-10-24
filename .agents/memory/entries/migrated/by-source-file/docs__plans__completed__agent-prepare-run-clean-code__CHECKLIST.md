---
schema_version: 1
id: "MEM-SRC-85bc7a9016b241ed"
title: "Source mirror: docs/plans/completed/agent-prepare-run-clean-code/CHECKLIST.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/agent-prepare-run-clean-code/CHECKLIST.md"
source_sha256: "85bc7a9016b241ed2ff7effb89ec59cfbae676fe72b4a7ccedfb789b5766096e"
mirror_body_sha256: "85bc7a9016b241ed2ff7effb89ec59cfbae676fe72b4a7ccedfb789b5766096e"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
# Checklist: Fix AgentState::prepare_run Parameter Count

- [x] Define `PrepareAgentRunOptions` in `crates/ui/src/agent_state.rs`
- [x] Refactor `AgentState::prepare_run` to use `PrepareAgentRunOptions`
- [x] Update call site in `submit_typed_agent_prompt`
- [x] Update unit test call sites
- [x] Resolve merge with `main`: keep `AgentRunPreparation` (equivalent DTO already on `main`) and drop duplicate `PrepareAgentRunOptions`
- [x] Re-run `cargo test -p db-pro-core -p db-pro-ui` after merge
- [x] Re-run `cargo clippy -p db-pro-core -p db-pro-ui --all-targets -- -D warnings` after merge
- [x] Re-run `cargo fmt --all -- --check` after merge
