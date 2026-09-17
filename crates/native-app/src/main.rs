use std::error::Error;
use std::sync::mpsc::Sender;
use std::thread;

use db_pro_core::application::sql_builder::{FilterOp, SortClause, SortDir, TableFilter};
use db_pro_core::domain::query::CellValue;
use db_pro_runtime::{spawn_worker, DbProRuntime, RuntimeCommand, RuntimeEvent, RuntimeRequestId};
use db_pro_ui::{
    AgentMessage, AgentRole, DbProApp, DbProTheme, RequestId, TaskBridge, UiCell, UiCheckConstraint, UiColumn,
    UiCommand, UiConnectionDraft, UiConnectionSummary, UiDependencyDirection, UiDependencyKind, UiDriver, UiEvent,
    UiFunctionSummary, UiQueryError, UiQueryExecutionOutput, UiQueryFolderSummary, UiQueryResult, UiSavedQuerySummary,
    UiSchemaColumn, UiSchemaForeignKey, UiSchemaSummary, UiSslMode, UiStatementOutput, UiTableColumn,
    UiTableDataFilter, UiTableDataSort, UiTableDependency, UiTableFilterOperator, UiTableForeignKey, UiTableIndex,
    UiTableInfo, UiTableMutation, UiTableSummary, UiTriggerSummary, UiViewSummary,
};
use eframe::egui;
use tokio::runtime::Builder;

#[cfg(feature = "capture")]
mod capture;
mod translate;
#[cfg(test)]
mod translate_tests;

#[cfg(test)]
pub(crate) use translate::draft_to_domain;
use translate::{translate_command, translate_event};

fn main() -> Result<(), Box<dyn Error>> {
    init_tracing();
    // Local cargo binaries must not poke the OS keyring — every launch was prompting
    // Keychain / Credential Manager. Packaged installs keep the default (keyring on).
    configure_dev_secret_store();
    let tokio_runtime = Builder::new_multi_thread().enable_all().build()?;
    let data_dir = resolve_data_dir();
    let (bridge, command_rx, event_tx) = TaskBridge::with_channels();

    let (runtime_tx, runtime_rx) = tokio_runtime.block_on(async {
        let runtime = DbProRuntime::new(data_dir).await?;
        seed_default_connection(&runtime).await;
        Ok::<_, Box<dyn Error>>(spawn_worker(runtime, 64))
    })?;

    let command_runtime_tx = runtime_tx.clone();
    let command_handle = tokio_runtime.handle().clone();
    let picker_event_tx = event_tx.clone();
    thread::spawn(move || {
        let send_picked = |request_id, kind: &str, path: Option<String>| {
            // Fire-and-forget: a picker result is only dropped once the UI has
            // gone away, which means the whole app is shutting down.
            let _ = picker_event_tx.send(UiEvent::FilePicked {
                request_id,
                kind: kind.to_owned(),
                path,
            });
        };
        while let Ok(command) = command_rx.recv() {
            match command {
                UiCommand::PickSqliteFile { request_id } => {
                    let path = rfd::FileDialog::new()
                        .add_filter("SQLite database", &["db", "sqlite", "sqlite3"])
                        .pick_file()
                        .map(|path| path.to_string_lossy().into_owned());
                    send_picked(request_id, "sqlite", path);
                    continue;
                }
                UiCommand::PickBackupFile { request_id } => {
                    let path = rfd::FileDialog::new()
                        .set_title("Choose backup output")
                        .save_file()
                        .map(|path| path.to_string_lossy().into_owned());
                    send_picked(request_id, "backup", path);
                    continue;
                }
                UiCommand::PickRestoreFile { request_id } => {
                    let path = rfd::FileDialog::new()
                        .set_title("Choose backup to restore")
                        .pick_file()
                        .map(|path| path.to_string_lossy().into_owned());
                    send_picked(request_id, "restore", path);
                    continue;
                }
                UiCommand::PickWorkspaceFolder { request_id } => {
                    let path = rfd::FileDialog::new()
                        .set_title("Open workspace folder")
                        .pick_folder()
                        .map(|path| path.to_string_lossy().into_owned());
                    send_picked(request_id, "workspace-folder", path);
                    continue;
                }
                UiCommand::PickSshPrivateKey { request_id } => {
                    let path = rfd::FileDialog::new()
                        .pick_file()
                        .map(|path| path.to_string_lossy().into_owned());
                    send_picked(request_id, "ssh-key", path);
                    continue;
                }
                command => {
                    let Some(command) = translate_command(command) else {
                        continue;
                    };
                    let send_result = command_handle.block_on(command_runtime_tx.send(command));
                    if send_result.is_err() {
                        break;
                    }
                }
            }
        }
    });

    spawn_event_pump(runtime_rx, event_tx, tokio_runtime.handle().clone());

    run_native_app(bridge)
}

fn init_tracing() {
    // Best-effort: a global tracing subscriber may already be installed when
    // the app is embedded in a host process, which is not a failure.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "db_pro_runtime=info".to_owned()))
        .try_init();
}

/// Name of the legacy, working-directory-local state directory.
const LEGACY_DATA_DIR_NAME: &str = ".db-pro-data";
/// Application directory name used inside the per-user platform data root.
const PLATFORM_APP_DIR_NAME: &str = "DB Pro";

/// Resolves the directory holding the application state (`meta.db`, `secrets/`).
///
/// Precedence: `DB_PRO_DATA_DIR`, then an *existing* working-directory-local
/// `.db-pro-data`, then the per-user platform data directory. A bundle launched
/// through LaunchServices runs with `/` as its working directory, so a legacy
/// directory that does not already exist is never created there: that failed
/// with `CreateDataDir(Os { code: 30, kind: ReadOnlyFilesystem })` and the app
/// quit before it opened a window.
fn resolve_data_dir() -> std::path::PathBuf {
    let override_dir = non_empty_env("DB_PRO_DATA_DIR").map(std::path::PathBuf::from);
    let cwd = std::env::current_dir().ok();
    let legacy_exists = cwd
        .as_deref()
        .is_some_and(|dir| dir.join(LEGACY_DATA_DIR_NAME).is_dir());

    choose_data_dir(override_dir, cwd.as_deref(), legacy_exists, platform_data_dir())
}

/// The precedence rule behind [`resolve_data_dir`], free of process-global state
/// so it can be exercised directly.
fn choose_data_dir(
    override_dir: Option<std::path::PathBuf>,
    cwd: Option<&std::path::Path>,
    legacy_exists: bool,
    platform_dir: Option<std::path::PathBuf>,
) -> std::path::PathBuf {
    if let Some(dir) = override_dir {
        return dir;
    }
    if legacy_exists {
        if let Some(dir) = cwd {
            return dir.join(LEGACY_DATA_DIR_NAME);
        }
    }
    if let Some(dir) = platform_dir {
        return dir;
    }
    cwd.map(|dir| dir.join(LEGACY_DATA_DIR_NAME))
        .unwrap_or_else(|| std::path::PathBuf::from(LEGACY_DATA_DIR_NAME))
}

/// Per-user state root offered by the platform, if one can be resolved.
fn platform_data_dir() -> Option<std::path::PathBuf> {
    if cfg!(target_os = "macos") {
        non_empty_env("HOME").map(|home| {
            std::path::PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join(PLATFORM_APP_DIR_NAME)
        })
    } else if cfg!(target_os = "windows") {
        non_empty_env("APPDATA").map(|appdata| std::path::PathBuf::from(appdata).join(PLATFORM_APP_DIR_NAME))
    } else {
        non_empty_env("XDG_DATA_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| non_empty_env("HOME").map(|home| std::path::PathBuf::from(home).join(".local").join("share")))
            .map(|data_root| data_root.join("db-pro"))
    }
}

/// Reads an environment variable, treating an empty value as unset.
fn non_empty_env(key: &str) -> Option<std::ffi::OsString> {
    std::env::var_os(key).filter(|value| !value.is_empty())
}

/// For binaries launched from `target/debug` or `target/release`, disable the OS keyring
/// unless the developer explicitly opted in. Packaged installs are unaffected.
fn configure_dev_secret_store() {
    use db_pro_runtime::{DISABLE_KEYRING_ENV, USE_KEYRING_ENV};

    if non_empty_env(USE_KEYRING_ENV).is_some() || non_empty_env(DISABLE_KEYRING_ENV).is_some() {
        return;
    }
    if !is_cargo_target_binary() {
        return;
    }
    // SAFETY: single-threaded — called before the tokio runtime is built.
    std::env::set_var(DISABLE_KEYRING_ENV, "1");
    tracing::info!("cargo-target binary: OS keyring disabled (set {USE_KEYRING_ENV}=1 to enable Keychain prompts)");
}

fn is_cargo_target_binary() -> bool {
    std::env::current_exe()
        .ok()
        .as_ref()
        .and_then(|path| path.to_str())
        .is_some_and(|path| {
            path.contains("/target/debug/")
                || path.contains("/target/release/")
                || path.contains("\\target\\debug\\")
                || path.contains("\\target\\release\\")
        })
}

/// The developer-convenience connection seeded on first launch, if any.
///
/// Debug builds seed the developer's local PostgreSQL fixture so `cargo run`
/// starts with a usable connection. Release builds must not: a shipped binary
/// must never write a developer's private connection metadata into a user's
/// connection store (goal-3 §16 — never package local connection metadata or
/// credentials).
#[cfg(debug_assertions)]
fn developer_preset_connection() -> Option<db_pro_core::domain::connection::ConnectionConfig> {
    Some(db_pro_core::domain::connection::ConnectionConfig {
        name: "Xe Lạc Hồng (PostgreSQL)".to_owned(),
        host: "localhost".to_owned(),
        port: 5432,
        database: "fullstack_starter".to_owned(),
        username: "postgres".to_owned(),
        driver: db_pro_core::domain::connection::DriverType::Postgres,
        ssl_mode: db_pro_core::domain::connection::SslMode::Disable,
        ssh_tunnel: None,
        ssh_profile_id: None,
        ssl_root_cert_path: None,
        ssl_client_cert_path: None,
        ssl_client_key_path: None,
        query_timeout_ms: 30_000,
        max_rows: 500,
        color: Some("#6366f1".to_owned()),
        tags: vec!["docker".to_owned(), "xe-lac-hong".to_owned()],
        group: None,
        favorite: false,
        environment: Default::default(),
        readonly: false,
    })
}

/// Release builds seed nothing — see `developer_preset_connection`.
#[cfg(not(debug_assertions))]
fn developer_preset_connection() -> Option<db_pro_core::domain::connection::ConnectionConfig> {
    None
}

/// Log text for a failed seeding attempt, kept next to the preset it names so
/// the developer label does not survive into a release binary.
#[cfg(debug_assertions)]
const SEED_FAILURE_MESSAGE: &str = "failed to seed default Xe Lạc Hồng connection";
#[cfg(not(debug_assertions))]
const SEED_FAILURE_MESSAGE: &str = "failed to seed the developer default connection";

/// Seeds the demo PostgreSQL connection the first time the app runs.
async fn seed_default_connection(runtime: &DbProRuntime) {
    let Some(config) = developer_preset_connection() else {
        return;
    };
    let database = config.database.clone();
    let port = config.port;
    let existing = runtime.connections().list().await.unwrap_or_default();
    if existing
        .iter()
        .any(|c| c.config.database == database && c.config.port == port)
    {
        return;
    }
    if let Err(err) = runtime.connections().create(config, "postgres").await {
        tracing::warn!("{SEED_FAILURE_MESSAGE}: {err}");
    }
}

/// Forwards translated runtime events to the UI, stopping when the UI is gone.
fn spawn_event_pump(
    mut runtime_rx: tokio::sync::mpsc::Receiver<RuntimeEvent>,
    event_tx: Sender<UiEvent>,
    event_handle: tokio::runtime::Handle,
) {
    event_handle.spawn(async move {
        while let Some(event) = runtime_rx.recv().await {
            if let Some(event) = translate_event(event) {
                if event_tx.send(event).is_err() {
                    break;
                }
            }
        }
    });
}

/// Pins the initial window size from `DB_PRO_WINDOW_SIZE` (`1280x800`), instead of
/// maximizing.
///
/// UI acceptance evidence has to be captured at exact viewports
/// (`docs/10-egui-native-migration-plan.md`), and a maximized window renders at
/// whatever the monitor happens to be. Unset or malformed means "maximize as usual",
/// so this is inert for a normal launch.
fn capture_window_size() -> Option<[f32; 2]> {
    let raw = non_empty_env("DB_PRO_WINDOW_SIZE")?;
    parse_window_size(&raw.to_string_lossy())
}

/// The parsing half of [`capture_window_size`], free of process-global state.
fn parse_window_size(raw: &str) -> Option<[f32; 2]> {
    let (width, height) = raw.split_once(['x', 'X'])?;
    let width: f32 = width.trim().parse().ok()?;
    let height: f32 = height.trim().parse().ok()?;
    if width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0 {
        Some([width, height])
    } else {
        None
    }
}

fn run_native_app(bridge: TaskBridge) -> Result<(), Box<dyn Error>> {
    let pinned_size = capture_window_size();
    tracing::info!(?pinned_size, "capture: resolved window size override");
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("DB Pro")
        .with_min_inner_size([1024.0, 640.0]);
    viewport = match pinned_size {
        Some(size) => viewport.with_inner_size(size),
        None => viewport.with_maximized(true),
    };

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "DB Pro",
        options,
        Box::new(move |creation_context| {
            // Re-apply the product default after eframe restores its persisted window
            // frame — unless a capture run pinned an exact viewport.
            if pinned_size.is_none() {
                creation_context
                    .egui_ctx
                    .send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            }
            DbProTheme::install_fonts(&creation_context.egui_ctx);
            let app = DbProApp::with_task_bridge_and_storage(bridge, creation_context.storage);
            Ok(wrap_for_capture(app))
        }),
    )?;
    Ok(())
}

/// Wraps the app in the evidence capture driver when one was requested.
#[cfg(feature = "capture")]
fn wrap_for_capture(app: DbProApp) -> Box<dyn eframe::App> {
    capture::CaptureApp::wrap(app)
}

/// Without the `capture` feature the app runs unwrapped.
#[cfg(not(feature = "capture"))]
fn wrap_for_capture(app: DbProApp) -> Box<dyn eframe::App> {
    Box::new(app)
}

#[cfg(test)]
mod tests {
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
}
