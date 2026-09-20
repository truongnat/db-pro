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
