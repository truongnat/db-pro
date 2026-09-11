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
