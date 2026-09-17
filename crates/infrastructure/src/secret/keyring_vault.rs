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
        // Empty service name: no keyring layer exists, so the round-trip is proven through the
        // session store alone and no real OS keyring is touched (the same deterministic seam the
        // other #243 tests use). A non-empty name would reach the platform keyring on this host and
        // fail on a CI runner without one.
        let vault = KeyringVault::new("", fallback_dir.clone()).with_session_fallback();
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

    #[test]
    fn fallback_flags_follow_the_builder_calls() {
        let dir = PathBuf::from("/tmp/db-pro-fallback-flags");

        let plain = KeyringVault::new("db-pro-flags", dir.clone());
        assert!(!plain.file_fallback_enabled());
        assert!(!plain.session_fallback_enabled());

        let session = KeyringVault::new("db-pro-flags", dir.clone()).with_session_fallback();
        assert!(!session.file_fallback_enabled());
        assert!(session.session_fallback_enabled());

        let file = KeyringVault::new("db-pro-flags", dir).with_fallback();
        assert!(file.file_fallback_enabled());
        assert!(!file.session_fallback_enabled());
    }

    /// Regression guard for #142: a vault without the file fallback must neither read nor
    /// write `secrets.json`, even when an older build left one behind.
    #[tokio::test]
    async fn disabled_file_fallback_never_touches_a_stale_fallback_file() {
        let dir = std::env::temp_dir().join(format!("db-pro-stale-fallback-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("secrets.json");
        std::fs::write(&path, b"written by an older build").expect("stale file");

        let vault = KeyringVault::new("db-pro-stale-fallback-test", dir.clone()).with_session_fallback();
        let key = format!("stale/{}", std::process::id());

        vault
            .store_secret(&key, "<REDACTED>")
            .await
            .expect("session store should succeed");
        assert_eq!(
            std::fs::read(&path).expect("stale file must still exist"),
            b"written by an older build",
            "a disabled file fallback must not rewrite the stale file"
        );
        assert_eq!(
            vault
                .retrieve_secret(&key)
                .await
                .expect("session read should succeed")
                .as_deref(),
            Some("<REDACTED>"),
            "the value must come from the session store, never from the stale file"
        );

        let absent = format!("stale/absent/{}", std::process::id());
        assert_eq!(
            vault.retrieve_secret(&absent).await.expect("missing key lookup"),
            None,
            "a key missing from the OS keyring must not resolve to the stale file"
        );

        let _ = std::fs::remove_file(&path);
    }

    // -- #243: a store that persisted nothing, and a delete that did not take effect, must not be
    //    reported as success ------------------------------------------------------------------

    /// An empty service name makes `keyring::Entry::new` fail with
    /// `Invalid("service", "cannot be empty")` on every platform (verified on this host), so a vault
    /// built with it has **no usable keyring layer** without any test-only injection. With no
    /// fallback enabled either, the value has nowhere to go and the store must say so instead of
    /// reporting success (#243 K-1).
    #[tokio::test]
    async fn store_secret_reports_when_no_store_accepts_the_value() {
        let vault = KeyringVault::new("", std::env::temp_dir().join("db-pro-no-store"));

        let error = vault
            .store_secret("no-store/probe", "<REDACTED>")
            .await
            .expect_err("a store that persisted nothing must not report success");

        assert!(
            error.to_string().contains("the secret was not stored"),
            "the error must name the outcome, not just bubble a platform message: {error}"
        );
    }

    /// Characterisation test for the behaviour this change deliberately leaves alone.
    ///
    /// A session-only store still reports success, because turning that into a user-visible warning
    /// ("saved, but this will not survive a restart") needs a warning channel the runtime does not
    /// have, and choosing between that warning and a hard failure is the open decision recorded in
    /// #243. Pinned here so a future change to it is deliberate rather than accidental.
    #[tokio::test]
    async fn store_secret_still_succeeds_when_only_the_session_store_holds_the_secret() {
        let vault = KeyringVault::new("", std::env::temp_dir().join("db-pro-session-only")).with_session_fallback();
        let key = "session-only/probe";

        vault
            .store_secret(key, "<REDACTED>")
            .await
            .expect("the session store accepts the value, so the in-session workflow still works");
        assert_eq!(
            vault.retrieve_secret(key).await.expect("session read").as_deref(),
            Some("<REDACTED>")
        );
    }

    /// `delete_secret` used to discard every layer's result, so a delete that could not take effect
    /// was indistinguishable from a successful one and the surviving entry was an invisible orphan
    /// (#243 K-2). The directory is made read-only after the store so the encrypted-file delete
    /// cannot be persisted.
    #[cfg(unix)]
    #[tokio::test]
    async fn delete_secret_reports_a_store_that_could_not_delete() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::temp_dir().join(format!("db-pro-delete-fail-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        // Empty service name: no keyring layer, so no real credential ever leaves this test.
        let vault = KeyringVault::new("", dir.clone()).with_fallback();
        let key = "delete/probe";
        vault
            .store_secret(key, "<REDACTED>")
            .await
            .expect("the encrypted-file fallback stores the value");

        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o500)).expect("chmod read-only");
        let error = vault
            .delete_secret(key)
            .await
            .expect_err("a delete that could not be persisted must not report success");
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).expect("chmod back");

        assert!(
            error.to_string().contains("encrypted-file fallback"),
            "the error must name the store that still holds the secret: {error}"
        );
        assert!(
            error.to_string().contains("orphan"),
            "the caller has to be told what the consequence is: {error}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The goal state of a delete is "the key is gone", so deleting something that was never there
    /// is a success, not a failure.
    #[tokio::test]
    async fn delete_secret_of_a_missing_key_is_not_a_failure() {
        let vault = KeyringVault::new("", std::env::temp_dir().join("db-pro-delete-missing")).with_fallback();

        vault
            .delete_secret("delete/never-stored")
            .await
            .expect("deleting an absent key reaches the goal state and must not fail");
    }

    /// `delete_secret` used to report "orphan may remain" for *every* `delete_credential`
    /// failure, which is wrong when the keyring layer is simply unavailable: an unavailable
    /// keyring could not have received the value via `store_secret`'s best-effort write, so
    /// nothing can be orphaned there. This pins the classification directly — no real keyring
    /// and no mock — across every reachable variant.
    #[test]
    fn delete_secret_treats_unavailable_keyring_as_not_an_orphan() {
        let unavailable =
            |reason: &str| keyring::Error::PlatformFailure(Box::new(std::io::Error::other(reason.to_owned())));

        // Deleted, absent, and unreachable all reach "nothing is in the keyring".
        assert_eq!(keyring_delete_failure(Ok(())), None);
        assert_eq!(keyring_delete_failure(Err(keyring::Error::NoEntry)), None);
        assert_eq!(
            keyring_delete_failure(Err(unavailable(
                "org.freedesktop.secrets is not provided by any .service file"
            ))),
            None
        );
        assert_eq!(
            keyring_delete_failure(Err(keyring::Error::NoStorageAccess(Box::new(std::io::Error::other(
                "credential store is locked"
            ))))),
            None
        );

        // A failure that is not "unavailable" is still an orphan and must name the store.
        let orphan = keyring_delete_failure(Err(keyring::Error::Ambiguous(Vec::new())))
            .expect("a non-availability delete failure is an orphan");
        assert!(orphan.contains("OS keyring"), "must name the store: {orphan}");
    }

    /// An empty service name disables the keyring layer deterministically (see `keyring_entry`
    /// and `build_secret_store`). With no fallback store attached, retrieval must report the
    /// unreachable-store condition — *not* pretend the key is present and *not* crash with a
    /// fatal "keyring entry creation failed: Attribute service name is empty". This pins the
    /// corrected behaviour: the disabled keyring is treated like an unavailable one.
    #[tokio::test]
    async fn retrieve_secret_with_disabled_keyring_and_no_fallback_reports_unavailable() {
        let vault = KeyringVault::new("", std::env::temp_dir().join("db-pro-entry-error"));

        let error = vault
            .retrieve_secret("entry-error/probe")
            .await
            .expect_err("a disabled keyring with no fallback must not be reported as a missing key");

        assert!(
            error
                .to_string()
                .contains("OS keyring unavailable and fallback is disabled"),
            "the error has to distinguish an unreachable store from an absent key: {error}"
        );
    }

    /// With the keyring disabled (empty service name) and a fallback store attached, a missing
    /// secret returns `Ok(None)` instead of the fatal "keyring entry creation failed". Callers
    /// (e.g. `ConnectionService::connect`) then map it to the correct "password not found in
    /// secret store" rather than blocking connection creation. This is the regression guard for
    /// the new-connection Configuration Error.
    #[tokio::test]
    async fn retrieve_secret_with_disabled_keyring_falls_back_to_session_store() {
        let vault = KeyringVault::new("", std::env::temp_dir().join("db-pro-entry-fallback")).with_session_fallback();

        let value = vault
            .retrieve_secret("missing/probe")
            .await
            .expect("a disabled keyring must fall through to the fallback, not error");

        assert_eq!(
            value, None,
            "a secret absent from the fallback is genuinely absent, not a fatal error"
        );
    }
}
