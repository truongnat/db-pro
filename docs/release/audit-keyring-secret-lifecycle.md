# Keyring & Encrypted-Fallback Secret Lifecycle Audit

> Baseline SHA: `8d6aa54` (post-#142 fix)
> Issue: #126
> Source: `crates/infrastructure/src/secret/`, `crates/core/src/application/connection_service.rs`, `crates/core/src/domain/secret.rs`

## Credential storage lifecycle matrix

| Event | Metadata state | Keyring state | Fallback state | Retrieval | Failure behavior |
|---|---|---|---|---|---|
| Create connection | Saved to meta.DB | `store_secret(conn_id, password)` | N/A (fallback disabled) | `retrieve_secret(conn_id)` | Error propagated to user |
| Edit connection (password change) | Updated in meta.DB | `store_secret(conn_id, new_pw)` overwrites | N/A | Returns new password | Error propagated, old password may be lost |
| Delete connection | Removed from meta.DB | `delete_secret(conn_id)` | N/A | N/A | `NoEntry` is silently ignored (correct) |
| Connect | Read from meta.DB | `retrieve_secret(conn_id)` | N/A | Password for connection | Error → connection fails |
| Reconnect on startup | Loaded from meta.DB | `retrieve_secret(conn_id)` | N/A | Password for reconnect | Error → connection skipped, user notified |
| Orphan secret (conn deleted but secret remains) | No matching conn_id | Secret persists | N/A | Never retrieved | Orphan — no cleanup mechanism |
| Keyring unavailable at runtime | meta.DB OK | Error | N/A (disabled) | Error | Connection fails with `EncryptionFailed` |

## Keyring configuration (post-#142)

| Component | Value | Source |
|---|---|---|
| Service name | `com.dbpro.app` | `lib.rs:50` |
| Account key format | `secret-{connection_id}` | `domain/secret.rs` |
| macOS backend | `security-framework` 3.x (Keychain) | `apple-native` feature |
| Linux backend | `linux-keyutils` 0.2 (kernel keyring) | `linux-native` feature |
| Windows backend | `windows-sys` 0.60 (Credential Manager) | `windows-native` feature |
| Fallback in production | **Disabled** | `with_fallback()` removed from `lib.rs` |

## Findings

### F1 [P2] — No orphan secret cleanup

When a connection is deleted, `delete_secret(conn_id)` is called. If the keyring call fails silently (e.g., `NoEntry` is ignored), the secret may remain in the keyring indefinitely. Over time, orphan secrets accumulate.

**Impact**: Minimal — orphan secrets are small, keyed by UUID, and inaccessible without the connection ID. No security risk.

**Recommendation**: ACCEPT RC1. Add periodic orphan cleanup in a future release.

### F2 [P2] — Edit connection password: no atomic swap

When editing a connection's password, `connection_service.rs:104` calls `store_secret` with the new password. If this fails, the old password is lost from the keyring (overwritten or deleted). The code attempts to restore with `store_secret(conn_id, old_pw)` on failure, but if the restore also fails, the user loses the stored password.

**Impact**: Low — the user can re-enter the password. The connection metadata is intact.

**Recommendation**: ACCEPT RC1. Document as known limitation.

### F3 [P2] — Linux kernel keyutils: session-scoped

The `linux-native` feature uses `linux-keyutils` (kernel keyutils), which stores credentials in the session keyring. Credentials do NOT survive reboot. For persistent storage on Linux, `linux-native-sync-persistent` (libsecret/dbus) would be needed.

**Impact**: Linux users must re-enter passwords after reboot. This is acceptable for v0.1 but should be documented.

**Recommendation**: ACCEPT RC1. Document in known limitations. Consider `linux-native-sync-persistent` for v0.2 (requires libsecret).

### F4 [P2] — No secret rotation or expiry mechanism

Secrets are stored indefinitely with no expiry or rotation. This is expected for a database IDE — users manage their own database credentials.

**Recommendation**: No action needed.

### F5 [P2] — Encrypted fallback crypto review

The fallback uses Argon2 (default params) for key derivation from the service name + AES-256-GCM for encryption. The key derivation salt is derived from the service name (not random). This is **intentionally weak** — the fallback is dev-only.

Post-#142, the fallback is disabled in production. The crypto is adequate for development use.

**Recommendation**: No action needed. Fallback is disabled in production.

### F6 [P2] — Debug output redaction

`KeyringVault::Debug` implementation correctly redacts the service name (`[REDACTED]`). The fallback store does not log secret values. The `tracing::info!` for fallback storage logs only the key name, not the value.

**Recommendation**: No action needed. Redaction is correct.

## Summary

| Severity | Count | Findings |
|---|---|---|
| P1 | 0 | — |
| P2 | 6 | F1-F6 (all ACCEPT RC1) |

**Conclusion**: Post-#142, the keyring lifecycle is production-safe. Real OS backends are configured for all platforms. Fallback is disabled. Secrets follow a clean create/read/update/delete lifecycle. Linux session-scoped storage is the only notable limitation (credentials don't survive reboot).
