# Verification

## Automated evidence

- `cargo fmt --all -- --check` — PASS.
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
- `cargo test -p db-pro-infrastructure --test ssh_backup_runtime_verification -- --ignored --nocapture` — PENDING: live isolated SSH server, pre-provisioned OpenSSH known-hosts entry, and PostgreSQL target required; the test exercises pg_dump, psql restore, and a post-restore query through the tunnel. It skips explicitly when `DB_PRO_SSH_*` is not configured, so the existing CI `--include-ignored` suite does not claim live SSH coverage. A local attempt reached the SSH process but was correctly rejected because the ephemeral host key was not trusted.
- `cargo test -p db-pro-core application::export_service::tests` — PASS: 8 export
  serialization, validation, read-only policy, integer precision, and coordinate
  overflow tests.
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
- External PostgreSQL command timeout regression — PASS on Unix via
  `external_command_timeout_returns_query_timeout`.
- `cargo test --workspace` — PASS: 246 core unit, 28 SQLite integration, 55
  infrastructure unit, 14 PostgreSQL integration tests ignored, plus all runtime,
  native, UI, schema regression, and doc tests passed.
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
- `cargo check --workspace` after SSH readiness changes — PASS.
- Full current gate — PASS: `cargo fmt --all -- --check`, `cargo clippy --workspace
  --all-targets -- -D warnings`, `cargo test --workspace`, release build for
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
| PostgreSQL | Unit policy/timeout coverage PASS | PASS (isolated `postgres:18.2`, 14/14) | Live introspection/query/transaction and batch rollback pass; SSH backup/tunnel execution pending |
| SQLite | Integration timeout/recovery PASS | PASS (in-memory provider) | Native UI runtime evidence is outside this core-only slice |

## Scope check

- UI/native files: source diff contains no files under `crates/ui` or `crates/native-app`.
- Release build: PASS for `db-pro-core` and `db-pro-infrastructure`; the native
  application release gate is outside this core-only hardening slice.
