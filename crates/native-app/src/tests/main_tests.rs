//! Tests for `../main.rs` (moved out of the source file).
use super::*;
use std::path::{Path, PathBuf};

#[test]
fn data_dir_override_wins_over_every_other_candidate() {
    let chosen = choose_data_dir(
        Some(PathBuf::from("/override/state")),
        Some(Path::new("/work")),
        true,
        Some(PathBuf::from("/platform/DB Pro")),
    );

    assert_eq!(chosen, PathBuf::from("/override/state"));
}

#[test]
fn existing_legacy_dir_wins_over_the_platform_dir() {
    let chosen = choose_data_dir(
        None,
        Some(Path::new("/work")),
        true,
        Some(PathBuf::from("/platform/DB Pro")),
    );

    assert_eq!(chosen, PathBuf::from("/work/.db-pro-data"));
}

/// Regression guard for the packaged-app startup blocker: a bundle launched
/// through LaunchServices runs with `cwd=/`, where no legacy directory
/// exists, so resolving `/.db-pro-data` made `DbProRuntime::new` fail with
/// `CreateDataDir(ReadOnlyFilesystem)` and the app quit before opening a
/// window. The per-user platform directory must be chosen instead.
#[test]
fn platform_dir_is_used_when_no_legacy_dir_exists() {
    let platform_dir = PathBuf::from("/platform/DB Pro");

    let chosen = choose_data_dir(None, Some(Path::new("/")), false, Some(platform_dir.clone()));

    assert_eq!(chosen, platform_dir);
}

#[test]
fn cwd_fallback_is_used_only_when_nothing_else_resolves() {
    let chosen = choose_data_dir(None, Some(Path::new("/work")), false, None);

    assert_eq!(chosen, PathBuf::from("/work/.db-pro-data"));

    // An unresolvable working directory must degrade to a relative path
    // rather than panic.
    let chosen = choose_data_dir(None, None, false, None);

    assert_eq!(chosen, PathBuf::from(".db-pro-data"));
}

/// Regression guard for the S-2 release-hygiene finding: the developer
/// connection preset is seeded by debug builds only. `cargo test --release`
/// exercises the release half of this assertion.
#[test]
fn developer_connection_preset_is_gated_to_debug_builds() {
    let preset = developer_preset_connection();

    assert_eq!(
        preset.is_some(),
        cfg!(debug_assertions),
        "the developer connection preset must exist in debug builds only"
    );

    if let Some(config) = preset {
        assert_eq!(config.name, "Xe Lạc Hồng (PostgreSQL)");
        assert_eq!(config.database, "fullstack_starter");
        assert_eq!(config.port, 5432);
    }
}

#[test]
fn sqlite_draft_drops_postgres_only_credentials() {
    let draft = UiConnectionDraft {
        driver: UiDriver::Sqlite,
        database: "/tmp/app.sqlite".to_owned(),
        password: "should-not-cross-boundary".to_owned(),
        ssh_tunnel_enabled: true,
        ssh_host: "bastion".to_owned(),
        ..Default::default()
    };

    let (config, password) = draft_to_domain(draft).expect("valid SQLite draft");

    assert!(password.is_empty());
    assert!(config.ssh_tunnel.is_none());
}

#[test]
fn window_size_override_parses_the_documented_gate_viewports() {
    for (raw, expected) in [
        ("1280x800", [1280.0, 800.0]),
        ("1440x900", [1440.0, 900.0]),
        ("1920x1080", [1920.0, 1080.0]),
        ("1280X800", [1280.0, 800.0]),
        (" 1280 x 800 ", [1280.0, 800.0]),
    ] {
        assert_eq!(parse_window_size(raw), Some(expected), "parsing {raw:?}");
    }
}

/// A malformed override must fall back to the maximized default rather than
/// pinning a degenerate window (or panicking on the launch path).
#[test]
fn malformed_window_size_falls_back_to_maximized() {
    for raw in [
        "",
        "1280",
        "1280x",
        "x800",
        "0x800",
        "1280x0",
        "-5x800",
        "widexhigh",
        "1280x800x600",
    ] {
        assert_eq!(parse_window_size(raw), None, "expected {raw:?} to be rejected");
    }
}

#[test]
fn application_icon_asset_is_square_and_decodes_for_native_windowing() {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../../assets/db-pro-logo.png"))
        .expect("the packaged application icon must be a valid PNG");

    assert_eq!(icon.width, icon.height);
    assert_eq!(icon.width, 256);
    assert_eq!(icon.rgba.len(), (icon.width * icon.height * 4) as usize);
}
