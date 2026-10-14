//! Tests for `../pg_settings.rs` (moved out of the source file).
use super::*;

#[test]
fn sensitive_names_detected() {
    assert!(is_sensitive_setting_name("ssl_key_file"));
    assert!(!is_sensitive_setting_name("work_mem"));
}

#[test]
fn preview_alter_system_quotes_value() {
    let preview = preview_alter_system("work_mem", "64MB").unwrap();
    assert!(preview.sql.contains("ALTER SYSTEM SET work_mem = '64MB'"));
    assert!(preview_alter_system("ssl_key_file", "x").is_err());
}

#[test]
fn session_mutable_contexts() {
    let mut s = PgSetting {
        name: "work_mem".into(),
        setting: "4MB".into(),
        unit: Some("kB".into()),
        category: "Resource Usage".into(),
        short_desc: None,
        context: "user".into(),
        vartype: "string".into(),
        source: "default".into(),
        min_val: None,
        max_val: None,
        enumvals: None,
        boot_val: None,
        reset_val: None,
        pending_restart: false,
        sensitive: false,
    };
    assert!(s.session_mutable());
    s.context = "postmaster".into();
    assert!(!s.session_mutable());
}
