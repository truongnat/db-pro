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
    /// **Development and CI only.** The encryption key is derived from the
    /// service name, which is not a secret. The shipping wiring
    /// (`DbProRuntime::new`) enables this only for debug builds or when
    /// `DB_PRO_ALLOW_FILE_SECRET_FALLBACK` is set explicitly (#142).
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

    /// Whether the encrypted-file fallback is enabled for this vault.
    pub fn file_fallback_enabled(&self) -> bool {
        self.allow_fallback
    }

    /// Whether the in-memory session fallback is enabled for this vault.
    pub fn session_fallback_enabled(&self) -> bool {
        self.allow_session_fallback
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
        if self.service_name.is_empty() {
            // An empty service name disables the keyring layer deterministically
            // across platforms (some `keyring` backends accept empty strings).
            return Err(keyring::Error::Invalid(
                "service name is empty".into(),
                "disallowed".into(),
            ));
        }
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

/// Classify the result of `Entry::delete_credential` for the orphan check.
///
/// The goal state of a delete is "the key is gone". That is reached three ways: the
/// credential was deleted, there was no entry to delete, or the keyring layer is
/// *unavailable*. An unavailable keyring cannot have received the value through
/// `store_secret`'s best-effort write, so nothing can be orphaned there — the same
/// `is_keyring_unavailable` distinction `retrieve_secret` already makes (and the reverse
/// of `store_secret`, which only writes when the keyring accepts the value). Anything else
/// is an entry this vault could not delete, which must be reported (#243 K-2).
fn keyring_delete_failure(delete_result: Result<(), keyring::Error>) -> Option<String> {
    match delete_result {
        Ok(()) | Err(keyring::Error::NoEntry) => None,
        Err(error) if is_keyring_unavailable(&error) => None,
        Err(error) => Some(format!("OS keyring: {error}")),
    }
}

#[async_trait]
impl SecretStore for KeyringVault {
    async fn store_secret(&self, key: &str, value: &str) -> Result<(), DbError> {
        let mut stored = false;
        if self.allow_fallback {
            self.store_fallback(key, value)?;
            stored = true;
        }
        if self.allow_session_fallback {
            self.store_session_secret(key, value)?;
            stored = true;
        }

        // Best-effort OS keyring write. A refusal is the documented degradation path of a release
        // build (the session store keeps the secret for this process only), but it must not be
        // silent: #243 (K-1) records the silent version, where a denied keyring still reported the
        // connection as saved and the user only found out after a restart, when connecting failed
        // with "password not found in secret store".
        if let Ok(entry) = self.keyring_entry(key) {
            match entry.set_password(value) {
                Ok(()) => stored = true,
                Err(error) => tracing::warn!(
                    %error,
                    "OS keyring did not accept the secret; it will not survive a restart unless another store holds it"
                ),
            }
        }

        if stored {
            Ok(())
        } else {
            Err(DbError::Internal(
                "the secret was not stored: the OS keyring did not accept it and no fallback store is enabled, so it \
                 would be lost at restart"
                    .to_owned(),
            ))
        }
    }

    // cc-scan:allow COMPLEXITY — classifier/dispatch ladder — one case per branch
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
            // Empty service name ⇒ keyring deterministically disabled (see `build_secret_store`):
            // treat like "unavailable" and fall through to the fallback instead of fatal erroring.
            Err(e) if self.service_name.is_empty() => {
                tracing::debug!("OS keyring disabled (empty service name): {e}");
                return self.retrieve_unavailable_secret(key);
            }
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
        let mut failures: Vec<String> = Vec::new();

        if self.allow_fallback {
            match self.get_or_init_fallback() {
                Ok(store) => {
                    if let Err(error) = store.delete(key) {
                        failures.push(format!("encrypted-file fallback: {error}"));
                    }
                }
                Err(error) => failures.push(format!("encrypted-file fallback unavailable: {error}")),
            }
        }
        if self.allow_session_fallback {
            if let Err(error) = self.delete_session_secret(key) {
                failures.push(format!("session store: {error}"));
            }
        }
        if let Ok(entry) = self.keyring_entry(key) {
            if let Some(failure) = keyring_delete_failure(entry.delete_credential()) {
                failures.push(failure);
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            // #243 (K-2): a delete that did not take effect has to be reported, or the surviving
            // entry is an orphan nobody can see -- the `SecretStore` port has no enumeration, so the
            // caller is the only place this can surface.
            Err(DbError::Internal(format!(
                "the secret could not be deleted from every credential store ({}) -- an orphan entry may remain",
                failures.join("; ")
            )))
        }
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
        tracing::info!("secret also stored in the encrypted-file fallback (development/CI fallback)");
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
#[path = "tests/keyring_vault_tests.rs"]
mod tests;
