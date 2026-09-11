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
- No claim of live PostgreSQL or native UI verification in this wave.

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
- PostgreSQL and SQLite unit/integration coverage is updated independently where the
  behavior differs.
