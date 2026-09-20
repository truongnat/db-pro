# Checklist: Fix AgentState::prepare_run Parameter Count

- [x] Define `PrepareAgentRunOptions` in `crates/ui/src/agent_state.rs`
- [x] Refactor `AgentState::prepare_run` to use `PrepareAgentRunOptions`
- [x] Update call site in `submit_typed_agent_prompt`
- [x] Update unit test call sites
- [x] Run `cargo test -p db-pro-core -p db-pro-ui`
- [x] Run `cargo clippy -p db-pro-core -p db-pro-ui --all-targets -- -D warnings`
- [x] Run `cargo fmt --all -- --check`
