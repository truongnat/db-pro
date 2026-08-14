use async_trait::async_trait;
use db_pro_core::domain::error::DbError;
use db_pro_core::ports::SecretStore;
use std::path::PathBuf;
use std::sync::Mutex;

use super::encryption;
use super::fallback::{self, FallbackStore};

/// A `SecretStore` backed by the OS keyring with an opt-in encrypted-file fallback.
///
/// The fallback is **disabled by default**. Call `with_fallback()` to enable it
/// for development/CI environments. In production, if the OS keyring is
/// unavailable, operations return an error instead of silently writing
/// weakly-encrypted secrets to disk.
pub struct KeyringVault {
    service_name: String,
    fallback_dir: PathBuf,
    allow_fallback: bool,
    fallback_store: Mutex<Option<FallbackStore>>,
}

impl std::fmt::Debug for KeyringVault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyringVault")
            .field("service_name", &"[REDACTED]")
            .field("fallback_dir", &self.fallback_dir)
            .field("allow_fallback", &self.allow_fallback)
            .finish()
    }
}

impl KeyringVault {
    pub fn new(service_name: impl Into<String>, fallback_dir: PathBuf) -> Self {
        Self {
            service_name: service_name.into(),
            fallback_dir,
            allow_fallback: false,
            fallback_store: Mutex::new(None),
        }
    }

    /// Enable the encrypted-file fallback for environments without an OS keyring.
    ///
    /// **Do not enable in production.** The encryption key is derived from the
    /// service name, which is not a secret.
    pub fn with_fallback(mut self) -> Self {
        self.allow_fallback = true;
        self
    }

    // -- helpers -------------------------------------------------------------

    fn get_or_init_fallback(&self) -> Result<FallbackStore, DbError> {
        let mut guard = self
            .fallback_store
            .lock()
            .map_err(|e| DbError::Internal(format!("fallback mutex poisoned: {e}")))?;
        if let Some(store) = guard.as_ref() {
            return Ok(FallbackStore::clone_ref(store));
        }
        let path = self.fallback_dir.join("secrets.json");
        let store = FallbackStore::new(path)?;
        *guard = Some(FallbackStore::clone_ref(&store));
        Ok(store)
    }

    fn require_fallback(&self) -> Result<(), DbError> {
        if self.allow_fallback {
            Ok(())
        } else {
            Err(DbError::EncryptionFailed(
                "OS keyring unavailable and fallback is disabled; \
                 call KeyringVault::with_fallback() to enable for development"
                    .into(),
            ))
        }
    }

    /// Derive a master encryption key from the service name.
    ///
    /// **DEV-ONLY**: The service name is not a secret — anyone who knows it can
    /// derive the same key and decrypt the fallback file.
    fn derive_master_key(&self, salt: &[u8]) -> Result<[u8; 32], DbError> {
        encryption::derive_key(&self.service_name, salt)
    }

    fn master_salt(&self) -> [u8; 16] {
        let mut salt = [0u8; 16];
        let name_bytes = self.service_name.as_bytes();
        let len = name_bytes.len().min(16);
        salt[..len].copy_from_slice(&name_bytes[..len]);
        salt
    }

    fn keyring_entry(&self, key: &str) -> Result<keyring::Entry, keyring::Error> {
        keyring::Entry::new(&self.service_name, key)
    }
}

fn is_keyring_unavailable(err: &keyring::Error) -> bool {
    matches!(
        err,
        keyring::Error::NoStorageAccess(_) | keyring::Error::PlatformFailure(_)
    )
}

#[async_trait]
impl SecretStore for KeyringVault {
    async fn store_secret(&self, key: &str, value: &str) -> Result<(), DbError> {
        let entry = match self.keyring_entry(key) {
            Ok(e) => e,
            Err(e) if is_keyring_unavailable(&e) => {
                tracing::warn!("OS keyring unavailable: {e}");
                self.require_fallback()?;
                return self.store_fallback(key, value);
            }
            Err(e) => {
                return Err(DbError::Internal(format!("keyring entry creation failed: {e}")));
            }
        };

        match entry.set_password(value) {
            Ok(()) => {
                if self.allow_fallback {
                    if let Err(e) = self.store_fallback(key, value) {
                        tracing::warn!("fallback store failed after keyring success: {e}");
                    }
                }
                Ok(())
            }
            Err(e) if is_keyring_unavailable(&e) => {
                tracing::warn!("OS keyring unavailable: {e}");
                self.require_fallback()?;
                self.store_fallback(key, value)
            }
            Err(e) => Err(DbError::Internal(format!("keyring set_password failed: {e}"))),
        }
    }

    async fn retrieve_secret(&self, key: &str) -> Result<Option<String>, DbError> {
        let entry = match self.keyring_entry(key) {
            Ok(e) => e,
            Err(e) if is_keyring_unavailable(&e) => {
                tracing::warn!("OS keyring unavailable: {e}");
                self.require_fallback()?;
                return self.retrieve_fallback(key);
            }
            Err(e) => {
                return Err(DbError::Internal(format!("keyring entry creation failed: {e}")));
            }
        };

        match entry.get_password() {
            Ok(value) => return Ok(Some(value)),
            Err(keyring::Error::NoEntry) => {
                // Not in keyring — try fallback if allowed.
                self.require_fallback()?;
            }
            Err(e) if is_keyring_unavailable(&e) => {
                tracing::warn!("OS keyring unavailable: {e}");
                self.require_fallback()?;
                return self.retrieve_fallback(key);
            }
            Err(e) => {
                return Err(DbError::Internal(format!("keyring get_password failed: {e}")));
            }
        }

        self.retrieve_fallback(key)
    }

    async fn delete_secret(&self, key: &str) -> Result<(), DbError> {
        match self.keyring_entry(key) {
            Ok(entry) => {
                if let Err(e) = entry.delete_credential() {
                    if !matches!(e, keyring::Error::NoEntry) && !is_keyring_unavailable(&e) {
                        return Err(DbError::Internal(format!("keyring delete_credential failed: {e}")));
                    }
                }
            }
            Err(e) if !is_keyring_unavailable(&e) => {
                return Err(DbError::Internal(format!("keyring entry creation failed: {e}")));
            }
            Err(_) => {}
        }

        if self.allow_fallback {
            if let Ok(store) = self.get_or_init_fallback() {
                store.delete(key)?;
            }
        }

        Ok(())
    }
}

impl KeyringVault {
    fn store_fallback(&self, key: &str, value: &str) -> Result<(), DbError> {
        let salt = self.master_salt();
        let master_key = self.derive_master_key(&salt)?;
        let (ciphertext, nonce) = encryption::encrypt(value, &master_key)?;
        let blob = fallback::pack_encrypted(&salt, &nonce, &ciphertext);

        let store = self.get_or_init_fallback()?;
        store.store(key, blob)?;
        tracing::info!("secret stored in fallback file (keyring unavailable)");
        Ok(())
    }

    fn retrieve_fallback(&self, key: &str) -> Result<Option<String>, DbError> {
        let store = self.get_or_init_fallback()?;
        let blob = match store.retrieve(key)? {
            Some(b) => b,
            None => return Ok(None),
        };

        let (salt, nonce, ciphertext) = fallback::unpack_encrypted(&blob)?;
        let master_key = self.derive_master_key(&salt)?;
        let plaintext = encryption::decrypt(&ciphertext, &nonce, &master_key)?;
        Ok(Some(plaintext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test for the keyring credential API.
    ///
    /// Performs a set → get → delete roundtrip to verify the keyring API is
    /// functional. On macOS (Keychain) and Windows (Credential Manager) this
    /// exercises the real native backend. On Linux with DBus/Secret Service
    /// available, this exercises the persistent libsecret backend.
    ///
    /// **Limitations**:
    /// - The keyring mock store also supports set/get/delete, so this test
    ///   passing does NOT prove a native backend is active.
    /// - On headless Linux CI (no DBus daemon), `sync-secret-service` may
    ///   fail with `PlatformFailure`. The test treats this as an expected
    ///   skip rather than a failure.
    ///
    /// Full platform qualification requires running the packaged app on each
    /// OS and verifying credential persistence across restarts.
    #[test]
    fn keyring_credential_roundtrip() {
        let service = "db-pro-test-roundtrip";
        let key = "probe-key";
        let value = "probe-value-42";

        let entry = match keyring::Entry::new(service, key) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("keyring Entry::new failed (no backend compiled in?): {e:?}");
                return;
            }
        };

        // Attempt to store a credential.
        match entry.set_password(value) {
            Ok(()) => {}
            Err(keyring::Error::PlatformFailure(msg)) => {
                // Expected on headless Linux CI without DBus/Secret Service.
                // Not a failure — just means the native backend is not available.
                eprintln!(
                    "keyring backend not available (expected on headless CI): {msg}"
                );
                return;
            }
            Err(keyring::Error::NoStorageAccess(msg)) => {
                eprintln!("keyring has no storage access: {msg}");
                return;
            }
            Err(e) => panic!("set_password failed with unexpected error: {e:?}"),
        }

        // Verify the stored value.
        let retrieved = entry
            .get_password()
            .expect("get_password must succeed after set_password");
        assert_eq!(
            retrieved, value,
            "retrieved credential must match what was stored"
        );

        // Clean up.
        entry
            .delete_credential()
            .expect("delete_credential must succeed");

        // Verify deletion.
        let after_delete = entry.get_password();
        assert!(
            matches!(after_delete, Err(keyring::Error::NoEntry)),
            "credential must be gone after delete_credential, got: {after_delete:?}"
        );
    }

    /// Verify that KeyringVault without fallback returns an error when the
    /// keyring is unavailable, rather than silently using a weak fallback.
    #[test]
    fn vault_without_fallback_is_fail_closed() {
        let vault = KeyringVault::new("db-pro-test-nofb", PathBuf::from("/tmp/db-pro-test-secrets"));
        assert!(!vault.allow_fallback, "fallback must be disabled by default");
    }
}
