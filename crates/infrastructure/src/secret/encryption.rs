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

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SALT: &[u8] = b"deterministic-salt";

    #[test]
    fn derive_key_is_deterministic_for_same_password_and_salt() {
        let first = derive_key("test-password", TEST_SALT).expect("key derivation");
        let second = derive_key("test-password", TEST_SALT).expect("key derivation");

        assert_eq!(first, second);
    }

    #[test]
    fn encrypt_and_decrypt_round_trip() {
        let key = derive_key("test-password", TEST_SALT).expect("key derivation");
        let (ciphertext, nonce) = encrypt("secret value", &key).expect("encryption");

        assert_eq!(decrypt(&ciphertext, &nonce, &key).expect("decryption"), "secret value");
    }

    #[test]
    fn wrong_key_cannot_decrypt_ciphertext() {
        let key = derive_key("test-password", TEST_SALT).expect("key derivation");
        let wrong_key = derive_key("different-password", TEST_SALT).expect("key derivation");
        let (ciphertext, nonce) = encrypt("secret value", &key).expect("encryption");

        assert!(decrypt(&ciphertext, &nonce, &wrong_key).is_err());
    }

    #[test]
    fn corrupt_ciphertext_is_rejected() {
        let key = derive_key("test-password", TEST_SALT).expect("key derivation");
        let (mut ciphertext, nonce) = encrypt("secret value", &key).expect("encryption");
        ciphertext[0] ^= 0xff;

        assert!(decrypt(&ciphertext, &nonce, &key).is_err());
    }
}
