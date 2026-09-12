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
- Multi-statement scripts containing any mutation, including row-producing
  data-modifying CTEs and `EXPLAIN ANALYZE` mutations, use the atomic transaction
  path while preserving query-result routing.
- PostgreSQL backup refuses an existing destination before starting external
  processes and removes incomplete output after a failed command.
- SQLite backup publishes through an atomic no-overwrite operation and preserves a
  destination that appears during snapshot creation.
- Single- and multi-statement SQL validation uses one lexical splitter that preserves
  semicolons inside comments, quoted identifiers, and dollar-quoted bodies.
- PostgreSQL and SQLite batch execution explicitly rolls back on failure and waits
  for rollback completion after timeout interruption.
- Backup and restore resolve the persisted connection secret reference, with a
  default-key fallback only for legacy records that have no reference.
- Connect, Test Connection, and delete use the same default password-key fallback
  for legacy PostgreSQL records without `secret_ref`.
- Test SSH Tunnel uses the same key/password authentication mode as tunnel
  startup without exposing the password in process arguments.
- SSH tunnel passwords are stored separately through `SecretStore`, omitted
  from metadata and debug output, and hydrated only for provider/backup calls;
  create/update/delete lifecycle failures restore both database and SSH secret
  state.
- SQLite backup and restore do not depend on database credential availability;
  PostgreSQL continues to require its resolved secret.
- Successful connection updates and deletes invalidate their connection-scoped
  introspection cache; cache cleanup failure is non-fatal and observable.
- Excel export preserves exact `i64` values by writing values outside the exact
  IEEE-754 integer range as text and rejects row/column index conversions that
  would overflow the workbook API types.
- Table pagination rejects missing, non-integer, or negative count results with
  an explicit core error instead of silently defaulting or wrapping the value.
- Cross-connection data diff rejects negative provider counts and computes the
  signed row-count difference with overflow checks.
- Schema diff returns tables, columns, and indexes in deterministic sorted order.
- SQLite connection create/connect/test flows do not require or persist a
  database secret; PostgreSQL continues to require one, including on provider
  transitions.
- SQLite rejects SSH tunnel configuration at the domain boundary.
- User-management operations are capability-gated to PostgreSQL before invoking
  the provider manager; unsupported SQLite operations return a clear core error.
- Table DDL reconstruction emits executable SQLite foreign-key and index/table
  qualification syntax while preserving the existing PostgreSQL form.
- Table DDL reconstruction preserves introspected CHECK constraints for both
  PostgreSQL and SQLite.
- SQLite introspection extracts each CHECK expression independently while
  ignoring nested parentheses, literals, and comments.
- SQLite trigger introspection matches timing and event keywords outside quoted
  names, literals, comments, and the trigger body.
- PostgreSQL table DDL qualifies reconstructed index names with their source
  schema; SQLite keeps local index names unqualified.
- SQLite table DDL preserves table-level UNIQUE constraints without recreating
  SQLite-owned autoindex names; primary-key autoindexes are not emitted twice.
- PostgreSQL index introspection parses quoted identifiers and nested expressions
  without treating their commas or parentheses as column-list boundaries.
- PostgreSQL introspection preserves intentional NULL metadata defaults while
  propagating row decode/column errors as core errors.
- CTE safety classification ignores comments, quoted tokens, dollar-quoted
  bodies, and nested parentheses while detecting data-modifying CTEs.
- `MERGE` with a DELETE action is classified as destructive while non-delete
  merge actions remain ordinary writes.
- Opaque server-side or dynamically prepared execution statements are classified
  as destructive unless their internals can be inspected safely by the core.
- Query history rejects malformed persisted numeric metadata instead of silently
  replacing it with zero.
- A live isolated PostgreSQL fixture verifies backup and restore through the SSH
  tunnel, including restore into a fresh database and post-restore queryability.
- CI provisions an isolated SSHD/key/known-hosts fixture so the live backup
  verification runs with the same host-key verification contract.
- Metadata migration fails closed on malformed, negative, or future schema
  versions instead of silently applying an incompatible migration set.
- SQLite primary-key metadata decode errors propagate instead of silently
  converting a column to non-primary-key state.
- DML statements with a top-level `RETURNING` clause use the query-result route
  in single and atomic multi-statement execution, while nested CTE `RETURNING`
  clauses do not falsely mark the outer mutation as row-producing.
- Transactional query results pass the same `QueryResult::validate` boundary as
  direct query results, including partial results attached to rollback errors.
- SSH Test Tunnel applies a process-level deadline and kills the child process
  when the deadline expires.
- Export, table-data, and data-diff application services validate connector
  `QueryResult` payloads before consuming them.
- Schema diff compares schema/object identity structurally, without collisions
  from dots inside either identifier.
- Pagination and data-diff count consumers require exactly one validated scalar
  count row from the provider.
- Connection-name validation enforces its documented limit by Unicode character
  count rather than UTF-8 byte length.
- `QueryResult::validate` rejects mismatched returned-row counts while retaining
  the affected-row representation with no columns/rows.
- JSON export rejects duplicate column names instead of silently dropping values;
  CSV and Excel continue to preserve positional duplicate columns.
- `QueryResult::validate` rejects returned rows without columns while preserving
  affected-row results that contain neither columns nor returned rows.
- `QueryExecution` ignores non-terminal finish statuses and late success calls
  after leaving the `Running` state.
- `CellValue` temporal and network values preserve typed parameter semantics:
  PostgreSQL binds `TIME`/`TIMETZ`, `INTERVAL`, and `INET`/`CIDR` natively;
  SQLite binds the same values as explicit text because it has no equivalent
  native storage type.
- Query cancellation is an explicit provider contract: SQLite advertises
  cancellation, interrupts the active VM, and acknowledges actor recovery
  before reporting cancellation; PostgreSQL does not advertise cancellation
  until a provider-safe primitive exists.
- DDL submitted through QueryService invalidates the shared schema cache after
  success and after an unknown transaction outcome; confirmed rollbacks retain
  the existing cache.
- Create, update, and both connection-test paths share the same connection
  configuration validation boundary.
- PostgreSQL and SQLite unit/integration coverage is updated independently where the
  behavior differs.
