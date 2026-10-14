//! Tests for `../fallback.rs` (moved out of the source file).
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn test_file_path(label: &str) -> PathBuf {
    let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "db-pro-fallback-{label}-{}-{sequence}.json",
        std::process::id()
    ))
}

fn cleanup(path: &PathBuf) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_dir_all(path);
}

fn encrypted_test_blob() -> Vec<u8> {
    pack_encrypted(&[1; SALT_LEN], &[2; NONCE_LEN], b"ciphertext")
}

#[test]
fn store_round_trips_and_missing_key_returns_none() {
    let path = test_file_path("round-trip");
    let store = FallbackStore::new(path.clone()).expect("create fallback store");

    store.store("agent/key", encrypted_test_blob()).expect("store value");
    assert_eq!(
        store.retrieve("agent/key").expect("retrieve value"),
        Some(encrypted_test_blob())
    );
    assert_eq!(store.retrieve("missing").expect("retrieve missing value"), None);

    cleanup(&path);
}

#[test]
fn delete_removes_value_and_persists_across_reopen() {
    let path = test_file_path("delete-reopen");
    let store = FallbackStore::new(path.clone()).expect("create fallback store");
    store.store("agent/key", encrypted_test_blob()).expect("store value");
    drop(store);

    let reopened = FallbackStore::new(path.clone()).expect("reopen fallback store");
    assert_eq!(
        reopened.retrieve("agent/key").expect("retrieve persisted value"),
        Some(encrypted_test_blob())
    );
    reopened.delete("agent/key").expect("delete value");
    assert_eq!(reopened.retrieve("agent/key").expect("retrieve deleted value"), None);

    let reopened_after_delete = FallbackStore::new(path.clone()).expect("reopen after delete");
    assert_eq!(
        reopened_after_delete
            .retrieve("agent/key")
            .expect("retrieve deleted persisted value"),
        None
    );
    cleanup(&path);
}

#[test]
fn delete_of_missing_key_is_successful() {
    let path = test_file_path("delete-missing");
    let store = FallbackStore::new(path.clone()).expect("create fallback store");

    store.delete("missing").expect("deleting a missing key is idempotent");

    cleanup(&path);
}

#[test]
fn corrupt_fallback_file_is_reported() {
    let path = test_file_path("corrupt");
    std::fs::write(&path, b"not valid fallback json").expect("write corrupt fallback");

    let error = match FallbackStore::new(path.clone()) {
        Ok(_) => panic!("corrupt fallback must fail to load"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("failed to parse fallback file"));

    cleanup(&path);
}

#[test]
fn malformed_encrypted_blob_is_rejected() {
    assert!(unpack_encrypted(b"bad").is_err());

    let mut blob = encrypted_test_blob();
    blob[0] = b'X';
    assert!(unpack_encrypted(&blob).is_err());
}
