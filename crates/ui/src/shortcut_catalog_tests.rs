use super::*;
use crate::{UiCell, UiColumn, UiConnectionSummary, UiQueryResult, UiSslMode, UiTableColumn, UiTableInfo};

fn connect(app: &mut DbProApp) {
    *app.connection.catalog.connections_mut() = vec![UiConnectionSummary {
        id: "active".to_owned(),
        name: "Active".to_owned(),
        host: "localhost".to_owned(),
        port: 5432,
        database: "active".to_owned(),
        username: "postgres".to_owned(),
        driver: "PostgreSQL".to_owned(),
        ssl_mode: UiSslMode::Disable,
        readonly: false,
        tags: vec![],
        group: None,
        favorite: false,
        environment: "Development".to_owned(),
    }];
    *app.connection.lifecycle.active_connection_id_mut() = Some("active".to_owned());
    app.connection.lifecycle.set_connected(true);
    app.query.execution.query_row_limit = None;
}

fn idle(app: &mut DbProApp) {
    app.query.session.documents[0].execution_state = QueryExecutionState::Idle;
}

fn press(app: &mut DbProApp, ctx: &egui::Context, key: egui::Key, modifiers: egui::Modifiers) -> bool {
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1024.0, 640.0))),
        events: vec![
            egui::Event::ModifiersChanged(modifiers),
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            },
        ],
        ..Default::default()
    };
    let mut editor_held_keyboard = false;
    let _ = crate::test_frame::frame(ctx, input, |ui| {
        let mut scratch = String::new();
        ui.text_edit_singleline(&mut scratch).request_focus();
        editor_held_keyboard = ui.ctx().egui_wants_keyboard_input();
        app.query.editor.query_editor_focused = true;
        app.handle_shortcuts(ui.ctx());
    });
    editor_held_keyboard
}

/// Run, run-all, and format come from the catalog and still fire while the editor holds the keyboard.
#[test]
fn catalog_shortcuts_win_while_the_editor_is_focused() {
    let ids: Vec<_> = settings_model::default_keybinding_catalog()
        .iter()
        .map(|command| command.id)
        .collect();
    assert!(ids.contains(&"query.run"));
    assert!(ids.contains(&"query.run_all"));
    assert!(ids.contains(&"query.format"));
    let defaults = settings_model::KeybindingSettings::default();
    assert_eq!(defaults.resolved("query.run"), "f5");
    assert_eq!(defaults.resolved("query.run_all"), "mod+shift+enter");
    assert_eq!(defaults.resolved("query.format"), "mod+shift+f");

    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    connect(&mut app);
    app.set_active_query_text("SELECT 1;\nSELECT 2;");
    let ctx = egui::Context::default();

    let focused = press(&mut app, &ctx, egui::Key::F5, egui::Modifiers::NONE);
    assert!(focused, "the editor must hold the keyboard");
    let UiCommand::RunQuery { sql, .. } = command_rx.try_recv().expect("F5 runs while the editor is focused") else {
        panic!("expected RunQuery");
    };
    assert_eq!(sql, "SELECT 1;");

    idle(&mut app);
    let focused = press(
        &mut app,
        &ctx,
        egui::Key::Enter,
        egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    assert!(focused);
    assert!(
        command_rx.try_recv().is_err(),
        "Mod+Enter is not a hardcoded run chord when query.run is F5"
    );

    app.preferences.settings.keybindings.overrides.insert("query.run".into(), "mod+enter".into());
    let focused = press(
        &mut app,
        &ctx,
        egui::Key::Enter,
        egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    assert!(focused);
    let UiCommand::RunQuery { .. } = command_rx.try_recv().expect("rebound query.run handles Mod+Enter") else {
        panic!("expected RunQuery");
    };
    app.preferences.settings.keybindings.overrides.remove("query.run");
    idle(&mut app);

    let focused = press(
        &mut app,
        &ctx,
        egui::Key::Enter,
        egui::Modifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        },
    );
    assert!(focused);
    let UiCommand::RunQueryMulti { sql, .. } = command_rx.try_recv().expect("run-all") else {
        panic!("expected RunQueryMulti");
    };
    assert_eq!(sql, "SELECT 1;\nSELECT 2;");

    app.set_active_query_text("select 1 from t");
    let len = app.query.session.documents[0].buffer.len_bytes();
    app.query.session.documents[0].selection = crate::editor::SelectionRange::new(0, len);
    let focused = press(
        &mut app,
        &ctx,
        egui::Key::F,
        egui::Modifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        },
    );
    assert!(focused);
    let formatted = app.query.session.documents[0].buffer.text().to_owned();
    assert!(formatted.contains('\n'), "{formatted}");

    let before = app.query.session.documents.len();
    let focused = press(
        &mut app,
        &ctx,
        egui::Key::T,
        egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    assert!(focused);
    assert_eq!(app.query.session.documents.len(), before + 1);
    assert!(app.query.editor.editor_search_open == false);

    let focused = press(
        &mut app,
        &ctx,
        egui::Key::F,
        egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    assert!(focused);
    assert!(app.query.editor.editor_search_open);
}

fn shortcut_named(items: &[PaletteItem], title: &str) -> Option<String> {
    items
        .iter()
        .find(|item| item.title == title)
        .and_then(|item| item.shortcut.clone())
}

/// Palette chords are the catalog's resolved token, and Explain runs the query plan.
#[test]
fn palette_rows_use_resolved_shortcuts() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    connect(&mut app);
    app.set_active_query_text("SELECT 1");

    let bindings = &app.preferences.settings.keybindings;
    let commands = app.filtered_palette_items(PaletteMode::Commands);
    assert_eq!(shortcut_named(&commands, "Run query").as_deref(), Some(bindings.resolved("query.run").as_str()));
    assert_eq!(
        shortcut_named(&commands, "Format SQL").as_deref(),
        Some(bindings.resolved("query.format").as_str())
    );
    assert_eq!(
        shortcut_named(&commands, "New query").as_deref(),
        Some(bindings.resolved("query.new").as_str())
    );
    assert_eq!(
        shortcut_named(&commands, "New connection").as_deref(),
        Some(bindings.resolved("connection.new").as_str())
    );
    assert_eq!(
        shortcut_named(&commands, "Toggle explorer").as_deref(),
        Some(bindings.resolved("view.toggle_sidebar").as_str())
    );

    app.preferences
        .settings
        .keybindings
        .overrides
        .insert("query.run".into(), "mod+enter".into());
    let commands = app.filtered_palette_items(PaletteMode::Commands);
    assert_eq!(shortcut_named(&commands, "Run query").as_deref(), Some("mod+enter"));

    let quick = app.filtered_palette_items(PaletteMode::QuickOpen);
    let query_tab = quick
        .iter()
        .find(|item| matches!(item.action, PaletteAction::Query))
        .expect("query tab row");
    assert!(query_tab.shortcut.is_none());

    let explain = commands
        .iter()
        .find(|item| item.title == "Explain current query")
        .expect("explain row");
    assert!(!explain.subtitle.contains("Agent action"));
    let action = explain.action.clone();
    app.execute_palette_action(action, &egui::Context::default());
    let UiCommand::ExplainQuery { sql, analyze, .. } = command_rx.try_recv().expect("explain dispatches") else {
        panic!("expected ExplainQuery");
    };
    assert_eq!(sql, "SELECT 1");
    assert!(!analyze);
}

/// `table.open` is a catalog chord and opens the selected table on the palette path.
#[test]
fn table_open_shortcut_opens_the_selected_table() {
    let ids: Vec<_> = settings_model::default_keybinding_catalog()
        .iter()
        .map(|command| command.id)
        .collect();
    assert!(ids.contains(&"table.open"));
    assert_eq!(
        settings_model::KeybindingSettings::default().resolved("table.open"),
        "mod+o"
    );

    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    connect(&mut app);
    let ctx = egui::Context::default();
    let modifiers = egui::Modifiers {
        ctrl: true,
        ..Default::default()
    };

    let focused = press(&mut app, &ctx, egui::Key::O, modifiers);
    assert!(focused);
    assert_eq!(app.palette.mode, Some(PaletteMode::QuickOpen));
    assert_eq!(app.palette.scope, SearchScope::Schema);
    assert!(command_rx.try_recv().is_err());

    app.palette.mode = None;
    app.schema.explorer.selected_table = Some("orders".to_owned());
    let focused = press(&mut app, &ctx, egui::Key::O, modifiers);
    assert!(focused);
    assert_eq!(app.workspace.active_tab, WorkspaceTab::Table);
    assert_eq!(app.schema.explorer.selected_table.as_deref(), Some("orders"));
    let UiCommand::LoadTableInfo {
        connection_id,
        schema,
        table,
        ..
    } = command_rx.try_recv().expect("open table loads its structure")
    else {
        panic!("expected LoadTableInfo");
    };
    assert_eq!(connection_id, "active");
    assert_eq!(schema, "public");
    assert_eq!(table, "orders");
}

fn orders_result() -> UiQueryResult {
    UiQueryResult {
        columns: vec![
            UiColumn {
                name: "id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
            },
            UiColumn {
                name: "name".to_owned(),
                data_type: "text".to_owned(),
                nullable: false,
            },
        ],
        rows: vec![vec![UiCell::Number("1".to_owned()), UiCell::Text("Ada".to_owned())]],
        row_count: 1,
        duration_ms: 1,
    }
}

fn orders_table() -> UiTableInfo {
    UiTableInfo {
        schema: "public".to_owned(),
        name: "orders".to_owned(),
        row_count: Some(1),
        columns: vec![
            UiTableColumn {
                name: "id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
                is_primary_key: true,
                ..UiTableColumn::default()
            },
            UiTableColumn {
                name: "name".to_owned(),
                data_type: "text".to_owned(),
                nullable: false,
                ..UiTableColumn::default()
            },
        ],
        primary_key: Some(vec!["id".to_owned()]),
        indexes: Vec::new(),
        foreign_keys: Vec::new(),
        check_constraints: Vec::new(),
        dependencies: Vec::new(),
    }
}

fn grid_frame(
    app: &mut DbProApp,
    ctx: &egui::Context,
    result: &UiQueryResult,
    key: egui::Key,
    modifiers: egui::Modifiers,
    text_focused: bool,
) {
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1024.0, 640.0))),
        events: vec![
            egui::Event::ModifiersChanged(modifiers),
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            },
        ],
        ..Default::default()
    };
    let _ = crate::test_frame::frame(ctx, input, |ui| {
        if text_focused {
            let mut scratch = String::new();
            ui.text_edit_singleline(&mut scratch).request_focus();
        }
        app.handle_shortcuts(ui.ctx());
        app.draw_result_grid(ui, result);
    });
}

/// The focus command opens the result dock and lets copy, discard, and delete run.
#[test]
fn focus_result_grid_enables_copy_discard_and_delete() {
    assert_eq!(
        settings_model::KeybindingSettings::default().resolved("result.focus"),
        "mod+shift+r"
    );
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    connect(&mut app);
    app.table.state.table_info = Some(orders_table());
    app.table.data.selected_cell = Some((0, 1));
    app.table.data.selected_row = Some(0);
    let result = orders_result();
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let ctrl = egui::Modifiers {
        ctrl: true,
        ..Default::default()
    };

    grid_frame(&mut app, &ctx, &result, egui::Key::C, ctrl, true);
    assert_ne!(app.feedback.copy_status, "Cell copied");

    grid_frame(
        &mut app,
        &ctx,
        &result,
        egui::Key::R,
        egui::Modifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        },
        true,
    );
    assert!(app.workspace.bottom_panel_open);
    assert_eq!(
        app.query.output.active_tab_for_document(app.query.session.active_document().map(|document| document.id.as_str())),
        OutputTab::Results
    );
    assert_eq!(
        ctx.memory(|memory| memory.focused()),
        Some(result_grid_keyboard_view::result_grid_focus_id())
    );

    grid_frame(&mut app, &ctx, &result, egui::Key::C, ctrl, false);
    assert_eq!(app.feedback.copy_status, "Cell copied");

    app.table.editing.data_editing_cell = Some((0, 1));
    app.table.editing.data_edit_value = "Bea".to_owned();
    assert!(app.submit_data_cell_edit(&result, 0, 1));
    assert!(app.staged_cell_value(&result, 0, 1).is_some());
    grid_frame(&mut app, &ctx, &result, egui::Key::Z, ctrl, false);
    assert!(app.staged_cell_value(&result, 0, 1).is_none());

    app.table.data.selected_row = Some(0);
    grid_frame(&mut app, &ctx, &result, egui::Key::Delete, egui::Modifiers::NONE, false);
    assert!(app.staged_row_deleted(&result, 0));
}

fn token_matches(ctx: &egui::Context, token: &str, key: egui::Key, modifiers: egui::Modifiers) -> bool {
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 400.0))),
        events: vec![
            egui::Event::ModifiersChanged(modifiers),
            egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            },
        ],
        ..Default::default()
    };
    let mut matched = false;
    let _ = crate::test_frame::frame(ctx, input, |ui| {
        matched = ui.input(|input| settings_model::match_shortcut_token(input, token));
    });
    matched
}

/// The matcher accepts slash, digits, function keys, and alt, Save refuses an unknown token, then comment toggles.
#[test]
fn shortcut_matcher_accepts_extended_tokens_and_toggles_comment() {
    let ctx = egui::Context::default();
    let ctrl = egui::Modifiers {
        ctrl: true,
        ..Default::default()
    };
    assert!(token_matches(&ctx, "mod+/", egui::Key::Slash, ctrl));
    assert!(token_matches(&ctx, "0", egui::Key::Num0, egui::Modifiers::NONE));
    assert!(token_matches(
        &ctx,
        "alt+1",
        egui::Key::Num1,
        egui::Modifiers {
            alt: true,
            ..Default::default()
        }
    ));
    assert!(token_matches(&ctx, "f12", egui::Key::F12, egui::Modifiers::NONE));
    assert!(!token_matches(
        &ctx,
        "f5",
        egui::Key::F5,
        egui::Modifiers {
            alt: true,
            ..Default::default()
        }
    ));

    let mut bindings = settings_model::KeybindingSettings::default();
    assert!(!bindings.save_override("query.run", "f5", "mod+banana"));
    assert_eq!(bindings.resolved("query.run"), "f5");
    assert!(bindings.save_override("query.run", "f5", "mod+/"));
    assert_eq!(bindings.resolved("query.run"), "mod+/");
    assert!(bindings.save_override("query.run", "f5", "f12"));
    assert_eq!(bindings.resolved("query.run"), "f12");
    assert!(bindings.save_override("query.run", "f5", ""));
    assert_eq!(bindings.resolved("query.run"), "f5");
    assert_eq!(bindings.resolved("editor.comment"), "mod+/");

    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    connect(&mut app);
    app.set_active_query_text("SELECT 1");
    let focused = press(&mut app, &ctx, egui::Key::Slash, ctrl);
    assert!(focused);
    assert_eq!(app.query.session.documents[0].buffer.text(), "-- SELECT 1");
    let focused = press(&mut app, &ctx, egui::Key::Slash, ctrl);
    assert!(focused);
    assert_eq!(app.query.session.documents[0].buffer.text(), "SELECT 1");
}
