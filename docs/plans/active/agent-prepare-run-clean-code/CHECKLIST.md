# Checklist: Fix AgentState::prepare_run Parameter Count

- [x] Define `PrepareAgentRunOptions` in `crates/ui/src/agent_state.rs`
- [x] Refactor `AgentState::prepare_run` to use `PrepareAgentRunOptions`
- [x] Update call site in `submit_typed_agent_prompt`
- [x] Update unit test call sites
- [x] Resolve merge with `main`: keep `AgentRunPreparation` (equivalent DTO already on `main`) and drop duplicate `PrepareAgentRunOptions`
- [ ] Re-run `cargo test -p db-pro-core -p db-pro-ui` after merge
- [ ] Re-run `cargo clippy -p db-pro-core -p db-pro-ui --all-targets -- -D warnings` after merge
- [ ] Re-run `cargo fmt --all -- --check` after merge
