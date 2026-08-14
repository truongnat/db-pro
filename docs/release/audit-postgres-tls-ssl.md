# PostgreSQL TLS/SSL Mode Audit

> Baseline SHA: `8d6aa54`
> Issue: #125
> Source: `crates/core/src/domain/connection.rs`, `crates/infrastructure/src/postgres/connection_string.rs`

## TLS/SSL mode matrix

| UI value | Domain config | SQLx setting | Encrypts? | Verifies cert? | Verifies hostname? | Release status |
|---|---|---|---|---|---|---|
| `disable` (default) | `SslMode::Disable` | `PgSslMode::Disable` | No | No | No | Supported |
| `require` | `SslMode::Require` | `PgSslMode::Require` | Yes | No | No | Supported |
| `verify-ca` | `SslMode::VerifyCa` | `PgSslMode::VerifyCa` | Yes | Yes (CA) | No | Supported |
| `verify-full` | `SslMode::VerifyFull` | `PgSslMode::VerifyFull` | Yes | Yes (CA) | Yes | Supported |

## Findings

### F1 [P1] — Default SSL mode is `Disable`

The `SslMode` enum defaults to `Disable` (`#[default]` on `Disable` variant). This means connections to remote PostgreSQL servers send credentials and data in plaintext unless the user explicitly selects a TLS mode.

**Impact**: Users connecting to cloud databases (AWS RDS, Supabase, Neon, etc.) may unknowingly transmit passwords and query data over unencrypted connections.

**Recommendation**: ACCEPT RC1 with documentation. The default is intentional for local development (most common v0.1 use case). The connection dialog should warn users when connecting to non-localhost hosts with `Disable` mode. Add a UX warning in a future release.

### F2 [P2] — No custom CA certificate support

The `connection_string.rs` `build_options` function does not call `.ssl_root_cert()` on `PgConnectOptions`. Users cannot provide a custom CA certificate for self-signed or internal PKI servers.

**Impact**: Users with self-signed certificates or private CAs cannot use `VerifyCa` or `VerifyFull` modes. They must fall back to `Require` (no cert verification) or `Disable`.

**Recommendation**: ACCEPT RC1. Document as known limitation. Add custom CA support in a future release.

### F3 [P2] — No client certificate/key support

The `build_options` function does not call `.ssl_client_cert()` or `.ssl_client_key()`. Certificate-based authentication (mutual TLS) is not supported.

**Impact**: Users who rely on client certificate authentication cannot use v0.1. They must use password authentication.

**Recommendation**: ACCEPT RC1. Document as known limitation.

### F4 [P2] — No SSL mode validation for SQLite

SQLite connections don't use TLS (local file). The `ssl_mode` field is present in `ConnectionConfig` but irrelevant for SQLite. No validation prevents setting SSL mode for SQLite connections.

**Impact**: Minimal — the field is silently ignored for SQLite. Could confuse users.

**Recommendation**: ACCEPT RC1. Consider hiding SSL options for SQLite in the UI.

### F5 [P2] — Password passed in plaintext to PgConnectOptions

The `build_options` function receives the password as a `&str` parameter and passes it to `.password()`. This is correct behavior — SQLx handles the password securely during connection setup. The password is not logged or persisted in the connection options.

**Recommendation**: No action needed.

### F6 [P2] — No connection string exposure in error messages

SQLx may include connection parameters in error messages (e.g., "failed to connect to host:port"). The password is not included in SQLx error messages by default. However, the connection string could be logged via `tracing` if an error handler logs the full config.

**Recommendation**: Verify that no tracing log statement includes the full `ConnectionConfig` with password. Current audit shows no such logging.

## Summary

| Severity | Count | Findings |
|---|---|---|
| P1 | 1 | F1 (default SSL mode is Disable) |
| P2 | 5 | F2-F6 (all ACCEPT RC1) |

**Conclusion**: The TLS mode mapping is correct and complete for the 4 standard PostgreSQL SSL modes. The P1 finding (default Disable) is acceptable for v0.1 with documentation — most users connect to local databases. Custom CA and client certificates are not supported (documented limitations). No sensitive data is leaked in error messages or logs.
