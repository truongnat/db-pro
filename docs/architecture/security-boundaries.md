# Security Boundaries

> Status: **Implemented** (P2-08, P2-09)

## Credential Boundary

### Separation

```text
ConnectionConfig (metadata, non-secret)
  name, host, port, database, username, driver, ssl_mode, ssh_tunnel config

ConnectionSecret (credentials, secret)
  password, ssh_password, ssh_passphrase
```

### Source

- Domain model: `crates/core/src/domain/secret.rs`
- Secret store trait: `crates/core/src/ports/secret_store.rs`
- Keyring implementation: `crates/infrastructure/src/secret/keyring_vault.rs`
- Encrypted fallback: `crates/infrastructure/src/secret/fallback.rs`

### Rules

Credentials must **never** appear in:

1. Workspace persistence (SQLite meta store)
2. Recent connection lists
3. Logs or tracing output
4. Diagnostics bundles
5. Frontend localStorage
6. Error messages sent to frontend

### Storage

- Primary: OS keychain via `keyring` crate — selected in every build
  (`KEYRING_SERVICE = "com.dbpro.app"`, `crates/runtime/src/lib.rs:62`)
- Session fallback: in-memory only, enabled in every build
  (`crates/runtime/src/lib.rs:100`); a password that only ever reached this store must be
  re-entered after a restart
- Encrypted-file fallback: AES-256-GCM `secrets.json`. **Development/CI only** — the shipping
  wiring enables it for debug builds or when `DB_PRO_ALLOW_FILE_SECRET_FALLBACK` is `1`/`true`
  (`crates/runtime/src/lib.rs:83`, `:99`), and a release build that did not opt in never
  consults the file at all (`crates/infrastructure/src/secret/keyring_vault.rs:189`) (#142)
- Reference: `Connection.secret_ref` stores only the keyring key, not the password

### Redaction

`ConnectionSecret::redacted()` returns a `RedactedSecret` that only reveals whether
credentials exist (`has_password: bool`), never their values.

## Safety Policy

### Source

- Domain model: `crates/core/src/domain/safety.rs`

### Policy Model

```rust
ConnectionSafetyPolicy {
    read_only: bool,
    allow_ddl: bool,
    allow_destructive: bool,
    max_rows: Option<u64>,
    query_timeout_ms: Option<u64>,
}
```

### Enforcement

Backend-enforced. Frontend UI controls are convenience only.

```text
connection.readOnly = true
frontend bug sends: DELETE FROM users
backend → reject → READ_ONLY_VIOLATION
```

### Statement Classifier

| Category | Keywords |
|----------|----------|
| Read | SELECT, SHOW, EXPLAIN, TABLE, WITH...SELECT |
| Write | INSERT, UPDATE, DELETE, WITH...INSERT/UPDATE/DELETE |
| Ddl | CREATE, ALTER, DROP, TRUNCATE, RENAME |
| Destructive | DROP, TRUNCATE, DELETE (without WHERE) |

### Tauri Permissions

Current capability (`capabilities/default.json`):
- `core:default` only
- No filesystem scope expansion
- No shell access
- CSP restricts `default-src` to `'self'`

## Metadata Safety

Database metadata is treated as **untrusted input**:

- SQLite introspection uses parameterized queries (no string interpolation)
- PostgreSQL introspection uses `try_get().unwrap_or_default()` for nullable fields
- All identifiers (table names, column names) are handled as opaque strings
