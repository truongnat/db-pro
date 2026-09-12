# Verification

## Automated evidence

- `cargo fmt --all -- --check` — PASS at the prior clean-worktree gate; the
  current shared worktree has unrelated native-app formatting edits, so the
  latest global formatter run is not claimable for this core-only wave.
- `rustfmt --edition 2021 --check crates/infrastructure/src/sqlite/introspect.rs` — PASS for the SQLite core change. A later global formatter run is blocked by unrelated uncommitted native-app formatting changes; no native file is part of this core change.
- `git diff --check` — PASS.
- `cargo check --workspace` — PASS.
- `cargo clippy --workspace --all-targets -- -D warnings` — PASS.
- `cargo test -p db-pro-core backup_factory_receives_ssh_configuration` — PASS.
- `cargo test -p db-pro-core --lib application::connection_service::tests::disconnect_failure_keeps_handle_for_retry -- --exact` — PASS.
- `cargo test -p db-pro-infrastructure postgres::connector::tests::postgres_operation_timeout_returns_query_timeout -- --exact` — PASS.
- `cargo test -p db-pro-infrastructure postgres::connector::tests` — PASS: 2
  PostgreSQL SQL-builder/timeout tests; transaction and batch rollback are also
  covered by the live fixture below.
- `cargo test -p db-pro-infrastructure postgres::user_manager::tests` — PASS: 3
  identifier/privilege validation tests.
- `cargo test -p db-pro-core application::connection_service::tests` — PASS: 29
  connection lifecycle/update/delete/connectivity/duplicate-cleanup/cache-invalidation tests.
- `cargo test -p db-pro-core domain::safety::tests` — PASS: 34 safety classifier
  and policy tests, including comment-separated CTE mutations and quoted-token boundaries.
- `cargo test -p db-pro-core domain::safety::tests::cte_classifier_respects_lexical_boundaries_and_comment_separators -- --exact` — PASS: comment-separated CTE mutation is classified as destructive and rejected by the read-only policy.
- `cargo test -p db-pro-core domain::safety::tests::classify_merge_delete_as_destructive_but_update_as_write -- --exact` — PASS: MERGE DELETE actions are destructive while update-only and quoted DELETE text remain writes.
- `cargo test -p db-pro-core domain::safety::tests::classify_opaque_server_side_execution_as_destructive -- --exact` — PASS: DO, CALL, and EXECUTE cannot bypass a policy that forbids destructive operations.
- `cargo test -p db-pro-infrastructure meta::query_history_repo::tests::query_history_metrics_reject_invalid_values -- --exact` — PASS: malformed persisted duration and row-count values return an explicit error.
- `cargo test -p db-pro-infrastructure meta::migration::tests::malformed_schema_version_fails_closed -- --exact` — PASS: malformed persisted schema versions return an explicit internal error.
- `cargo test -p db-pro-infrastructure meta::migration::tests::unsupported_future_schema_version_is_rejected -- --exact` — PASS: databases newer than the binary are rejected before migrations run.
- `cargo test -p db-pro-infrastructure sqlite::introspect::tests --no-fail-fast` — PASS: SQLite introspection including primary-key identity remains valid after fallible `pk` metadata decoding.
- `cargo test -p db-pro-core application::query_service::tests --no-fail-fast` — PASS: 25 query-service tests, including query routing for `INSERT/UPDATE/DELETE ... RETURNING` and nested CTE boundary handling.
- `cargo test -p db-pro-core application::query_service::tests --no-fail-fast` — PASS after transaction-result validation: 26 query-service tests, including rejection of malformed transactional row/column shapes.
- `cargo test -p db-pro-infrastructure ssh::tunnel::tests -- --nocapture` — PASS: 3 SSH command tests, including bounded process timeout and child cleanup behavior.
- `cargo test -p db-pro-core application::export_service::tests::export_rejects_malformed_query_result_shape -- --exact` — PASS: malformed export result is rejected at the service boundary.
- `cargo test -p db-pro-core application::table_data_service::tests::fetch_rows_rejects_malformed_data_result_shape -- --exact` — PASS: malformed paginated data result is rejected at the service boundary.
- `cargo test -p db-pro-core --lib --no-fail-fast` — PASS: 252 core unit tests.
- `cargo clippy -p db-pro-core --all-targets -- -D warnings` — PASS.
- `cargo test -p db-pro-core application::schema_diff::tests --no-fail-fast` — PASS: 4 schema-diff tests, including dotted schema/object identity collision and unambiguous display coverage.
- `cargo test -p db-pro-core --lib --no-fail-fast` — PASS: 253 core unit tests after structured schema-diff keys.
- `cargo test -p db-pro-core application::table_data_service::tests::parse_total_count_rejects_non_scalar_result -- --exact` — PASS: pagination rejects a non-scalar count payload.
- `cargo test -p db-pro-core application::data_diff::tests::extract_count_rejects_non_scalar_provider_result -- --exact` — PASS: data-diff rejects a non-scalar count payload.
- `cargo test -p db-pro-core --lib --no-fail-fast` — PASS: 255 core unit tests after scalar count validation.
- `cargo test -p db-pro-core domain::connection::tests::validate_name_uses_character_count_for_unicode -- --exact` — PASS: Unicode connection-name 128/129-character boundary is enforced correctly.
- `cargo test -p db-pro-core --lib --no-fail-fast` — PASS: 256 core unit tests after Unicode name-length validation.
- `cargo test -p db-pro-core domain::query::tests --no-fail-fast` — PASS: 9 query-result contract tests, including row-count consistency, affected-row compatibility, and rejection of rows without columns.
- `cargo test -p db-pro-core --lib --no-fail-fast` — PASS: 262 core unit tests after rows-without-columns and execution-lifecycle validation.
- `cargo test -p db-pro-core domain::execution::tests --no-fail-fast` — PASS: 13 execution lifecycle tests, including invalid non-terminal finishes and late success callbacks.
- `cargo test -p db-pro-core application::query_service::tests::execute_multi_commit_failure_reports_unknown_outcome -- --exact` — PASS: core reports a commit failure as an unknown final outcome and preserves partial results.
- `cargo test -p db-pro-core application::export_service::tests::export_json_rejects_non_finite_float -- --exact` — PASS: JSON export rejects `NaN` instead of silently serializing it as `null`.
- `cargo test -p db-pro-core application::sql_builder::tests::date_cell_uses_typed_date_parameter -- --exact` — PASS: `CellValue::Date` maps to the existing typed date-capable parameter path.
- `cargo test -p db-pro-core application::sql_builder::tests::temporal_and_network_cells_use_typed_parameters -- --exact` and `cargo test -p db-pro-core domain::query::tests::typed_query_params_round_trip_through_ipc_json -- --exact` — PASS: typed time/interval/network cells are preserved by SQL building and IPC JSON.
- `cargo test -p db-pro-infrastructure postgres::query_mapper::tests --no-fail-fast` — PASS: 9 PostgreSQL mapper/binder tests, including strict TIME/TIMETZ and INTERVAL parsing.
- `cargo test -p db-pro-infrastructure --test integration sqlite_typed_temporal_and_network_parameters_remain_text --no-fail-fast` — PASS: SQLite receives the typed temporal/network values as text, matching its storage model.
- The same full ignored PostgreSQL integration command — PASS: 18/18 against an isolated temporary `postgres:18.2` fixture, including native TIME/TIMETZ/INTERVAL/INET parameter binding without explicit casts; the Docker container was removed after the run.
- After routing `SqliteActor` through the shared mapper, `cargo test -p db-pro-infrastructure --lib --no-fail-fast` — PASS: 61 tests, and `cargo test -p db-pro-infrastructure --test integration --no-fail-fast` — PASS: 30 tests.
- `cargo test -p db-pro-core application::sql_builder::tests::insert_rejects_empty_columns -- --exact` and `update_rejects_empty_columns` — PASS: empty mutation payloads fail at the core boundary instead of generating invalid SQL.
- `cargo test -p db-pro-infrastructure --test integration --no-fail-fast` — PASS: 29 SQLite integration tests, including deferred-foreign-key commit failure (`Commit + Unknown`) and preserved partial results.
- `DATABASE_URL=postgres://dbpro:dbpro_test@127.0.0.1:15434/dbpro_fixture cargo test -p db-pro-infrastructure --test pg_integration pg_transaction_commit_failure_reports_unknown_outcome --offline -- --ignored --exact --nocapture` — PASS: deferred PostgreSQL foreign-key failure is reported as `Commit + Unknown` with partial results preserved.
- `cargo test -p db-pro-core application::export_service::tests::export_json_rejects_duplicate_column_names -- --exact` — PASS: JSON export rejects duplicate object keys instead of silently dropping the earlier value.
- `DB_PRO_SSH_*`-configured `cargo test -p db-pro-infrastructure --test ssh_backup_runtime_verification -- --ignored --nocapture` — PASS: 1 live isolated SSH backup/restore test; pg_dump, database creation, psql restore, and post-restore query all completed through the tunnel with host-key verification enabled.
- CI configuration now provisions the SSHD/key/known-hosts fixture and exports the required `DB_PRO_SSH_*` variables before `cargo test --all -- --include-ignored`; live CI execution remains pending until that workflow run completes.
- `cargo test -p db-pro-core application::export_service::tests` — PASS: 10 export
  serialization, validation, read-only policy, duplicate-column, integer precision,
  and coordinate overflow tests.
- `cargo test -p db-pro-core application::table_data_service::tests` — PASS: 16
  table mutation, pagination, and count-validation tests.
- `cargo test -p db-pro-core application::data_diff::tests` — PASS: 2 data-diff
  count validation and checked-difference tests.
- `cargo test -p db-pro-core application::schema_diff::tests` — PASS: 3 schema
  diff qualification, comparison, and deterministic-order tests.
- `cargo test -p db-pro-core application::connection_service::tests` — PASS: 36
  tests, including validation before both connectivity test paths.
- `cargo test -p db-pro-core application::connection_service::tests` — PASS: 38
  tests, including legacy PostgreSQL default-secret fallback for connect and
  delete.
- `cargo test -p db-pro-infrastructure sqlite::introspect::tests --no-fail-fast` — PASS:
  11 SQLite introspection tests, including independent nested CHECK extraction,
  ignoring CHECK text in literals/comments, and trigger keyword boundaries.
- `cargo test -p db-pro-infrastructure sqlite::introspect::tests::introspection_marks_unique_constraint_indexes_without_reusing_internal_names -- --exact` — PASS: SQLite marks table-level UNIQUE autoindexes by origin while preserving their columns and uniqueness.
- `cargo test -p db-pro-infrastructure postgres::introspect::tests::test_quoted_identifier_with_parenthesis_and_comma -- --exact` — PASS: PostgreSQL index parsing preserves quoted identifier boundaries.
- `cargo test -p db-pro-infrastructure postgres::introspect::tests --no-fail-fast` — PASS: 14 PostgreSQL introspection/parser tests after fallible metadata decoding.
- `cargo test -p db-pro-core domain::connection::tests` — PASS: 13 connection
  validation and metadata-security tests, including SQLite SSH rejection.
- `cargo test -p db-pro-core application::user_service::tests` — PASS: 1
  SQLite capability-gate test; `cargo clippy -p db-pro-core --all-targets -- -D warnings` — PASS.
- `cargo test -p db-pro-core application::schema_service::tests::get_table_ddl_sqlite_uses_inline_foreign_keys_and_unqualified_names -- --exact` — PASS: SQLite DDL reconstruction uses inline foreign keys and local object names.
- `cargo test -p db-pro-core application::schema_service::tests --no-fail-fast` — PASS: 16 schema-service tests, including CHECK constraint reconstruction.
- The same schema-service suite asserts schema-qualified PostgreSQL index names and local SQLite index names.
- `cargo test -p db-pro-core execute_multi_routes_mutating_cte_through_transaction_while_preserving_rows` — PASS.
- `cargo test -p db-pro-core execute_multi_routes_select_then_update` — PASS.
- `cargo test -p db-pro-infrastructure backup::pg_dump::tests` — PASS: 2 tests,
  including existing-destination rejection and external command timeout.
- `cargo test -p db-pro-infrastructure backup::sqlite_backup::tests` — PASS: 2
  tests, including no-overwrite publish behavior.
- `cargo test -p db-pro-core backup_uses_persisted_custom_secret_reference -- --nocapture` — PASS.
- `cargo test -p db-pro-core restore_uses_persisted_custom_secret_reference -- --nocapture` — PASS.
- `cargo test -p db-pro-core application::backup_service::tests` — PASS: 5 tests,
  including SQLite backup/restore without a database secret.
- `cargo test -p db-pro-infrastructure ssh::tunnel::tests -- --nocapture` — PASS: 2
  command-construction tests for key and password authentication modes.
- `cargo test -p db-pro-core --no-fail-fast` — PASS: 246 core unit tests,
  including SSH metadata redaction and connection lifecycle hydration coverage.
- `cargo test -p db-pro-core application::sql_policy::tests` — PASS: 9 lexical
  boundary and statement-splitting tests.
- `DATABASE_URL=postgres://dbpro:dbpro_test@127.0.0.1:15434/dbpro_fixture cargo test -p db-pro-infrastructure --test pg_integration --offline -- --ignored` — PASS: 14/14 against an isolated temporary `postgres:18.2` fixture after fallible metadata decoding and the `enabled_flag` text cast; the container was removed after the run.
- The same full ignored PostgreSQL integration command — PASS: 15/15 against an isolated temporary `postgres:18.2` fixture, including commit-failure outcome reporting; the Docker container was removed after the run.
- The same full ignored PostgreSQL integration command — PASS: 16/16 against an isolated temporary `postgres:18.2` fixture, including live DATE filter binding; the Docker container was removed after the run.
- `cargo test -p db-pro-infrastructure --test pg_integration pg_query_decodes_native_temporal_and_network_values --offline -- --ignored --exact --nocapture` — PASS: native PostgreSQL `TIME`, `INTERVAL`, and `INET` values decode to typed `CellValue`s without placeholder fallback.
- The same full ignored PostgreSQL integration command — PASS: 17/17 against an isolated temporary `postgres:18.2` fixture, including temporal/network decoding and custom enum preservation; the Docker container was removed after the run.
- External PostgreSQL command timeout regression — PASS on Unix via
  `external_command_timeout_returns_query_timeout`.
- `cargo test --workspace --no-fail-fast` — PASS: 246 core unit, 28 SQLite
  integration, 57 infrastructure unit, 14 PostgreSQL integration tests ignored,
  plus runtime, native, UI, schema regression, and doc tests. The shared
  checkout required external native commit `9e7be46` before this verification;
  no native source was changed in the core hardening commits.
- `cargo test -p db-pro-core application::schema_service::tests::get_table_ddl_sqlite_uses_inline_foreign_keys_and_unqualified_names -- --exact` — PASS: reconstructed SQLite DDL preserves UNIQUE constraints without replaying the internal autoindex name.
- Targeted regression `sqlite_query_timeout_interrupts_vm_and_actor_recovers` — PASS.
- Targeted regression `sqlite_transaction_timeout_waits_for_rollback_before_returning` — PASS
  against the in-memory SQLite provider; the post-timeout count was zero.
- Targeted regression `sqlite_execute_batch_timeout_waits_for_rollback_before_returning` — PASS
  against the in-memory SQLite provider; the post-timeout count was zero and the actor recovered.
- `cargo build --release --locked -p db-pro-core -p db-pro-infrastructure` — PASS.
- `bash .skills/clean-code/scripts/clean-code-scan.sh --diff` — PASS: 8 pass,
  8 heuristic warning groups, 0 blocking failures. Remaining warnings cover
  test assertions, intentional cleanup sends, parser/module size, and unrelated
  native/UI helpers; none is a clippy or scanner blocker.
- Transaction commit-failure semantics — PASS for core, SQLite, and PostgreSQL:
  typed phase/outcome fields prevent false rollback claims, both providers
  preserve prior results, and deferred-constraint commit failures are covered
  against live fixtures. The separate CI SSH workflow execution remains pending.
- Latest shared-checkout verification: `cargo check --workspace` reaches all
  core/infrastructure/runtime/native crates but is blocked by the pre-existing
  dirty `crates/tauri-app/src/commands/query.rs` errors at lines 50 and 114
  (`DbErrorDto` passed to a helper requiring `&DbError`). This file is outside
  the core-only scope and was not modified.
- Query cancellation contract — PASS: 270 core tests, 62 infrastructure unit
  tests, 31 SQLite integration tests, and 4 runtime tests. The SQLite live
  cancellation regression interrupts the VM and waits for actor recovery;
  SQLite's capability is true only for that provider-safe actor path;
  PostgreSQL returns explicit `Unsupported` and its capability is false until
  a provider-safe cancellation primitive is implemented. Runtime cancellation
  failure emits `RuntimeEvent::Failed` instead of leaving a request pending.
- Scoped cancellation gates — PASS: `cargo check -p db-pro-core
  -p db-pro-infrastructure -p db-pro-runtime`, clippy with `-D warnings`,
  release build for the three scoped crates, scoped rustfmt check, and
  `clean-code-scan.sh --diff` with 0 blocking failures.
- QueryService DDL cache contract — PASS: targeted
  `execute_ddl_invalidates_schema_cache` test verifies successful DDL
  invalidates the shared cache; runtime wiring uses the same metadata cache as
  SchemaService. Unknown transaction outcomes invalidate through the same
  explicit cache boundary.
- `cargo check --workspace` after SSH readiness changes — PASS.
- Core/infrastructure scoped gate — PASS: file-scoped rustfmt for the SQLite
  change,
  `cargo clippy -p db-pro-core --all-targets -- -D warnings`,
  `cargo clippy -p db-pro-infrastructure --all-targets -- -D warnings`,
  `cargo test -p db-pro-core --lib` (246),
  `cargo test -p db-pro-infrastructure --lib` (57), release build for
  `db-pro-core` and `db-pro-infrastructure`, and `git diff --check`.

The multi-statement regression proves that a row-producing data-modifying CTE is
sent through `execute_transaction` with query-result routing preserved. Live
PostgreSQL transaction failure, transaction timeout rollback, batch statement
failure rollback, and batch timeout rollback are covered by the isolated fixture
run above. SQLite has equivalent
batch and transaction timeout/rollback coverage against its in-memory provider.

Source-only security check:

- `rg -n "StrictHostKeyChecking" crates/infrastructure` — no matches.

The PostgreSQL backup test proves the core factory preserves `ssh_tunnel`; actual
SSH-tunnel backup execution remains pending. The live fixture proves PostgreSQL
introspection/query behavior plus transaction and batch rollback after statement
failure and timeout. The backup destination test proves an existing PostgreSQL
artifact is rejected before external command execution.

## Provider matrix

| Provider | Automated | Live provider | Notes |
|---|---|---|---|
| PostgreSQL | Unit policy/timeout coverage PASS | PASS (isolated `postgres:18.2`, 18/18) | Live introspection/query/temporal-network decoding and native parameter binding/DATE binding/transaction commit outcome, batch rollback, and isolated SSH backup/tunnel pass; CI SSH workflow execution pending |
| SQLite | Integration timeout/recovery and typed text binding PASS | PASS (in-memory provider) | Shared actor/mapper path verified; native UI runtime evidence is outside this core-only slice |

## Scope check

- UI/native files: source diff contains no files under `crates/ui` or `crates/native-app`.
- Release build: PASS for `db-pro-core` and `db-pro-infrastructure`; the native
  application release gate is outside this core-only hardening slice.
