//! Tests for `../encryption.rs` (moved out of the source file).
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
