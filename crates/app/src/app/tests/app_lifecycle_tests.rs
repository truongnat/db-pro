use super::*;
use std::collections::HashMap;

#[derive(Default)]
struct MemoryStorage(HashMap<String, String>);

impl eframe::Storage for MemoryStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.0.insert(key.to_owned(), value);
    }

    fn remove_string(&mut self, key: &str) {
        self.0.remove(key);
    }

    fn flush(&mut self) {}
}

fn render(app: &mut DbProApp, ctx: &egui::Context, events: Vec<egui::Event>) -> egui::FullOutput {
    crate::test_frame::frame(
        ctx,
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0))),
            events,
            ..Default::default()
        },
        |ui| {
            // Match eframe: root Ui already exists when App::logic applies the theme.
            app.prepare_frame(ui.ctx());
            let frame_theme = app.theme;
            app.draw_shell(ui);
            assert_eq!(ui.visuals().dark_mode, frame_theme.dark_mode);
            assert_eq!(ui.visuals().panel_fill, frame_theme.surface_panel);
        },
    )
}

fn click(app: &mut DbProApp, ctx: &egui::Context, label: &str) {
    let output = render(app, ctx, Vec::new());
    let bounds = output
        .platform_output
        .accesskit_update
        .as_ref()
        .expect("accessible shell")
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing control: {label}"));
    let position = egui::pos2(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    let _ = render(app, ctx, vec![egui::Event::PointerMoved(position)]);
    for pressed in [true, false] {
        let _ = render(
            app,
            ctx,
            vec![egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            }],
        );
    }
}

fn assert_theme_persists(app: &mut DbProApp, ctx: &egui::Context, dark_mode: bool) {
    assert_eq!(app.preferences.dark_mode, dark_mode);
    assert_eq!(app.preferences.settings.appearance.dark_mode, dark_mode);
    for _ in 0..3 {
        let output = render(app, ctx, Vec::new());
        assert_eq!(app.theme.dark_mode, dark_mode);
        assert_eq!(
            ctx.theme(),
            if dark_mode {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            }
        );
        assert_eq!(app.preferences.settings.appearance.dark_mode, dark_mode);
        if app.workspace.activity == Activity::Settings {
            let selected = if dark_mode { "Dark" } else { "Light" };
            let node = output
                .platform_output
                .accesskit_update
                .as_ref()
                .expect("accessible settings")
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some(selected))
                .map(|(_, node)| node)
                .expect("selected appearance radio");
            assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::True));
        }
        assert!(
            output.shapes.iter().any(|shape| matches!(&shape.shape,
            egui::epaint::Shape::Rect(rect) if rect.fill == app.theme.surface_app)),
            "shell must paint the selected theme's surface"
        );
    }
    let mut storage = MemoryStorage::default();
    eframe::App::save(app, &mut storage);
    assert_eq!(storage.0.get("dbpro.native.dark-mode"), Some(&dark_mode.to_string()));
    let restored = DbProApp::with_task_bridge_and_storage(TaskBridge::default(), Some(&storage));
    assert_eq!(restored.preferences.dark_mode, dark_mode);
    assert_eq!(restored.preferences.settings.appearance.dark_mode, dark_mode);
    assert_eq!(restored.theme.dark_mode, dark_mode);
}

#[test]
fn toolbar_and_settings_switch_rendered_theme_and_persist_the_same_choice() {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    DbProTheme::install_fonts(&ctx);
    // Start with the opposite context style, as happens under a dark system theme.
    ctx.set_theme(egui::Theme::Dark);
    let mut app = DbProApp {
        initial_frames_count: 3,
        ..Default::default()
    };
    assert_theme_persists(&mut app, &ctx, false);

    click(&mut app, &ctx, "Switch to Dark Theme");
    assert_theme_persists(&mut app, &ctx, true);
    click(&mut app, &ctx, "Switch to Light Theme");
    assert_theme_persists(&mut app, &ctx, false);

    app.workspace.activity = Activity::Settings;
    app.preferences.section = settings_model::SettingsSection::Appearance;
    click(&mut app, &ctx, "Dark");
    assert_theme_persists(&mut app, &ctx, true);
    click(&mut app, &ctx, "Light");
    assert_theme_persists(&mut app, &ctx, false);
}

#[test]
fn shell_panel_boundaries_stay_borderless_in_both_themes() {
    for dark_mode in [false, true] {
        for output_open in [false, true] {
            let ctx = egui::Context::default();
            DbProTheme::install_fonts(&ctx);
            let mut app = DbProApp {
                initial_frames_count: 3,
                preferences: PreferencesState {
                    dark_mode,
                    ..Default::default()
                },
                workspace: WorkspaceFeatureState {
                    shell: WorkspaceShellState {
                        agent_open: true,
                        bottom_panel_open: output_open,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            for _ in 0..3 {
                let output = render(&mut app, &ctx, Vec::new());
                for shape in &output.shapes {
                    if let egui::epaint::Shape::Rect(rect) = &shape.shape {
                        let shell_boundary =
                            rect.rect.width() > 1200.0 || (rect.rect.height() > 400.0 && rect.rect.width() < 400.0);
                        if shell_boundary {
                            assert!(
                                rect.stroke.is_empty(),
                                "decorative shell boundary reappeared: {:?}, dark={dark_mode}, output={output_open}",
                                rect.rect
                            );
                        }
                    }
                    if let egui::epaint::Shape::LineSegment { points, stroke } = &shape.shape {
                        if points[0].distance(points[1]) > 400.0 {
                            assert!(stroke.is_empty(), "idle splitter must not paint a decorative line");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn borderless_sidebar_still_highlights_and_drags() {
    for dark_mode in [false, true] {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let mut app = DbProApp {
            initial_frames_count: 3,
            preferences: PreferencesState {
                dark_mode,
                ..Default::default()
            },
            ..Default::default()
        };
        for _ in 0..3 {
            let _ = render(&mut app, &ctx, Vec::new());
        }
        let original_width = app.workspace.sidebar_width;
        let grip = egui::pos2(
            crate::tokens::component::shell::ACTIVITY_RAIL_WIDTH + original_width,
            400.0,
        );
        let mut highlighted = false;
        for _ in 0..2 {
            let output = render(&mut app, &ctx, vec![egui::Event::PointerMoved(grip)]);
            highlighted = output.shapes.iter().any(|shape| {
                matches!(&shape.shape, egui::epaint::Shape::LineSegment { points, stroke }
                    if points[0].distance(points[1]) > 400.0 && stroke.color == app.theme.accent && !stroke.is_empty())
            });
        }
        assert!(highlighted, "hover must reveal the resize affordance");
        let _ = render(
            &mut app,
            &ctx,
            vec![egui::Event::PointerButton {
                pos: grip,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        let target = grip + egui::vec2(40.0, 0.0);
        let _ = render(&mut app, &ctx, vec![egui::Event::PointerMoved(target)]);
        let _ = render(
            &mut app,
            &ctx,
            vec![egui::Event::PointerButton {
                pos: target,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!((app.workspace.sidebar_width - original_width - 40.0).abs() < 1.0);
    }
}

/// Tab must walk the whole shell, not stall on a topbar label. The connection
/// name is re-registered as a window-drag target; making that drag focusable
/// turned the label into a focus sink that swallowed every later Tab press.
#[test]
fn tab_focus_traverses_topbar_into_the_workspace() {
    let (bridge, _rx, _tx) = crate::TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let _ = render(&mut app, &ctx, Vec::new());

    let mut focused_ids = Vec::new();
    let mut reached_workspace = false;
    for _ in 0..30 {
        if let Some(id) = ctx.memory(|m| m.focused()) {
            focused_ids.push(id.short_debug_format());
            if let Some(response) = ctx.read_response(id) {
                reached_workspace |= response.rect.left() > 320.0 && response.rect.top() > 100.0;
            }
        }
        let _ = render(
            &mut app,
            &ctx,
            vec![egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
    }

    let mut distinct = focused_ids.clone();
    distinct.sort();
    distinct.dedup();
    assert!(
        distinct.len() >= 10,
        "Tab should traverse the shell controls, saw {} stops: {focused_ids:?}",
        distinct.len()
    );
    assert!(
        focused_ids.windows(2).all(|pair| pair[0] != pair[1]),
        "Tab focus stalled on one widget: {focused_ids:?}"
    );
    assert!(reached_workspace, "Tab never reached the workspace content: {focused_ids:?}");
}

/// #545 — an enabled schedule used to pin `prepare_frame` to a flat
/// `request_repaint_after(50 ms)` (~20 wakeups/s idle) no matter how far the
/// next run was. These frames read the real `repaint_delay` from the egui
/// output for the far/near/in-flight cases.
mod scheduler_repaint_cadence_tests {
    use super::*;
    use db_pro_core::domain::saved_task::{SavedTask, SavedTaskPayload, TaskSchedule};

    fn app_with_schedule(next_run_secs: i64) -> DbProApp {
        let (bridge, _commands, _events) = TaskBridge::with_channels();
        let mut app = DbProApp::with_task_bridge(bridge);
        let now = chrono::Utc::now();
        let mut schedule = TaskSchedule::every_secs(3_600);
        schedule.next_run_at = Some(now + chrono::Duration::seconds(next_run_secs));
        app.saved_tasks.store.tasks.push(SavedTask {
            id: uuid::Uuid::new_v4(),
            name: "Hourly ping".to_owned(),
            description: String::new(),
            connection_id: "conn-1".to_owned(),
            payload: SavedTaskPayload::Sql { sql: "SELECT 1".to_owned() },
            created_at: now,
            updated_at: now,
            last_run: None,
            schedule: Some(schedule),
        });
        app
    }

    fn repaint_delay(app: &mut DbProApp) -> std::time::Duration {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        // Warm frames: consume the one-shot connections list request (no
        // worker answers it in the harness, so settle the flag to model the
        // issue's scenario — idle app, nothing in flight) and let egui's own
        // internal layout/font settle repaints drain. egui pins a frame's
        // stored delay to ZERO whenever the previous frame left an immediate
        // repaint outstanding (`begin_pass_repaint_logic`), and while it is
        // ZERO a larger later request cannot raise it — so the meaningful
        // delay only becomes observable once the shell stops requesting
        // immediate repaints (verified: from the 4th frame onward the only
        // cause left is prepare_frame's scheduler request).
        let mut delay = std::time::Duration::ZERO;
        for _ in 0..6 {
            app.connection.lifecycle.set_connections_request_pending(false);
            let output = render(app, &ctx, Vec::new());
            delay = output
                .viewport_output
                .get(&egui::ViewportId::ROOT)
                .expect("root viewport output")
                .repaint_delay;
        }
        delay
    }

    #[test]
    fn idle_far_schedule_sleeps_to_the_cap_not_50ms() {
        let delay = repaint_delay(&mut app_with_schedule(3_600));
        assert!(
            delay >= std::time::Duration::from_secs(295),
            "hourly task while idle must not repaint at 50 ms cadence, got {delay:?}"
        );
    }

    #[test]
    fn near_schedule_deadline_sleeps_exactly_to_the_deadline() {
        // 20 s away stays under the 5-minute cap → the delay tracks the slot.
        let delay = repaint_delay(&mut app_with_schedule(20));
        assert!(
            (std::time::Duration::from_secs(15)..std::time::Duration::from_secs(21)).contains(&delay),
            "near deadline should repaint at the real remaining time, got {delay:?}"
        );
    }

    #[test]
    fn in_flight_scheduled_run_keeps_fast_polling() {
        let mut app = app_with_schedule(3_600);
        let request_id = 7u64;
        app.saved_tasks.pending_runs.insert(
            request_id,
            crate::app::saved_task_state::PendingTaskRun {
                task_id: uuid::Uuid::new_v4(),
                trigger: db_pro_core::domain::saved_task::SavedTaskRunTrigger::Scheduled,
                started_at: chrono::Utc::now(),
                export_path: None,
                export_format: None,
            },
        );
        let delay = repaint_delay(&mut app);
        assert!(
            delay > std::time::Duration::ZERO && delay <= std::time::Duration::from_millis(60),
            "a dispatched task awaiting its runtime result must keep the 50 ms cadence, got {delay:?}"
        );
    }
}

#[test]
fn query_history_is_redacted_on_disk_but_keeps_the_session_text() {
    use db_pro_ui::UiQueryHistoryStatus;

    let mut app = DbProApp::default();
    app.query.editor.query_history_entries.push(UiQueryHistoryEntry {
        id: "h1".into(),
        sql: "CREATE ROLE bob PASSWORD 'hunter2' LOGIN;".into(),
        connection_id: None,
        schema: None,
        started_at: "2026-01-01T00:00:00Z".into(),
        duration_ms: 1,
        status: UiQueryHistoryStatus::Success,
        row_count: None,
        affected_rows: None,
        error_code: None,
        error_summary: None,
    });

    let mut storage = MemoryStorage::default();
    eframe::App::save(&mut app, &mut storage);
    let persisted = storage
        .0
        .get("dbpro.native.query-history-v1")
        .expect("query history must be persisted");
    assert!(!persisted.contains("hunter2"), "{persisted}");
    assert!(persisted.contains("***"), "{persisted}");
    assert!(
        app.query.editor.query_history_entries[0].sql.contains("hunter2"),
        "the live session must keep the statement the user ran"
    );

    app.preferences.settings.security.redact_secrets_in_logs = false;
    let mut verbatim = MemoryStorage::default();
    eframe::App::save(&mut app, &mut verbatim);
    let persisted = verbatim
        .0
        .get("dbpro.native.query-history-v1")
        .expect("query history must be persisted");
    assert!(persisted.contains("hunter2"), "opt-out stores verbatim: {persisted}");
}

/// The frame-level cursor reset must run *before* widgets draw. When it ran
/// after, it overwrote every `on_hover_cursor` in the app and hover cursors
/// disappeared everywhere — the audit's "cursor does not match behaviour".
#[test]
fn stale_cursor_is_cleared_before_widgets_draw() {
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    // A value left over from the previous pass.
    ctx.set_cursor_icon(egui::CursorIcon::Grab);

    let button_rect = std::cell::Cell::new(egui::Rect::NOTHING);
    let _ = crate::test_frame::frame(
        &ctx,
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 200.0))),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                crate::app::app_lifecycle::clear_stale_cursor(ui.ctx());
                button_rect.set(
                    crate::components::button::Button::new(DbProTheme::dark())
                        .text("Run")
                        .show(ui)
                        .rect,
                );
            });
        },
    );

    // Hover the button in the next pass, which also clears first.
    let output = crate::test_frame::frame(
        &ctx,
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 200.0))),
            events: vec![egui::Event::PointerMoved(button_rect.get().center())],
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                crate::app::app_lifecycle::clear_stale_cursor(ui.ctx());
                crate::components::button::Button::new(DbProTheme::dark())
                    .text("Run")
                    .show(ui);
            });
        },
    );

    assert_eq!(
        output.platform_output.cursor_icon,
        egui::CursorIcon::PointingHand,
        "clearing before drawing must not erase the widget's hover cursor"
    );

    // The other side of the contract, so the ordering cannot be "simplified"
    // later: a reset *after* drawing erases the hover cursor. That is the
    // regression this test exists for.
    let after_draw = crate::test_frame::frame(
        &ctx,
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 200.0))),
            events: vec![egui::Event::PointerMoved(button_rect.get().center())],
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                crate::components::button::Button::new(DbProTheme::dark())
                    .text("Run")
                    .show(ui);
                crate::app::app_lifecycle::clear_stale_cursor(ui.ctx());
            });
        },
    );
    assert_eq!(
        after_draw.platform_output.cursor_icon,
        egui::CursorIcon::Default,
        "a reset after drawing would erase the hover cursor — keep the clear first"
    );
}
