---
schema_version: 1
id: "MEM-SRC-3e0a7bec5a2d4ab9"
title: "Source mirror: docs/plans/completed/agent-prepare-run-clean-code/FINDINGS.md"
type: pattern
scope: project
status: needs-review
mirror_kind: full-file
mirror_wave: v2-by-source-file
source_path: "docs/plans/completed/agent-prepare-run-clean-code/FINDINGS.md"
source_sha256: "3e0a7bec5a2d4ab9e83e3b25d8d861e2043a30097a5c4f46df3b420b0ede37c4"
mirror_body_sha256: "3e0a7bec5a2d4ab9e83e3b25d8d861e2043a30097a5c4f46df3b420b0ede37c4"
truncated: false
created: "2026-10-24"
updated: "2026-10-24"
verified_on: unknown
confidence: low
tags: [migrated, by-source-file]
---
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
