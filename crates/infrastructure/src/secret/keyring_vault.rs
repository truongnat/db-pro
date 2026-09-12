use async_trait::async_trait;
use db_pro_core::domain::error::DbError;
use db_pro_core::ports::SecretStore;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use super::encryption;
use super::fallback::{self, FallbackStore};

/// A `SecretStore` backed by the OS keyring with optional session/file fallbacks.
///
/// The encrypted-file fallback is **disabled by default**. Call
/// `with_session_fallback()` to keep secrets only for the current process, or
/// `with_fallback()` for the opt-in development/CI file fallback.
pub struct KeyringVault {
    service_name: String,
    fallback_dir: PathBuf,
    allow_fallback: bool,
    fallback_store: Mutex<Option<FallbackStore>>,
    allow_session_fallback: bool,
    session_secrets: Mutex<HashMap<String, String>>,
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
            allow_session_fallback: false,
            session_secrets: Mutex::new(HashMap::new()),
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

    /// Keep credentials in memory when the OS keyring is unavailable.
    ///
    /// This preserves the current-session workflow without writing secrets to
    /// disk. The user must enter the password again after restarting the app.
    pub fn with_session_fallback(mut self) -> Self {
        self.allow_session_fallback = true;
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
        if self.allow_fallback || self.allow_session_fallback {
            Ok(())
        } else {
            Err(DbError::EncryptionFailed(
                "OS keyring unavailable and fallback is disabled; \
                 call KeyringVault::with_session_fallback() or \
                 KeyringVault::with_fallback() to enable a fallback"
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

    fn store_session_secret(&self, key: &str, value: &str) -> Result<(), DbError> {
        let mut secrets = self
            .session_secrets
            .lock()
            .map_err(|e| DbError::Internal(format!("session secret mutex poisoned: {e}")))?;
        secrets.insert(key.to_owned(), value.to_owned());
        Ok(())
    }

    fn retrieve_session_secret(&self, key: &str) -> Result<Option<String>, DbError> {
        let secrets = self
            .session_secrets
            .lock()
            .map_err(|e| DbError::Internal(format!("session secret mutex poisoned: {e}")))?;
        Ok(secrets.get(key).cloned())
    }

    fn delete_session_secret(&self, key: &str) -> Result<(), DbError> {
        let mut secrets = self
            .session_secrets
            .lock()
            .map_err(|e| DbError::Internal(format!("session secret mutex poisoned: {e}")))?;
        secrets.remove(key);
        Ok(())
    }

    fn retrieve_unavailable_secret(&self, key: &str) -> Result<Option<String>, DbError> {
        self.require_fallback()?;
        if self.allow_fallback {
            self.retrieve_fallback(key)
        } else {
            self.retrieve_session_secret(key)
        }
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
        if self.allow_fallback {
            self.store_fallback(key, value)?;
        }
        if self.allow_session_fallback {
            self.store_session_secret(key, value)?;
        }

        // Best-effort storage to OS keyring if available (without failing if keyring denied)
        if let Ok(entry) = self.keyring_entry(key) {
            let _ = entry.set_password(value);
        }

        Ok(())
    }

    async fn retrieve_secret(&self, key: &str) -> Result<Option<String>, DbError> {
        // Check local encrypted vault first if enabled (avoids OS keychain prompt on every launch)
        if self.allow_fallback {
            if let Ok(Some(value)) = self.retrieve_fallback(key) {
                return Ok(Some(value));
            }
        }
        if self.allow_session_fallback {
            if let Ok(Some(value)) = self.retrieve_session_secret(key) {
                return Ok(Some(value));
            }
        }

        let entry = match self.keyring_entry(key) {
            Ok(e) => e,
            Err(e) if is_keyring_unavailable(&e) => {
                tracing::warn!("OS keyring unavailable: {e}");
                return self.retrieve_unavailable_secret(key);
            }
            Err(e) => {
                return Err(DbError::Internal(format!("keyring entry creation failed: {e}")));
            }
        };

        match entry.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => {
                if !self.allow_fallback && !self.allow_session_fallback {
                    Ok(None)
                } else if self.allow_fallback {
                    self.retrieve_fallback(key)
                } else {
                    self.retrieve_session_secret(key)
                }
            }
            Err(e) if is_keyring_unavailable(&e) => {
                tracing::warn!("OS keyring unavailable: {e}");
                self.retrieve_unavailable_secret(key)
            }
            Err(e) => Err(DbError::Internal(format!("keyring get_password failed: {e}"))),
        }
    }

    async fn delete_secret(&self, key: &str) -> Result<(), DbError> {
        if self.allow_fallback {
            if let Ok(store) = self.get_or_init_fallback() {
                let _ = store.delete(key);
            }
        }
        if self.allow_session_fallback {
            let _ = self.delete_session_secret(key);
        }
        if let Ok(entry) = self.keyring_entry(key) {
            let _ = entry.delete_credential();
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

    #[test]
    fn new_vault_fails_closed_without_os_keyring() {
        let vault = KeyringVault::new("db-pro-test", PathBuf::from("/tmp/db-pro-test-secrets"));
        assert!(matches!(vault.require_fallback(), Err(DbError::EncryptionFailed(_))));
    }

    #[tokio::test]
    async fn missing_os_keyring_secret_returns_none_without_fallback() {
        let vault = KeyringVault::new("com.dbpro.app", PathBuf::from("/tmp/db-pro-diagnostic-secrets"));
        let key = format!("diagnostic/missing/{}", std::process::id());

        match vault.retrieve_secret(&key).await {
            Ok(None) => {}
            Ok(Some(_)) => panic!("diagnostic key unexpectedly exists"),
            Err(DbError::EncryptionFailed(message)) if message.contains("OS keyring unavailable") => {}
            Err(error) => panic!("missing key lookup failed: {error}"),
        }
    }

    #[tokio::test]
    async fn session_fallback_round_trips_without_writing_a_file() {
        let fallback_dir = std::env::temp_dir().join(format!("db-pro-session-{}", std::process::id()));
        let vault = KeyringVault::new("db-pro-session-test", fallback_dir.clone()).with_session_fallback();
        let key = format!("session/{}", std::process::id());

        vault
            .store_secret(&key, "<REDACTED>")
            .await
            .expect("session store should succeed");
        assert_eq!(
            vault
                .retrieve_secret(&key)
                .await
                .expect("session read should succeed")
                .as_deref(),
            Some("<REDACTED>")
        );
        assert!(!fallback_dir.join("secrets.json").exists());
        vault.delete_secret(&key).await.expect("session delete should succeed");
    }
}
