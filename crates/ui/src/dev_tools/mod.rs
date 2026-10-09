// cc-scan:allow LONG_FUNCTION — the inspector window is a linear list of collapsing sections.
//! Debug-build-only UI inspector (developer tools panel).
//!
//! Backed by egui 0.36's built-in inspection APIs —
//! [`egui::Context::inspection_ui`], [`egui::DebugOptions`], and
//! `ViewportState::hits`/`prev_pass.widgets` widget hit-testing. No audit rules
//! or analysis live here; they build on this state later.
//!
//! Compiled out of release builds (`debug_assertions`), so production UI and
//! behavior are unchanged. Toggle with `mod+shift+i` or start it open with the
//! `DB_PRO_INSPECTOR` environment variable (used by capture runs).

use egui::{Align2, Color32, Pos2, RichText, ScrollArea, Ui, Window};

pub mod audit;
pub mod geometry;
pub mod report;
pub mod screen;
pub mod semantic;

use crate::dev_tools::audit::{AuditIssue, AuditReport};
use crate::dev_tools::geometry::{is_area_layer, transform_debug, GeometrySnapshot, SelectedWidget, WidgetGeometry};
use crate::dev_tools::semantic::{AccessKitCapture, SemanticSnapshot};
use crate::theme::DbProTheme;

/// Environment variable that opens the inspector on the first frame.
/// Lets deterministic capture runs document the panel without input injection.
const OPEN_ON_START_ENV: &str = "DB_PRO_INSPECTOR";

/// `x,y` in window points: programmatically pin a widget selection, so capture
/// runs can document the selection/highlight path without input injection.
const PICK_ENV: &str = "DB_PRO_INSPECTOR_PICK";

/// Paints a deliberately-broken fixture (overlapping + zero-size widgets) so
/// the audit has deterministic runtime evidence. Debug-only; no-op in release
/// captures and ignored when unset.
const AUDIT_FIXTURE_ENV: &str = "DB_PRO_INSPECTOR_AUDIT_FIXTURE";

/// Prints every audit issue (rule / role / widget / evidence) to stderr once —
/// used to classify findings during development runs.
const AUDIT_DUMP_ENV: &str = "DB_PRO_INSPECTOR_DUMP";

/// The inspector window's [`egui::Id`]; its layer id filters inspector widgets
/// out of every geometry listing and pick.
fn inspector_id() -> egui::Id {
    egui::Id::new("dbpro_dev_tools_inspector")
}

/// State for the developer-tools inspector.
pub struct DevToolsState {
    /// Whether the inspector window is open.
    pub open: bool,
    /// Pinned widget selection; persists while the pointer moves and is
    /// re-resolved each frame so the highlight follows scrolling/resizing.
    pub selected: Option<SelectedWidget>,
    /// Programmatic pick requested via [`PICK_ENV`], consumed on first draw.
    pending_pick: Option<Pos2>,
    /// Latest rule-engine result over the last snapshot; `None` until the
    /// first audited frame. Recomputed every draw — a widget that disappears
    /// drops out of `issues` automatically.
    audit: Option<AuditReport>,
    /// Previous frame's audit — the baseline for issue identity diffs
    /// (new/resolved), only compared when the widget set is equivalent.
    prev_audit: Option<AuditReport>,
    /// Set after the first `DB_PRO_INSPECTOR_DUMP` dump so it prints once.
    audit_dumped: bool,
    /// Frames audited since open — lets the dump wait for warm semantic data
    /// but still fire if it never arrives.
    audit_frames: usize,
    /// Headless audit emit (`DB_PRO_AUDIT_JSON`): consumed by the capture
    /// driver via `take_audit_emission` — `DevToolsState` owns the pipeline,
    /// so the report + exit decision live here too.
    pending_audit: Option<(serde_json::Value, i32)>,
    /// Consecutive frames whose layout fingerprint matched the previous one —
    /// the stability signal the headless emit waits for.
    stable_streak: usize,
}

impl Default for DevToolsState {
    fn default() -> Self {
        Self {
            open: std::env::var_os(OPEN_ON_START_ENV).is_some_and(|v| !v.is_empty() && v != "0"),
            selected: None,
            pending_pick: parse_pick_env(),
            audit: None,
            prev_audit: None,
            audit_dumped: false,
            audit_frames: 0,
            pending_audit: None,
            stable_streak: 0,
        }
    }
}

fn parse_pick_env() -> Option<Pos2> {
    let raw = std::env::var_os(PICK_ENV)?;
    let raw = raw.to_string_lossy();
    let (x, y) = raw.trim().split_once(',')?;
    Some(Pos2::new(x.trim().parse().ok()?, y.trim().parse().ok()?))
}

impl DevToolsState {
    /// Draw the inspector window, refresh the widget snapshot, and paint the
    /// selection highlight. Cheap no-op when closed.
    /// Take a pending headless-audit emission (JSON + exit code), if the last
    /// `draw` produced one. `None` when `DB_PRO_AUDIT_JSON` isn't set or the
    /// pipeline isn't warm yet.
    pub fn take_audit_emission(&mut self) -> Option<(serde_json::Value, i32)> {
        self.pending_audit.take()
    }

    pub fn draw(&mut self, ctx: &egui::Context, theme: DbProTheme) {
        // Headless audit (`DB_PRO_AUDIT_JSON`) runs the pipeline with the
        // window closed — same snapshots and rules, no inspector UI.
        let headless = std::env::var_os(report::OUTPUT_ENV).is_some_and(|v| !v.is_empty());
        if !self.open && !headless {
            return;
        }

        if audit_fixture_enabled() {
            paint_audit_fixture(ctx);
        }
        if let Some(mode) = screen_fixture_mode() {
            paint_screen_fixture(ctx, &mode);
        }

        // AccessKit tree feeds the semantic rules; plugin + flag are
        // idempotent, and updates start flowing the pass after enabling.
        AccessKitCapture::ensure_installed(ctx);
        let semantic = AccessKitCapture::latest(ctx)
            .as_ref()
            .and_then(SemanticSnapshot::from_update);

        let snapshot = GeometrySnapshot::collect(ctx, inspector_id());
        self.update_selection(ctx, &snapshot);
        self.paint_highlight(ctx);
        let report = audit::run_audit(ctx, &snapshot, semantic.as_ref());
        self.audit_frames += 1;
        // Wait for warm data: a widget set plus the accesskit tree (which lags
        // a frame or two). Fall back after 60 frames so the dump can't miss.
        // Warm = the audited snapshot is STABLE (same layout fingerprint two
        // frames in a row — an Area's first-paint sizing pass registers
        // children `enabled=false`, so emitting on a fixed frame number could
        // audit that fake snapshot) AND the accesskit tree has data (it lags
        // a frame or two). 120-frame fallback covers accesskit-off.
        // Snapshots describe the previous pass, so a surface opened this frame
        // only shows up next frame: a change can never hide inside a run of
        // three identical fingerprints — one matching frame isn't enough.
        let stable = self
            .audit
            .as_ref()
            .is_some_and(|prev| prev.layout_fingerprint == report.layout_fingerprint);
        self.stable_streak = if stable { self.stable_streak + 1 } else { 0 };
        // Readiness gate (`DB_PRO_AUDIT_READY=<min-widgets>`): scenario-defined
        // minimum, checked in addition to stability — deterministic per surface.
        let ready_min = std::env::var(report::READY_ENV)
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1);
        let warm = report.widget_count >= ready_min
            && (self.stable_streak >= 2 || self.audit_frames > 30)
            && (report
                .rules
                .iter()
                .any(|r| r.rule_id == "semantic.missing_name" && r.stats.evaluated > 0)
                || self.audit_frames > 120);
        if !self.audit_dumped && warm && std::env::var_os(AUDIT_DUMP_ENV).is_some_and(|v| !v.is_empty() && v != "0") {
            self.audit_dumped = true;
            dump_audit_issues(&report, &snapshot);
        }
        // Headless mode (`DB_PRO_AUDIT_JSON`): render the report the frame the
        // pipeline goes warm — the capture driver collects it via
        // `take_audit_emission` and exits with the verdict code.
        if self.pending_audit.is_none() && warm && std::env::var_os(report::OUTPUT_ENV).is_some_and(|v| !v.is_empty()) {
            let rc = report::ReportContext {
                viewport: std::env::var("DB_PRO_WINDOW_SIZE").unwrap_or_else(|_| "unknown".to_owned()),
                fixture: audit_fixture_enabled(),
                frames: self.audit_frames as u32,
                scenario: std::env::var(report::SCENARIO_ENV).unwrap_or_else(|_| "shell".to_owned()),
                theme_label: if theme.dark_mode {
                    "Dark".to_owned()
                } else {
                    "Light".to_owned()
                },
            };
            let mut json = report::render_report(&report, ctx, &rc);
            if let Some(base) = std::env::var_os(report::BASELINE_ENV) {
                if let Ok(sig) = serde_json::from_value::<report::Signature>(json["signature"].clone()) {
                    let diff =
                        report::diff_with_baseline(std::path::Path::new(&base), &sig, &report::issue_keys(&report));
                    json["baseline"] = match diff {
                        Ok(d) => d.to_json(),
                        Err(e) => serde_json::json!({"comparable": false, "error": e}),
                    };
                } else {
                    json["baseline"] = serde_json::json!({"comparable": false, "reason": "no local signature"});
                }
            }
            self.pending_audit = Some((json, report::exit_code(&report)));
        }
        self.prev_audit = self.audit.replace(report);

        if !self.open {
            // Headless: pipeline ran, emission captured — no inspector window.
            return;
        }
        let mut open = self.open;
        Window::new("🛠 Inspector")
            .id(inspector_id())
            .open(&mut open)
            .default_size(egui::vec2(380.0, 480.0))
            .vscroll(false)
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .id_salt("dbpro_dev_tools_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        draw_selected_section(ui, ctx, &mut self.selected, &snapshot, theme);
                        draw_audit_section(
                            ui,
                            self.audit.as_ref(),
                            self.prev_audit.as_ref(),
                            &mut self.selected,
                            &snapshot,
                            theme,
                        );
                        draw_widget_hits(ui, ctx, theme);
                        draw_debug_paint_toggles(ui, ctx);
                        egui::CollapsingHeader::new(section("egui state"))
                            .id_salt("dbpro_dev_tools_egui_state")
                            .default_open(false)
                            .show(ui, |ui| ctx.inspection_ui(ui));
                    });
            });
        self.open = open;
    }

    /// Apply a pending programmatic pick, a real pointer press, or Escape.
    /// The inspector's own widgets are never pickable.
    fn update_selection(&mut self, ctx: &egui::Context, snapshot: &GeometrySnapshot) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.selected = None;
            return;
        }

        if let Some(pos) = self.pending_pick {
            // Programmatic pick: widgets are registered on the *next* pass, so
            // a pick that finds nothing yet must retry rather than clear.
            if let Some(picked) = snapshot.pick(ctx, pos) {
                self.pending_pick = None;
                self.selected = Some(SelectedWidget {
                    picked: picked.clone(),
                    live: None,
                });
            }
        } else if let Some(pos) = ctx.input(|i| {
            if i.pointer.primary_pressed() {
                i.pointer.interact_pos()
            } else {
                None
            }
        }) {
            self.selected = snapshot.pick(ctx, pos).map(|picked| SelectedWidget {
                picked: picked.clone(),
                live: None,
            });
            // Clicks on empty space produce None → clears; inspector clicks
            // never reach here because its layer is filtered out of `snapshot`.
        }

        // Keep the selection pinned to the live widget so the highlight tracks
        // scrolling and layout shifts; remember it disappeared if it vanishes.
        if let Some(selected) = &mut self.selected {
            selected.live = snapshot.find(selected.picked.id).cloned();
        }
    }

    /// Stroke the selected widget's live rect on the debug layer.
    fn paint_highlight(&self, ctx: &egui::Context) {
        let Some(selected) = &self.selected else {
            return;
        };
        let current = selected.current();
        let painter = ctx.debug_painter();
        let rect = current.to_global_rect(ctx);
        let interact = ctx
            .layer_transform_to_global(current.layer)
            .map(|t| t * current.interact_rect.to_egui())
            .unwrap_or_else(|| current.interact_rect.to_egui());
        if interact != rect {
            painter.debug_rect(interact, Color32::from_rgb(255, 171, 0), "");
        }
        painter.debug_rect(rect, Color32::from_rgb(255, 60, 60), "");
        painter.debug_text(
            rect.left_bottom() + egui::vec2(0.0, 4.0),
            Align2::LEFT_TOP,
            Color32::from_rgb(255, 200, 80),
            current.id_debug.as_str(),
        );
        ctx.request_repaint();
    }
}

fn section(title: &str) -> RichText {
    RichText::new(format!("🔧 {title}")).strong()
}

/// Live checkboxes for egui `Options` (screen reader, id-clash warnings, zoom,
/// `Style.debug` overlays under the 🐛 Debug header). Mutates a clone and writes
/// back on change — drawing widgets inside `all_styles_mut` would deadlock the
/// context lock.
fn draw_debug_paint_toggles(ui: &mut Ui, ctx: &egui::Context) {
    egui::CollapsingHeader::new(section("Debug paint"))
        .id_salt("dbpro_dev_tools_debug_paint")
        .default_open(true)
        .show(ui, |ui| {
            let prev = ctx.options(|o| o.clone());
            let mut options = prev.clone();
            options.ui(ui);
            if options != prev {
                ctx.options_mut(|o| *o = options);
            }
        });
}

/// Pinned selection detail: identity, geometry, clipping, semantics.
fn draw_selected_section(
    ui: &mut Ui,
    ctx: &egui::Context,
    selected: &mut Option<SelectedWidget>,
    snapshot: &GeometrySnapshot,
    theme: DbProTheme,
) {
    egui::CollapsingHeader::new(section("Selection"))
        .id_salt("dbpro_dev_tools_selection")
        .default_open(true)
        .show(ui, |ui| match selected.as_mut() {
            None => {
                ui.label(RichText::new("click a widget to pin it — Esc or empty click clears").color(theme.text_muted));
            }
            Some(sel) => {
                let w = sel.current().clone();
                let live = sel.live.is_some();
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&w.id_debug).monospace().strong());
                    if !live {
                        ui.label(RichText::new("(widget gone — showing last seen)").color(theme.text_muted));
                    }
                    if ui.small_button("clear").clicked() {
                        *selected = None;
                    }
                });
                draw_geometry_detail(ui, ctx, &w, snapshot, theme);
            }
        });
}

fn draw_geometry_detail(
    ui: &mut Ui,
    ctx: &egui::Context,
    w: &WidgetGeometry,
    snapshot: &GeometrySnapshot,
    theme: DbProTheme,
) {
    egui::Grid::new(ui.id().with("geom"))
        .num_columns(2)
        .spacing(egui::vec2(8.0, 2.0))
        .show(ui, |ui| {
            if let Some(typ) = &w.widget_type {
                row(ui, theme, "type", typ);
            }
            if let Some(label) = &w.label {
                row(ui, theme, "label", label);
            }
            if let Some(hint) = &w.hint_text {
                row(ui, theme, "hint", hint);
            }
            // Every Ui owns a WidgetRect in 0.36, so the parent resolves to a
            // real container entry when it was painted this pass.
            match snapshot.find(w.parent_id) {
                Some(parent) => row(
                    ui,
                    theme,
                    "parent",
                    &format!(
                        "{} ({:.0}×{:.0})",
                        parent.id_debug,
                        parent.rect.width(),
                        parent.rect.height()
                    ),
                ),
                None => row(ui, theme, "parent", &w.parent_debug),
            }
            row(
                ui,
                theme,
                "layer",
                &format!("{} ({})", w.layer_debug, transform_debug(ctx, w.layer)),
            );

            let rect_label = format!(
                "{:.0}×{:.0} @ {:.0},{:.0}",
                w.rect.width(),
                w.rect.height(),
                w.rect.min_x,
                w.rect.min_y
            );
            row(ui, theme, "rect (local)", &rect_label);
            let global = w.to_global_rect(ctx);
            row(
                ui,
                theme,
                "rect (screen)",
                &format!(
                    "{:.0}×{:.0} @ {:.0},{:.0}",
                    global.width(),
                    global.height(),
                    global.left(),
                    global.top()
                ),
            );
            row(
                ui,
                theme,
                "interact rect",
                &format!(
                    "{:.0}×{:.0} @ {:.0},{:.0}",
                    w.interact_rect.width(),
                    w.interact_rect.height(),
                    w.interact_rect.min_x,
                    w.interact_rect.min_y
                ),
            );
            if w.clipped {
                row(ui, theme, "clipped", "interact_rect < rect (parent clip bounds it)");
            }

            let mut senses = Vec::new();
            if w.senses_click {
                senses.push("click");
            }
            if w.senses_drag {
                senses.push("drag");
            }
            if w.focusable {
                senses.push("focusable");
            }
            if !w.enabled {
                senses.push("disabled");
            }
            row(ui, theme, "sense", &senses.join(" "));

            if let Some(v) = w.value {
                row(ui, theme, "value", &format!("{v}"));
            }
            if let Some(sel) = w.selected {
                row(ui, theme, "selected", &sel.to_string());
            }
            if let Some(text) = &w.current_text_value {
                row(ui, theme, "text", text);
            }
        });
}

fn row(ui: &mut Ui, theme: DbProTheme, name: &str, value: &str) {
    ui.label(
        RichText::new(name)
            .color(theme.text_muted)
            .size(crate::tokens::FONT_SIZE_CAPTION),
    );
    ui.label(RichText::new(value).monospace().size(crate::tokens::FONT_SIZE_CAPTION));
    ui.end_row();
}

/// Widgets currently under the pointer (excluding the inspector), from the
/// previous-pass hit test — the real stacked list, top widget last.
fn draw_widget_hits(ui: &mut Ui, ctx: &egui::Context, theme: DbProTheme) {
    egui::CollapsingHeader::new(section("Under pointer"))
        .id_salt("dbpro_dev_tools_hits")
        .default_open(true)
        .show(ui, |ui| {
            let hits = ctx.viewport(|v| v.hits.clone());
            let pointer = ctx.pointer_hover_pos();
            ui.horizontal(|ui| {
                ui.label("Pointer");
                ui.monospace(pointer.map_or_else(|| "—".to_owned(), |p| format!("{:.1}, {:.1}", p.x, p.y)));
            });
            let widgets: Vec<_> = hits
                .contains_pointer
                .iter()
                .filter(|w| !is_area_layer(w.layer_id, inspector_id()))
                .collect();
            if widgets.is_empty() {
                ui.label(RichText::new("hover the UI to inspect widgets").color(theme.text_muted));
                return;
            }
            ui.label(
                RichText::new(format!("{} stacked — front-most last", widgets.len()))
                    .size(crate::tokens::FONT_SIZE_CAPTION)
                    .color(theme.text_muted),
            );
            for widget in widgets {
                draw_hit_row(ui, widget, theme);
            }
        });
}

fn draw_hit_row(ui: &mut Ui, widget: &egui::WidgetRect, theme: DbProTheme) {
    ui.group(|ui| {
        ui.set_max_width(ui.available_width());
        ui.horizontal_wrapped(|ui| {
            ui.monospace(widget.id.short_debug_format());
            ui.label(
                RichText::new(widget.layer_id.short_debug_format())
                    .size(crate::tokens::FONT_SIZE_CAPTION)
                    .color(theme.text_muted),
            );
            let mut flags = Vec::new();
            if widget.sense.senses_click() {
                flags.push("click");
            }
            if widget.sense.senses_drag() {
                flags.push("drag");
            }
            if widget.sense.is_focusable() {
                flags.push("focusable");
            }
            if !widget.enabled {
                flags.push("disabled");
            }
            if !flags.is_empty() {
                ui.label(
                    RichText::new(flags.join(" "))
                        .size(crate::tokens::FONT_SIZE_CAPTION)
                        .color(theme.accent),
                );
            }
        });
        ui.monospace(format!(
            "rect {:.0}x{:.0} @ {:.0},{:.0}",
            widget.rect.width(),
            widget.rect.height(),
            widget.rect.left(),
            widget.rect.top()
        ));
    });
}

/// Whether the audit fixture is enabled (env `DB_PRO_INSPECTOR_AUDIT_FIXTURE`).
pub(crate) fn audit_fixture_enabled() -> bool {
    std::env::var_os(AUDIT_FIXTURE_ENV).is_some_and(|v| !v.is_empty() && v != "0")
}

/// Deliberately broken widgets in one `Area` (same layer id → same-area
/// overlap the `overlap.suspicious` rule flags): two buttons whose hitboxes
/// overlap >50% without containment, plus a zero-size interactive widget.
/// Only painted when [`AUDIT_FIXTURE_ENV`] is set — used for capture runs.
fn paint_audit_fixture(ctx: &egui::Context) {
    egui::Area::new(egui::Id::new("dbpro_audit_fixture"))
        .order(egui::Order::Middle)
        .fixed_pos(Pos2::new(600.0, 500.0))
        .show(ctx, |ui| {
            ui.put(
                egui::Rect::from_min_size(Pos2::new(600.0, 500.0), egui::vec2(120.0, 28.0)),
                egui::Button::new("audit base"),
            );
            ui.put(
                egui::Rect::from_min_size(Pos2::new(615.0, 507.0), egui::vec2(120.0, 28.0)),
                egui::Button::new("audit overlapping"),
            );
            // allocate_rect registers a WidgetRect exactly as given — a
            // click-sensing widget with a 0×0 hitbox (unreachable).
            ui.allocate_rect(
                egui::Rect::from_min_size(Pos2::new(600.0, 560.0), egui::vec2(0.0, 0.0)),
                egui::Sense::click(),
            );
        });
}

/// Whether the screen fixture is enabled (`DB_PRO_AUDIT_SCREEN_FIXTURE`).
/// `ugly` paints intentional defects; any other non-empty value paints a
/// compositionally clean layout. Both live in one dedicated Area/layer so the
/// screen analyzer scopes itself to just the fixture.
fn screen_fixture_mode() -> Option<String> {
    std::env::var(screen::SCREEN_FIXTURE_ENV)
        .ok()
        .filter(|v| !v.is_empty() && v != "0")
}

/// Virtual "screen" the fixture presents to the analyzer — a full IDE layout
/// in miniature: top bar, side rail, content column, status bar. `ugly`
/// breaks composition on purpose (dominant rail, outlier gap, off-screen
/// widget); `good` keeps everything consistent. `ui.put` inside per-zone
/// `new_child` UIs gives each zone its own `parent_id`, which is how the
/// analyzer sees the pane structure.
fn paint_screen_fixture(ctx: &egui::Context, mode: &str) {
    let frame = screen::SCREEN_FIXTURE_RECT;
    egui::Area::new(screen::screen_fixture_layer_id().id)
        .order(egui::Order::Middle)
        .movable(false)
        .interactable(false)
        .fixed_pos(frame.min)
        .show(ctx, |ui| {
            let ugly = mode == "ugly";
            let w = frame.width();
            let rail_w = if ugly { w * 0.62 } else { w * 0.18 };
            // Each zone is its own Ui with an explicit id_salt → distinct
            // parent_id per zone, the analyzer's pane boundary signal.
            // `allocate_rect` + `widget_info` registers a named interactive
            // widget under the zone ui (unlike `put`, which wraps every
            // widget in its own anonymous child Ui).
            let place = |zone: &mut Ui, rect: egui::Rect, name: &str| {
                let resp = zone.allocate_rect(rect, egui::Sense::click());
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, name.to_owned()));
            };
            let zones: [(egui::Rect, &str); 4] = [
                (egui::Rect::from_min_size(frame.min, egui::vec2(w, 36.0)), "bar"),
                (
                    egui::Rect::from_min_size(
                        Pos2::new(frame.min.x, frame.min.y + 40.0),
                        egui::vec2(rail_w, frame.height() - 80.0),
                    ),
                    "rail",
                ),
                (
                    egui::Rect::from_min_size(
                        Pos2::new(frame.min.x + rail_w + 8.0, frame.min.y + 40.0),
                        egui::vec2(w - rail_w - 8.0, frame.height() - 80.0),
                    ),
                    "content",
                ),
                (
                    egui::Rect::from_min_max(Pos2::new(frame.min.x, frame.max.y - 28.0), frame.max),
                    "status",
                ),
            ];
            for (rect, name) in zones {
                let mut zone = ui.new_child(egui::UiBuilder::new().id_salt(egui::Id::new(name)).max_rect(rect));
                match name {
                    "bar" => {
                        for i in 0..4 {
                            place(
                                &mut zone,
                                egui::Rect::from_min_size(
                                    rect.min + egui::vec2(8.0 + i as f32 * 88.0, 4.0),
                                    egui::vec2(80.0, 28.0),
                                ),
                                &format!("bar{i}"),
                            );
                        }
                    }
                    "rail" => {
                        // Varied heights keep the rail non-grid-like — a
                        // navigation panel of mixed sections, which is what
                        // makes the ugly variant's dominance measurable.
                        let mut y = rect.min.y + 8.0;
                        for i in 0..6 {
                            let h = 28.0 + (i % 3) as f32 * 14.0;
                            place(
                                &mut zone,
                                egui::Rect::from_min_size(Pos2::new(rect.min.x + 8.0, y), egui::vec2(rail_w - 16.0, h)),
                                &format!("rail{i}"),
                            );
                            y += h + 8.0;
                        }
                    }
                    "content" => {
                        let mut y = rect.min.y + 8.0;
                        for i in 0..5 {
                            if i > 0 {
                                y += if ugly && i == 3 { 100.0 } else { 8.0 };
                            }
                            place(
                                &mut zone,
                                egui::Rect::from_min_size(
                                    Pos2::new(rect.min.x + 8.0, y),
                                    egui::vec2(rect.width() - 16.0, 32.0),
                                ),
                                &format!("row{i}"),
                            );
                            y += 32.0;
                        }
                    }
                    "status" => {
                        for i in 0..2 {
                            place(
                                &mut zone,
                                egui::Rect::from_min_size(
                                    rect.min + egui::vec2(8.0 + i as f32 * 108.0, 2.0),
                                    egui::vec2(100.0, 24.0),
                                ),
                                &format!("status{i}"),
                            );
                        }
                    }
                    _ => {}
                }
            }
            if ugly {
                // Interactive widget fully outside the fixture screen —
                // the off-screen finding's fixture evidence.
                let mut stray = ui.new_child(egui::UiBuilder::new().id_salt(egui::Id::new("stray")).max_rect(
                    egui::Rect::from_min_size(
                        Pos2::new(frame.max.x + 20.0, frame.min.y + 40.0),
                        egui::vec2(80.0, 28.0),
                    ),
                ));
                let resp = stray.allocate_rect(
                    egui::Rect::from_min_size(
                        Pos2::new(frame.max.x + 20.0, frame.min.y + 40.0),
                        egui::vec2(80.0, 28.0),
                    ),
                    egui::Sense::click(),
                );
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "offscreen".to_owned()));
            }
        });
}

/// Audit results: per-rule PASS/issue tallies + a clickable issue list.
/// Clicking an issue pins its widget — the highlight follows it across frames.
fn draw_audit_section(
    ui: &mut Ui,
    audit: Option<&AuditReport>,
    prev: Option<&AuditReport>,
    selected: &mut Option<SelectedWidget>,
    snapshot: &GeometrySnapshot,
    theme: DbProTheme,
) {
    // Identity diff: new/resolved issue keys — meaningful only when both
    // audits ran over the same widget set (`diff` returns None otherwise).
    let diff = audit
        .zip(prev)
        .and_then(|(now, before)| now.diff(before))
        .map(|(new, resolved)| format!(" (+{} new, −{} resolved)", new.len(), resolved.len()))
        .unwrap_or_default();
    let title = match audit {
        Some(a) if a.issues.is_empty() => format!("Audit — clean ({} widgets){diff}", a.widget_count),
        Some(a) => format!(
            "Audit — {} issue(s) in {} widgets{diff}",
            a.issues.len(),
            a.widget_count
        ),
        None => "Audit".to_owned(),
    };
    egui::CollapsingHeader::new(section(&title))
        .id_salt("dbpro_dev_tools_audit")
        .default_open(true)
        .show(ui, |ui| {
            let Some(report) = audit else {
                ui.label(RichText::new("no snapshot yet").color(theme.text_muted));
                return;
            };
            // Per-rule verdict: FAIL(n) / PASS / SKIP / N/A — zero evaluated
            // can never show PASS.
            for r in &report.rules {
                let (color, verdict) = match r.verdict() {
                    audit::Verdict::Fail => (theme.danger, format!("{} {}", audit::Verdict::Fail.label(), r.issues)),
                    audit::Verdict::Pass => (theme.success, audit::Verdict::Pass.label().to_owned()),
                    audit::Verdict::Skipped => (theme.warning, audit::Verdict::Skipped.label().to_owned()),
                    audit::Verdict::NotApplicable => {
                        (theme.text_muted, audit::Verdict::NotApplicable.label().to_owned())
                    }
                };
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(&verdict)
                            .color(color)
                            .monospace()
                            .size(crate::tokens::FONT_SIZE_CAPTION),
                    );
                    ui.label(
                        RichText::new(r.rule_id)
                            .monospace()
                            .size(crate::tokens::FONT_SIZE_CAPTION),
                    );
                    let mut meta = format!("{} evaluated", r.stats.evaluated);
                    if r.stats.skipped > 0 {
                        meta.push_str(&format!(
                            " · {} skipped ({})",
                            r.stats.skipped,
                            r.stats.skip_reason.map_or("unknown", |s| s.label())
                        ));
                    }
                    if r.stats.not_applicable > 0 {
                        meta.push_str(&format!(" · {} n/a", r.stats.not_applicable));
                    }
                    ui.label(
                        RichText::new(meta)
                            .size(crate::tokens::FONT_SIZE_CAPTION)
                            .color(theme.text_muted),
                    );
                });
            }
            if let Some(screen) = report.screen.as_ref() {
                ui.separator();
                draw_screen_section(ui, screen, theme);
            }
            if report.issues.is_empty() {
                return;
            }
            ui.separator();
            // Issue rows: severity + rule + widget id + evidence. Click pins.
            let mut pin: Option<egui::Id> = None;
            for issue in &report.issues {
                draw_issue_row(ui, issue, snapshot, theme, &mut pin);
            }
            if let Some(found) = pin.and_then(|id| snapshot.find(id)) {
                *selected = Some(SelectedWidget {
                    picked: found.clone(),
                    live: None,
                });
            }
        });
}

/// Screen-level composition summary: measured metrics + screen findings.
/// Screen findings are informational — they never join the issue list or
/// flip the audit verdict, so they render muted, not severity-colored.
fn draw_screen_section(ui: &mut Ui, screen: &screen::ScreenAnalysis, theme: DbProTheme) {
    ui.label(RichText::new("Screen").strong().size(crate::tokens::FONT_SIZE_CAPTION));
    if !screen.sufficient {
        let note = screen.gaps.iter().map(|g| g.label()).collect::<Vec<_>>().join(", ");
        ui.label(
            RichText::new(format!("insufficient data ({note})"))
                .size(crate::tokens::FONT_SIZE_CAPTION)
                .color(theme.text_muted),
        );
        return;
    }
    let kinds = screen
        .regions
        .iter()
        .map(|r| r.kind.label())
        .collect::<Vec<_>>()
        .join(", ");
    ui.label(
        RichText::new(format!(
            "{} regions [{}] · {:.0}% widget coverage",
            screen.regions.len(),
            kinds,
            screen.density.widget_coverage * 100.0
        ))
        .monospace()
        .size(crate::tokens::FONT_SIZE_CAPTION)
        .color(theme.text_secondary),
    );
    if let Some(mode) = screen.rhythm.mode_gap {
        ui.label(
            RichText::new(format!(
                "rhythm {:.0}px ({:.0}% of {} gaps · {} outliers)",
                mode,
                screen.rhythm.mode_share.unwrap_or(0.0) * 100.0,
                screen.rhythm.gaps_measured,
                screen.rhythm.outliers
            ))
            .monospace()
            .size(crate::tokens::FONT_SIZE_CAPTION)
            .color(theme.text_secondary),
        );
    }
    for f in &screen.findings {
        let color = match f.severity {
            audit::Severity::Error => theme.danger,
            audit::Severity::Warning => theme.warning,
        };
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!("{}·{}", f.severity.label(), f.kind.label()))
                    .color(color)
                    .size(crate::tokens::FONT_SIZE_CAPTION),
            );
            ui.label(RichText::new(f.rule).monospace().size(crate::tokens::FONT_SIZE_CAPTION));
        });
        ui.label(
            RichText::new(&f.evidence)
                .size(crate::tokens::FONT_SIZE_CAPTION)
                .color(theme.text_secondary),
        );
    }
}

fn draw_issue_row(
    ui: &mut Ui,
    issue: &AuditIssue,
    snapshot: &GeometrySnapshot,
    theme: DbProTheme,
    pin: &mut Option<egui::Id>,
) {
    let severity_color = match issue.severity {
        audit::Severity::Error => theme.danger,
        audit::Severity::Warning => theme.warning,
    };
    ui.group(|ui| {
        ui.set_max_width(ui.available_width());
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(issue.severity.label())
                    .color(severity_color)
                    .strong()
                    .size(crate::tokens::FONT_SIZE_CAPTION),
            );
            ui.label(
                RichText::new(issue.rule_id)
                    .monospace()
                    .size(crate::tokens::FONT_SIZE_CAPTION),
            );
            ui.label(
                RichText::new(format!("confidence {}", issue.confidence.label()))
                    .size(crate::tokens::FONT_SIZE_CAPTION)
                    .color(theme.text_muted),
            );
        });
        if ui
            .link(
                RichText::new(&issue.widget_debug)
                    .monospace()
                    .size(crate::tokens::FONT_SIZE_CAPTION),
            )
            .on_hover_text("pin widget")
            .clicked()
        {
            *pin = Some(issue.widget_id);
        }
        if let Some(other) = issue.other_id {
            let other_debug = snapshot
                .find(other)
                .map(|w| w.id_debug.clone())
                .unwrap_or_else(|| other.short_debug_format());
            ui.label(
                RichText::new(format!("with {other_debug}"))
                    .size(crate::tokens::FONT_SIZE_CAPTION)
                    .color(theme.text_muted),
            );
        }
        ui.label(
            RichText::new(&issue.evidence)
                .size(crate::tokens::FONT_SIZE_CAPTION)
                .color(theme.text_secondary),
        );
    });
}

/// One-line-per-issue stderr dump for `DB_PRO_INSPECTOR_DUMP` runs.
fn dump_audit_issues(report: &AuditReport, snapshot: &GeometrySnapshot) {
    eprintln!(
        "=== audit dump: {} issue(s), {} widgets ===",
        report.issues.len(),
        report.widget_count
    );
    for r in &report.rules {
        eprintln!(
            "RULE {} {} eval={} skip={} na={}",
            r.verdict().label(),
            r.rule_id,
            r.stats.evaluated,
            r.stats.skipped,
            r.stats.not_applicable
        );
    }
    // Full widget list — correlating issues to positions needs the geometry
    // the report doesn't carry (widget rects aren't in the JSON by design).
    for (layer_id, ws) in &snapshot.layers {
        for w in ws {
            eprintln!(
                "WID {} layer={} parent={} rect={:?} click={} enabled={} type={:?} label={:?}",
                w.id_debug, w.layer_debug, w.parent_debug, w.rect, w.senses_click, w.enabled, w.widget_type, w.label
            );
        }
        let _ = layer_id;
    }
    if let Some(scr) = report.screen.as_ref() {
        for f in &scr.findings {
            eprintln!(
                "SCREEN {} {} kind={} region={:?} wid={:?} :: {}",
                f.severity.label(),
                f.rule,
                f.kind.label(),
                f.region,
                f.widget_debug,
                f.evidence
            );
        }
    }
    for i in &report.issues {
        // Join back to geometry for the widget's kind + position — the context
        // needed to tell a real defect from an instrumentation gap.
        let geo = snapshot
            .find(i.widget_id)
            .map(|w| {
                format!(
                    " layer={} rect={:?}→{:?} parent={}",
                    w.layer_debug, w.rect, w.interact_rect, w.parent_debug
                )
            })
            .unwrap_or_else(|| " (no geometry join)".to_owned());
        eprintln!(
            "ISSUE {} {} wid={} other={:?}{} :: {}",
            i.severity.label(),
            i.rule_id,
            i.widget_debug,
            i.other_id.map(|o| o.short_debug_format()),
            geo,
            i.evidence
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspector_starts_closed_without_env() {
        std::env::remove_var(OPEN_ON_START_ENV);
        assert!(!DevToolsState::default().open);
    }

    #[test]
    fn inspector_window_draws_and_closes() {
        let ctx = egui::Context::default();
        let mut state = DevToolsState {
            open: true,
            selected: None,
            pending_pick: None,
            audit: None,
            prev_audit: None,
            audit_dumped: false,
            audit_frames: 0,
            pending_audit: None,
            stable_streak: 0,
        };
        let _ = crate::test_frame::frame(&ctx, egui::RawInput::default(), |ui| {
            state.draw(ui.ctx(), DbProTheme::dark());
        });
        assert!(state.open, "window must stay open while undismissed");

        let mut closed = DevToolsState {
            open: false,
            selected: None,
            pending_pick: None,
            audit: None,
            prev_audit: None,
            audit_dumped: false,
            audit_frames: 0,
            pending_audit: None,
            stable_streak: 0,
        };
        let out = crate::test_frame::frame(&ctx, egui::RawInput::default(), |ui| {
            closed.draw(ui.ctx(), DbProTheme::dark());
        });
        assert!(out.shapes.is_empty(), "closed inspector must paint nothing");
    }

    #[test]
    fn pick_env_parses_xy() {
        std::env::set_var(PICK_ENV, "120.5, 64");
        assert_eq!(parse_pick_env(), Some(Pos2::new(120.5, 64.0)));
        std::env::set_var(PICK_ENV, "not-a-pair");
        assert_eq!(parse_pick_env(), None);
        std::env::remove_var(PICK_ENV);
    }

    fn plain_input(ctx_size: egui::Vec2) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, ctx_size)),
            ..Default::default()
        }
    }

    #[test]
    fn draw_populates_audit_report() {
        let ctx = egui::Context::default();
        let mut state = DevToolsState {
            open: true,
            selected: None,
            pending_pick: None,
            audit: None,
            prev_audit: None,
            audit_dumped: false,
            audit_frames: 0,
            pending_audit: None,
            stable_streak: 0,
        };
        // Pass 1 registers widgets; pass 2 audits them.
        for _ in 0..2 {
            let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
                let _ = ui.button("target");
                state.draw(ui.ctx(), DbProTheme::dark());
            });
        }
        let report = state.audit.as_ref().expect("draw must run the audit");
        assert_eq!(report.rules.len(), 12, "every registered rule must report");
        assert!(report.widget_count > 0);
    }

    #[test]
    fn audit_fixture_paints_broken_widgets() {
        // The intentional-error fixture must actually feed the engine:
        // overlap + zero-size widgets drawn via paint_audit_fixture land in
        // the next pass's snapshot and produce issues.
        let ctx = egui::Context::default();
        // Fixture paints via Area::show — must run inside a pass (fonts).
        for _ in 0..3 {
            let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
                paint_audit_fixture(ui.ctx());
            });
        }
        let snapshot = GeometrySnapshot::collect(&ctx, inspector_id());
        let report = audit::run_audit(&ctx, &snapshot, None);
        let overlap = report.issues.iter().any(|i| i.rule_id == "overlap.suspicious");
        let zero = report.issues.iter().any(|i| i.rule_id == "interactive.zero_size");
        assert!(overlap, "fixture must produce an overlap issue");
        assert!(zero, "fixture must produce a zero-size issue");
    }

    #[test]
    fn semantic_tree_and_tab_focus_flow_through_capture() {
        // End-to-end: enable accesskit, paint two buttons, press Tab — the
        // captured TreeUpdate must carry nodes for both widgets, a resolvable
        // parent (root), and focus must land on an existing node (or root),
        // which is exactly what `focus.orphaned` audits.
        let ctx = egui::Context::default();
        AccessKitCapture::ensure_installed(&ctx);

        let paint = |ui: &mut egui::Ui| {
            let _ = ui.button("first");
            let _ = ui.button("second");
        };
        for _ in 0..3 {
            let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), paint);
        }
        // Tab keypress moves egui keyboard focus between focusable widgets.
        let _ = crate::test_frame::frame(
            &ctx,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
                events: vec![egui::Event::Key {
                    key: egui::Key::Tab,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
                ..Default::default()
            },
            paint,
        );

        let update = AccessKitCapture::latest(&ctx).expect("capture plugin must see a tree update");
        let sem = SemanticSnapshot::from_update(&update).expect("update carries nodes");
        // Buttons appear as semantic nodes joined to their widget ids.
        assert!(
            sem.nodes.values().any(|n| n.role == egui::accesskit::Role::Button),
            "buttons must emit Button nodes"
        );
        // Focus is always resolvable: root when nothing focused, else a node.
        if let Some(f) = sem.focus {
            assert!(sem.nodes.contains_key(&f), "focus {:?} must be a real node", f);
        }
        // The audit agrees: no orphaned focus, no missing names (both buttons
        // are labelled by their text).
        let report = audit::run_audit(&ctx, &GeometrySnapshot::collect(&ctx, inspector_id()), Some(&sem));
        assert!(
            report.issues.iter().all(|i| i.rule_id != "focus.orphaned"),
            "focus must not be orphaned"
        );
        assert!(
            report.issues.iter().all(|i| i.rule_id != "semantic.missing_name"),
            "named buttons must not flag"
        );
    }

    #[test]
    fn pointer_press_pins_the_widget_under_the_click() {
        let ctx = egui::Context::default();
        let mut state = DevToolsState {
            open: true,
            selected: None,
            pending_pick: None,
            audit: None,
            prev_audit: None,
            audit_dumped: false,
            audit_frames: 0,
            pending_audit: None,
            stable_streak: 0,
        };

        // Paint the button in the same pass the press lands — picks resolve
        // against the previous pass's widgets, so the target must exist there.
        let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
            ui.put(
                egui::Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(80.0, 24.0)),
                egui::Button::new("pick me"),
            );
        });

        // Pass 2: press inside the button while the inspector draws.
        let press = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
            events: vec![
                egui::Event::PointerMoved(Pos2::new(140.0, 112.0)),
                egui::Event::PointerButton {
                    pos: Pos2::new(140.0, 112.0),
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            ..Default::default()
        };
        let _ = crate::test_frame::frame(&ctx, press, |ui| {
            ui.put(
                egui::Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(80.0, 24.0)),
                egui::Button::new("pick me"),
            );
            state.draw(ui.ctx(), DbProTheme::dark());
        });
        let selected = state.selected.as_ref().expect("click must pin a widget");
        assert!(
            selected.picked.senses_click,
            "picked widget should be the clickable button"
        );
        assert!(selected.picked.rect.to_egui().contains(Pos2::new(140.0, 112.0)));
    }

    #[test]
    fn selection_survives_pointer_move_and_escape_clears() {
        let ctx = egui::Context::default();
        let mut state = DevToolsState {
            open: true,
            selected: None,
            pending_pick: Some(Pos2::new(140.0, 112.0)),
            audit: None,
            prev_audit: None,
            audit_dumped: false,
            audit_frames: 0,
            pending_audit: None,
            stable_streak: 0,
        };

        // Pass 1 paints the target; the pending pick resolves on the next pass
        // once the widget exists in prev_pass.
        let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
            ui.put(
                egui::Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(80.0, 24.0)),
                egui::Button::new("pin me"),
            );
            state.draw(ui.ctx(), DbProTheme::dark());
        });
        assert!(
            state.pending_pick.is_some(),
            "pick must wait for the target to register"
        );
        let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
            ui.put(
                egui::Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(80.0, 24.0)),
                egui::Button::new("pin me"),
            );
            state.draw(ui.ctx(), DbProTheme::dark());
        });
        assert!(state.selected.is_some(), "pending pick must pin the widget");

        // Pass 2: pointer moved away — selection must stay.
        let _ = crate::test_frame::frame(
            &ctx,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
                events: vec![egui::Event::PointerMoved(Pos2::new(400.0, 400.0))],
                ..Default::default()
            },
            |ui| {
                ui.put(
                    egui::Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(80.0, 24.0)),
                    egui::Button::new("pin me"),
                );
                state.draw(ui.ctx(), DbProTheme::dark());
            },
        );
        assert!(state.selected.is_some(), "pointer move must not drop the pin");

        // Pass 3: Escape clears the pin.
        let _ = crate::test_frame::frame(
            &ctx,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
                events: vec![egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
                ..Default::default()
            },
            |ui| {
                ui.put(
                    egui::Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(80.0, 24.0)),
                    egui::Button::new("pin me"),
                );
                state.draw(ui.ctx(), DbProTheme::dark());
            },
        );
        assert!(state.selected.is_none(), "Escape must clear the pin");
    }

    #[test]
    fn inspector_widgets_are_not_pickable() {
        let ctx = egui::Context::default();
        let mut state = DevToolsState {
            open: true,
            selected: None,
            pending_pick: None,
            audit: None,
            prev_audit: None,
            audit_dumped: false,
            audit_frames: 0,
            pending_audit: None,
            stable_streak: 0,
        };

        // Pass 1 draws the inspector itself plus a background button.
        let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
            ui.put(
                egui::Rect::from_min_size(Pos2::new(20.0, 20.0), egui::vec2(40.0, 20.0)),
                egui::Button::new("bg"),
            );
            state.draw(ui.ctx(), DbProTheme::dark());
        });

        // Pass 2: pick inside the inspector window's area — selection must
        // not capture an inspector widget.
        state.pending_pick = Some(Pos2::new(200.0, 40.0));
        let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
            state.draw(ui.ctx(), DbProTheme::dark());
        });
        if let Some(sel) = &state.selected {
            assert!(
                sel.picked.layer.id != inspector_id(),
                "inspector widget leaked into the pick: {:?}",
                sel.picked.id_debug
            );
        }
    }

    #[test]
    fn snapshot_picks_the_front_most_of_overlapping_widgets() {
        let ctx = egui::Context::default();

        let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
            ui.put(
                egui::Rect::from_min_size(Pos2::new(50.0, 50.0), egui::vec2(100.0, 100.0)),
                egui::Button::new("under"),
            );
            ui.put(
                egui::Rect::from_min_size(Pos2::new(80.0, 80.0), egui::vec2(100.0, 100.0)),
                egui::Button::new("over"),
            );
        });

        let snapshot = GeometrySnapshot::collect(&ctx, inspector_id());
        let picked = snapshot
            .pick(&ctx, Pos2::new(100.0, 100.0))
            .expect("overlap point must hit");

        // The "over" button was painted last → it wins the shared region.
        // `ui.put` also registers the child Ui on the same rect, so take the
        // last entry, not the first.
        let over =
            snapshot.layers.iter().flat_map(|(_, ws)| ws.iter()).rfind(|w| {
                w.rect.to_egui() == egui::Rect::from_min_size(Pos2::new(80.0, 80.0), egui::vec2(100.0, 100.0))
            });
        assert_eq!(Some(picked.id), over.map(|w| w.id), "front-most widget must win");
    }

    #[test]
    fn foreground_popup_wins_over_background_widget() {
        let ctx = egui::Context::default();

        let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
            ui.put(
                egui::Rect::from_min_size(Pos2::new(50.0, 50.0), egui::vec2(200.0, 200.0)),
                egui::Button::new("background"),
            );
            egui::Area::new(egui::Id::new("test_popup"))
                .order(egui::Order::Foreground)
                .fixed_pos(Pos2::new(100.0, 100.0))
                .show(ui.ctx(), |ui| {
                    ui.add(egui::Button::new("popup item").min_size(egui::vec2(80.0, 24.0)));
                });
        });

        let snapshot = GeometrySnapshot::collect(&ctx, inspector_id());
        let picked = snapshot
            .pick(&ctx, Pos2::new(120.0, 112.0))
            .expect("popup widget must be pickable");
        assert_eq!(
            picked.layer.order,
            egui::Order::Foreground,
            "foreground popup layer must beat the background widget, picked: {:?}",
            picked.layer_debug
        );
    }

    #[test]
    fn selection_follows_the_widget_after_scrolling() {
        let ctx = egui::Context::default();
        let mut state = DevToolsState {
            open: true,
            selected: None,
            pending_pick: None,
            audit: None,
            prev_audit: None,
            audit_dumped: false,
            audit_frames: 0,
            pending_audit: None,
            stable_streak: 0,
        };

        let draw_scroller = |ui: &mut Ui| {
            egui::ScrollArea::vertical()
                .id_salt("geom_scroll_test")
                .max_height(100.0)
                .show(ui, |ui| {
                    for i in 0..40 {
                        let _ = ui.button(format!("row {i}"));
                    }
                });
        };

        // Pass 1: draw scroller + inspector.
        let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
            draw_scroller(ui);
            state.draw(ui.ctx(), DbProTheme::dark());
        });

        // Pick the 2nd visible row — `widget_info`/`label` is only recorded
        // for interacted widgets, so locate by sense + vertical position.
        let snapshot = GeometrySnapshot::collect(&ctx, inspector_id());
        let mut rows: Vec<_> = snapshot
            .layers
            .iter()
            .flat_map(|(_, ws)| ws.iter())
            .filter(|w| w.senses_click && w.enabled && w.rect.height() > 10.0 && w.rect.height() < 60.0 && !w.clipped)
            .collect();
        rows.sort_by(|a, b| {
            a.rect
                .min_y
                .partial_cmp(&b.rect.min_y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let row = rows.get(1).expect("a visible scrolled row must exist");
        let center = row.to_global_rect(&ctx).center();
        state.pending_pick = Some(center);

        // Pass 2: the pending pick resolves against pass-1 widgets. Picking a
        // row's rect may pin the row's inner Ui or the row itself — the
        // contract is that *a* widget covering the point is pinned.
        let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
            draw_scroller(ui);
            state.draw(ui.ctx(), DbProTheme::dark());
        });
        let picked = state.selected.as_ref().expect("pending pick must pin a widget");
        let to_local = ctx.layer_transform_from_global(picked.picked.layer).unwrap_or_default();
        let local_center = to_local * center;
        assert!(
            picked.picked.rect.to_egui().contains(local_center),
            "pinned widget must cover the picked point"
        );
        // Pass 3: scroll down; the pinned widget's live rect must move.
        let _ = crate::test_frame::frame(
            &ctx,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
                events: vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -60.0),
                    modifiers: egui::Modifiers::NONE,
                    phase: egui::TouchPhase::Move,
                }],
                ..Default::default()
            },
            |ui| {
                draw_scroller(ui);
                state.draw(ui.ctx(), DbProTheme::dark());
            },
        );

        if let Some(live) = state.selected.as_ref().and_then(|s| s.live.as_ref()) {
            assert!(
                (live.rect.min_y - state.selected.as_ref().unwrap().picked.rect.min_y).abs() > 1.0 || live.clipped,
                "scrolling must update the pinned widget's live geometry"
            );
        }
        // If the row scrolled out of `prev_pass` entirely, `live` is None —
        // which the detail view surfaces as "widget gone"; both are valid.
    }
}
