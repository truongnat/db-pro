//! Tests for `../secret.rs` (moved out of the source file).
use super::*;

#[test]
fn empty_secret_has_none() {
    let s = ConnectionSecret::empty();
    assert!(!s.has_any());
}

#[test]
fn with_password_has_any() {
    let s = ConnectionSecret::with_password("secret123");
    assert!(s.has_any());
}

#[test]
fn redacted_hides_values() {
    let s = ConnectionSecret {
        password: Some("hunter2".into()),
        ssh_password: Some("ssh_pass".into()),
        ssh_passphrase: None,
    };
    let r = s.redacted();
    assert!(r.has_password);
    assert!(r.has_ssh_password);
    assert!(!r.has_ssh_passphrase);
}

#[test]
fn redacted_display_format() {
    let s = ConnectionSecret::with_password("secret");
    let display = s.redacted().to_string();
    assert!(display.contains("password=***"));
    assert!(!display.contains("secret"));
}

#[test]
fn redacted_empty_display() {
    let s = ConnectionSecret::empty();
    assert_eq!(s.redacted().to_string(), "(no secrets)");
}

#[test]
fn secret_key_format() {
    assert_eq!(secret_key("abc-123", fields::PASSWORD), "connection:abc-123:password");
}
