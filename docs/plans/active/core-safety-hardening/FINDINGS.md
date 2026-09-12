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

Decision: publish the temporary snapshot with an atomic no-overwrite hard-link and
only remove the temporary name after the destination link succeeds.

## P2 — SQL statement boundary handling misses PostgreSQL lexical forms

`reject_multi_statement` and `split_statements` only protected single-quoted
strings. Semicolons inside quoted identifiers, comments, or PostgreSQL dollar-quoted
function/DO bodies were therefore treated as statement separators. This could reject
valid single statements or split a function body into unrelated commands before
transaction classification.

Decision: use one byte-aware splitter that skips single/double quotes, nested block
comments, line comments, and dollar-quoted bodies, and reuse it for single-statement
validation.

## P1 — Batch execution can return before rollback

`PostgresConnector::execute_batch` previously wrapped the complete transaction in
`tokio::time::timeout`. A timeout dropped the SQLx transaction without awaiting
rollback, so the method could report failure while the connection resource was
still cleaning up. SQLite had the same race in the opposite direction: its actor
continued rolling back after the async caller had already returned a timeout.

Impact: a following operation could race transaction cleanup, and the batch API
did not prove its atomicity contract after a timeout or statement failure.

Decision: apply the deadline per PostgreSQL batch operation, explicitly rollback
on timeout/error/row-count overflow, and wait for SQLite's actor response after
interrupting it. Preserve an unknown PostgreSQL commit outcome instead of
client-cancelling `COMMIT`.

## P1 — Backup and restore ignore a persisted custom secret reference

`BackupService` loaded only `ConnectionConfig` and reconstructed the default
`connection/{id}/password` key. The full `Connection` record stores
`secret_ref`, which is used by normal connect and can point to a migrated or
custom credential.

Impact: a connection could connect successfully while PostgreSQL backup or
restore failed authentication or used a stale credential.

Decision: load the full connection record for backup and restore, resolve its
persisted `secret_ref`, and retain the default key only as a legacy fallback.

## P2 — SSH password authentication is ignored by Test SSH Tunnel

`SshTunnel::start` invokes `sshpass -e ssh` when the tunnel password is present,
but `SshTunnel::test` always invokes `ssh` directly and never supplies that
password. The connectivity test therefore exercises a different authentication
path from the real tunnel startup.

Impact: valid password-authenticated tunnels can be reported as failed and the
user cannot verify the same SSH configuration that connect/backup will use.

Decision: share the authentication-mode selection with the test command and keep
the password in `SSHPASS`, outside process arguments.

## P1 — SSH tunnel passwords can be persisted in connection metadata

`SshTunnelConfig.password` was embedded in `ConnectionConfig`, and the metadata
repository serializes the complete `Connection` record to `meta.db`. A saved
connection could therefore contain the SSH password in plaintext, while the
derived `Debug` implementation could also expose it in diagnostics or logs.

Impact: a local metadata copy, API serialization path, or debug log could
disclose credentials used to reach the database network.

Decision: keep legacy metadata readable but make the password write-only for
serialization, redact it from `Debug`, store it under the connection's
`SecretStore` SSH key, hydrate it only for connect/test/backup/restore, and
compensate secret changes on lifecycle failures.

## P2 — SQLite backup depends on an irrelevant database secret

`BackupService::password_for` retrieved a database password for both providers,
although `SqliteBackupEngine` ignores the password entirely. When the OS keyring
was unavailable, a SQLite backup could fail before the engine was invoked even
though the database file itself was accessible.

Decision: return an empty credential for SQLite backup/restore and keep secret
resolution mandatory only for PostgreSQL.

## P1 — Connection updates can serve stale introspection metadata

`SchemaService` caches `IntrospectResult` by `ConnectionId`. `ConnectionService`
can update the host, database, or other connection target without touching that
cache. A later non-forced introspection can therefore return tables and columns
from the previous target, even though the active connection now points elsewhere.

Decision: invalidate the connection-scoped introspection cache after a successful
connection update or delete, with cache failure logged as non-fatal because the
next introspection can rebuild the cache.

## P1 — Excel export can corrupt large BIGINT values

`ExportService::export_excel` converted every `CellValue::Int64` to `f64` before
writing the workbook. IEEE-754 doubles cannot represent every `i64` exactly above
2^53, so a valid BIGINT such as `9007199254740993` could be exported as a
different number. The same loop also used unchecked `usize` to `u16`/`u32`
casts for worksheet coordinates.

Impact: an export presented as a recovery/reporting artifact can silently change
database values, and sufficiently wide or tall results can wrap their target
coordinates before the spreadsheet writer sees them.

Decision: keep integers as numeric Excel cells within the exact `f64` integer
range and write larger values as text, preserving their decimal representation;
use checked coordinate conversions and return a validation error on overflow.

## P2 — Table pagination silently wraps invalid count results

`TableDataService::fetch_rows` accepted only an `Int64` count, but used `as u64`
and then defaulted every missing, null, or wrong-typed result to `0`. A negative
or malformed provider response could therefore become either a huge page count
or a misleading empty result without an error.

Impact: pagination metadata can drive incorrect page navigation and hide a
provider/count-query failure from the caller.

Decision: parse the count through `u64::try_from` and return an explicit core
error for missing, non-integer, or negative values.

## P2 — Cross-connection data diff trusts invalid row counts

`DataDiffService` extracted `COUNT(*)` as a signed `i64` without rejecting a
negative provider value, then computed `source_count - target_count` directly.
Real database counts are non-negative, but a malformed adapter result could
produce an invalid diff or arithmetic overflow before the result reached the
cross-connection API.

Impact: comparison output can report impossible row counts or fail through an
unchecked arithmetic panic in debug/test builds.

Decision: reject negative counts at extraction and use checked arithmetic for
the signed difference, preserving the existing domain/API type.

## P2 — Schema diff output order is nondeterministic

`SchemaDiff` built its missing/common table, column, and index collections from
`HashSet` differences and intersections. Hash iteration order is not a stable
output contract, so identical schemas could produce differently ordered diff
payloads between process runs.

Impact: the comparison report and any consumer that renders or snapshots it can
flicker, creating noisy reviews and making real changes harder to identify.

Decision: use `BTreeSet` for set-based comparison so all emitted names and the
common-table traversal are deterministically sorted.

## P1 — SQLite connection lifecycle incorrectly requires a database secret

`ConnectionService::create`, `connect`, and `test_connectivity_with_secret`
treated every driver like PostgreSQL. A SQLite database file does not use a
database password, but the lifecycle still wrote/read `secret_ref` and failed
when the OS keyring was unavailable.

Impact: a valid local SQLite connection could not be created or opened in a
keyring-unavailable environment, and provider state contained an unnecessary
credential dependency.

Decision: require database secrets only for PostgreSQL, keep SQLite records free
of database `secret_ref`, remove the old secret when changing PostgreSQL to
SQLite, and require a new password when changing SQLite to PostgreSQL.

## P2 — SQLite accepts unusable SSH tunnel configuration

`ConnectionConfig::validate` validated SSH fields for SQLite even though the
SQLite provider has no remote endpoint or SSH tunnel implementation. Such a
record could be persisted with credentials that the provider would silently
ignore.

Impact: the connection configuration advertised a path that could not be used,
and callers could mistakenly assume the local SQLite file was tunnel-protected.

Decision: reject SSH tunnel configuration for SQLite at the core domain boundary.

## P1 — User management is not capability-gated for SQLite

`UserService` checked only the persisted readonly flag before forwarding role
and privilege operations to `UserManager`. The capability model marks SQLite as
having no server sessions, but the application boundary did not enforce that
contract.

Impact: a SQLite connection could enter a PostgreSQL-oriented role/session path,
producing a misleading provider error or allowing future adapters to execute an
operation the provider does not support.

Decision: require PostgreSQL server-session capability before every user list,
role, or privilege operation and return `DbError::Unsupported` before the
provider manager is called.

## P1 — Reconstructed table DDL is not executable on SQLite

`SchemaService::get_table_ddl` generated foreign keys as PostgreSQL-only
`ALTER TABLE ... ADD CONSTRAINT` statements and qualified SQLite table/index
names with `"main".`. SQLite requires foreign keys inside `CREATE TABLE` and
does not accept those qualified forms in the generated index statement.

Impact: applying or reusing the DDL shown for a SQLite table fails, so the
schema reconstruction path cannot safely reproduce a table with relationships
and indexes.

Decision: make table DDL generation provider-aware. SQLite embeds named foreign
keys in `CREATE TABLE` and uses unqualified local object names; PostgreSQL keeps
the existing post-create constraint statements.

## P1 — Reconstructed table DDL drops CHECK constraints

Both providers populate `IntrospectResult::check_constraints`, but
`SchemaService::get_table_ddl` did not select or emit those constraints.

Impact: applying the generated table script could recreate a table without its
original validation rules, allowing rows that the source database rejected.

Decision: select constraints belonging to the requested table and emit them as
named inline `CONSTRAINT ... CHECK ...` definitions in the reconstructed table.

## P1 — Reconstructed PostgreSQL indexes depend on `search_path`

The generated index statement qualified the table but left the index name
unqualified. PostgreSQL therefore creates the index in the active session's
schema/search path instead of necessarily preserving the source index schema.

Impact: replaying DDL for a table in a non-default schema can create the index
in the wrong namespace or fail on a name collision, leaving the reconstructed
schema inconsistent.

Decision: qualify PostgreSQL index names with the introspected index schema;
SQLite continues to use local unqualified names required by its syntax.

## P1 — Connection test paths bypass provider validation

`ConnectionService::create` and `update` validate `ConnectionConfig`, but
`test_connectivity` and `test_connectivity_with_secret` forwarded drafts
directly to the connector. This allowed a SQLite draft with an SSH tunnel to be
silently handled by a provider that cannot support SSH tunneling.

Impact: the UI could report a successful test for a configuration that cannot
be saved or connected using the same semantics, hiding a provider mismatch at
the connection boundary.

Decision: centralize configuration validation and require it in create, update,
and both test-connectivity paths before any secret hydration or connector call.

## P1 — EXPLAIN safety classification uses substring matching

`classify_explain_safety` previously searched the entire SQL string for
`ANALYZE`. A plain `EXPLAIN` containing a literal or identifier with that text
could therefore be treated as `EXPLAIN ANALYZE`; the uppercase-then-slice
implementation could also calculate an invalid byte offset for Unicode text
and panic inside the safety gate.

Decision: parse `ANALYZE` as an option keyword in the original SQL, support
PostgreSQL's parenthesized options and comments between keywords, and classify
only the actual inner statement when execution is enabled.

## P1 — CTE DELETE safety check uses substring matching

The `WITH` classifier used `remaining.contains("WHERE")` for its outer
`DELETE`. A comment or literal containing `WHERE` could make a DELETE without
an actual predicate look non-destructive, bypassing a policy that allows
writes but forbids destructive operations.

Decision: reuse the token-aware DELETE predicate check used by ordinary
statements, ignoring comments, quoted identifiers, and string literals.

## P1 — Destructive DELETE inside a CTE is downgraded to Write

`classify_cte_safety` treated every data-modifying CTE as `Write`, including
`WITH deleted AS (DELETE FROM users RETURNING ...) SELECT ...`. A policy that
allows writes but forbids destructive operations could therefore permit a
DELETE without a predicate when it was wrapped in a CTE.

Decision: retain the destructive classification of DELETE bodies inside CTEs
while preserving row-producing CTE routing as a query result.

## P1 — Legacy PostgreSQL secret fallback is inconsistent across lifecycle paths

`test_connectivity_with_secret` already falls back to the default
`connection/{id}/password` key when a legacy PostgreSQL record has no
`secret_ref`, but `ConnectionService::connect` previously rejected the same
record. `delete` also skipped secret cleanup when `secret_ref` was absent.

Impact: a migrated connection could pass Test Connection but fail to open in
the application, and deleting it could leave its default database credential
orphaned in the secret store.

Decision: use the persisted `secret_ref` when present and the default key for
legacy PostgreSQL records across connect and delete, while retaining SQLite's
no-database-secret behavior.

## P1 — SQLite CHECK introspection merges independent constraints

`sqlite::introspect::introspect_check_constraints` tracked the parenthesis depth
of the entire `CREATE TABLE` statement and only closed a CHECK when that outer
depth returned to zero. A table with multiple CHECK clauses, or a nested CHECK
expression, therefore produced one merged definition instead of one definition
per constraint.

Impact: `SchemaService::get_table_ddl` could emit malformed or semantically
incorrect reconstructed SQLite DDL, losing the source table's validation
invariants when the DDL was reused.

Decision: scan CHECK keywords outside strings, quoted identifiers, and comments,
then match each expression's own parenthesis pair with nested-expression support.

## P2 — SQLite trigger metadata uses substring matching

`parse_sqlite_trigger_sql` searched the trigger header with `contains()` for
`BEFORE`, `AFTER`, `INSERT`, `UPDATE`, and `DELETE`. A quoted trigger or table
name containing one of those words could therefore override the actual timing
or event clause.

Impact: core introspection returned incorrect trigger metadata, which could make
schema inspection and trigger presentation misleading for valid SQLite names.

Decision: reuse the lexical keyword scanner for the header and ignore quoted
identifiers, literals, comments, and the trigger body before matching clauses.

## P1 — SQLite DDL reconstruction replays internal UNIQUE autoindexes

SQLite reports table-level `UNIQUE` constraints through `PRAGMA index_list` as
internal `sqlite_autoindex_*` indexes. Treating those rows as ordinary indexes
made reconstructed DDL emit `CREATE UNIQUE INDEX` for a name owned by SQLite;
the script then failed instead of reproducing the source constraint.

Impact: copying or replaying a valid SQLite table definition could fail, or a
filter-only fix could silently drop uniqueness from the reconstructed schema.

Decision: preserve the index origin in the core schema model, omit primary-key
autoindexes because the table primary-key definition already recreates them,
render UNIQUE-constraint indexes as inline `UNIQUE (...)` definitions, and emit
`CREATE INDEX` only for user-created indexes.

## P1 — PostgreSQL index introspection splits quoted index keys

`parse_index_columns` used parenthesis depth and a raw comma split. PostgreSQL
allows quoted table and column identifiers containing parentheses or commas, so
valid definitions such as an index on `"a,b"` were returned as two malformed
column names.

Impact: schema metadata was incorrect and reconstructed PostgreSQL index DDL
could target the wrong columns or fail to execute for valid special identifiers.

Decision: find the index key list and its closing parenthesis while ignoring
quoted identifiers and literals, then split only on commas outside quotes and
nested expressions.

## P1 — PostgreSQL introspection swallows metadata decode errors

Several PostgreSQL metadata fields used `try_get(...).unwrap_or_default()` (or
an equivalent fallback). A NULL value and a real decode/column error were both
turned into an empty string, so the introspection result could contain an object
with missing schema, timing, definition, or function metadata while reporting
success. Live fixture verification also exposed that `pg_trigger.tgenabled` was
returned as PostgreSQL `CHAR`, which the old fallback hid and the new fallible
decode correctly surfaced.

Impact: callers may cache or render corrupted schema metadata and later produce
incorrect DDL without seeing the provider error that caused it.

Decision: decode metadata through fallible helpers, use `Option<String>` only for
columns that are intentionally nullable, cast the one-character trigger state
to text at the SQL boundary, and propagate all other row errors as `DbError`.

## P1 — CTE mutation detection misses comment-separated keywords

The CTE safety classifier searched character slices and required whitespace
after `INSERT`, `UPDATE`, or `DELETE`. PostgreSQL permits comments between
tokens, so `DELETE/* comment */FROM ...` was not recognized as a data-modifying
CTE; an outer `SELECT` could then be classified as read-only.

Impact: a mutating CTE could bypass the read-only policy, and a destructive
DELETE inside the CTE could bypass the destructive-operation policy.

Decision: tokenize CTE structure while skipping comments, quoted identifiers,
string literals, and dollar-quoted bodies; inspect mutation and predicate
keywords at their actual parenthesis depth, with conservative destructive
classification when the CTE structure cannot be parsed.

## P1 — MERGE DELETE actions bypass destructive policy

The safety classifier had no explicit `MERGE` branch, so every merge fell
through to generic `Write`. PostgreSQL and SQL Server permit a `WHEN MATCHED
THEN DELETE` action inside a merge, which is destructive even when the outer
statement does not contain a standalone DELETE command.

Impact: a connection configured to allow writes but forbid destructive
operations could still execute a merge that deletes matched rows.

Decision: classify `MERGE` as `Destructive` when a real DELETE keyword appears
outside literals, identifiers, comments, and dollar-quoted bodies; retain
`Write` for merge statements without a DELETE action.

## P1 — Opaque server-side execution bypasses destructive policy

`DO`, `CALL`, and top-level `EXECUTE` statements can run procedural or prepared
server-side logic whose mutations are not visible to the client-side statement
classifier. They previously fell through to generic `Write`, so a connection
that allowed ordinary writes but prohibited destructive operations could still
execute hidden `DELETE`, `TRUNCATE`, or dynamic DDL logic.

Decision: classify these opaque execution forms as `Destructive` conservatively.
The core cannot prove their internals are non-destructive without provider-aware
parsing and routine metadata, so restricted connections must require explicit
destructive-operation permission.

## P2 — Query history masks malformed numeric metadata

The query-history repository parsed persisted `duration_ms` and `row_count`
values with `unwrap_or(0)`. A damaged or manually edited `meta.db` row therefore
looked like a valid zero-duration, zero-row query instead of surfacing the
corrupt metadata to the caller.

Decision: parse both metrics as unsigned integers and return a named internal
error when either value is invalid, including negative or fractional text.

## P2 — Metadata migration masks invalid or future schema versions

`current_version()` converted any parse failure to `0`, so a damaged
`schema_version` row could make the application attempt migrations from the
beginning. It also allowed a database with a version newer than the binary to
continue startup without an explicit compatibility error.

Decision: parse the persisted version as an unsigned integer, return an internal
error for missing or malformed values, and reject versions above
`LATEST_VERSION` before applying migrations.

## P2 — SQLite primary-key metadata decode errors are swallowed

SQLite column introspection used `unwrap_or(0)` when decoding the `pk` field from
`PRAGMA table_info`. A provider metadata decode failure therefore changed a
primary-key column into an ordinary column, which can later make table editing
lose its row identity contract.

Decision: use the rusqlite row error path directly so malformed primary-key
metadata aborts the introspection rather than returning an incomplete schema.

## P1 — Multi-statement routing drops rows from DML `RETURNING`

`QueryService::execute_multi` used the first keyword to decide whether each
statement should go through `query` or `execute`. `INSERT`, `UPDATE`, and
`DELETE` statements with a top-level `RETURNING` clause were therefore sent
through the affected-row path, which discarded the returned rows. The same
problem applied to a row-producing mutation inside an atomic multi-statement
transaction.

Impact: a successful mutation can appear to have no result set, making query
output, history, and follow-up automation lose data that the database returned.

Decision: classify top-level `RETURNING` mutations as query-routed while still
keeping `RETURNING` inside a CTE body out of the outer statement's result-route
decision. The mutation remains safety-classified as a write/destructive
operation and still uses the transaction path when the script contains
multiple statements.

## P2 — Transaction query results bypass the core shape invariant

`QueryService::execute` and the non-transaction branch of
`execute_multi` validate every `QueryResult`, but the transaction branch
converted `TransactionStatementResult::Query` values directly. A provider
response with a different number of cells than its column metadata could
therefore cross the core boundary without the invariant check.

Impact: malformed provider data can reach result consumers and cause
inconsistent rendering, serialization, or downstream row processing.

Decision: validate transaction query results before returning them, for both
successful transactions and the partial results attached to a rolled-back
failure. A malformed result becomes an indexed multi-query error and is not
returned as usable data.

## P2 — SSH Test Tunnel has no process-level timeout

`SshTunnel::test` set OpenSSH's `ConnectTimeout`, but awaited
`ssh/sshpass.output()` without an application deadline. A connection can
complete and still leave the remote command or process teardown waiting
indefinitely, so the core request has no bounded completion path.

Impact: Test SSH Tunnel can hang a runtime operation indefinitely and retain a
child process after the network path stops responding.

Decision: wrap the test command in a bounded timeout and enable
`kill_on_drop`, returning a typed connection-timeout error when the deadline
expires.

## P2 — Core result consumers bypass `QueryResult` shape validation

`QueryService` validates direct and transactional query results, but
`ExportService`, `TableDataService`, and `DataDiffService` consumed connector
results without calling the same row/column shape invariant. A malformed
provider response could therefore be exported, used by the data grid, or
interpreted as a count before the core rejected it.

Impact: invalid result payloads can produce misleading artifacts or inconsistent
downstream state instead of one explicit core error.

Decision: validate every connector `QueryResult` at these application-service
boundaries before export, pagination, or diff processing.

## P2 — Schema diff flattens dotted identifiers into ambiguous keys

`SchemaDiff` built internal set keys by concatenating `schema` and object name
with a dot, then recovered the pair with `split_once('.')`. PostgreSQL and
SQLite both permit quoted identifiers containing dots, so a schema named
`tenant.prod` with table `orders` collides with schema `tenant` and table
`prod.orders`; a dotted schema can also be split into the wrong pair for
column comparison.

Impact: schema diff can omit real table changes or attach column/type changes to
the wrong object.

Decision: keep schema and object names as a typed tuple for comparison and only
flatten them at the final display boundary, quoting dotted/quoted parts so the
result remains distinguishable to callers.

## P2 — Count consumers accept non-scalar provider results

`TableDataService::parse_total_count` and `DataDiffService::extract_count`
selected the first cell without checking that the provider returned the one-row,
one-column scalar shape promised by `SELECT COUNT(*)`. Extra rows or columns
could therefore be silently ignored and a wrong count could drive pagination or
cross-connection comparison.

Impact: malformed provider responses can produce an apparently valid but wrong
row count.

Decision: require one column, one row, and `row_count == 1` before parsing the
count value; reject all other shapes explicitly.

## P2 — Connection-name limit counts bytes instead of characters

`ConnectionConfig::validate` compared `String::len()` with a limit documented
and reported as characters. UTF-8 names such as a 128-character Vietnamese or
accented name therefore exceeded the byte count and were rejected despite
being within the configured limit.

Impact: valid non-ASCII connection names fail validation inconsistently with the
public contract.

Decision: count Unicode scalar values with `chars().count()` and cover the exact
128/129-character boundary.

## P2 — Query result validation accepts inconsistent row counts

`QueryResult::validate` verified each row's cell count but did not compare
`row_count` with the number of returned rows. A malformed provider result could
therefore carry valid-looking cells and an incorrect count into history, export,
pagination, or execution metrics.

Impact: downstream consumers can report a count different from the actual
returned payload.

Decision: require matching counts whenever the result has columns, while
preserving the existing affected-row representation (`columns == []`, `rows ==
[]`, `row_count > 0`).

## P1 — JSON export silently overwrites duplicate column names

`ExportService::export_json` represents each row as a JSON object and inserts
values by `ColumnMeta.name`. SQL results can legally contain duplicate output
names, such as `SELECT 1 AS value, 2 AS value`; inserting both values into a
map silently keeps only the last one.

Impact: a successful JSON export can contain incomplete, corrupted row data
without warning, while the positional CSV and Excel formats still retain both
columns.

Decision: reject JSON export when column names are not unique and return an
explicit validation error. Keep CSV and Excel positional output unchanged.

## P1 — Query results can contain rows without column metadata

`QueryResult::validate` checked each row against `columns.len()`, but treated
an empty column list as a valid affected-row shape even when `rows` was
non-empty. A malformed provider result with zero columns and one empty row
therefore crossed the boundary; JSON export serialized it as `{}` and lost the
row's payload.

Impact: downstream consumers can silently discard returned data instead of
rejecting an invalid provider response.

Decision: reject any result with returned rows but no columns. Preserve the
valid affected-row representation with no columns and no returned rows.

## P2 — Query execution lifecycle accepts invalid late transitions

`QueryExecution::finish` relied on `debug_assert!` for a non-terminal target,
so a release build could move an execution back to `Created` or `Running` and
still stamp `finished_at`. `succeed` also wrote result metrics before calling
the idempotent `finish`, allowing a late success callback after timeout/error to
overwrite the terminal execution's metrics.

Impact: execution status and metrics can disagree, making cancellation,
timeout, and result reporting unreliable.

Decision: make invalid finish statuses and success calls outside `Running`
no-ops. Preserve the existing idempotent terminal behavior.

## P1 — Transaction commit failures are reported as rolled back

`QueryService` formats every `TransactionFailure` as `transaction rolled back`.
The PostgreSQL and SQLite connectors explicitly preserve the commit outcome by
not cancelling `COMMIT`, but return a `TransactionFailure` when `commit` itself
fails. The failure currently has `statement_index == statements.len()` and no
rollback confirmation, so the application can report a rollback that was never
verified and point at a non-existent statement.

Impact: after a commit transport or server error, the user can believe changes
were rolled back while the final database state is unknown. The reported
statement index is also not a valid failing statement index, which can mislead
selection and diagnostics.

Decision: extend the core transaction-failure contract with an explicit phase
and rollback/outcome status. Preserve confirmed rollback for statement failures;
represent commit failures as an unknown final outcome instead of claiming
atomic rollback. PostgreSQL and SQLite live coverage now pass; the remaining
plan-level pending item is the separate CI SSH workflow execution.

## P1 — JSON export silently converts non-finite floats to null

`ExportService::cell_to_json` previously used `Number::from_f64(...).unwrap_or
(Null)`. PostgreSQL can return IEEE non-finite `FLOAT8` values such as `NaN`,
which are not valid JSON numbers; the fallback therefore made a successful
export silently lose the original value.

Impact: exported JSON can contain a plausible `null` while the database value
was non-null, causing data corruption in a supposedly lossless export path.

Decision: reject non-finite floats with an explicit validation error. Keep the
existing CSV/Excel behavior unchanged because their serialization contracts are
separate and this fix is scoped to JSON's strict number model.

## P1 — Date cells are downgraded to text query parameters

`sql_builder::cell_to_param` converted `CellValue::Date` to
`QueryParam::Text`. The PostgreSQL parameter binder already has a typed
date-only path under `QueryParam::DateTime`, parsing `YYYY-MM-DD` as
`NaiveDate`; the text mapping bypassed that path for table filters and row
mutations on `DATE` columns.

Impact: PostgreSQL DATE comparisons or mutations can fail with an operator/type
error instead of executing the requested row operation. SQLite remains
compatible because its date representation is text-backed.

Decision: map date cells to the existing typed date-capable parameter variant
and add a builder regression test. Runtime PostgreSQL execution remains covered
by the provider integration gate when a fixture is available.

## P2 — Empty row mutations generate invalid SQL

`build_insert` and `build_update` accepted empty column/value lists because the
lengths matched. They then generated `INSERT ... () VALUES ()` or `UPDATE ...
SET  WHERE ...`, delegating a deterministic input error to the provider.

Impact: malformed editor payloads produce opaque SQL errors instead of a clear
core validation error; an all-default insert is not supported by this builder
API and must not be implied by an invalid statement.

Decision: reject empty insert/update column sets at the core SQL-builder
boundary. Add explicit validation regressions; a future `DEFAULT VALUES` flow
would be a separate feature with its own provider matrix.

## P1 — PostgreSQL known-type decode errors are silently coerced

`postgres::query_mapper::decode_cell` previously used a fallback for every
known type. Binary `TIME`, `INTERVAL`, and `INET` values could therefore be
returned as raw text or an `<unsupported value: TYPE>` placeholder while the
query still succeeded. A subsequent strict decode pass also showed that custom
PostgreSQL enum values cannot be decoded by asking SQLx for `String` directly.

Impact: query results can look valid while losing native type/value fidelity;
downstream table editing and export consumers then operate on corrupted or
misleading cells.

Decision: decode PostgreSQL temporal and network values through their native
SQLx/raw representations, preserve textual custom types such as enums, and
propagate known-type decode errors instead of inventing placeholder values.
Add live temporal/network and full PostgreSQL integration coverage.

## P1 — Temporal and network cells are downgraded to text parameters

`sql_builder::cell_to_param` converted `CellValue::Time`, `Interval`, and
`Inet` to `QueryParam::Text`. PostgreSQL therefore received a `TEXT` parameter
for comparisons against native `TIME`, `INTERVAL`, or `INET`/`CIDR` columns;
the provider cannot reliably resolve those operators without a caller-supplied
cast, and row mutations used the same broken path.

Impact: valid values read from PostgreSQL can fail when used in table filters or
updates, while a text-backed SQLite path hides the provider-specific contract
gap.

Decision: add serialized `QueryParam::Time`, `Interval`, and `Inet` variants;
bind them to native PostgreSQL types with strict parsing, and intentionally
bind them as text for SQLite. Cover both providers and verify PostgreSQL
placeholder type inference without explicit SQL casts.
