use async_trait::async_trait;
use db_pro_core::domain::error::DbError;
use db_pro_core::ports::SecretStore;
use std::path::PathBuf;

use super::encryption;
use super::fallback::{self, FallbackStore};

/// A `SecretStore` backed by the OS keyring with an encrypted-file fallback.
///
/// When the OS credential manager is unavailable (e.g. headless CI), secrets
/// are transparently stored in an encrypted JSON file inside `fallback_dir`.
pub struct KeyringVault {
    service_name: String,
    fallback_dir: PathBuf,
}

impl std::fmt::Debug for KeyringVault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyringVault")
            .field("service_name", &"[REDACTED]")
            .field("fallback_dir", &self.fallback_dir)
            .finish()
    }
}

impl KeyringVault {
    pub fn new(service_name: impl Into<String>, fallback_dir: PathBuf) -> Self {
        Self {
            service_name: service_name.into(),
            fallback_dir,
        }
    }

    // -- helpers -------------------------------------------------------------

    fn fallback_store(&self) -> Result<FallbackStore, DbError> {
        let path = self.fallback_dir.join("secrets.json");
        FallbackStore::new(path)
    }

    /// Derive a master encryption key from the service name.
    ///
    /// **DEV-ONLY**: The service name is not a secret — anyone who knows it can
    /// derive the same key and decrypt the fallback file. This exists solely so
    /// the application can run in environments without an OS keyring (e.g.
    /// headless CI). Before production use, either the OS keyring must be
    /// mandatory or the user must supply a master password that is stored in
    /// platform secure storage.
    fn derive_master_key(&self, salt: &[u8]) -> Result<[u8; 32], DbError> {
        encryption::derive_key(&self.service_name, salt)
    }

    /// Fixed salt used for master-key derivation (MVP).
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

/// Returns `true` when the error indicates the OS credential manager itself
/// is not accessible (as opposed to a simple "entry not found").
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
                tracing::warn!("OS keyring unavailable, falling back to encrypted file: {e}");
                return self.store_fallback(key, value);
            }
            Err(e) => {
                return Err(DbError::Internal(format!("keyring entry creation failed: {e}")));
            }
        };

        match entry.set_password(value) {
            Ok(()) => Ok(()),
            Err(e) if is_keyring_unavailable(&e) => {
                tracing::warn!("OS keyring unavailable, falling back to encrypted file: {e}");
                self.store_fallback(key, value)
            }
            Err(e) => Err(DbError::Internal(format!("keyring set_password failed: {e}"))),
        }
    }

    async fn retrieve_secret(&self, key: &str) -> Result<Option<String>, DbError> {
        let entry = match self.keyring_entry(key) {
            Ok(e) => e,
            Err(e) if is_keyring_unavailable(&e) => {
                tracing::warn!("OS keyring unavailable, trying fallback file: {e}");
                return self.retrieve_fallback(key);
            }
            Err(e) => {
                return Err(DbError::Internal(format!("keyring entry creation failed: {e}")));
            }
        };

        match entry.get_password() {
            Ok(value) => return Ok(Some(value)),
            Err(keyring::Error::NoEntry) => {
                // Not in keyring — try fallback file.
            }
            Err(e) if is_keyring_unavailable(&e) => {
                tracing::warn!("OS keyring unavailable, trying fallback file: {e}");
                return self.retrieve_fallback(key);
            }
            Err(e) => {
                return Err(DbError::Internal(format!("keyring get_password failed: {e}")));
            }
        }

        // Try fallback.
        self.retrieve_fallback(key)
    }

    async fn delete_secret(&self, key: &str) -> Result<(), DbError> {
        // Attempt keyring deletion (ignore NoEntry — just means it was never
        // stored there or already removed).
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
            Err(_) => {
                // Keyring unavailable — skip keyring deletion.
            }
        }

        // Also remove from fallback file (ignore error if file doesn't exist).
        if let Ok(store) = self.fallback_store() {
            store.delete(key)?;
        }

        Ok(())
    }
}

// -- private fallback helpers ------------------------------------------------

impl KeyringVault {
    fn store_fallback(&self, key: &str, value: &str) -> Result<(), DbError> {
        let salt = self.master_salt();
        let master_key = self.derive_master_key(&salt)?;
        let (ciphertext, nonce) = encryption::encrypt(value, &master_key)?;
        let blob = fallback::pack_encrypted(&salt, &nonce, &ciphertext);

        let store = self.fallback_store()?;
        store.store(key, blob)?;
        tracing::info!("secret stored in fallback file (keyring unavailable)");
        Ok(())
    }

    fn retrieve_fallback(&self, key: &str) -> Result<Option<String>, DbError> {
        let store = self.fallback_store()?;
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
