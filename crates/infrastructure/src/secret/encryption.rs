use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::Argon2;
use db_pro_core::domain::error::DbError;
use rand::RngCore;

const NONCE_LEN: usize = 12;

pub fn derive_key(master_password: &str, salt: &[u8]) -> Result<[u8; 32], DbError> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(master_password.as_bytes(), salt, &mut key)
        .map_err(|e| DbError::EncryptionFailed(format!("key derivation failed: {e}")))?;
    Ok(key)
}

pub fn encrypt(plaintext: &str, key: &[u8; 32]) -> Result<(Vec<u8>, [u8; NONCE_LEN]), DbError> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| DbError::EncryptionFailed(format!("encryption failed: {e}")))?;
    Ok((ciphertext, nonce_bytes))
}

pub fn decrypt(ciphertext: &[u8], nonce: &[u8; NONCE_LEN], key: &[u8; 32]) -> Result<String, DbError> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| DbError::EncryptionFailed(format!("decryption failed: {e}")))?;
    String::from_utf8(plaintext).map_err(|e| DbError::EncryptionFailed(format!("invalid utf8: {e}")))
}
