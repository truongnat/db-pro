---
schema_version: 1
id: "MEM-PLAN-core-safety-hardening-7f993863c1f4"
title: "Plan archive: Core Safety Hardening"
type: bugfix
scope: project
status: needs-review
created: "2026-10-24"
updated: "2026-10-24"
verified_on: "unknown"
confidence: medium
tags: [migrated, docs-plans, bugfix, core-safety-hardening]
mirror_wave: v1-plan-evidence-excerpt
mirror_excerpt_sha256: "7f993863c1f4ba5a87a38cdae797d05aca910c214ba799c13d62272a1aa7ce0b"
applies_to:
  repo: "db-pro"
  stack: "see sources"
  versions: "see plan state at migration"
sources:
  - "docs/plans/active/core-safety-hardening/PLAN.md"
  - "docs/plans/active/core-safety-hardening/FINDINGS.md"
  - "docs/plans/active/core-safety-hardening/VERIFICATION.md"
related: []
supersedes: null
---

# Tóm tắt

Migrate từ `docs/plans/active/core-safety-hardening/`. Loại suy luận: `bugfix`.

Entry tự động từ `docs/plans/`. Đối chiếu code hiện tại trước khi đặt `status: active`.

## Nội dung gốc (rút gọn)

## Source: `PLAN.md`

# Core Safety Hardening

## Goal

Close the highest-risk correctness and security gaps in the core database path
without changing the native UI or adding product features.

## Scope

- `crates/core`: application safety and mutation invariants.
- `crates/infrastructure`: PostgreSQL/SQLite execution behavior and SSH command safety.
- `crates/runtime`: only if required to preserve the core contract at the runtime boundary.

## Non-goals

- No changes to `crates/ui`, `crates/native-app`, Tauri, Agent/MCP, or visual design.
- No schema migration, import workflow, or new provider feature.
- No claim of native UI runtime verification in this core-only wave; SSH
  verification is limited to the isolated fixture and CI evidence recorded below.

## Findings driving this wave

1. `QueryService::explain` bypasses the persisted safety policy, so `EXPLAIN ANALYZE`
   can execute a mutation on a read-only connection.
2. Table mutations pass through `affected_rows = 0` as success, hiding stale or missing
   row identities.
3. SQLite ignores the configured query timeout because its actor has no interrupt path.
4. SSH commands disable host-key verification.
5. PostgreSQL backup/restore factories discard `ssh_tunnel`, so external backup
   commands bypass the connection's configured tunnel.
6. Row updates allow primary-key columns even though the staged mutation contract
   identifies the target using the original primary key.
7. External PostgreSQL backup commands have no configured timeout.
8. Connection disconnect removes the registry entry before the connector has
   successfully released the resource.
9. Connection testing bypasses SSH even though actual connection establishment
   uses the tunnel.
10. PostgreSQL applies the configured operation timeout to query/mutation paths but
    leaves connect, Test Connection, introspection, and Explain unbounded.
11. PostgreSQL user-management interpolates untrusted identifiers and privilege
    text into role/permission statements.
12. Connection updates change runtime and secret state in a non-compensating order,
    which can lose an active session or leave a password/secret reference out of
    sync when persistence fails.
13. PostgreSQL transaction timeout can return before SQLx has completed rollback.
14. Connection deletion can remove the secret before repository deletion succeeds.
15. Test Connection resolves the default secret key instead of a persisted custom
    `secret_ref`.
16. Pagination values can overflow when converted to provider integer parameters.
17. Destructive DELETE detection treats `WHERE` inside identifiers/comments as a
    real clause.
18. Concurrent duplicate connects can lose the newly opened handle when its first
    cleanup attempt fails.
19. SQLite transaction timeout returns before the actor has completed rollback.
20. Export queries bypass the persisted read-only safety policy.
21. Multi-statement data-modifying CTEs and `EXPLAIN ANALYZE` can be routed outside
    the atomic transaction path because result classification is reused as the
    mutation detector.
22. PostgreSQL backup passes its destination directly to `pg_dump -f`, allowing an
    existing backup artifact to be overwritten.
23. SQLite backup checks destination existence before snapshot creation but uses
    replacement `rename()` at publish time, leaving a race window.
24. SQL statement boundary handling only understands single-quoted strings and
    mishandles comments, quoted identifiers, and PostgreSQL dollar-quoted bodies.
25. Batch execution can report a timeout or statement failure before provider
    rollback has completed, allowing the next operation to race cleanup.
26. Backup and restore resolve only the default password key and discard a
    persisted custom `secret_ref`.
27. SSH Test Tunnel ignores password authentication even though tunnel startup
    supports it through `sshpass`.
28. `SshTunnelConfig.password` is part of the serialized connection metadata,
    allowing SSH credentials to be persisted in `meta.db` and exposed through
    ordinary debug/serialization paths.
29. SQLite backup and restore ask the `SecretStore` for a database password even
    though the SQLite backup engine does not consume one.
30. Connection updates can change the target database while leaving the
    connection-id keyed introspection cache intact.
31. Excel export converts every `i64` to `f64` and uses unchecked row/column
    index casts, which can corrupt large BIGINT values or wrap oversized results.
32. Table pagination converts a malformed or negative `COUNT(*)` result to `u64`
    with a silent `0` fallback, allowing invalid provider data to become a
    misleading pagination state or a huge wrapped row count.
33. Cross-connection data diff accepts negative counts and computes the signed
    difference without checked arithmetic.
34. Schema diff collects set differences directly from `HashSet`, making the
    order of tables, columns, and indexes unstable across runs.
35. Connection lifecycle always requires a database secret, even for SQLite,
    although the SQLite provider has no database credential boundary.
36. Core connection validation accepts SSH tunnel configuration for SQLite,
    although the provider cannot use SSH tunneling.
37. User-management operations are exposed through `UserService` without
    checking the provider capability, so SQLite can reach a PostgreSQL role API.
38. Table DDL reconstruction always emits PostgreSQL-style foreign-key and
    schema-qualified statements, so a SQLite table script cannot be executed.
39. Table DDL reconstruction ignores introspected CHECK constraints, so a
    recreated table can lose data-validation invariants.
40. Reconstructed PostgreSQL indexes omit the source schema from the index
    name, so creation depends on the session `search_path`.
41. Connection test paths bypass `ConnectionConfig::validate`, allowing invalid
    provider-specific drafts to reach a connector.
42. EXPLAIN safety classification searches for `ANALYZE` by substring and can
    slice the original SQL with an invalid Unicode offset.
43. CTE DELETE safety classification searches for `WHERE` by substring and can
    misclassify a destructive delete when comments or literals contain it.
44. Data-modifying CTE safety classification downgrades `DELETE` without a
    predicate to `Write`, bypassing policies that forbid destructive operations.
45. Legacy PostgreSQL connections without `secret_ref` could pass Test Connection
    through the default password key but fail normal connect and leave that
    credential orphaned on delete.
46. SQLite CHECK introspection uses the outer `CREATE TABLE` parenthesis depth,
    so multiple or nested CHECK expressions are merged into an invalid definition.
47. SQLite trigger introspection searches timing/event keywords by substring, so
    quoted trigger or table names can corrupt the reported trigger metadata.
48. SQLite introspection exposes internal UNIQUE autoindex names as ordinary
    indexes, so reconstructed table DDL attempts to recreate a name SQLite owns.
49. PostgreSQL index-column parsing splits quoted identifiers containing commas,
    producing incorrect metadata and invalid reconstructed index DDL.
50. PostgreSQL introspection converts metadata decode errors into empty strings,
    allowing corrupted rows to appear as valid schema metadata.
51. CTE safety classification requires whitespace after mutation keywords and
    scans only single-quoted strings, so comment-separated mutations can bypass
    the read-only/destructive policy.
52. PostgreSQL/SQL Server `MERGE` statements fall through to generic `Write`,
    so a `WHEN MATCHED THEN DELETE` action bypasses destructive-operation policy.
53. Opaque server-side execution statements (`DO`, `CALL`, and `EXECUTE`) fall
    through to generic `Write`, even though they can execute hidden or dynamic
    destructive mutations that the client-side classifier cannot inspect.
54. Query history converts malformed persisted `duration_ms` and `row_count`
    values to zero, masking metadata corruption as a valid history entry.
55. Metadata migration treats malformed schema versions as version `0` and does
    not reject a database schema newer than the binary supports.
56. SQLite column introspection converts a primary-key metadata decode error to
    `false`, which can erase row identity information from the schema snapshot.
57. Multi-statement routing treats DML with a top-level `RETURNING` clause as
    affected-row-only execution and drops the returned rows.
58. Transactional multi-query execution returns provider `QueryResult` values
    without applying the core row/column shape invariant.
59. SSH Test Tunnel bounds only connection establishment and can await the
    external process forever after connection setup.
60. Export, table-data, and data-diff services consume connector query results
    without applying the core row/column shape invariant.
61. Schema diff encodes schema/object identity into a dotted string, so valid
    identifiers containing dots can be split incorrectly or collide.
62. Pagination and data-diff count consumers accept the first cell of a
    non-scalar result instead of requiring the `COUNT(*)` scalar contract.
63. Connection-name validation measures UTF-8 bytes while reporting a
    character limit, rejecting valid non-ASCII names at the boundary.
64. `QueryResult::validate` checks row cell shape but accepts a mismatched
    `row_count`, allowing incorrect result metrics to cross core boundaries.
65. JSON export uses column names as object keys, so a result with duplicate
    column names silently overwrites an earlier value.
66. `QueryResult::validate` accepts non-empty rows without column metadata,
    allowing consumers such as JSON export to discard every cell.
67. `QueryExecution` accepts invalid lifecycle transitions and can overwrite
    terminal execution metrics through a late success callback.
68. PostgreSQL query mapping can silently coerce failed native-value decodes to
    raw text or placeholder cells.
69. Temporal and network cells are downgraded to `TEXT` parameters before
    PostgreSQL filter and mutation binding.
70. Runtime query cancellation reports success without invoking a provider
    cancellation primitive; PostgreSQL is advertised as cancellable while the
    runtime only drops the awaiting future, and SQLite cancellation does not
    wait for actor recovery.
71. DDL submitted through `QueryService` does not invalidate the
    connection-scoped introspection cache, including the unknown-outcome case
    of an atomic transaction.

## Acceptance criteria

- Explain validates against the same safety policy as query execution.
- Update/delete with zero affected rows return a typed `NotFound`/conflict error.
- SQLite query execution interrupts the SQLite VM when the configured timeout expires.
- SSH uses the OpenSSH default known-host verification behavior and never passes
  `StrictHostKeyChecking=no`.
- PostgreSQL backup/restore receives the complete connection configuration and routes
  external commands through the configured SSH tunnel.
- SSH tunnel startup confirms the local forward is listening and fails if the SSH
  process exits before readiness.
- Row mutation rejects primary-key edits until the identity contract supports them.
- PostgreSQL backup/restore commands stop at the connection timeout.
- Failed disconnect preserves the active handle so cleanup can be retried.
- Connection testing uses the same effective SSH-routed configuration as connect.
- PostgreSQL connect, Test Connection, introspection, and Explain honor the
  configured operation timeout.
- PostgreSQL role and privilege mutations quote identifiers and accept only the
  supported table-privilege vocabulary.
- Connection updates persist configuration and secret changes with compensation;
  active sessions are disconnected only after persistence succeeds.
- Multi-statement scripts containing any muta

… [truncated for memory entry; see source file]


## Source: `FINDINGS.md`

# Findings

## P1 — Explain safety bypass

`QueryService::execute` validates the connection policy, but `QueryService::explain`
only rejects multi-statements before calling the connector. PostgreSQL wraps the SQL
in `EXPLAIN (FORMAT JSON)`, and `EXPLAIN ANALYZE` executes the inner statement.

Impact: a read-only connection can execute a write through the Explain path.

Decision: reuse the existing safety classifier before invoking the connector.

## P1 — Zero-row mutation reported as success

The table service returns the connector's raw affected-row count and the runtime maps
any `Ok(_)` result to generic success. A missing/stale primary-key target therefore
looks successful.

Impact: callers can discard staged state after no database row was changed.

Decision: table mutations return `DbError::NotFound` when zero rows are affected. This
keeps callers from treating any zero-row mutation as a successful state change.

## P1 — SQLite timeout not enforced

SQLite execution runs on a dedicated actor thread and the async handle only waits on
a oneshot. Dropping the wait does not stop SQLite's VM.

Impact: a timed-out/cancelled request can keep the actor occupied indefinitely.

Decision: retain the actor model but expose rusqlite's `InterruptHandle`; a timeout
interrupts the VM and returns `DbError::QueryTimeout`.

## P1 — SSH host verification disabled

Both SSH command paths pass `StrictHostKeyChecking=no`.

Impact: SSH tunnels are vulnerable to host impersonation.

Decision: remove the bypass and let OpenSSH enforce its configured known-host policy.

## P1 — PostgreSQL backup bypasses configured SSH tunnel

`BackupService` previously built `PgDumpEngine` from only host, port, database and
username. The persisted `ssh_tunnel` was therefore lost before `pg_dump`, `psql`, or
`pg_restore` ran.

Impact: a backup/restore requested for an SSH-routed connection could connect directly
to the database host, fail unexpectedly, or violate the connection boundary.

Decision: pass the complete `ConnectionConfig` into the engine and keep the tunnel
handle alive for the duration of each external backup/restore command.

## P2 — SSH tunnel readiness race

Tunnel startup previously returned after a fixed sleep for key authentication and
immediately for password authentication. Callers could start database work before
the local forward was listening, while an early SSH process failure was not surfaced.

Decision: require `ExitOnForwardFailure=yes`, poll the local forward with a bounded
deadline, and surface early process exit as a connection error.

## P1 — Primary-key edit conflicts with row identity contract

`TableDataService::update_row` previously accepted a primary-key column in the
updated column list. The caller identifies the row with the original primary key;
when multiple staged edits are applied, a later edit can still target the old key.

Decision: reject primary-key updates at the core boundary until an explicit identity
rewrite/concurrency contract is implemented.

## P2 — External backup command has no timeout

The PostgreSQL backup engine used `Command::output()` without a deadline. A stuck
`pg_dump`, `psql`, or `pg_restore` could keep a runtime operation pending forever.

Decision: reuse the validated connection timeout and kill the child process when the
deadline expires.

## P1 — Failed disconnect loses cleanup handle

`ConnectionService::disconnect` previously unregistered the connection before
calling the connector. If the connector failed to close the pool/actor, the registry
forgot the handle and subsequent cleanup or retry became impossible.

Decision: call the connector first and unregister only after successful resource
release; retain the registry entry on failure.

## P1 — SSH connection test bypasses tunnel

`CompositeConnector::connect` rewrites the database endpoint to the local forwarded
port, but `test_connection` passed the original remote endpoint directly to the
provider connector.

Decision: derive one effective configuration for both connect and test, keeping the
tunnel alive until the test finishes.

## P1 — PostgreSQL operation timeout coverage is incomplete

The PostgreSQL connector applied `query_timeout_ms` to query, execute, batch, and
transaction operations, but `connect`, Test Connection, introspection, and Explain
awaited SQLx futures without a deadline. A blocked database or expensive Explain
could therefore outlive the configured runtime boundary.

Decision: use one timeout wrapper for all PostgreSQL operations that return a core
`DbError`, including pool establishment and the provider metadata/Explain paths.

## P1 — PostgreSQL user-management SQL accepts unsafe interpolated input

Role, schema, and table values were inserted into quoted SQL without escaping
embedded double quotes. The privilege value was inserted as raw SQL syntax. A
crafted request could break out of an identifier or inject additional privilege
clauses/statements.

Decision: quote every identifier at the infrastructure boundary and allow only the
supported PostgreSQL table privileges (ALL, SELECT, INSERT, UPDATE, DELETE,
TRUNCATE, REFERENCES, and TRIGGER).

## P1 — Connection update can leave runtime and secret state inconsistent

ConnectionService::update disconnected an active connection before saving the new
configuration. A repository failure therefore returned an error while losing the
working session. Password updates also stored a new secret without assigning a
missing legacy secret_ref, and a failed save only restored an existing password,
not the previous absence of one.

Decision: snapshot the previous connection, persist the secret/config first, attach
the derived secret reference for legacy records, and compensate secret/config state
when persistence or the post-persist disconnect fails.

## P1 — PostgreSQL transaction timeout can return before rollback

`PostgresConnector::execute_transaction` previously wrapped the whole transaction
future in `tokio::time::timeout`. When the deadline elapsed, dropping SQLx's
`Transaction` only scheduled rollback from `Drop`; it did not await rollback before
returning `TransactionFailure`, despite the `DbConnector` contract requiring that
rollback be complete first. The timeout also discarded the statement index and any
successful results collected before the timeout.

Decision: enforce the transaction deadline per begin/statement operation, explicitly
rollback on timeout, preserve the failing statement index and prior results, and do
not client-cancel COMMIT because that would make the commit outcome ambiguous.

## P1 — Connection deletion can lose a credential before repository deletion

`ConnectionService::delete` deleted the secret first and only then deleted the
connection record. If repository deletion failed, the record remained but its
credential had already been removed, making the saved connection unrecoverable.

Decision: read the existing secret before cleanup, delete the secret, and compensate
by restoring it when repository deletion fails. Secret-store failure still stops the
operation before the connection record is removed.

## P1 — Test connectivity ignores a persisted custom secret reference

`ConnectionService::connect` and update logic resolve `Connection.secret_ref`, but
`test_connectivity_with_secret` always read `connection/{id}/password`. Connections
using a migrated or custom secret key could therefore test with a missing or stale
credential even though normal connect would use another secret.

Decision: resolve the persisted connection's `secret_ref` for empty-password tests,
falling back to the default key only for unsaved/legacy records without a reference.

## P2 — Pagination values could overflow the database integer parameter

`sql_builder::build_select` converted public `u64` limit/offset values to signed
`i64` parameters with a lossy cast. A value above `i64::MAX` became negative and
produced invalid provider SQL or an opaque database error.

Decision: validate both pagination values with a checked conversion and return a
domain validation error before building SQL.

## P1 — Destructive DELETE detection accepts keyword substrings

`is_delete_without_where` used a raw `contains("WHERE")` check. A table named
`somewhere`, a quoted identifier, or a comment containing `WHERE` could therefore
make a DELETE without a predicate appear non-destructive and bypass a policy with
`allow_destructive = false`.

Decision: scan SQL tokens while ignoring quoted strings, quoted identifiers, and
comments, then match `WHERE` as a complete keyword.

## P1 — Duplicate connect cleanup can lose an opened handle

`ConnectionService::connect` atomically kept the first registry handle, but when a
second concurrent connect won a provider handle and its cleanup disconnect failed,
the second handle was dropped without an owner. The active connection remained
usable, while the duplicate provider resource could leak permanently.

Decision: retain failed duplicate handles in a per-connection pending-cleanup queue
and retry them during `disconnect`, including when the primary registry handle is
already absent.

## P1 — SQLite transaction timeout can return before rollback

`SqliteHandle::execute_transaction` interrupted the actor and immediately returned a
synthetic timeout failure while the actor still owned the transaction. The next
command could therefore race transaction cleanup, and the returned failure did not
prove the required rollback contract.

Decision: keep the oneshot receiver after interrupt and wait for the actor's explicit
rollback result. Preserve a known committed result, map an interrupted statement to
`QueryTimeout`, and retain an explicit rollback failure as an internal cleanup error.

## P1 — Export queries bypass the read-only safety policy

`ExportService` rejected multi-statements but sent the remaining SQL directly to
`DbConnector::query` without loading the connection's persisted policy. A query such
as `DELETE ... RETURNING` could therefore mutate a read-only connection through CSV,
JSON, or Excel export.

Decision: resolve the connection policy before export and validate the SQL through
the same core safety classifier used by query and Explain paths.

## P1 — Multi-statement mutation can bypass the transaction path

`QueryService::execute_multi` used the result-routing classifier to decide whether
the script contained a write. PostgreSQL data-modifying CTEs and `EXPLAIN ANALYZE`
can return rows while also mutating data, so they were marked as read and a script
such as `WITH deleted AS (...) SELECT ...; SELECT ...` ran statement-by-statement
instead of through `DbConnector::execute_transaction`.

Impact: a later statement failure could leave an earlier mutation committed, and
the core atomic multi-statement contract was not upheld for these SQL forms.

Decision: use the safety classifier for transaction selection while retaining the
result-routing classifier only to preserve query rows from row-producing mutations.

## P1 — PostgreSQL backup can overwrite an existing artifact

`PgDumpEngine` passed the requested destination directly to `pg_dump -f`. The
command may truncate and replace an existing backup before the core receives a
result, unlike the SQLite backup path which rejects an existing destination.

Impact: a backup operation aimed at the wrong path can destroy the only known copy
of an older recovery artifact.

Decision: reserve the destination with atomic `create_new` before starting the SSH
tunnel or `pg_dump`, and remove the reserved file when the command fails.

## P1 — SQLite backup publish can overwrite a raced destination

`SqliteBackupEngine` checked `dst.exists()` before creating a temporary snapshot,
but published the result with `rename()`. A destination created after that check
could be replaced during publish.

Impact: a concurrent backup or file operation could lose an existing recovery
artifact even though the initial preflight reported a free path.

Decision: publish the tem

… [truncated for memory entry; see source file]


## Source: `VERIFICATION.md`

# Verification

> **[CORRECTED 2026-09-14 — V01-06 evidence audit]** The `PASS` lines below are prose records of
> commands that were run; no command output, log, or capture for the live-PostgreSQL/SSH runs is
> retrievable from the repository, and on HEAD the `pg_integration` suite runs as **0 passed /
> 18 ignored** and the SSH backup test as **0 passed / 1 ignored** without their fixtures. The
> live-fixture claims below are therefore **not re-asserted** and are not part of the v0.1
> evidence set (`docs/release/evidence/v01-06/04-v01-01-05-evidence-audit.md` §5, cross-cutting
> cause 2). The original record is retained, unretracted. State stays `RUNTIME_VERIFY`.

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
  SQLite capability-gate test; `cargo clippy -p db-pro-core --all-targets -- -D

… [truncated for memory entry; see source file]

