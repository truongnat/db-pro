use db_pro_core::domain::error::DbError;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

const MAGIC: &[u8; 4] = b"DBP1";
const ALGO_ARGON2_AES: u8 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

// Minimum blob size: magic(4) + algo(1) + salt(16) + nonce(12) = 33
const MIN_BLOB_LEN: usize = 4 + 1 + SALT_LEN + NONCE_LEN;

/// Pack salt, nonce, and ciphertext into the versioned encrypted blob format.
///
/// Format: `DBP1 | algo(1B) | salt(16B) | nonce(12B) | ciphertext(variable)`
pub fn pack_encrypted(salt: &[u8], nonce: &[u8; NONCE_LEN], ciphertext: &[u8]) -> Vec<u8> {
    let mut blob = Vec::with_capacity(4 + 1 + SALT_LEN + NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(MAGIC);
    blob.push(ALGO_ARGON2_AES);
    blob.extend_from_slice(salt);
    blob.extend_from_slice(nonce);
    blob.extend_from_slice(ciphertext);
    blob
}

/// Unpack an encrypted blob into (salt, nonce, ciphertext).
#[allow(clippy::type_complexity)]
pub fn unpack_encrypted(blob: &[u8]) -> Result<(Vec<u8>, [u8; NONCE_LEN], Vec<u8>), DbError> {
    if blob.len() < MIN_BLOB_LEN {
        return Err(DbError::EncryptionFailed("encrypted blob too short".into()));
    }
    if &blob[0..4] != MAGIC {
        return Err(DbError::EncryptionFailed("invalid blob magic header".into()));
    }
    if blob[4] != ALGO_ARGON2_AES {
        return Err(DbError::EncryptionFailed(format!(
            "unsupported algo version: {}",
            blob[4]
        )));
    }

    let salt = blob[5..5 + SALT_LEN].to_vec();
    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&blob[5 + SALT_LEN..5 + SALT_LEN + NONCE_LEN]);
    let ciphertext = blob[5 + SALT_LEN + NONCE_LEN..].to_vec();

    Ok((salt, nonce, ciphertext))
}

// ---------------------------------------------------------------------------
// Hex encode / decode (inline to avoid extra dependency)
// ---------------------------------------------------------------------------

fn hex_encode(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() * 2);
    for &b in data {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn hex_decode(s: &str) -> Result<Vec<u8>, DbError> {
    if !s.len().is_multiple_of(2) {
        return Err(DbError::Internal("hex string has odd length".into()));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| DbError::Internal(format!("invalid hex: {e}"))))
        .collect()
}

// ---------------------------------------------------------------------------
// FallbackStore — versioned encrypted-file fallback
// ---------------------------------------------------------------------------

pub struct FallbackStore {
    file_path: PathBuf,
    entries: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl Clone for FallbackStore {
    fn clone(&self) -> Self {
        Self {
            file_path: self.file_path.clone(),
            entries: Arc::clone(&self.entries),
        }
    }
}

impl FallbackStore {
    /// Create a reference to an existing store (shares the same in-memory entries).
    pub fn clone_ref(store: &FallbackStore) -> Self {
        store.clone()
    }

    /// Create a new `FallbackStore`, loading existing entries from `file_path`
    /// if the file already exists.
    pub fn new(file_path: PathBuf) -> Result<Self, DbError> {
        let entries = if file_path.exists() {
            let contents = std::fs::read_to_string(&file_path)
                .map_err(|e| DbError::Io(format!("failed to read fallback file: {e}")))?;
            let map: HashMap<String, String> = serde_json::from_str(&contents)
                .map_err(|e| DbError::Internal(format!("failed to parse fallback file: {e}")))?;
            let mut decoded = HashMap::with_capacity(map.len());
            for (k, v) in map {
                decoded.insert(k, hex_decode(&v)?);
            }
            decoded
        } else {
            HashMap::new()
        };

        Ok(Self {
            file_path,
            entries: Arc::new(RwLock::new(entries)),
        })
    }

    /// Store an encrypted blob under `key`, persisting to disk.
    pub fn store(&self, key: &str, encrypted_blob: Vec<u8>) -> Result<(), DbError> {
        {
            let mut entries = self
                .entries
                .write()
                .map_err(|e| DbError::Internal(format!("lock poisoned: {e}")))?;
            entries.insert(key.to_string(), encrypted_blob);
        }
        self.persist()
    }

    /// Retrieve the encrypted blob for `key`, if it exists.
    pub fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>, DbError> {
        let entries = self
            .entries
            .read()
            .map_err(|e| DbError::Internal(format!("lock poisoned: {e}")))?;
        Ok(entries.get(key).cloned())
    }

    /// Delete the entry for `key` and persist.
    pub fn delete(&self, key: &str) -> Result<(), DbError> {
        {
            let mut entries = self
                .entries
                .write()
                .map_err(|e| DbError::Internal(format!("lock poisoned: {e}")))?;
            entries.remove(key);
        }
        self.persist()
    }

    /// Write the current in-memory entries to disk as JSON (hex-encoded blobs).
    ///
    /// Uses atomic write (temp file + rename) to prevent corruption on crash.
    /// Sets file permission to 0600 on Unix to restrict access to the owner.
    ///
    /// **Security note**: The encryption key is derived from the service name,
    /// which is not a secret. This fallback is intended for development only.
    /// Production deployments must use the OS keyring or a user-provided master
    /// key stored in platform secure storage.
    fn persist(&self) -> Result<(), DbError> {
        let entries = self
            .entries
            .read()
            .map_err(|e| DbError::Internal(format!("lock poisoned: {e}")))?;
        let encoded: HashMap<&str, String> = entries.iter().map(|(k, v)| (k.as_str(), hex_encode(v))).collect();
        let json = serde_json::to_string_pretty(&encoded)
            .map_err(|e| DbError::Internal(format!("failed to serialize fallback: {e}")))?;

        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| DbError::Io(format!("failed to create fallback dir: {e}")))?;
        }

        let tmp_path = self
            .file_path
            .with_extension(format!("json.tmp.{}", std::process::id()));
        std::fs::write(&tmp_path, &json)
            .map_err(|e| DbError::Io(format!("failed to write fallback temp file: {e}")))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&tmp_path, perms)
                .map_err(|e| DbError::Io(format!("failed to set fallback file permissions: {e}")))?;
        }

        std::fs::rename(&tmp_path, &self.file_path)
            .map_err(|e| DbError::Io(format!("failed to rename fallback temp file: {e}")))?;
        Ok(())
    }
}
