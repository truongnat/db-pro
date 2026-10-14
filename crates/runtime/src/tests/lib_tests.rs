//! Tests for `../lib.rs` (moved out of the source file).
use super::*;

#[test]
fn file_fallback_needs_a_debug_build_or_an_explicit_opt_in() {
    assert!(file_secret_fallback_enabled_for(true, None));
    assert!(file_secret_fallback_enabled_for(true, Some("0")));
    assert!(!file_secret_fallback_enabled_for(false, None));
    assert!(!file_secret_fallback_enabled_for(false, Some("")));
    assert!(!file_secret_fallback_enabled_for(false, Some("0")));
    assert!(!file_secret_fallback_enabled_for(false, Some("yes")));
    assert!(file_secret_fallback_enabled_for(false, Some("1")));
    assert!(file_secret_fallback_enabled_for(false, Some(" true ")));
    assert!(file_secret_fallback_enabled_for(false, Some("TRUE")));
}

#[test]
fn os_keyring_defaults_off_in_debug_and_on_in_release() {
    assert!(!os_keyring_enabled_for(true, None, None));
    assert!(os_keyring_enabled_for(false, None, None));
    assert!(!os_keyring_enabled_for(false, None, Some("1")));
    assert!(!os_keyring_enabled_for(false, None, Some("true")));
    assert!(os_keyring_enabled_for(true, Some("1"), None));
    assert!(
        os_keyring_enabled_for(true, Some("1"), Some("1")),
        "USE_KEYRING wins over DISABLE"
    );
}

/// Regression guard for #142: the shipping wiring selects the OS keyring when enabled, and
/// selects the encrypted-file fallback only when [`file_secret_fallback_enabled`] allows it.
#[test]
fn shipping_secret_store_follows_the_build_profile() {
    let secrets_dir = std::env::temp_dir().join(format!("db-pro-runtime-secrets-{}", std::process::id()));
    let vault = build_secret_store(secrets_dir);
    let keyring_on = os_keyring_enabled();
    let opt_in = std::env::var(ALLOW_FILE_SECRET_FALLBACK_ENV).ok();
    let expect_file = !keyring_on || file_secret_fallback_enabled_for(cfg!(debug_assertions), opt_in.as_deref());

    assert!(
        vault.session_fallback_enabled(),
        "a build without an OS keyring item must degrade to the in-memory session fallback"
    );
    assert_eq!(
        vault.file_fallback_enabled(),
        expect_file,
        "the encrypted-file fallback must follow keyring-disable + build profile / opt-in"
    );
}

/// The release half of the guard above, compiled only when `debug_assertions` is off, so
/// `cargo test -p db-pro-runtime --release` is what runs it. It fails deliberately when
/// [`ALLOW_FILE_SECRET_FALLBACK_ENV`] or [`DISABLE_KEYRING_ENV`] is set in the environment:
/// the release default is what is being pinned here, and the gate run does not set either.
#[cfg(not(debug_assertions))]
#[test]
fn release_secret_store_never_selects_the_file_fallback_by_default() {
    assert!(
        os_keyring_enabled(),
        "release tests must not set {DISABLE_KEYRING_ENV}; unset it to pin the shipping default"
    );
    let secrets_dir = std::env::temp_dir().join(format!("db-pro-runtime-release-secrets-{}", std::process::id()));
    let vault = build_secret_store(secrets_dir);

    assert!(
        vault.session_fallback_enabled(),
        "the release store keeps the in-memory session fallback so a missing keyring item is recoverable"
    );
    assert!(
        !vault.file_fallback_enabled(),
        "a release build must not open secrets.json unless {ALLOW_FILE_SECRET_FALLBACK_ENV} is set"
    );
}
