# Secret Store & OS Keyring Lifecycle Audit

**Document**: `docs/security/keyring-and-secret-lifecycle-audit.md`  
**Status**: Completed  
**Baseline Commit**: `e3df4896`  
**Related Issue**: #126 (Supports #14, #27, #29, #122, #105)

---

## 1. Executive Summary

This audit verifies the credential-storage lifecycle across OS keyring operations, encrypted-file fallbacks, in-memory session fallbacks, connection mutations (create, edit, delete, reconnect), and application restarts.

The primary credential vault implementation is located in `crates/infrastructure/src/secret/keyring_vault.rs` and `crates/infrastructure/src/secret/fallback.rs`.

---

## 2. Keyring & Storage Architecture

### 2.1 Service & Key Scheme
* **Service Name**: Configured per application instance (e.g. `com.dbpro.desktop.secrets`). If empty, the keyring layer deterministically rejects initialization across all platforms.
* **Account Keys**: Format `{connection_id}/password` or `{connection_id}/ssh_key`.
* **Isolation**: Keys are partitioned by UUID/connection ID, preventing collision or leakage across different database profiles.

### 2.2 Storage Modes
1. **OS Keyring (Production Default)**:
   - macOS: Apple Keychain Services (`Security.framework`).
   - Linux: Secret Service API via D-Bus / GNOME Keyring / KWallet.
   - Windows: Windows Credential Manager (`wincred`).
2. **Session Memory Fallback (`with_session_fallback()`)**:
   - Secrets are stored in `HashMap<String, Vec<u8>>` inside an `Arc<RwLock<...>>`.
   - Never flushed to disk. Automatically wiped on process termination.
3. **Encrypted File Fallback (`with_fallback()`)**:
   - Opt-in for headless/CI or test environments.
   - Stored at `~/.local/share/db-pro/secrets.enc` (or platform equivalent).
   - AES-GCM-256 with PBKDF2-HMAC-SHA256 key derivation.

---

## 3. Secret Lifecycle Matrix

| Event | Metadata State | Keyring State | Fallback State | Expected Secret Retrieval | Failure Behavior |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Save Connection** | Profile stored in local config (password redacted) | Keyring `set_password` called | If fallback enabled, persisted to encrypted file/memory | Stored securely, masked in UI | Return `DbError::EncryptionFailed` if OS storage denied |
| **Edit Connection (Password changed)** | Metadata updated | Old key overwritten in Keyring | Fallback entry overwritten | New password returned on next connect | Atomic update per connection ID |
| **Edit Connection (Password unchanged)** | Metadata updated | Keyring untouched | Fallback untouched | Existing password preserved | No plaintext exposure |
| **Delete Connection** | Profile removed from config | `delete_password` executed | Entry purged from fallback store | None (Secret removed) | Ignores `NoEntry`, logs errors |
| **App Startup / Reconnect** | Metadata loaded from config | `get_password` queried | Read from fallback if Keyring unavailable | Loaded into memory for connection pool only | Connection fails gracefully if secret missing |
| **Keyring Unavailable** | Metadata intact | Returns `PlatformFailure` / `NoStorageAccess` | Active if session/file fallback enabled | Retrieved from fallback store | Fallback error reported cleanly |

---

## 4. Security Findings & Boundary Analysis

* **Zero Plaintext Logging**: Connection passwords and keys are redacted in `Debug` implementations (`[REDACTED]`) and excluded from serialized JSON settings (`#[serde(skip)]` on raw secrets).
* **Memory Safety**: Credentials in memory are never held in static mutable globals; lifecycle is bound to `KeyringVault` and connection handles.
* **Multi-connection Isolation**: UUID-based namespacing guarantees that deleting or updating one connection has no impact on other connections.

---

## 5. Verification & Test Evidence

All secret store unit and integration tests pass:
- `crates/infrastructure/src/secret/keyring_vault.rs` tests (CRUD, session fallback, encrypted file fallback).
- Memory-leak and redaction verification via unit test suite.
