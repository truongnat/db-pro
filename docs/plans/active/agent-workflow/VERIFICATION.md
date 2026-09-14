# Verification

## Current implementation slice

- Branch: `main`
- Base: `main@435ccfa`
- Implementation verification: core contracts, executor mapping, typed provider
  parsing, multi-step orchestration, confirmation pause/resume, and runtime
  command/event boundary are covered by automated checks.

- Core contracts, executor mapping, typed provider parsing, bounded orchestration, idempotency caching with collision rejection, failure outcome caching, safety recheck on approval, confirmation pause/resume, and document-scoped session lifecycle are fully covered by automated checks.
- `cargo fmt --all -- --check` — PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo test --workspace` — PASS (550 tests passed: 291 core, 21 runtime, 238 UI tests).
- `cargo build --release --locked -p db-pro-native` — PASS.
- Hardening test matrix verified:
  1. `mutation_run_query_executes_once_and_repeats_replay_without_database_re_execution`: DB runner invocation count strictly = 1 on repeated call_id.
  2. `tool_call_collision_with_different_input_fails_with_protocol_error`: Different tool/input on same call_id rejects with protocol collision error.
  3. `late_agent_workflow_events_are_ignored_after_cancellation`: Terminal sessions drop late events.
  4. `closing_query_tab_cleans_up_agent_session_and_cancels_active_run`: Closing tab cleans session and issues cancel.
  5. `open_agent_result_in_workspace`: Explicit sample rows labeling when total row count exceeds sampled rows.

The tests cover stale document/version rejection, UTF-8 patch boundaries,
confirmation-gated mutation/unknown SQL, bounded schema retrieval, bounded
result samples, serialized context limits, document snapshot routing, and
executor UTF-8-safe output handling.

The workflow tests additionally cover mode/tool permissions, run-id validation,
stale tool requests, patch preview/confirmation, mutation confirmation,
rejection without ending the run, cancellation cleanup, and the invariant that
no synthetic run ID is created after a run finishes.

## Required evidence

- Core unit tests for patch safety, bounded context, result summaries, and
  execution permissions.
- Workspace format/check/clippy/test gates after implementation.
- PostgreSQL and SQLite runtime verification for agent inspection/execution.
- Native UI evidence for patch preview, confirmation, cancellation, and
  background-tab routing.
- Native Agent panel consumption of workflow events and provider live
  tool-call evidence.
