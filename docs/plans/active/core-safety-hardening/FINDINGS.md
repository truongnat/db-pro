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
