//! Deterministic UI evidence capture.
//!
//! eframe's built-in `EFRAME_SCREENSHOT_TO` harness fires on a fixed egui pass and
//! reads the framebuffer *after* `swap_buffers`, so it captures the frame before
//! the one it was asked for. On this app that is the frame before the connection
//! list arrives, so every capture showed a start page still in its loading state.
//!
//! This driver waits for the layout to settle and then requests the screenshot
//! itself, so the PNG holds the state the run was actually asked to document.
//!
//! Enabled by `DB_PRO_CAPTURE_TO=<path.png>` together with `DB_PRO_WINDOW_SIZE`.

use std::path::PathBuf;

use db_pro_ui::DbProApp;
use eframe::egui;

/// Environment variable naming the PNG to write.
const PATH_ENV: &str = "DB_PRO_CAPTURE_TO";

/// Environment variable overriding [`SETTLE_FRAMES`], so a run can deliberately
/// document a transient state such as the loading skeletons.
const SETTLE_ENV: &str = "DB_PRO_CAPTURE_SETTLE_FRAMES";

/// Environment variable that, when set, asks the capture run to open the
/// new-connection dialog before capturing. This documents the password input and
/// eye toggle — the affected surface for the input click-steal fix — instead of the
/// default window. Inert for a normal launch.
const NEW_CONNECTION_ENV: &str = "DB_PRO_CAPTURE_NEW_CONNECTION";

/// Environment variable that opens the New Connection dialog with a stable
/// validation error for the error-state acceptance capture.
const CONNECTION_ERROR_ENV: &str = "DB_PRO_CAPTURE_CONNECTION_ERROR";

/// Environment variable that holds the Welcome connection request pending for
/// the loading-state acceptance capture.
const LOADING_ENV: &str = "DB_PRO_CAPTURE_LOADING";

/// Environment variable that, when set, asks the capture run to open the
/// Edit Connection dialog with a test draft, so the password input + eye toggle
/// on the edit surface can be documented. Inert for a normal launch.
const EDIT_CONNECTION_ENV: &str = "DB_PRO_CAPTURE_EDIT_CONNECTION";

/// When set, switch to the Query workspace before capturing (UI05 editor-first shots).
const QUERY_WORKSPACE_ENV: &str = "DB_PRO_CAPTURE_QUERY";

/// When set with [`QUERY_WORKSPACE_ENV`], force light theme for the Query capture.
const QUERY_LIGHT_ENV: &str = "DB_PRO_CAPTURE_QUERY_LIGHT";

/// When set, switch to the Table workspace before capturing (data grid / structure shots).
const TABLE_WORKSPACE_ENV: &str = "DB_PRO_CAPTURE_TABLE";

/// When set with [`TABLE_WORKSPACE_ENV`], force light theme for the Table capture.
const TABLE_LIGHT_ENV: &str = "DB_PRO_CAPTURE_TABLE_LIGHT";

/// When set with [`TABLE_WORKSPACE_ENV`], capture the deterministic Profile pane.
const TABLE_PROFILE_ENV: &str = "DB_PRO_CAPTURE_TABLE_PROFILE";

/// When set with [`TABLE_WORKSPACE_ENV`], capture the deterministic Indexes pane.
const TABLE_INDEXES_ENV: &str = "DB_PRO_CAPTURE_TABLE_INDEXES";

/// When set with [`TABLE_WORKSPACE_ENV`], focus the condition and show a
/// deterministic column completion result instead of the ordinary table state.
const TABLE_COMPLETION_ENV: &str = "DB_PRO_CAPTURE_TABLE_COMPLETION";

/// When set, switch to the ER Diagram canvas before capturing.
const DIAGRAM_WORKSPACE_ENV: &str = "DB_PRO_CAPTURE_DIAGRAM";

/// When set, switch to the Settings activity before capturing.
const SETTINGS_WORKSPACE_ENV: &str = "DB_PRO_CAPTURE_SETTINGS";

/// When set, open the Agent workspace panel before capturing.
const AGENT_WORKSPACE_ENV: &str = "DB_PRO_CAPTURE_AGENT";

/// When set, switch to the native Component Gallery before capturing.
const COMPONENT_GALLERY_ENV: &str = "DB_PRO_CAPTURE_COMPONENT_GALLERY";

/// When set, switch the sidebar to the History activity with seeded execution
/// records before capturing (unified execution history shots).
const HISTORY_ACTIVITY_ENV: &str = "DB_PRO_CAPTURE_HISTORY";

/// Show the schema-compare workspace seeded with a diff + plan recorded
/// against a divergent target (exercises the spec-09 safety lock).
const COMPARE_WORKSPACE_ENV: &str = "DB_PRO_CAPTURE_COMPARE";

/// When set, open the Explorer with a non-default object filter before
/// capturing (object-filter workbench evidence shots).
const EXPLORER_FILTER_ENV: &str = "DB_PRO_CAPTURE_FILTER";

/// When set, open Quick Open with its Schema filter selected before capturing.
const QUICK_OPEN_ENV: &str = "DB_PRO_CAPTURE_QUICK_OPEN";

/// When set with [`QUICK_OPEN_ENV`], use the light theme for the Quick Open capture.
const QUICK_OPEN_LIGHT_ENV: &str = "DB_PRO_CAPTURE_QUICK_OPEN_LIGHT";

/// When set with [`QUICK_OPEN_ENV`], show the Quick Open empty state.
const QUICK_OPEN_EMPTY_ENV: &str = "DB_PRO_CAPTURE_QUICK_OPEN_EMPTY";

/// Environment variable pinning the viewport size for evidence runs (the same key
/// `main.rs` reads for the initial window). The capture driver re-asserts it each
/// frame so the window cannot maximize itself away from the requested size.
const SIZE_ENV: &str = "DB_PRO_WINDOW_SIZE";

/// Parses [`SIZE_ENV`] into a viewport size, or `None` when unset/malformed.
fn capture_size_from_env() -> Option<egui::Vec2> {
    let raw = std::env::var(SIZE_ENV).ok()?;
    let (w, h) = raw.split_once(['x', 'X'])?;
    let w: f32 = w.trim().parse().ok()?;
    let h: f32 = h.trim().parse().ok()?;
    if w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 {
        Some(egui::Vec2::new(w, h))
    } else {
        None
    }
}

/// Frames to render before asking for a screenshot. The connection list arrives
/// asynchronously within the first couple of frames; the margin covers a viewport
/// resize landing and the first layout pass settling.
const SETTLE_FRAMES: u32 = 60;

/// The settle frame count, honouring [`SETTLE_ENV`]. A malformed or zero value
/// falls back to the default, so a typo cannot produce a blank capture.
fn settle_frames() -> u32 {
    std::env::var(SETTLE_ENV)
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|frames| *frames > 0)
        .unwrap_or(SETTLE_FRAMES)
}

/// Wraps [`DbProApp`] to write one framebuffer PNG and then close.
pub(super) struct CaptureApp {
    inner: DbProApp,
    path: PathBuf,
    settle: u32,
    frames: u32,
    requested: bool,
    opened_dialog: bool,
    prepared_loading: bool,
    pinned: Option<egui::Vec2>,
}

impl CaptureApp {
    /// Wraps `app` when [`PATH_ENV`] is set, otherwise hands it back untouched so a
    /// normal launch carries no capture behaviour at all.
    pub(super) fn wrap(inner: DbProApp) -> Box<dyn eframe::App> {
        match std::env::var_os(PATH_ENV).filter(|value| !value.is_empty()) {
            Some(raw) => Box::new(Self {
                inner,
                path: PathBuf::from(raw),
                settle: settle_frames(),
                frames: 0,
                requested: false,
                opened_dialog: false,
                prepared_loading: false,
                pinned: capture_size_from_env(),
            }),
            None => Box::new(inner),
        }
    }

    /// Writes a captured frame to `self.path`, reporting the outcome. Named
    /// `write_png` rather than `save` because `eframe::App` already defines `save`.
    fn write_png(&self, image: &egui::ColorImage) -> bool {
        let [width, height] = image.size;
        let Ok(width) = u32::try_from(width) else {
            tracing::error!(path = %self.path.display(), width, "capture: framebuffer width exceeds PNG limits");
            return false;
        };
        let Ok(height) = u32::try_from(height) else {
            tracing::error!(path = %self.path.display(), height, "capture: framebuffer height exceeds PNG limits");
            return false;
        };
        let rgba: Vec<u8> = image.pixels.iter().flat_map(|pixel| pixel.to_array()).collect();
        match image::save_buffer(&self.path, &rgba, width, height, image::ColorType::Rgba8) {
            Ok(()) => {
                tracing::info!(path = %self.path.display(), width, height, "capture: wrote framebuffer");
                true
            }
            Err(err) => {
                tracing::error!(path = %self.path.display(), %err, "capture: failed to write framebuffer");
                false
            }
        }
    }
}

impl eframe::App for CaptureApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.prepare_loading();
        self.inner.update(ctx, frame);
        self.open_requested_surface();
        self.pin_viewport(ctx);
        if self.capture_screenshot(ctx) {
            return;
        }
        self.advance_capture(ctx);
    }
}

impl CaptureApp {
    fn prepare_loading(&mut self) {
        if !self.prepared_loading && std::env::var_os(LOADING_ENV).is_some() {
            self.inner.prepare_loading_for_capture();
            self.prepared_loading = true;
        }
    }

    fn open_requested_surface(&mut self) {
        if self.opened_dialog || self.frames < 2 {
            return;
        }
        // Evidence hook: when asked, open the new-connection dialog so the capture
        // documents the password input + eye toggle (the affected surface for the
        // input click-steal fix) instead of the default window. Gated by an env var
        // so a normal launch is unaffected.
        if std::env::var_os(NEW_CONNECTION_ENV).is_some() {
            self.inner.open_new_connection_for_capture();
            self.opened_dialog = true;
        } else if std::env::var_os(CONNECTION_ERROR_ENV).is_some() {
            self.inner.open_connection_error_for_capture();
            self.opened_dialog = true;
        } else if std::env::var_os(EDIT_CONNECTION_ENV).is_some() {
            self.inner.open_edit_connection_for_capture();
            self.opened_dialog = true;
        } else if std::env::var_os(QUERY_WORKSPACE_ENV).is_some() {
            if std::env::var_os(QUERY_LIGHT_ENV).is_some() {
                self.inner.open_query_workspace_for_capture_light();
            } else {
                self.inner.open_query_workspace_for_capture();
            }
            self.opened_dialog = true;
        } else if std::env::var_os(TABLE_WORKSPACE_ENV).is_some() {
            if std::env::var_os(TABLE_PROFILE_ENV).is_some() {
                self.inner
                    .open_table_profile_for_capture(std::env::var_os(TABLE_LIGHT_ENV).is_some());
            } else if std::env::var_os("DB_PRO_CAPTURE_TABLE_STRUCTURE").is_some() {
                self.inner
                    .open_table_structure_for_capture(std::env::var_os(TABLE_LIGHT_ENV).is_some());
            } else if std::env::var_os("DB_PRO_CAPTURE_TABLE_FOREIGN_KEYS").is_some() {
                self.inner
                    .open_table_foreign_keys_for_capture(std::env::var_os(TABLE_LIGHT_ENV).is_some());
            } else if std::env::var_os("DB_PRO_CAPTURE_TABLE_CONSTRAINTS").is_some() {
                self.inner
                    .open_table_constraints_for_capture(std::env::var_os(TABLE_LIGHT_ENV).is_some());
            } else if std::env::var_os("DB_PRO_CAPTURE_TABLE_DEPENDENCIES").is_some() {
                self.inner
                    .open_table_dependencies_for_capture(std::env::var_os(TABLE_LIGHT_ENV).is_some());
            } else if std::env::var_os(TABLE_INDEXES_ENV).is_some() {
                self.inner
                    .open_table_indexes_for_capture(std::env::var_os(TABLE_LIGHT_ENV).is_some());
            } else if std::env::var_os(TABLE_COMPLETION_ENV).is_some() {
                self.inner
                    .open_table_condition_completion_for_capture(std::env::var_os(TABLE_LIGHT_ENV).is_some());
            } else if std::env::var_os(TABLE_LIGHT_ENV).is_some() {
                self.inner.open_table_workspace_for_capture_light();
            } else {
                self.inner.open_table_workspace_for_capture();
            }
            self.opened_dialog = true;
        } else if std::env::var_os(DIAGRAM_WORKSPACE_ENV).is_some() {
            self.inner.open_diagram_workspace_for_capture();
            self.opened_dialog = true;
        } else if std::env::var_os(SETTINGS_WORKSPACE_ENV).is_some() {
            self.inner.open_settings_workspace_for_capture();
            self.opened_dialog = true;
        } else if std::env::var_os(AGENT_WORKSPACE_ENV).is_some() {
            self.inner.open_agent_workspace_for_capture();
            self.opened_dialog = true;
        } else if std::env::var_os(COMPONENT_GALLERY_ENV).is_some() {
            self.inner.open_component_gallery_for_capture();
            self.opened_dialog = true;
        } else if std::env::var_os(EXPLORER_FILTER_ENV).is_some() {
            self.inner.open_explorer_filter_for_capture();
            self.opened_dialog = true;
        } else if std::env::var_os(HISTORY_ACTIVITY_ENV).is_some() {
            self.inner.open_history_activity_for_capture();
            self.opened_dialog = true;
        } else if std::env::var_os(COMPARE_WORKSPACE_ENV).is_some() {
            self.inner.open_schema_compare_for_capture();
            self.opened_dialog = true;
        } else if std::env::var_os(QUICK_OPEN_ENV).is_some() {
            self.inner.open_quick_open_for_capture(
                std::env::var_os(QUICK_OPEN_LIGHT_ENV).is_some(),
                std::env::var_os(QUICK_OPEN_EMPTY_ENV).is_some(),
            );
            self.opened_dialog = true;
        }
    }

    fn pin_viewport(&self, ctx: &egui::Context) {
        // Re-assert the pinned viewport size every frame so the window cannot
        // maximize itself away from the requested evidence size.
        if let Some(size) = self.pinned {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
        }
    }

    fn capture_screenshot(&self, ctx: &egui::Context) -> bool {
        // The reply to `ViewportCommand::Screenshot`.
        let captured = ctx.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = captured {
            self.write_png(&image);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            true
        } else {
            false
        }
    }

    fn advance_capture(&mut self, ctx: &egui::Context) {
        self.frames += 1;
        if self.frames >= self.settle && !self.requested {
            self.requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
        }
        // The app repaints on demand, so without this the settle count would stall
        // on an idle frame and the screenshot would never be requested.
        ctx.request_repaint();
    }
}
