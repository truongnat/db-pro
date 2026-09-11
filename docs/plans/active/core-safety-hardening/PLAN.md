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
- No claim of native UI runtime or live SSH-tunnel verification in this wave.

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
- PostgreSQL and SQLite unit/integration coverage is updated independently where the
  behavior differs.
