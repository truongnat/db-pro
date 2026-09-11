use super::diagram_view::diagram_candidates;
use super::*;

fn result() -> UiQueryResult {
    UiQueryResult {
        columns: vec![
            crate::UiColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
            },
            crate::UiColumn {
                name: "name".to_owned(),
                data_type: "text".to_owned(),
                nullable: false,
            },
        ],
        rows: vec![
            vec![
                crate::UiCell::Number("2".to_owned()),
                crate::UiCell::Text("Beta".to_owned()),
            ],
            vec![
                crate::UiCell::Number("1".to_owned()),
                crate::UiCell::Text("Alpha".to_owned()),
            ],
            vec![
                crate::UiCell::Number("3".to_owned()),
                crate::UiCell::Text("Gamma".to_owned()),
            ],
        ],
        row_count: 3,
        duration_ms: 2,
    }
}

#[test]
fn filter_returns_original_row_indexes() {
    let app = DbProApp {
        grid_filter: "gamma".to_owned(),
        ..Default::default()
    };
    let value = result();
    assert_eq!(
        crate::filtered_sorted_indexes(&value, &app.grid_filter, app.grid_sort_column, app.grid_sort_desc),
        vec![2]
    );
}

#[test]
fn sort_is_stable_over_filtered_indexes() {
    let mut app = DbProApp {
        grid_sort_column: Some(0),
        ..Default::default()
    };
    let value = result();
    assert_eq!(
        crate::filtered_sorted_indexes(&value, &app.grid_filter, app.grid_sort_column, app.grid_sort_desc),
        vec![1, 0, 2]
    );
    app.grid_sort_desc = true;
    assert_eq!(
        crate::filtered_sorted_indexes(&value, &app.grid_filter, app.grid_sort_column, app.grid_sort_desc),
        vec![2, 0, 1]
    );
}

#[test]
fn cell_text_keeps_null_and_json_visible() {
    assert_eq!(crate::cell_text(&crate::UiCell::Null), "NULL");
    assert_eq!(
        crate::cell_text(&crate::UiCell::Json("{\"ok\":true}".to_owned())),
        "{\"ok\":true}"
    );
}

#[test]
fn displayed_row_number_tracks_database_page_offset() {
    assert_eq!(crate::displayed_row_number(0, 0), 1);
    assert_eq!(crate::displayed_row_number(100, 0), 101);
    assert_eq!(crate::displayed_row_number(100, 49), 150);
}

#[test]
fn composite_primary_key_identity_preserves_each_cell_type() {
    let result = UiQueryResult {
        columns: vec![
            crate::UiColumn {
                name: "tenant_id".to_owned(),
                data_type: "INTEGER".to_owned(),
                nullable: false,
            },
            crate::UiColumn {
                name: "item_id".to_owned(),
                data_type: "TEXT".to_owned(),
                nullable: false,
            },
        ],
        rows: vec![vec![UiCell::Number("7".to_owned()), UiCell::Text("sku-42".to_owned())]],
        row_count: 1,
        duration_ms: 0,
    };
    let info = UiTableInfo {
        schema: "main".to_owned(),
        name: "inventory".to_owned(),
        row_count: Some(1),
        columns: Vec::new(),
        primary_key: Some(vec!["tenant_id".to_owned(), "item_id".to_owned()]),
        indexes: Vec::new(),
        foreign_keys: Vec::new(),
    };

    let (columns, values) = DbProApp::row_identity(&result, &info, 0).expect("row identity expected");

    assert_eq!(columns, vec!["tenant_id", "item_id"]);
    assert_eq!(
        values,
        vec![UiCell::Number("7".to_owned()), UiCell::Text("sku-42".to_owned())]
    );
}

#[test]
fn update_value_keeps_empty_text_and_parses_typed_values() {
    assert_eq!(
        DbProApp::parse_update_value("", "TEXT").unwrap(),
        UiCell::Text(String::new())
    );
    assert_eq!(
        DbProApp::parse_update_value("false", "BOOLEAN").unwrap(),
        UiCell::Boolean(false)
    );
    assert!(DbProApp::parse_update_value("not-an-int", "INTEGER").is_err());
}

#[test]
fn insert_value_respects_column_types() {
    assert_eq!(
        DbProApp::parse_insert_value("42", "INTEGER").unwrap(),
        Some(crate::UiCell::Number("42".to_owned()))
    );
    assert_eq!(
        DbProApp::parse_insert_value("true", "BOOLEAN").unwrap(),
        Some(crate::UiCell::Boolean(true))
    );
    assert_eq!(
        DbProApp::parse_insert_value("{\"active\":true}", "JSONB").unwrap(),
        Some(crate::UiCell::Json("{\"active\":true}".to_owned()))
    );
    assert_eq!(
        DbProApp::parse_insert_value("12.50", "NUMERIC(10,2)").unwrap(),
        Some(crate::UiCell::Text("12.50".to_owned()))
    );
    assert_eq!(
        DbProApp::parse_insert_value("1.20e1", "DECIMAL(10,2)").unwrap(),
        Some(crate::UiCell::Text("1.20e1".to_owned()))
    );
}

#[test]
fn insert_value_rejects_invalid_typed_input() {
    assert!(DbProApp::parse_insert_value("maybe", "BOOLEAN").is_err());
    assert!(DbProApp::parse_insert_value("not-json", "JSON").is_err());
    assert!(DbProApp::parse_insert_value("4.2", "INTEGER").is_err());
    assert!(DbProApp::parse_insert_value("12.345", "NUMERIC(10,2)").is_err());
    assert!(DbProApp::parse_insert_value("123456789.01", "NUMERIC(10,2)").is_err());
    assert!(DbProApp::parse_update_value("", "DECIMAL(10,2)").is_err());
}

#[test]
fn table_edits_stage_until_explicit_apply() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.connections = vec![UiConnectionSummary {
        id: "conn-1".to_owned(),
        name: "Local".to_owned(),
        host: "localhost".to_owned(),
        port: 5432,
        database: "app".to_owned(),
        username: "postgres".to_owned(),
        driver: "PostgreSQL".to_owned(),
        readonly: false,
    }];
    app.active_connection_id = Some("conn-1".to_owned());
    app.connected = true;
    app.selected_table = Some("customers".to_owned());
    app.table_info = Some(UiTableInfo {
        schema: "public".to_owned(),
        name: "customers".to_owned(),
        row_count: Some(1),
        columns: vec![
            crate::UiTableColumn {
                name: "id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
                default: None,
                is_primary_key: true,
            },
            crate::UiTableColumn {
                name: "name".to_owned(),
                data_type: "text".to_owned(),
                nullable: false,
                default: None,
                is_primary_key: false,
            },
        ],
        primary_key: Some(vec!["id".to_owned()]),
        indexes: Vec::new(),
        foreign_keys: Vec::new(),
    });
    app.data_edit_value = "Updated".to_owned();
    let value = UiQueryResult {
        columns: vec![
            crate::UiColumn {
                name: "id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
            },
            crate::UiColumn {
                name: "name".to_owned(),
                data_type: "text".to_owned(),
                nullable: false,
            },
        ],
        rows: vec![vec![
            UiCell::Number("1".to_owned()),
            UiCell::Text("Original".to_owned()),
        ]],
        row_count: 1,
        duration_ms: 1,
    };

    app.submit_data_cell_edit(&value, 0, 1);

    assert_eq!(app.staged_changes.len(), 1);
    assert!(command_rx.try_recv().is_err());
    app.apply_staged_changes();
    assert!(matches!(command_rx.try_recv(), Ok(UiCommand::UpdateTableRow { .. })));
}

#[test]
fn explain_query_uses_selected_connection_and_switches_output() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.connections = vec![UiConnectionSummary {
        id: "conn-1".to_owned(),
        name: "Local".to_owned(),
        host: "localhost".to_owned(),
        port: 5432,
        database: "app".to_owned(),
        username: "postgres".to_owned(),
        driver: "PostgreSQL".to_owned(),
        readonly: false,
    }];
    app.active_connection_id = Some("conn-1".to_owned());
    app.connected = true;
    app.query_text = "SELECT 1".to_owned();

    app.explain_query();

    let UiCommand::ExplainQuery {
        request_id,
        connection_id,
        sql,
    } = command_rx.try_recv().expect("explain command expected")
    else {
        panic!("expected ExplainQuery");
    };
    assert_eq!(connection_id, "conn-1");
    assert_eq!(sql, "SELECT 1");
    assert_eq!(app.explain_request, Some(request_id));
    assert_eq!(app.output_tab, OutputTab::Explain);
}

#[test]
fn closing_agent_restores_sidebar_state_after_narrow_window() {
    let mut app = DbProApp::default();
    let ctx = egui::Context::default();
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1024.0, 640.0))),
        ..Default::default()
    });

    app.set_agent_open(true, &ctx);
    assert!(!app.sidebar_open);

    app.set_agent_open(false, &ctx);
    assert!(app.sidebar_open);
    let _ = ctx.end_pass();
}

#[test]
fn selected_connection_is_not_shown_as_connected() {
    let connection = UiConnectionSummary {
        id: "conn-1".to_owned(),
        name: "Local".to_owned(),
        host: "localhost".to_owned(),
        port: 5432,
        database: "app".to_owned(),
        username: "postgres".to_owned(),
        driver: "PostgreSQL".to_owned(),
        readonly: false,
    };
    let mut app = DbProApp {
        connections: vec![connection],
        active_connection_id: Some("conn-1".to_owned()),
        connected: false,
        ..Default::default()
    };

    assert_eq!(app.active_connection_name(), "Local");
    assert_eq!(app.connection_indicator(&app.connections[0]).1, app.theme.accent);
    assert_eq!(app.statusbar_state().2, "Not connected");
    assert!(!app.can_mutate_active_connection());
    app.connected = true;
    assert_eq!(app.connection_indicator(&app.connections[0]).1, app.theme.success);
    assert_eq!(app.statusbar_state().2, "Connected");
    assert!(app.can_mutate_active_connection());
    app.connections[0].readonly = true;
    assert!(!app.can_mutate_active_connection());
    app.runtime_message = "Table data failed · timeout".to_owned();
    assert!(app.has_runtime_error());
    assert_eq!(app.statusbar_state().2, "Connected");
    app.connected = false;
    assert_eq!(app.statusbar_state().2, "Runtime error");
}

#[test]
fn provider_capabilities_gate_provider_specific_actions() {
    let sqlite = UiConnectionSummary {
        id: "sqlite".to_owned(),
        name: "SQLite".to_owned(),
        host: String::new(),
        port: 0,
        database: "app.db".to_owned(),
        username: String::new(),
        driver: "SQLite".to_owned(),
        readonly: false,
    };
    let postgres = UiConnectionSummary {
        driver: "PostgreSQL".to_owned(),
        ..sqlite.clone()
    };

    let sqlite_app = DbProApp {
        connections: vec![sqlite],
        active_connection_id: Some("sqlite".to_owned()),
        ..Default::default()
    };
    let postgres_app = DbProApp {
        connections: vec![postgres],
        active_connection_id: Some("sqlite".to_owned()),
        ..Default::default()
    };

    let sqlite_capabilities = sqlite_app.active_capabilities().expect("SQLite capabilities");
    assert!(!sqlite_capabilities.schema.functions);
    assert!(!sqlite_capabilities.features.server_sessions);
    assert!(sqlite_capabilities.features.backup);

    let postgres_capabilities = postgres_app.active_capabilities().expect("PostgreSQL capabilities");
    assert!(postgres_capabilities.schema.functions);
    assert!(postgres_capabilities.query.explain);
    assert!(postgres_capabilities.features.server_sessions);
}

#[test]
fn closing_query_document_restores_the_next_valid_document() {
    let mut app = DbProApp::default();
    app.new_query_document();
    app.query_text = "select 2".to_owned();
    app.persist_active_query_document();

    app.close_query_document(0);

    assert_eq!(app.query_documents.len(), 1);
    assert_eq!(app.active_query_document, 0);
    assert_eq!(app.query_text, "select 2");
    assert_eq!(app.runtime_message, "Closed Query 2");
}

#[test]
fn quick_open_filters_workspaces_by_title_and_description() {
    let app = DbProApp {
        palette_query: "relationship".to_owned(),
        ..Default::default()
    };
    let items = app.filtered_palette_items(PaletteMode::QuickOpen);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "ER diagram");
}

#[test]
fn active_schema_prefers_user_selection_and_loaded_schema_metadata() {
    let mut app = DbProApp::default();
    app.schema.schemas = vec!["public".to_owned(), "tenant1".to_owned()];

    assert_eq!(app.active_schema(), "public");
    app.selected_schema = Some("tenant1".to_owned());
    assert_eq!(app.active_schema(), "tenant1");
}

#[test]
fn active_schema_columns_do_not_include_other_schemas() {
    let mut app = DbProApp::default();
    app.schema.schemas = vec!["public".to_owned(), "tenant1".to_owned()];
    app.schema.columns = vec!["legacy_global_column".to_owned()];
    app.schema.table_details = vec![
        UiTableSummary {
            schema: "public".to_owned(),
            name: "customers".to_owned(),
            row_count: None,
            columns: vec![crate::UiSchemaColumn {
                name: "customer_id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
                is_primary_key: true,
            }],
            foreign_keys: Vec::new(),
        },
        UiTableSummary {
            schema: "tenant1".to_owned(),
            name: "orders".to_owned(),
            row_count: None,
            columns: vec![crate::UiSchemaColumn {
                name: "order_id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
                is_primary_key: true,
            }],
            foreign_keys: Vec::new(),
        },
    ];

    assert_eq!(app.active_schema_column_names(), vec!["customer_id"]);
    app.selected_schema = Some("tenant1".to_owned());
    assert_eq!(app.active_schema_column_names(), vec!["order_id"]);
}

#[test]
fn diagram_search_matches_schema_table_and_column_names() {
    let table = UiTableSummary {
        schema: "tenant1".to_owned(),
        name: "orders".to_owned(),
        row_count: None,
        columns: vec![crate::UiSchemaColumn {
            name: "customer_id".to_owned(),
            data_type: "integer".to_owned(),
            nullable: false,
            is_primary_key: false,
        }],
        foreign_keys: Vec::new(),
    };

    assert!(matches_diagram_search(&table, "tenant"));
    assert!(matches_diagram_search(&table, "order"));
    assert!(matches_diagram_search(&table, "customer"));
    assert!(!matches_diagram_search(&table, "invoice"));
}

#[test]
fn diagram_candidates_bound_clones_until_show_all_is_explicit() {
    let tables: Vec<_> = (0..8)
        .map(|index| UiTableSummary {
            schema: "public".to_owned(),
            name: format!("table_{index}"),
            row_count: None,
            columns: Vec::new(),
            foreign_keys: Vec::new(),
        })
        .collect();

    let (candidate_count, visible) = diagram_candidates(&tables, "", false, 5);
    assert_eq!(candidate_count, 8);
    assert_eq!(visible.len(), 6);

    let (candidate_count, search_results) = diagram_candidates(&tables, "table_7", true, 5);
    assert_eq!(candidate_count, 1);
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].name, "table_7");
}

#[test]
fn ddl_impact_summary_uses_plain_language_for_destructive_operations() {
    assert!(ddl_impact_summary("DROP TABLE customers", "customers").contains("recovery requires a backup"));
    assert!(
        ddl_impact_summary("ALTER TABLE customers ADD COLUMN note TEXT", "customers").contains("changes its structure")
    );
    assert!(ddl_impact_summary("CREATE INDEX", "customers").contains("adds or rebuilds"));
}

#[test]
fn explorer_search_matches_table_names_case_insensitively() {
    assert!(matches_explorer_table("customer_orders", "orders"));
    assert!(matches_explorer_table("CustomerOrders", "customer"));
    assert!(matches_explorer_table("customer_orders", ""));
    assert!(!matches_explorer_table("customer_orders", "invoice"));

    let tables = (0..=EXPLORER_MAX_TABLES)
        .map(|index| format!("orders_{index}"))
        .collect::<Vec<_>>();
    let (matching_count, visible_tables) = filtered_explorer_tables(&tables, "orders");
    assert_eq!(matching_count, EXPLORER_MAX_TABLES + 1);
    assert_eq!(visible_tables.len(), EXPLORER_MAX_TABLES);
}

#[test]
fn command_palette_new_query_keeps_a_query_entry_point() {
    let mut app = DbProApp::default();
    let ctx = egui::Context::default();
    app.execute_palette_action(PaletteAction::NewQuery, &ctx);

    assert_eq!(app.active_tab, WorkspaceTab::Query);
    assert_eq!(app.query_documents.len(), 2);
    assert!(app.palette_mode.is_none());
}

#[test]
fn command_palette_refresh_schema_bypasses_the_metadata_cache() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.connections = vec![UiConnectionSummary {
        id: "active".to_owned(),
        name: "Active".to_owned(),
        host: "localhost".to_owned(),
        port: 5432,
        database: "active".to_owned(),
        username: "postgres".to_owned(),
        driver: "PostgreSQL".to_owned(),
        readonly: false,
    }];
    app.active_connection_id = Some("active".to_owned());
    app.connected = true;
    let ctx = egui::Context::default();

    app.execute_palette_action(PaletteAction::RefreshSchema, &ctx);

    let UiCommand::IntrospectSchema {
        connection_id,
        force_refresh,
        ..
    } = command_rx.try_recv().expect("schema refresh command expected")
    else {
        panic!("expected IntrospectSchema command");
    };
    assert_eq!(connection_id, "active");
    assert!(force_refresh);
    assert_eq!(app.runtime_message, "Refreshing schema…");
}

#[test]
fn loading_connections_does_not_introspect_before_connecting() {
    let (bridge, command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    event_tx
        .send(UiEvent::ConnectionsLoaded {
            request_id: crate::RequestId(1),
            connections: vec![UiConnectionSummary {
                id: "local".to_owned(),
                name: "Local".to_owned(),
                host: "127.0.0.1".to_owned(),
                port: 5432,
                database: "postgres".to_owned(),
                username: "postgres".to_owned(),
                driver: "PostgreSQL".to_owned(),
                readonly: false,
            }],
        })
        .expect("connections should be queued");

    app.apply_runtime_events();

    assert!(command_rx.try_recv().is_err());
}

#[test]
fn connected_event_starts_schema_and_metadata_loading() {
    let (bridge, command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.pending_connection_request = Some(crate::RequestId(1));
    event_tx
        .send(UiEvent::Connected {
            request_id: crate::RequestId(1),
            connection_id: "local".to_owned(),
        })
        .expect("connected event should be queued");

    app.apply_runtime_events();

    assert!(matches!(command_rx.try_recv(), Ok(UiCommand::IntrospectSchema { .. })));
    assert!(matches!(command_rx.try_recv(), Ok(UiCommand::ListSavedQueries { .. })));
    assert!(matches!(command_rx.try_recv(), Ok(UiCommand::ListQueryFolders { .. })));
}

#[test]
fn command_palette_shortcut_is_available_from_the_native_shell() {
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let ctx = egui::Context::default();
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1024.0, 640.0))),
        modifiers: egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
        events: vec![egui::Event::Key {
            key: egui::Key::K,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                ..Default::default()
            },
        }],
        ..Default::default()
    });

    app.handle_shortcuts(&ctx);

    assert_eq!(app.palette_mode, Some(PaletteMode::QuickOpen));
    let _ = ctx.end_pass();
}

#[test]
fn command_palette_shortcut_accepts_mac_command_modifier() {
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let ctx = egui::Context::default();
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1024.0, 640.0))),
        modifiers: egui::Modifiers {
            mac_cmd: true,
            command: true,
            ..Default::default()
        },
        events: vec![egui::Event::Key {
            key: egui::Key::K,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                mac_cmd: true,
                command: true,
                ..Default::default()
            },
        }],
        ..Default::default()
    });

    app.handle_shortcuts(&ctx);

    assert_eq!(app.palette_mode, Some(PaletteMode::QuickOpen));
    let _ = ctx.end_pass();
}

#[test]
fn native_text_edit_maps_linux_ctrl_to_command_shortcuts() {
    let ctx = egui::Context::default();
    let id = egui::Id::new("native-text-edit-shortcuts");
    let mut value = "select me".to_owned();
    let screen_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1024.0, 640.0));

    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(screen_rect),
        ..Default::default()
    });
    egui::CentralPanel::default().show(&ctx, |ui| {
        ui.add(egui::TextEdit::singleline(&mut value).id(id)).request_focus();
    });
    let _ = ctx.end_pass();

    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(screen_rect),
        modifiers,
        events: vec![egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }],
        ..Default::default()
    });
    egui::CentralPanel::default().show(&ctx, |ui| {
        ui.add(egui::TextEdit::singleline(&mut value).id(id));
    });
    let _ = ctx.end_pass();

    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(screen_rect),
        events: vec![egui::Event::Text("replaced".to_owned())],
        ..Default::default()
    });
    egui::CentralPanel::default().show(&ctx, |ui| {
        ui.add(egui::TextEdit::singleline(&mut value).id(id));
    });
    let _ = ctx.end_pass();

    assert_eq!(value, "replaced");
}

#[test]
fn global_panel_shortcuts_do_not_steal_text_input_combinations() {
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let ctx = egui::Context::default();
    let id = egui::Id::new("shortcut-routing-input");
    let screen_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1024.0, 640.0));
    let mut value = String::new();

    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(screen_rect),
        ..Default::default()
    });
    egui::CentralPanel::default().show(&ctx, |ui| {
        ui.add(egui::TextEdit::singleline(&mut value).id(id)).request_focus();
    });
    let _ = ctx.end_pass();

    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(screen_rect),
        modifiers,
        events: vec![egui::Event::Key {
            key: egui::Key::B,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }],
        ..Default::default()
    });

    app.handle_shortcuts(&ctx);

    assert!(app.sidebar_open);
    let _ = ctx.end_pass();
}

#[test]
fn global_palette_shortcuts_do_not_steal_text_input_combinations() {
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let ctx = egui::Context::default();
    let id = egui::Id::new("palette-shortcut-input");
    let screen_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1024.0, 640.0));
    let mut value = String::new();

    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(screen_rect),
        ..Default::default()
    });
    egui::CentralPanel::default().show(&ctx, |ui| {
        ui.add(egui::TextEdit::singleline(&mut value).id(id)).request_focus();
    });
    let _ = ctx.end_pass();

    let modifiers = egui::Modifiers {
        ctrl: true,
        command: true,
        ..Default::default()
    };
    ctx.begin_pass(egui::RawInput {
        screen_rect: Some(screen_rect),
        modifiers,
        events: vec![egui::Event::Key {
            key: egui::Key::K,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }],
        ..Default::default()
    });

    app.handle_shortcuts(&ctx);

    assert_eq!(app.palette_mode, None);
    let _ = ctx.end_pass();
}

#[test]
fn failed_connection_request_clears_connecting_state_and_keeps_error() {
    let (bridge, _command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.connected = true;
    app.pending_connection_request = Some(crate::RequestId(42));
    event_tx
        .send(UiEvent::QueryFailed {
            request_id: crate::RequestId(42),
            message: "auth failed".to_owned(),
        })
        .expect("connection failure should be queued");

    app.apply_runtime_events();

    assert!(!app.connected);
    assert_eq!(app.pending_connection_request, None);
    assert_eq!(app.connection_error, "auth failed");
    assert_eq!(app.runtime_message, "Connection failed · auth failed");
}

#[test]
fn connection_mutation_refreshes_the_explorer_without_waiting_for_another_frame() {
    let (bridge, command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.connections_requested = true;
    app.pending_connection_request = Some(crate::RequestId(7));
    event_tx
        .send(UiEvent::OperationCompleted {
            request_id: crate::RequestId(7),
            operation: "connection.created".to_owned(),
        })
        .expect("connection mutation should be queued");

    app.apply_runtime_events();

    assert!(app.connections_requested);
    assert!(matches!(command_rx.try_recv(), Ok(UiCommand::ListConnections { .. })));
}

#[test]
fn sql_diagnostics_allow_expression_selects_without_from() {
    let diagnostics = DbProApp::parse_sql_diagnostics("SELECT 1 AS ok;", "PostgreSQL");

    assert!(diagnostics.is_empty());
}

#[test]
fn failed_schema_request_is_visible_and_retryable() {
    let (bridge, _command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.schema_request = Some(crate::RequestId(7));
    event_tx
        .send(UiEvent::QueryFailed {
            request_id: crate::RequestId(7),
            message: "missing field `from_columns`".to_owned(),
        })
        .expect("schema failure should be queued");

    app.apply_runtime_events();

    assert_eq!(app.schema_request, None);
    assert_eq!(app.schema_error.as_deref(), Some("missing field `from_columns`"));
    assert_eq!(
        app.runtime_message,
        "Schema introspection failed · missing field `from_columns`"
    );
}

#[test]
fn stale_schema_event_cannot_replace_the_selected_connection_schema() {
    let (bridge, _command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.schema.tables = vec!["current_table".to_owned()];
    app.schema_request = Some(crate::RequestId(2));
    event_tx
        .send(UiEvent::SchemaLoaded {
            request_id: crate::RequestId(1),
            schema: UiSchemaSummary {
                schemas: Vec::new(),
                tables: vec!["stale_table".to_owned()],
                columns: Vec::new(),
                table_details: Vec::new(),
                views: Vec::new(),
                triggers: Vec::new(),
                functions: Vec::new(),
            },
        })
        .expect("stale schema event should be queued");

    app.apply_runtime_events();

    assert_eq!(app.schema.tables, vec!["current_table"]);
    assert_eq!(app.schema_request, Some(crate::RequestId(2)));
}

#[test]
fn connection_test_success_is_invalidated_when_the_draft_changes() {
    let (bridge, command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.connection_draft.name = "Local".to_owned();
    app.connection_draft.database = "app".to_owned();
    app.dispatch_connection_command(false);

    let request_id = match command_rx.try_recv().expect("test command expected") {
        UiCommand::TestConnection { request_id, .. } => request_id,
        _ => panic!("expected TestConnection command"),
    };
    app.connection_draft.database = "other".to_owned();
    event_tx
        .send(UiEvent::OperationCompleted {
            request_id,
            operation: "connection.tested".to_owned(),
        })
        .expect("test result should be queued");

    app.apply_runtime_events();

    assert!(!app.connection_test_valid);
    assert_eq!(app.runtime_message, "Connection changed · test again before saving");
}

#[test]
fn schema_refresh_reloads_the_selected_table_after_summary_completion() {
    let (bridge, command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.connections = vec![UiConnectionSummary {
        id: "active".to_owned(),
        name: "Active".to_owned(),
        host: String::new(),
        port: 0,
        database: "active".to_owned(),
        username: String::new(),
        driver: "SQLite".to_owned(),
        readonly: false,
    }];
    app.active_connection_id = Some("active".to_owned());
    app.selected_table = Some("customers".to_owned());
    app.active_tab = WorkspaceTab::Table;
    app.refresh_table_info_after_schema = true;
    event_tx
        .send(UiEvent::SchemaLoaded {
            request_id: crate::RequestId(1),
            schema: UiSchemaSummary {
                schemas: vec!["main".to_owned()],
                tables: vec!["customers".to_owned()],
                columns: vec!["id".to_owned()],
                table_details: Vec::new(),
                views: Vec::new(),
                triggers: Vec::new(),
                functions: Vec::new(),
            },
        })
        .expect("schema event should be queued");

    app.apply_runtime_events();

    let UiCommand::LoadTableInfo {
        connection_id,
        schema,
        table,
        ..
    } = command_rx.try_recv().expect("table metadata refresh expected")
    else {
        panic!("expected LoadTableInfo command");
    };
    assert_eq!(connection_id, "active");
    assert_eq!(schema, "main");
    assert_eq!(table, "customers");
    assert!(!app.refresh_table_info_after_schema);
}

#[test]
fn schema_refresh_returns_to_welcome_when_selected_table_disappears() {
    let (bridge, _command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.selected_table = Some("deleted_table".to_owned());
    app.active_tab = WorkspaceTab::Table;
    event_tx
        .send(UiEvent::SchemaLoaded {
            request_id: crate::RequestId(1),
            schema: UiSchemaSummary {
                schemas: Vec::new(),
                tables: vec!["remaining_table".to_owned()],
                columns: Vec::new(),
                table_details: Vec::new(),
                views: Vec::new(),
                triggers: Vec::new(),
                functions: Vec::new(),
            },
        })
        .expect("schema event should be queued");

    app.apply_runtime_events();

    assert_eq!(app.selected_table, None);
    assert_eq!(app.active_tab, WorkspaceTab::Welcome);
}

#[test]
fn closing_workspace_tab_clears_its_resource_and_requests() {
    let mut app = DbProApp {
        active_tab: WorkspaceTab::Table,
        selected_table: Some("customers".to_owned()),
        table_info_request: Some(crate::RequestId(1)),
        table_ddl_request: Some(crate::RequestId(2)),
        table_data_request: Some(crate::RequestId(3)),
        table_data_result: Some(result()),
        ..Default::default()
    };

    app.request_close_workspace_tab(WorkspaceTab::Table);

    assert_eq!(app.active_tab, WorkspaceTab::Welcome);
    assert_eq!(app.selected_table, None);
    assert_eq!(app.table_info_request, None);
    assert_eq!(app.table_ddl_request, None);
    assert_eq!(app.table_data_request, None);
    assert_eq!(app.table_data_result, None);
}

#[test]
fn query_dispatch_uses_the_active_connection_not_the_first_connection() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.connections = vec![
        UiConnectionSummary {
            id: "first".to_owned(),
            name: "First".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "first".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            readonly: false,
        },
        UiConnectionSummary {
            id: "active".to_owned(),
            name: "Active".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "active".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            readonly: false,
        },
    ];
    app.active_connection_id = Some("active".to_owned());
    app.connected = true;
    app.dispatch_query();

    let UiCommand::RunQuery { connection_id, .. } = command_rx.try_recv().expect("query command expected") else {
        panic!("expected RunQuery command");
    };
    assert_eq!(connection_id, "active");
}

#[test]
fn ddl_apply_dispatch_requires_an_explicit_request_and_uses_active_connection() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.connections = vec![UiConnectionSummary {
        id: "active".to_owned(),
        name: "Active".to_owned(),
        host: "localhost".to_owned(),
        port: 5432,
        database: "active".to_owned(),
        username: "postgres".to_owned(),
        driver: "PostgreSQL".to_owned(),
        readonly: false,
    }];
    app.active_connection_id = Some("active".to_owned());
    app.connected = true;
    app.table_ddl = Some("CREATE TABLE \"public\".\"audit\" (id INTEGER)".to_owned());

    app.submit_ddl();

    let UiCommand::ExecuteDdl { connection_id, sql, .. } = command_rx.try_recv().expect("DDL command expected") else {
        panic!("expected ExecuteDdl command");
    };
    assert_eq!(connection_id, "active");
    assert_eq!(sql, "CREATE TABLE \"public\".\"audit\" (id INTEGER)");
    assert!(app.ddl_execution_request.is_some());
}
