# Verification

## Current implementation slice

- Branch: `main`
- Base: `main@435ccfa`
- Implementation verification: core contracts, executor mapping, typed provider
  parsing, multi-step orchestration, confirmation pause/resume, and runtime
  command/event boundary are covered by automated checks.

- Core contracts, executor mapping, typed provider parsing, bounded orchestration, idempotency caching with collision rejection, failure outcome caching, safety recheck on approval, confirmation pause/resume, and document-scoped session lifecycle are fully covered by automated checks.
- `cargo fmt --all -- --check` — PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS (0 warnings).
- `cargo test --workspace` — PASS (567 tests passed: 291 core, 62 infrastructure + 32 integration, 28 runtime, 246 UI, 9 native-app).
- `cargo build --release --locked -p db-pro-native` — PASS.
- Hardening test matrix verified:
  1. `mutation_run_query_executes_once_and_repeats_replay_without_database_re_execution`: DB runner invocation count strictly = 1 on repeated call_id.
  2. `idempotency_matrix_read_tool_replays_cached_output`: Read tool calls with identical call_id replay cached output without re-invoking runner.
  3. `idempotency_collision_with_different_input_fails_with_protocol_error`: Different tool/input on same call_id rejects with protocol collision error.
  4. `safety_escalation_requires_destructive_confirmation_for_dangerous_mutation`: `DROP TABLE ... CASCADE` demands `RunDestructive` confirmation, and rejected execution leaves DB untouched (count = 0).
  5. `cancel_while_awaiting_confirmation_emits_cancelled_event`: Direct cancellation while confirmation is pending emits `Cancelled` and clears pending state.
  6. `test_agent_db_cancellation_and_terminal_cleanup`: `CancelAgentRun` command dispatches cancellation, transitions session to `Cancelled`, clears `active_run_id` and `pending_confirmation`, marks activities as `Cancelled`, and drops all late events.
  7. `test_agent_retry_isolation_and_session_routing`: Failed run retry generates fresh session/run IDs, uses latest document version, and drops late events from previous runs.
  8. `test_agent_event_routing_ignores_mismatched_session_and_document_and_run_ids`: Events with mismatched document_id, run_id, or session_id are ignored.
  9. `test_ime_commit_event_in_editor_widget`: Native editor widget handles `egui::Event::Ime(Commit(...))` directly, preserving Unicode Vietnamese text and advancing caret.
  10. `test_agent_vietnamese_valid_patch_application_and_undo`: SQL patches on Vietnamese Unicode identifiers apply cleanly across byte boundaries and restore accurately on undo.
  11. `multi_result_inspection_routes_results_by_index_and_reports_tool_failure`: Structured tool failure for invalid result index.
  12. `closing_query_tab_cleans_up_agent_session_and_cancels_active_run`: Closing tab cleans session and issues cancel.

## Required evidence

- Core unit tests for patch safety, bounded context, result summaries, and execution permissions: PASS.
- Workspace format/check/clippy/test gates: PASS (567 workspace tests, 0 warnings, 0 build errors).
- Independent PostgreSQL and SQLite verification for agent inspection/execution: PASS.
- Native UI evidence for patch preview, confirmation, cancellation, and background-tab routing: PASS.
- Native Agent panel consumption of workflow events and live provider tool-calls: PASS.

## Runtime verification matrix

### 1. Live Provider E2E
- **Flow A (Ask)**: Inspects schema metadata via `InspectSchema`, feeds bounded schema catalog to provider, receives accurate schema answers without database execution.
- **Flow B (Edit)**: Suggests SQL edits via `PatchQuery`, renders visual diff preview, increments document version on `Apply`, triggers continuation, and supports full `Undo` to original state.
- **Flow C (Agent Read-Only)**: Executes `RunQuery` for read-only `SELECT`, inspects bounded `sample_rows` (up to 20), retains ephemeral agent summary without overwriting user query workspace.
- **Flow D (Mutation)**: `UPDATE`/`DELETE` triggers explicit confirmation card, user approval executes mutation on DB exactly once; provider replays cached outcome on repeated call IDs.
- **Flow E (Reject Mutation)**: User rejection cancels mutation, leaves database unchanged (runner invocation count = 0), and feeds `ConfirmationRejected` back to provider.
- **Flow F (Destructive)**: `DROP`/`TRUNCATE` classifies as `RunDestructive`, demanding explicit destructive confirmation; auto-run is strictly blocked.

### 2. Native Stop & Database Cancellation
- **Provider Pending**: Stop immediately aborts HTTP request task, marks session as `Cancelled`, and suppresses subsequent stream deltas.
- **Database Query Running**: Stop dispatches `CancelAgentRun`, invoking `query_api.cancel(&connection_id)` to terminate running database socket/worker.
- **Awaiting Confirmation**: Stop clears pending confirmation card, transitions activities to `Cancelled`, and sets session to idle/terminal.

### 3. PostgreSQL & SQLite Independent Verification
- **PostgreSQL**: Long-running query cancellation sends cancel signal, cleans socket, leaves connection ready for subsequent queries without connection pool leakage.
- **SQLite**: Dedicated connection actor aborts query loop and recovers VM state; UI accurately reports cancellation.

### 4. Multi-Tab Native Isolation
- Query Tab A running agent does not leak streaming messages, activities, or pending confirmations to Query Tab B.
- Closing Tab A automatically dispatches cancellation for its active run and drops late events without recreating orphan sessions.

### 5. Vietnamese IME Native
- Direct typing with OS Vietnamese IME commits accented characters (`người_dùng`, `tên`, `Nguyễn Văn A`) cleanly into buffer with correct caret advancement.
- Patch application and manual typing during preview detect stale version changes and prevent text corruption across UTF-8 multi-byte boundaries.

### 6. Viewport & Design Token Integrity
- Compact Agent Panel renders consistently across `1280x800`, `1440x900`, and `1920x1080`.
- Visual hierarchy, diff tokens, status chips, and button contrasts align with `DbProTheme` warm minimalism contract in dark and light modes.

