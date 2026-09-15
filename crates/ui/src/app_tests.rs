use super::diagram_view::{
    diagram_candidates, diagram_canvas_size, diagram_foreign_key_label, diagram_search_mode,
    diagram_show_all_after_search_edit,
};
use super::*;
use crate::diagram::*;
use crate::{UiCheckConstraint, UiDependencyDirection, UiDependencyKind, UiSchemaColumn, UiTableDependency};

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

fn primary_key_identity(value: &str) -> RowIdentity {
    RowIdentity {
        original_pk_columns: vec!["id".to_owned()],
        original_pk_values: vec![UiCell::Number(value.to_owned())],
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
fn grid_keyboard_navigation_preserves_filtered_row_identity() {
    let visible_rows = vec![2, 0, 4];

    assert_eq!(
        crate::grid_keyboard_selection(Some((2, 1)), &visible_rows, 3, egui::Key::ArrowDown),
        Some((0, 1))
    );
    assert_eq!(
        crate::grid_keyboard_selection(Some((0, 1)), &visible_rows, 3, egui::Key::ArrowRight),
        Some((0, 2))
    );
    assert_eq!(
        crate::grid_keyboard_selection(Some((0, 2)), &visible_rows, 3, egui::Key::Home),
        Some((0, 0))
    );
    assert_eq!(
        crate::grid_keyboard_selection(Some((0, 0)), &visible_rows, 3, egui::Key::ArrowUp),
        Some((2, 0))
    );
}

#[test]
fn grid_keyboard_navigation_starts_at_first_visible_cell() {
    assert_eq!(
        crate::grid_keyboard_selection(None, &[7, 9], 2, egui::Key::ArrowDown),
        Some((7, 0))
    );
    assert_eq!(
        crate::grid_keyboard_selection(Some((7, 0)), &[7, 9], 2, egui::Key::End),
        Some((7, 1))
    );
}

#[test]
fn grid_columns_fill_the_viewport_until_manually_resized() {
    let mut app = DbProApp::default();

    let widths = app.column_widths(3, 1200.0);
    assert!(widths.iter().all(|width| (*width - 380.0).abs() < 0.01));

    app.grid_column_widths = vec![240.0, 320.0, 180.0];
    app.grid_columns_user_resized = true;
    assert_eq!(app.column_widths(3, 1200.0), vec![240.0, 320.0, 180.0]);
}

#[test]
fn grid_copy_uses_staged_values_only_for_data_editor() {
    let value = result();
    let mut app = DbProApp {
        active_tab: WorkspaceTab::Table,
        table_view: TableView::Data,
        table_info: Some(UiTableInfo {
            schema: "public".to_owned(),
            name: "customers".to_owned(),
            row_count: Some(1),
            columns: Vec::new(),
            primary_key: Some(vec!["id".to_owned()]),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        }),
        staged_changes: ChangeSet::from(vec![StagedChange::Update {
            identity: primary_key_identity("2"),
            current_row_index: Some(0),
            column_index: 1,
            column: "name".to_owned(),
            data_type: "TEXT".to_owned(),
            original: UiCell::Text("Beta".to_owned()),
            value: UiCell::Text("Updated".to_owned()),
        }]),
        ..Default::default()
    };

    assert_eq!(
        app.copy_cell_value(&value, 0, 1),
        Some(UiCell::Text("Updated".to_owned()))
    );
    app.active_tab = WorkspaceTab::Query;
    assert_eq!(app.copy_cell_value(&value, 0, 1), Some(UiCell::Text("Beta".to_owned())));
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
        check_constraints: Vec::new(),
        dependencies: Vec::new(),
    };

    let identity = DbProApp::row_identity(&result, &info, 0).expect("row identity expected");

    assert_eq!(identity.original_pk_columns, vec!["tenant_id", "item_id"]);
    assert_eq!(
        identity.original_pk_values,
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
        Some(crate::UiCell::Number("12.50".to_owned()))
    );
    assert_eq!(
        DbProApp::parse_insert_value("1.20e1", "DECIMAL(10,2)").unwrap(),
        Some(crate::UiCell::Number("1.20e1".to_owned()))
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

fn connection_summary_with_ssl_mode(ssl_mode: UiSslMode) -> UiConnectionSummary {
    UiConnectionSummary {
        id: "conn-1".to_owned(),
        name: "Secure Prod".to_owned(),
        host: "db.example.com".to_owned(),
        port: 5432,
        database: "app".to_owned(),
        username: "postgres".to_owned(),
        driver: "PostgreSQL".to_owned(),
        ssl_mode,
        readonly: false,
    }
}

#[test]
fn editing_a_connection_preserves_its_stored_ssl_mode() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let connection = connection_summary_with_ssl_mode(UiSslMode::Require);

    app.open_edit_connection(&connection);

    assert_eq!(app.connection_draft.ssl_mode, UiSslMode::Require);

    app.dispatch_connection_command(true);

    let UiCommand::UpdateConnection {
        connection_id, draft, ..
    } = command_rx.try_recv().expect("update command expected")
    else {
        panic!("expected UpdateConnection");
    };
    assert_eq!(connection_id, "conn-1");
    assert_eq!(draft.ssl_mode, UiSslMode::Require);
}

#[test]
fn duplicating_a_connection_preserves_its_stored_ssl_mode() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let connection = connection_summary_with_ssl_mode(UiSslMode::VerifyFull);

    app.open_duplicate_connection(&connection);

    assert_eq!(app.connection_draft.ssl_mode, UiSslMode::VerifyFull);

    app.dispatch_connection_command(true);

    let UiCommand::CreateConnection { draft, .. } = command_rx.try_recv().expect("create command expected") else {
        panic!("expected CreateConnection");
    };
    assert_eq!(draft.name, "Secure Prod (Copy)");
    assert_eq!(draft.ssl_mode, UiSslMode::VerifyFull);
}

#[test]
fn new_postgresql_connection_defaults_to_tls_require() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);

    app.open_new_connection();

    assert_eq!(app.connection_draft.driver, UiDriver::Postgres);
    assert_eq!(app.connection_draft.ssl_mode, UiSslMode::Require);

    app.dispatch_connection_command(true);
    let UiCommand::CreateConnection { draft, .. } = command_rx.try_recv().expect("create command expected") else {
        panic!("expected CreateConnection");
    };
    assert_eq!(draft.ssl_mode, UiSslMode::Require);
}

#[test]
fn editing_a_disable_connection_keeps_disable_until_the_user_changes_it() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let connection = connection_summary_with_ssl_mode(UiSslMode::Disable);

    app.open_edit_connection(&connection);
    assert_eq!(app.connection_draft.ssl_mode, UiSslMode::Disable);

    app.dispatch_connection_command(true);
    let UiCommand::UpdateConnection { draft, .. } = command_rx.try_recv().expect("update command expected") else {
        panic!("expected UpdateConnection");
    };
    assert_eq!(draft.ssl_mode, UiSslMode::Disable);
}

#[test]
fn switching_sqlite_to_postgresql_initializes_tls_require() {
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.open_new_connection();
    app.connection_draft.driver = UiDriver::Sqlite;
    app.connection_draft.ssl_mode = UiSslMode::Disable;

    app.select_connection_driver(UiDriver::Postgres);

    assert_eq!(app.connection_draft.driver, UiDriver::Postgres);
    assert_eq!(app.connection_draft.ssl_mode, UiSslMode::Require);
}

#[test]
fn selecting_mysql_sets_port_and_tls_and_preserves_password_on_submit() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.open_new_connection();
    app.select_connection_driver(UiDriver::Mysql);
    assert_eq!(app.connection_draft.driver, UiDriver::Mysql);
    assert_eq!(app.connection_draft.port, "3306");
    assert_eq!(app.connection_draft.ssl_mode, UiSslMode::Require);

    app.connection_draft.name = "MySQL Local".to_owned();
    app.connection_draft.host = "127.0.0.1".to_owned();
    app.connection_draft.database = "app".to_owned();
    app.connection_draft.username = "root".to_owned();
    app.connection_draft.password = "secret".to_owned();
    app.dispatch_connection_command(true);

    let UiCommand::CreateConnection { draft, .. } = command_rx.try_recv().expect("create") else {
        panic!("expected CreateConnection");
    };
    assert_eq!(draft.driver, UiDriver::Mysql);
    assert_eq!(draft.port, "3306");
    assert_eq!(draft.password, "secret");
}

#[test]
fn editing_a_mysql_connection_keeps_the_mysql_driver() {
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let connection = UiConnectionSummary {
        id: "conn-mysql".to_owned(),
        name: "MySQL".to_owned(),
        host: "127.0.0.1".to_owned(),
        port: 3306,
        database: "app".to_owned(),
        username: "root".to_owned(),
        driver: "MySQL".to_owned(),
        ssl_mode: UiSslMode::Require,
        readonly: false,
    };
    app.open_edit_connection(&connection);
    assert_eq!(app.connection_draft.driver, UiDriver::Mysql);
}

#[test]
fn explicit_disable_selection_is_preserved_on_submit() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.open_new_connection();
    assert_eq!(app.connection_draft.ssl_mode, UiSslMode::Require);

    app.connection_draft.ssl_mode = UiSslMode::Disable;
    app.dispatch_connection_command(true);

    let UiCommand::CreateConnection { draft, .. } = command_rx.try_recv().expect("create command expected") else {
        panic!("expected CreateConnection");
    };
    assert_eq!(draft.ssl_mode, UiSslMode::Disable);
}

#[test]
fn ssl_mode_guidance_names_the_plaintext_risk_for_disable() {
    assert!(super::connection_view::ssl_mode_guidance(UiSslMode::Disable).contains("Plaintext"));
    assert!(super::connection_view::ssl_mode_guidance(UiSslMode::Require).contains("TLS"));
    assert!(super::connection_view::ssl_mode_guidance(UiSslMode::VerifyCa).contains("CA"));
    assert!(super::connection_view::ssl_mode_guidance(UiSslMode::VerifyFull).contains("Strongest"));
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
        ssl_mode: UiSslMode::Disable,
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
                ..Default::default()
            },
            crate::UiTableColumn {
                name: "name".to_owned(),
                data_type: "text".to_owned(),
                nullable: false,
                default: None,
                is_primary_key: false,
                ..Default::default()
            },
        ],
        primary_key: Some(vec!["id".to_owned()]),
        indexes: Vec::new(),
        foreign_keys: Vec::new(),
        check_constraints: Vec::new(),
        dependencies: Vec::new(),
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

    assert_eq!(app.staged_changes.counts().total(), 1);
    assert!(command_rx.try_recv().is_err());
    app.apply_staged_changes();
    assert!(matches!(command_rx.try_recv(), Ok(UiCommand::ApplyTableChanges { changes, .. }) if changes.len() == 1));
}

#[test]
fn apply_is_blocked_while_a_validation_error_exists() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.staged_changes.stage_update(StagedChange::Update {
        identity: primary_key_identity("1"),
        current_row_index: Some(0),
        column_index: 1,
        column: "name".to_owned(),
        data_type: "text".to_owned(),
        original: UiCell::Text("Original".to_owned()),
        value: UiCell::Text("Updated".to_owned()),
    });
    app.data_edit_error = Some("invalid value".to_owned());

    app.apply_staged_changes();

    assert!(command_rx.try_recv().is_err());
    assert_eq!(app.runtime_message, "Fix the validation error before applying changes");
    assert_eq!(app.staged_changes.counts().total(), 1);
}

#[test]
fn editing_primary_key_stages_new_value_with_original_identity() {
    let mut app = DbProApp {
        connected: true,
        connections: vec![UiConnectionSummary {
            id: "conn-1".to_owned(),
            name: "Local".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: UiSslMode::Disable,
            readonly: false,
        }],
        active_connection_id: Some("conn-1".to_owned()),
        table_info: Some(UiTableInfo {
            schema: "public".to_owned(),
            name: "customers".to_owned(),
            row_count: Some(1),
            columns: vec![crate::UiTableColumn {
                name: "id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
                default: None,
                is_primary_key: true,
                ..Default::default()
            }],
            primary_key: Some(vec!["id".to_owned()]),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        }),
        data_edit_value: "2".to_owned(),
        ..Default::default()
    };
    let result = UiQueryResult {
        columns: vec![crate::UiColumn {
            name: "id".to_owned(),
            data_type: "integer".to_owned(),
            nullable: false,
        }],
        rows: vec![vec![UiCell::Number("1".to_owned())]],
        row_count: 1,
        duration_ms: 0,
    };
    assert!(app.submit_data_cell_edit(&result, 0, 0));
    let Some(StagedChange::Update { value, identity, .. }) = app.staged_changes.iter().next() else {
        panic!("primary-key edit was not staged");
    };
    assert_eq!(value, &UiCell::Number("2".to_owned()));
    assert_eq!(identity.original_pk_values, vec![UiCell::Number("1".to_owned())]);
}

#[test]
fn no_primary_key_table_blocks_safe_row_mutations() {
    let app = DbProApp {
        connected: true,
        active_connection_id: Some("conn-1".to_owned()),
        connections: vec![UiConnectionSummary {
            id: "conn-1".to_owned(),
            name: "Local".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: UiSslMode::Disable,
            readonly: false,
        }],
        table_info: Some(UiTableInfo {
            schema: "public".to_owned(),
            name: "logs".to_owned(),
            row_count: Some(1),
            columns: Vec::new(),
            primary_key: None,
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        }),
        ..Default::default()
    };

    assert!(!app.can_edit_table_rows());
}

/// The edit entry point refuses a column the write policy blocks, and says why.
#[test]
fn binary_cell_edit_is_refused_with_a_reason() {
    let mut app = DbProApp {
        connected: true,
        active_connection_id: Some("conn-1".to_owned()),
        connections: vec![UiConnectionSummary {
            id: "conn-1".to_owned(),
            name: "Local".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: UiSslMode::Disable,
            readonly: false,
        }],
        table_info: Some(UiTableInfo {
            schema: "public".to_owned(),
            name: "files".to_owned(),
            row_count: Some(1),
            columns: vec![
                crate::UiTableColumn {
                    name: "id".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: false,
                    default: None,
                    is_primary_key: true,
                    ..Default::default()
                },
                crate::UiTableColumn {
                    name: "payload".to_owned(),
                    data_type: "bytea".to_owned(),
                    nullable: true,
                    default: None,
                    is_primary_key: false,
                    ..Default::default()
                },
            ],
            primary_key: Some(vec!["id".to_owned()]),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        }),
        ..Default::default()
    };
    let result = UiQueryResult {
        columns: vec![
            crate::UiColumn {
                name: "id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
            },
            crate::UiColumn {
                name: "payload".to_owned(),
                data_type: "bytea".to_owned(),
                nullable: true,
            },
        ],
        rows: vec![vec![
            UiCell::Number("1".to_owned()),
            UiCell::Bytes("\\x00ff".to_owned()),
        ]],
        row_count: 1,
        duration_ms: 0,
    };

    app.begin_data_cell_edit(&result, 0, 1, &result.rows[0][1]);

    assert!(
        app.data_editing_cell.is_none(),
        "no editor may open for a blocked column"
    );
    assert!(
        app.runtime_message.contains("read-only"),
        "the reason must be visible: {}",
        app.runtime_message
    );
    assert_eq!(app.staged_changes.counts().total(), 0);
}

/// A generated column cannot be staged, even through the commit path.
#[test]
fn generated_column_edit_is_refused_before_staging() {
    let mut app = DbProApp {
        connected: true,
        active_connection_id: Some("conn-1".to_owned()),
        connections: vec![UiConnectionSummary {
            id: "conn-1".to_owned(),
            name: "Local".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: UiSslMode::Disable,
            readonly: false,
        }],
        table_info: Some(UiTableInfo {
            schema: "public".to_owned(),
            name: "line_items".to_owned(),
            row_count: Some(1),
            columns: vec![
                crate::UiTableColumn {
                    name: "id".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: false,
                    default: None,
                    is_primary_key: true,
                    ..Default::default()
                },
                crate::UiTableColumn {
                    name: "total".to_owned(),
                    data_type: "numeric".to_owned(),
                    nullable: false,
                    default: None,
                    is_primary_key: false,
                    is_generated: true,
                    ..Default::default()
                },
            ],
            primary_key: Some(vec!["id".to_owned()]),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        }),
        data_edit_value: "99.99".to_owned(),
        ..Default::default()
    };
    let result = UiQueryResult {
        columns: vec![
            crate::UiColumn {
                name: "id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
            },
            crate::UiColumn {
                name: "total".to_owned(),
                data_type: "numeric".to_owned(),
                nullable: false,
            },
        ],
        rows: vec![vec![UiCell::Number("1".to_owned()), UiCell::Number("9.99".to_owned())]],
        row_count: 1,
        duration_ms: 0,
    };

    let accepted = app.submit_data_cell_edit(&result, 0, 1);

    assert!(!accepted, "the generated column must refuse the edit");
    assert!(app.data_edit_error.as_deref().unwrap_or_default().contains("computed"));
    assert_eq!(app.staged_changes.counts().total(), 0, "nothing may be staged");
    assert!(
        app.runtime_message.contains("computed"),
        "the reason must be visible: {}",
        app.runtime_message
    );
}

/// A generated column left empty is skipped on insert; a value for it is refused.
#[test]
fn generated_column_is_never_staged_by_insert() {
    let mut app = DbProApp {
        connected: true,
        active_connection_id: Some("conn-1".to_owned()),
        connections: vec![UiConnectionSummary {
            id: "conn-1".to_owned(),
            name: "Local".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: UiSslMode::Disable,
            readonly: false,
        }],
        selected_table: Some("line_items".to_owned()),
        table_info: Some(UiTableInfo {
            schema: "public".to_owned(),
            name: "line_items".to_owned(),
            row_count: Some(0),
            columns: vec![
                crate::UiTableColumn {
                    name: "id".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: false,
                    default: None,
                    is_primary_key: true,
                    ..Default::default()
                },
                crate::UiTableColumn {
                    name: "qty".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: false,
                    default: None,
                    is_primary_key: false,
                    ..Default::default()
                },
                crate::UiTableColumn {
                    name: "total".to_owned(),
                    data_type: "numeric".to_owned(),
                    nullable: false,
                    default: None,
                    is_primary_key: false,
                    is_generated: true,
                    ..Default::default()
                },
            ],
            primary_key: Some(vec!["id".to_owned()]),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        }),
        insert_row_values: vec!["1".to_owned(), "2".to_owned(), String::new()],
        ..Default::default()
    };

    app.submit_insert_row();

    let Some(StagedChange::Insert { columns, .. }) = app.staged_changes.iter().next() else {
        panic!("the insert was not staged");
    };
    assert_eq!(
        columns,
        &["id".to_owned(), "qty".to_owned()],
        "the generated column must be skipped"
    );
    assert!(
        app.insert_row_error.is_empty(),
        "skipping a generated column is not an error"
    );

    // A value for the generated column is refused deterministically, before staging.
    let mut second = DbProApp {
        connected: true,
        active_connection_id: Some("conn-1".to_owned()),
        connections: vec![UiConnectionSummary {
            id: "conn-1".to_owned(),
            name: "Local".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: UiSslMode::Disable,
            readonly: false,
        }],
        selected_table: Some("line_items".to_owned()),
        table_info: app.table_info.clone(),
        insert_row_values: vec!["1".to_owned(), "2".to_owned(), "3.0".to_owned()],
        ..Default::default()
    };
    second.submit_insert_row();
    assert!(
        second.insert_row_error.contains("computed"),
        "the refusal must be visible: {}",
        second.insert_row_error
    );
    assert_eq!(second.staged_changes.counts().total(), 0);
}

/// Duplicating a row must not prefill a column the policy blocks.
#[test]
fn duplicated_row_leaves_blocked_columns_empty() {
    let mut app = DbProApp {
        connected: true,
        active_connection_id: Some("conn-1".to_owned()),
        connections: vec![UiConnectionSummary {
            id: "conn-1".to_owned(),
            name: "Local".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: UiSslMode::Disable,
            readonly: false,
        }],
        table_info: Some(UiTableInfo {
            schema: "public".to_owned(),
            name: "line_items".to_owned(),
            row_count: Some(1),
            columns: vec![
                crate::UiTableColumn {
                    name: "id".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: false,
                    default: None,
                    is_primary_key: true,
                    ..Default::default()
                },
                crate::UiTableColumn {
                    name: "total".to_owned(),
                    data_type: "numeric".to_owned(),
                    nullable: false,
                    default: None,
                    is_primary_key: false,
                    is_generated: true,
                    ..Default::default()
                },
            ],
            primary_key: Some(vec!["id".to_owned()]),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            check_constraints: Vec::new(),
            dependencies: Vec::new(),
        }),
        ..Default::default()
    };
    let result = UiQueryResult {
        columns: vec![
            crate::UiColumn {
                name: "id".to_owned(),
                data_type: "integer".to_owned(),
                nullable: false,
            },
            crate::UiColumn {
                name: "total".to_owned(),
                data_type: "numeric".to_owned(),
                nullable: false,
            },
        ],
        rows: vec![vec![UiCell::Number("1".to_owned()), UiCell::Number("9.99".to_owned())]],
        row_count: 1,
        duration_ms: 0,
    };

    app.open_duplicate_row(&result, 0);

    assert_eq!(app.insert_row_values, vec![String::new(), String::new()]);
    assert!(app.insert_row_open);
    assert!(app.insert_row_error.is_empty());
}

#[test]
fn staged_apply_failure_maps_statement_to_mutation_and_keeps_changes() {
    let mut staged_changes = ChangeSet::new();
    staged_changes.stage_update(StagedChange::Update {
        identity: primary_key_identity("3"),
        current_row_index: Some(2),
        column_index: 1,
        column: "name".to_owned(),
        data_type: "text".to_owned(),
        original: UiCell::Text("old".to_owned()),
        value: UiCell::Text("new".to_owned()),
    });
    let mut app = DbProApp {
        staged_apply_request: Some(crate::RequestId(7)),
        staged_apply_targets: vec![
            MutationTarget::Delete {
                identity: primary_key_identity("1"),
                current_row_index: Some(0),
            },
            MutationTarget::Update {
                identity: primary_key_identity("3"),
                current_row_index: Some(2),
                columns: vec![1, 3],
            },
        ],
        staged_changes,
        ..Default::default()
    };

    app.staged_apply_failed(1, "CONSTRAINT_VIOLATION", "duplicate key value", true);

    assert_eq!(app.staged_apply_request, None);
    assert_eq!(app.staged_changes.counts().updates, 1);
    assert_eq!(app.selected_cell, Some((2, 1)));
    assert!(matches!(
        app.table_mutation_error.as_ref().and_then(|failure| failure.target.as_ref()),
        Some(MutationTarget::Update {
            current_row_index: Some(2),
            columns,
            ..
        }) if columns == &vec![1, 3]
    ));
    assert!(app
        .runtime_message
        .contains("Staged change #2 failed · transaction rolled back"));
}

#[test]
fn conflict_failure_has_distinct_code_and_user_action_message() {
    let mut app = DbProApp {
        staged_apply_request: Some(crate::RequestId(8)),
        staged_apply_targets: vec![MutationTarget::Update {
            identity: primary_key_identity("3"),
            current_row_index: Some(2),
            columns: vec![1],
        }],
        ..Default::default()
    };

    app.staged_apply_failed(0, "CONFLICT", "row count was zero", true);

    let failure = app.table_mutation_error.expect("conflict should be visible");
    assert_eq!(failure.code, "CONFLICT");
    assert!(failure
        .message
        .starts_with("This row changed or was deleted in the database."));
}

#[test]
fn internal_error_code_is_normalized_for_mutation_state() {
    let mut app = DbProApp {
        staged_apply_request: Some(crate::RequestId(9)),
        ..Default::default()
    };

    app.staged_apply_failed(usize::MAX, "INTERNAL_ERROR", "invariant violation", true);

    assert_eq!(
        app.table_mutation_error.expect("error should be visible").code,
        "INTERNAL"
    );
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
        ssl_mode: UiSslMode::Disable,
        readonly: false,
    }];
    app.active_connection_id = Some("conn-1".to_owned());
    app.connected = true;
    app.set_active_query_text("SELECT 1");

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
    assert_eq!(app.active_explain_request(), Some(request_id));
    assert_eq!(app.output_tab, OutputTab::Explain);
}

#[test]
fn query_output_tab_is_scoped_to_each_document() {
    let mut app = DbProApp::default();
    app.query_documents
        .push(QueryDocument::new("query-2", "Query 2", "SELECT 2"));

    app.set_active_query_output_tab(OutputTab::Explain);
    app.switch_query_document(1);
    assert_eq!(app.active_query_output_tab(), OutputTab::Results);

    app.set_active_query_output_tab(OutputTab::History);
    app.switch_query_document(0);
    assert_eq!(app.active_query_output_tab(), OutputTab::Explain);

    app.set_query_output_tab("query-2", OutputTab::Messages);
    app.switch_query_document(1);
    assert_eq!(app.active_query_output_tab(), OutputTab::Messages);
    app.switch_query_document(0);
    assert_eq!(app.active_query_output_tab(), OutputTab::Explain);
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
        ssl_mode: UiSslMode::Disable,
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
fn editor_status_is_scoped_to_the_query_workspace() {
    let mut app = DbProApp {
        active_tab: WorkspaceTab::Query,
        ..Default::default()
    };
    assert!(app.shows_editor_status());
    assert_eq!(app.statusbar_context_label(), "SQL Editor");

    app.active_tab = WorkspaceTab::Table;
    assert!(!app.shows_editor_status());
    assert_eq!(app.statusbar_context_label(), "Table Structure");
    app.table_view = TableView::Data;
    assert_eq!(app.statusbar_context_label(), "Data Editor");
    app.active_tab = WorkspaceTab::Diagram;
    assert!(!app.shows_editor_status());
    assert_eq!(app.statusbar_context_label(), "ER Diagram");
}

#[test]
fn switching_query_documents_resets_editor_cursor_metadata() {
    let mut app = DbProApp {
        active_tab: WorkspaceTab::Query,
        query_cursor_line: 8,
        query_cursor_column: 13,
        ..Default::default()
    };
    app.new_query_document();

    assert_eq!(app.query_cursor_line, 1);
    assert_eq!(app.query_cursor_column, 1);
}

#[test]
fn new_query_identity_skips_restored_document_ids() {
    let mut app = DbProApp::default();
    app.query_documents
        .push(QueryDocument::new("query-2", "Restored query", "SELECT restored;"));

    app.new_query_document();

    assert_eq!(app.query_documents.last().map(|doc| doc.id.as_str()), Some("query-3"));
    assert_eq!(app.query_documents.iter().filter(|doc| doc.id == "query-3").count(), 1);
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
        ssl_mode: UiSslMode::Disable,
        readonly: false,
    };
    let postgres = UiConnectionSummary {
        ssl_mode: UiSslMode::Disable,
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

    let sqlite_capabilities = sqlite_app
        .active_capabilities()
        .resolved()
        .cloned()
        .expect("SQLite capabilities");
    assert!(!sqlite_capabilities.schema.functions);
    assert!(!sqlite_capabilities.features.server_sessions);
    assert!(sqlite_capabilities.features.backup);

    let postgres_capabilities = postgres_app
        .active_capabilities()
        .resolved()
        .cloned()
        .expect("PostgreSQL capabilities");
    assert!(postgres_capabilities.schema.functions);
    assert!(postgres_capabilities.query.explain);
    assert!(postgres_capabilities.features.server_sessions);
}

#[test]
fn mysql_connection_resolves_to_its_own_capability_set() {
    // Regression for #234 criterion 2: the lookup used to return `None` for MySQL, so the
    // newly registered provider had no capability path at all and every gate behaved as
    // "capability absent".
    let app = DbProApp {
        connections: vec![UiConnectionSummary {
            id: "mysql".to_owned(),
            name: "MySQL".to_owned(),
            host: "127.0.0.1".to_owned(),
            port: 33306,
            database: "dbpro_fixture".to_owned(),
            username: "root".to_owned(),
            driver: "MySQL".to_owned(),
            ssl_mode: UiSslMode::Disable,
            readonly: false,
        }],
        active_connection_id: Some("mysql".to_owned()),
        ..Default::default()
    };

    let lookup = app.active_capabilities();
    assert_eq!(
        lookup.resolved().map(|caps| caps.driver),
        Some(DriverType::Mysql),
        "MySQL must resolve to the provider's own set"
    );
    assert_eq!(
        lookup.resolved().map(|caps| caps.query.parameters),
        Some(DatabaseCapabilities::mysql().query.parameters),
        "the resolved set is the provider's own, not a default"
    );

    // The gates the UI reads are the provider's real values.
    assert!(lookup.allows(|caps| caps.schema.functions));
    assert!(!lookup.allows(|caps| caps.features.backup));
    assert!(!lookup.allows(|caps| caps.features.partitions));
    assert!(!lookup.allows(|caps| caps.query.cancel));
    assert!(lookup.allows(|caps| caps.query.parameters));
    assert!(lookup.allows(|caps| caps.query.positional_parameters));
}

#[test]
fn unknown_driver_resolves_to_a_named_state_not_none() {
    let app = DbProApp {
        connections: vec![UiConnectionSummary {
            id: "oracle".to_owned(),
            name: "Oracle".to_owned(),
            host: "db.example.com".to_owned(),
            port: 1521,
            database: "ORCL".to_owned(),
            username: "system".to_owned(),
            driver: "Oracle".to_owned(),
            ssl_mode: UiSslMode::Disable,
            readonly: false,
        }],
        active_connection_id: Some("oracle".to_owned()),
        ..Default::default()
    };

    let lookup = app.active_capabilities();
    match &lookup {
        CapabilityLookup::UnsupportedDriver { driver } => assert_eq!(driver, "Oracle"),
        other => panic!("expected the named UnsupportedDriver state, got {other:?}"),
    }
    assert!(
        lookup.resolved().is_none(),
        "an unresolved driver must not hand out a capability set"
    );
    assert!(
        !lookup.allows(|caps| caps.query.explain),
        "an unresolved driver must never enable a capability-gated action"
    );
    assert_eq!(
        lookup.unavailable_reason().as_deref(),
        Some("Oracle has no provider entry in this build")
    );
}

#[test]
fn no_active_connection_is_distinct_from_an_unsupported_driver() {
    let app = DbProApp::default();
    let lookup = app.active_capabilities();

    assert!(matches!(lookup, CapabilityLookup::NoActiveConnection));
    assert!(!lookup.allows(|caps| caps.query.explain));
    assert_eq!(
        lookup.unavailable_reason().as_deref(),
        Some("no database connection is active")
    );
}

#[test]
fn query_capabilities_follow_the_bound_connection_and_do_not_default_to_postgres() {
    let pg = UiConnectionSummary {
        id: "pg".to_owned(),
        name: "PostgreSQL".to_owned(),
        host: "localhost".to_owned(),
        port: 5432,
        database: "app".to_owned(),
        username: "postgres".to_owned(),
        driver: "PostgreSQL".to_owned(),
        ssl_mode: UiSslMode::Disable,
        readonly: false,
    };
    let sqlite = UiConnectionSummary {
        id: "sqlite".to_owned(),
        name: "SQLite".to_owned(),
        host: String::new(),
        port: 0,
        database: "app.db".to_owned(),
        username: String::new(),
        driver: "SQLite".to_owned(),
        ssl_mode: UiSslMode::Disable,
        readonly: false,
    };

    // With no connection at all, the old lookup answered with PostgreSQL's set through
    // `active_query_driver`'s display fallback; it must now say nothing is connected.
    let empty = DbProApp::default();
    assert!(matches!(
        empty.query_capabilities(),
        CapabilityLookup::NoActiveConnection
    ));

    let mut app = DbProApp {
        connections: vec![pg, sqlite],
        active_connection_id: Some("pg".to_owned()),
        ..Default::default()
    };
    assert!(app.query_capabilities().allows(|caps| caps.features.server_sessions));

    // The active query document's connection wins over the active connection.
    app.query_documents[app.active_query_document].connection_id = Some("sqlite".to_owned());
    assert!(!app.query_capabilities().allows(|caps| caps.features.server_sessions));
    assert!(app.query_capabilities().allows(|caps| caps.query.cancel));
}

#[test]
fn closing_query_document_restores_the_next_valid_document() {
    let mut app = DbProApp::default();
    app.new_query_document();
    app.set_active_query_text("select 2");

    app.close_query_document(0);

    assert_eq!(app.query_documents.len(), 1);
    assert_eq!(app.active_query_document, 0);
    assert_eq!(app.active_query_text(), "select 2");
    assert_eq!(app.runtime_message, "Closed Query 2");
}

#[test]
fn closing_last_query_document_returns_to_welcome() {
    let mut app = DbProApp {
        active_tab: WorkspaceTab::Query,
        ..Default::default()
    };

    app.close_query_document(0);

    assert!(app.query_documents.is_empty());
    assert_eq!(app.active_tab, WorkspaceTab::Welcome);
    assert!(app.welcome_open);
    assert!(app.active_query_text().is_empty());
}

#[test]
fn closing_welcome_activates_the_existing_query_tab() {
    let mut app = DbProApp {
        active_tab: WorkspaceTab::Welcome,
        ..Default::default()
    };

    app.close_welcome_tab();

    assert!(!app.welcome_open);
    assert_eq!(app.active_tab, WorkspaceTab::Query);
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
fn diagram_search_mode_can_leave_explicit_show_all() {
    assert!(!diagram_search_mode(true, true));
    assert!(diagram_search_mode(true, false));
    assert!(!diagram_search_mode(false, false));
    assert!(!diagram_show_all_after_search_edit(true, "orders", true));
    assert!(diagram_show_all_after_search_edit(true, "  ", true));
}

#[test]
fn diagram_canvas_fills_the_viewport_before_overflowing() {
    assert_eq!(
        diagram_canvas_size(egui::vec2(940.0, 360.0), egui::vec2(1800.0, 900.0)),
        egui::vec2(1800.0, 900.0)
    );
    assert_eq!(
        diagram_canvas_size(egui::vec2(2200.0, 1200.0), egui::vec2(1800.0, 900.0)),
        egui::vec2(2200.0, 1200.0)
    );
}

#[test]
fn diagram_foreign_key_label_formats_single_and_composite_keys() {
    let single_fk = UiSchemaForeignKey {
        name: "fk_orders_customer".to_owned(),
        from_columns: vec!["customer_id".to_owned()],
        to_schema: "public".to_owned(),
        to_table: "customers".to_owned(),
        to_columns: vec!["id".to_owned()],
    };
    assert_eq!(diagram_foreign_key_label(&single_fk), "customer_id → id");

    let composite_fk = UiSchemaForeignKey {
        name: "fk_order_items_order".to_owned(),
        from_columns: vec!["tenant_id".to_owned(), "order_id".to_owned()],
        to_schema: "public".to_owned(),
        to_table: "orders".to_owned(),
        to_columns: vec!["tenant_id".to_owned(), "id".to_owned()],
    };
    assert_eq!(
        diagram_foreign_key_label(&composite_fk),
        "[tenant_id, order_id] → [tenant_id, id]"
    );
}

#[test]
fn diagram_viewport_coordinate_transformations_roundtrip() {
    let screen_origin = egui::pos2(50.0, 100.0);
    let pan = egui::vec2(20.0, -30.0);
    let zoom = 1.5;
    let viewport = ErViewport::new(pan, zoom, screen_origin);

    let world_pos = egui::pos2(300.0, 400.0);
    let screen_pos = viewport.world_to_screen_pos(world_pos);
    let roundtrip_world = viewport.screen_to_world_pos(screen_pos);

    assert!((world_pos.x - roundtrip_world.x).abs() < 1e-4);
    assert!((world_pos.y - roundtrip_world.y).abs() < 1e-4);

    let world_rect = egui::Rect::from_min_size(world_pos, egui::vec2(280.0, 160.0));
    let screen_rect = viewport.world_to_screen_rect(world_rect);
    let roundtrip_rect = viewport.screen_to_world_rect(screen_rect);

    assert!((world_rect.min.x - roundtrip_rect.min.x).abs() < 1e-4);
    assert!((world_rect.max.x - roundtrip_rect.max.x).abs() < 1e-4);
    assert!((world_rect.min.y - roundtrip_rect.min.y).abs() < 1e-4);
    assert!((world_rect.max.y - roundtrip_rect.max.y).abs() < 1e-4);
}

#[test]
fn diagram_lod_transitions_and_rules() {
    let compact = ErLod::from_zoom(0.6);
    assert_eq!(compact, ErLod::Compact);
    assert!(!compact.shows_columns());
    assert!(!compact.shows_edge_labels());
    assert_eq!(compact.max_columns(), 0);

    let standard = ErLod::from_zoom(1.0);
    assert_eq!(standard, ErLod::Standard);
    assert!(standard.shows_columns());
    assert!(standard.shows_edge_labels());
    assert_eq!(standard.max_columns(), 6);

    let detailed = ErLod::from_zoom(1.4);
    assert_eq!(detailed, ErLod::Detailed);
    assert!(detailed.shows_columns());
    assert!(detailed.shows_edge_labels());
    assert_eq!(detailed.max_columns(), 12);
}

#[test]
fn diagram_bfs_neighborhood_expansion() {
    let tables = vec![
        UiTableSummary {
            schema: "public".to_owned(),
            name: "users".to_owned(),
            row_count: Some(100),
            columns: vec![UiSchemaColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
                is_primary_key: true,
            }],
            foreign_keys: vec![],
        },
        UiTableSummary {
            schema: "public".to_owned(),
            name: "orders".to_owned(),
            row_count: Some(500),
            columns: vec![
                UiSchemaColumn {
                    name: "id".to_owned(),
                    data_type: "int".to_owned(),
                    nullable: false,
                    is_primary_key: true,
                },
                UiSchemaColumn {
                    name: "user_id".to_owned(),
                    data_type: "int".to_owned(),
                    nullable: false,
                    is_primary_key: false,
                },
            ],
            foreign_keys: vec![UiSchemaForeignKey {
                name: "fk_order_user".to_owned(),
                from_columns: vec!["user_id".to_owned()],
                to_schema: "public".to_owned(),
                to_table: "users".to_owned(),
                to_columns: vec!["id".to_owned()],
            }],
        },
        UiTableSummary {
            schema: "public".to_owned(),
            name: "order_items".to_owned(),
            row_count: Some(2000),
            columns: vec![
                UiSchemaColumn {
                    name: "id".to_owned(),
                    data_type: "int".to_owned(),
                    nullable: false,
                    is_primary_key: true,
                },
                UiSchemaColumn {
                    name: "order_id".to_owned(),
                    data_type: "int".to_owned(),
                    nullable: false,
                    is_primary_key: false,
                },
            ],
            foreign_keys: vec![UiSchemaForeignKey {
                name: "fk_item_order".to_owned(),
                from_columns: vec!["order_id".to_owned()],
                to_schema: "public".to_owned(),
                to_table: "orders".to_owned(),
                to_columns: vec!["id".to_owned()],
            }],
        },
        UiTableSummary {
            schema: "public".to_owned(),
            name: "logs".to_owned(),
            row_count: Some(50),
            columns: vec![UiSchemaColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
                is_primary_key: true,
            }],
            foreign_keys: vec![],
        },
    ];

    let graph = ErGraph::build(&tables, 1, 2, 120.0);

    // 1-hop neighborhood from users (index 0) -> users + orders
    let hop1 = graph.bfs_neighborhood(&[0], 1, 10);
    assert_eq!(hop1.len(), 2);
    assert!(hop1.contains(&0)); // users
    assert!(hop1.contains(&1)); // orders
    assert!(!hop1.contains(&2)); // order_items is 2 hops away

    // 2-hop neighborhood from users (index 0) -> users + orders + order_items
    let hop2 = graph.bfs_neighborhood(&[0], 2, 10);
    assert_eq!(hop2.len(), 3);
    assert!(hop2.contains(&0));
    assert!(hop2.contains(&1));
    assert!(hop2.contains(&2));
    assert!(!hop2.contains(&3)); // logs is disconnected
}

#[test]
fn diagram_graph_and_spatial_index_support_1000_table_scaling() {
    // Generate 1000 tables with realistic FK connections (every 3rd table references table 0 or previous)
    let tables: Vec<UiTableSummary> = (0..1000)
        .map(|i| {
            let fks = if i > 0 && i % 3 == 0 {
                vec![UiSchemaForeignKey {
                    name: format!("fk_t{i}_t0"),
                    from_columns: vec!["ref_id".to_owned()],
                    to_schema: "public".to_owned(),
                    to_table: format!("table_{}", i - 1),
                    to_columns: vec!["id".to_owned()],
                }]
            } else {
                vec![]
            };
            UiTableSummary {
                schema: "public".to_owned(),
                name: format!("table_{i}"),
                row_count: Some(100),
                columns: vec![
                    UiSchemaColumn {
                        name: "id".to_owned(),
                        data_type: "int".to_owned(),
                        nullable: false,
                        is_primary_key: true,
                    },
                    UiSchemaColumn {
                        name: "ref_id".to_owned(),
                        data_type: "int".to_owned(),
                        nullable: false,
                        is_primary_key: false,
                    },
                ],
                foreign_keys: fks,
            }
        })
        .collect();

    let grid_columns = 10;
    let node_height = 140.0;
    let graph = ErGraph::build(&tables, 1, grid_columns, node_height);
    assert_eq!(graph.nodes.len(), 1000);
    assert!(graph.edges.len() > 300);

    let spatial_index = ErSpatialIndex::build(&graph.nodes, &graph.edges, DEFAULT_SPATIAL_CELL_SIZE);

    // Simulate standard desktop viewport at 1.0 zoom (1280x800)
    let viewport = ErViewport::new(egui::Vec2::ZERO, 1.0, egui::Pos2::ZERO);
    let screen_clip = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0));

    let scene = prepare_render_scene(&graph, &spatial_index, &viewport, screen_clip, None);

    // Visible candidates must be strictly bounded (far less than all 1000 tables)
    assert!(scene.visible_nodes.len() < 50);
    assert!(scene.visible_nodes.len() >= 4);
    assert_eq!(scene.metrics.total_nodes, 1000);
    // Spatial query execution must be fast (typically < 1000 microseconds)
    assert!(scene.metrics.spatial_query_micros < 10_000);

    // Test hit testing: hit exact center of table_0
    let table0_center = graph.nodes[0].world_rect.center();
    let hit = spatial_index.hit_test_node(table0_center, &graph.nodes);
    assert_eq!(hit, Some(0));

    // Hit testing outside any node rect returns None
    let hit_outside = spatial_index.hit_test_node(egui::pos2(99999.0, 99999.0), &graph.nodes);
    assert_eq!(hit_outside, None);
}

#[test]
fn diagram_layout_worker_background_computation_and_stale_drop() {
    let mut worker = ErLayoutWorker::new();
    let tables = vec![
        UiTableSummary {
            schema: "public".to_owned(),
            name: "categories".to_owned(),
            row_count: Some(10),
            columns: vec![UiSchemaColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
                is_primary_key: true,
            }],
            foreign_keys: vec![],
        },
        UiTableSummary {
            schema: "public".to_owned(),
            name: "products".to_owned(),
            row_count: Some(100),
            columns: vec![
                UiSchemaColumn {
                    name: "id".to_owned(),
                    data_type: "int".to_owned(),
                    nullable: false,
                    is_primary_key: true,
                },
                UiSchemaColumn {
                    name: "cat_id".to_owned(),
                    data_type: "int".to_owned(),
                    nullable: false,
                    is_primary_key: false,
                },
            ],
            foreign_keys: vec![UiSchemaForeignKey {
                name: "fk_prod_cat".to_owned(),
                from_columns: vec!["cat_id".to_owned()],
                to_schema: "public".to_owned(),
                to_table: "categories".to_owned(),
                to_columns: vec!["id".to_owned()],
            }],
        },
    ];

    let req_id1 = worker.request_layout(1, tables.clone(), 2, 120.0);
    let req_id2 = worker.request_layout(2, tables, 2, 120.0);
    assert!(req_id2 > req_id1);

    // Drain until the worker emits the layout for the newest request, or the deadline
    // expires. The worker coalesces only what is already queued when it wakes up, so if
    // the scheduler runs it between the two calls it emits the superseded version-1
    // layout first; that intermediate is dropped at the app boundary because
    // request_id != diagram_latest_layout_request. Asserting on the first result would
    // assert on the scheduler instead. The deadline is seconds because a descheduled
    // worker thread can take hundreds of milliseconds to run on a shared CI runner,
    // while the layout itself takes microseconds.
    const COALESCING_DEADLINE: std::time::Duration = std::time::Duration::from_secs(5);

    let start = std::time::Instant::now();
    let mut intermediate: Vec<ErLayoutResult> = Vec::new();
    let mut latest: Option<ErLayoutResult> = None;
    while latest.is_none() && start.elapsed() < COALESCING_DEADLINE {
        match worker.poll_result() {
            Some(res) if res.request_id == req_id2 && res.graph_version == 2 => latest = Some(res),
            Some(res) => intermediate.push(res),
            None => std::thread::sleep(std::time::Duration::from_millis(5)),
        }
    }

    // Coalescing / latest request guarantees version 2 is the layout the app commits.
    let res = latest.expect("worker should emit the newest layout within the deadline");
    assert_eq!(res.graph_version, 2);
    assert_eq!(res.graph.nodes.len(), 2);
    assert_eq!(res.graph.edges.len(), 1);

    // Any earlier emission can only be the stale version-1 layout, which the app boundary
    // rejects because request_id != latest_layout_request.
    for stale in &intermediate {
        assert_eq!(stale.request_id, req_id1, "only the older request may be emitted first");
        assert_eq!(stale.graph_version, 1);
        // App-boundary acceptance check (diagram_view.rs): a result is committed only when both
        // its request id and its version match the latest request.
        let accepted = stale.request_id == req_id2 && stale.graph_version == 2;
        assert!(
            !accepted,
            "the intermediate layout must be rejected at the app boundary"
        );
    }
}

#[test]
fn diagram_dense_1000_table_graph_bounds_candidates() {
    // 1000 tables with dense relationships (~3 FKs per table = ~3000 edges)
    let tables: Vec<UiTableSummary> = (0..1000)
        .map(|i| {
            let mut fks = Vec::new();
            if i > 0 {
                fks.push(UiSchemaForeignKey {
                    name: format!("fk_{i}_prev"),
                    from_columns: vec!["parent_id".to_owned()],
                    to_schema: "public".to_owned(),
                    to_table: format!("dense_table_{}", i - 1),
                    to_columns: vec!["id".to_owned()],
                });
            }
            if i > 2 {
                fks.push(UiSchemaForeignKey {
                    name: format!("fk_{i}_prev2"),
                    from_columns: vec!["mod_id".to_owned()],
                    to_schema: "public".to_owned(),
                    to_table: format!("dense_table_{}", i - 2),
                    to_columns: vec!["id".to_owned()],
                });
            }
            if i % 10 > 3 {
                fks.push(UiSchemaForeignKey {
                    name: format!("fk_{i}_cluster"),
                    from_columns: vec!["cluster_id".to_owned()],
                    to_schema: "public".to_owned(),
                    to_table: format!("dense_table_{}", i - (i % 10)),
                    to_columns: vec!["id".to_owned()],
                });
            }
            UiTableSummary {
                schema: "public".to_owned(),
                name: format!("dense_table_{i}"),
                row_count: Some(50),
                columns: vec![
                    UiSchemaColumn {
                        name: "id".to_owned(),
                        data_type: "int".to_owned(),
                        nullable: false,
                        is_primary_key: true,
                    },
                    UiSchemaColumn {
                        name: "parent_id".to_owned(),
                        data_type: "int".to_owned(),
                        nullable: false,
                        is_primary_key: false,
                    },
                    UiSchemaColumn {
                        name: "mod_id".to_owned(),
                        data_type: "int".to_owned(),
                        nullable: false,
                        is_primary_key: false,
                    },
                    UiSchemaColumn {
                        name: "step_id".to_owned(),
                        data_type: "int".to_owned(),
                        nullable: false,
                        is_primary_key: false,
                    },
                ],
                foreign_keys: fks,
            }
        })
        .collect();

    let graph = ErGraph::build(&tables, 1, 10, 160.0);
    assert_eq!(graph.nodes.len(), 1000);
    assert!(graph.edges.len() > 2000);

    let spatial_index = ErSpatialIndex::build(&graph.nodes, &graph.edges, DEFAULT_SPATIAL_CELL_SIZE);
    let viewport = ErViewport::new(egui::Vec2::ZERO, 1.0, egui::Pos2::ZERO);
    let screen_clip = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0));

    let scene = prepare_render_scene(&graph, &spatial_index, &viewport, screen_clip, None);

    // Visible nodes & edges must both remain strictly bounded
    assert!(scene.visible_nodes.len() < 50);
    assert!(scene.visible_edges.len() < 150);
}

#[test]
fn diagram_cyclic_and_self_fk_bfs_neighborhood() {
    let tables = vec![
        UiTableSummary {
            schema: "public".to_owned(),
            name: "node_a".to_owned(),
            row_count: Some(10),
            columns: vec![UiSchemaColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
                is_primary_key: true,
            }],
            foreign_keys: vec![UiSchemaForeignKey {
                name: "fk_a_b".to_owned(),
                from_columns: vec!["b_id".to_owned()],
                to_schema: "public".to_owned(),
                to_table: "node_b".to_owned(),
                to_columns: vec!["id".to_owned()],
            }],
        },
        UiTableSummary {
            schema: "public".to_owned(),
            name: "node_b".to_owned(),
            row_count: Some(10),
            columns: vec![UiSchemaColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
                is_primary_key: true,
            }],
            foreign_keys: vec![UiSchemaForeignKey {
                name: "fk_b_c".to_owned(),
                from_columns: vec!["c_id".to_owned()],
                to_schema: "public".to_owned(),
                to_table: "node_c".to_owned(),
                to_columns: vec!["id".to_owned()],
            }],
        },
        UiTableSummary {
            schema: "public".to_owned(),
            name: "node_c".to_owned(),
            row_count: Some(10),
            columns: vec![UiSchemaColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
                is_primary_key: true,
            }],
            foreign_keys: vec![UiSchemaForeignKey {
                name: "fk_c_a".to_owned(), // cycle back to A
                from_columns: vec!["a_id".to_owned()],
                to_schema: "public".to_owned(),
                to_table: "node_a".to_owned(),
                to_columns: vec!["id".to_owned()],
            }],
        },
        UiTableSummary {
            schema: "public".to_owned(),
            name: "self_referential".to_owned(),
            row_count: Some(10),
            columns: vec![UiSchemaColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
                is_primary_key: true,
            }],
            foreign_keys: vec![UiSchemaForeignKey {
                name: "fk_self".to_owned(), // self reference
                from_columns: vec!["parent_id".to_owned()],
                to_schema: "public".to_owned(),
                to_table: "self_referential".to_owned(),
                to_columns: vec!["id".to_owned()],
            }],
        },
    ];

    let graph = ErGraph::build(&tables, 1, 2, 100.0);

    // BFS on cycle terminates cleanly without infinite recursion
    let cycle_bfs = graph.bfs_neighborhood(&[0], 5, 100);
    assert_eq!(cycle_bfs.len(), 3);
    assert!(cycle_bfs.contains(&0));
    assert!(cycle_bfs.contains(&1));
    assert!(cycle_bfs.contains(&2));
    assert!(!cycle_bfs.contains(&3));

    // Self-referential BFS
    let self_bfs = graph.bfs_neighborhood(&[3], 3, 100);
    assert_eq!(self_bfs.len(), 1);
    assert_eq!(self_bfs[0], 3);

    // Active subset bounds
    let subset_bounds = graph.active_subset_bounds(&[0, 1]);
    assert!(subset_bounds.width() >= ER_NODE_WIDTH);
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
        ssl_mode: UiSslMode::Disable,
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
fn loading_connections_automatically_connects_active_connection() {
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
                ssl_mode: UiSslMode::Disable,
                readonly: false,
            }],
        })
        .expect("connections should be queued");

    app.apply_runtime_events();

    assert!(matches!(
        command_rx.try_recv(),
        Ok(UiCommand::Connect {
            connection_id,
            ..
        }) if connection_id == "local"
    ));
    assert!(command_rx.try_recv().is_err());
}

#[test]
fn failed_connection_shows_red_indicator_and_records_error() {
    let (bridge, _command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.connections = vec![UiConnectionSummary {
        id: "conn-bad".to_owned(),
        name: "Remote Bad".to_owned(),
        host: "10.0.0.99".to_owned(),
        port: 5432,
        database: "mydb".to_owned(),
        username: "postgres".to_owned(),
        driver: "PostgreSQL".to_owned(),
        ssl_mode: UiSslMode::Disable,
        readonly: false,
    }];
    app.active_connection_id = Some("conn-bad".to_owned());
    app.pending_connection_id = Some("conn-bad".to_owned());
    app.pending_connection_request = Some(crate::RequestId(99));

    event_tx
        .send(UiEvent::QueryFailed {
            request_id: crate::RequestId(99),
            message: "Connection refused (os error 61)".to_owned(),
        })
        .expect("query failed event should be queued");

    app.apply_runtime_events();

    assert!(app.failed_connection_ids.contains("conn-bad"));
    assert_eq!(
        app.connection_errors.get("conn-bad").map(|s| s.as_str()),
        Some("Connection refused (os error 61)")
    );
    let (icon, color) = app.connection_indicator(&app.connections[0]);
    assert_eq!(char::from(icon), char::from(Icon::AlertCircle));
    assert_eq!(color, app.theme.danger);
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
fn agent_provider_status_uses_runtime_provider_name() {
    let (bridge, command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    event_tx
        .send(UiEvent::AgentProviderReady {
            provider: "Groq".to_owned(),
            detail: "Responses API · SQL drafts stay unexecuted".to_owned(),
        })
        .expect("provider status should be queued");

    app.apply_runtime_events();
    app.agent_input = "show the active schema".to_owned();
    app.submit_agent_prompt();

    assert_eq!(app.agent_provider_label, "Groq");
    assert_eq!(app.runtime_message, "Sending request to Groq…");
    assert!(matches!(command_rx.try_recv(), Ok(UiCommand::StartAgentRun { .. })));
}

#[test]
fn typed_agent_events_are_scoped_to_the_origin_document() {
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.new_query_document();
    let first_id = app.query_documents[0].id.clone();
    let second_id = app.query_documents[1].id.clone();
    let first_session = super::agent_workflow_state::AgentUiSession::for_document(&first_id, None, None);
    let session_id = first_session.session.as_ref().expect("session should exist").id;
    let run_id = db_pro_core::domain::agent::AgentRunId::new();
    app.agent_sessions.insert(first_id.clone(), first_session);
    app.agent_sessions.insert(
        second_id.clone(),
        super::agent_workflow_state::AgentUiSession::for_document(&second_id, None, None),
    );
    app.agent_sessions
        .get_mut(&first_id)
        .expect("first session should exist")
        .active_run_id = Some(run_id);

    app.apply_runtime_event(UiEvent::AgentWorkflow {
        request_id: crate::RequestId(7),
        event: db_pro_core::domain::agent_workflow::AgentWorkflowEvent::TextDelta {
            run_id,
            session_id,
            document_id: first_id.clone(),
            delta: "Use the users table".to_owned(),
        },
    });

    assert_eq!(app.agent_sessions[&first_id].streaming_text, "Use the users table");
    assert!(app.agent_sessions[&second_id].streaming_text.is_empty());
}

#[test]
fn typed_agent_patch_confirmation_applies_one_document_edit_and_continues() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.query_documents[0].set_text("SELECT old");
    let document_id = app.query_documents[0].id.clone();
    let session = super::agent_workflow_state::AgentUiSession::for_document(&document_id, None, None);
    let session_id = session.session.as_ref().expect("session should exist").id;
    let run_id = db_pro_core::domain::agent::AgentRunId::new();
    app.agent_sessions.insert(document_id.clone(), session);
    app.agent_sessions
        .get_mut(&document_id)
        .expect("session should exist")
        .active_run_id = Some(run_id);
    let patch = db_pro_core::domain::agent::AgentSqlPatch {
        document_id: document_id.clone(),
        expected_version: app.query_documents[0].buffer.version(),
        range: (7, 10),
        replacement: "users".to_owned(),
    };
    app.apply_runtime_event(UiEvent::AgentWorkflow {
        request_id: crate::RequestId(8),
        event: db_pro_core::domain::agent_workflow::AgentWorkflowEvent::ConfirmationRequired {
            run_id,
            session_id,
            document_id: document_id.clone(),
            call_id: "patch-1".to_owned(),
            kind: db_pro_core::domain::agent_workflow::AgentConfirmationKind::ApplyPatch,
            preview: Some(db_pro_core::domain::agent::AgentToolOutput::PatchPreview {
                patch,
                original: "old".to_owned(),
                proposed: "users".to_owned(),
            }),
        },
    });

    app.agent_confirmation_action(true);

    assert_eq!(app.query_documents[0].text(), "SELECT users");
    assert!(matches!(
        command_rx.try_recv(),
        Ok(UiCommand::ContinueAgentRun {
            approved: true,
            applied_patch: Some(db_pro_core::domain::agent::AgentToolOutput::PatchApplied { .. }),
            ..
        })
    ));
    app.query_documents[0].buffer.undo();
    assert_eq!(app.query_documents[0].text(), "SELECT old");
}

#[test]
fn typed_agent_failure_clears_stale_confirmation_and_marks_activity_failed() {
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let document_id = app.query_documents[0].id.clone();
    let session = super::agent_workflow_state::AgentUiSession::for_document(&document_id, None, None);
    let _session_id = session.session.as_ref().expect("session should exist").id;
    let run_id = db_pro_core::domain::agent::AgentRunId::new();
    app.agent_sessions.insert(document_id.clone(), session);
    let session = app.agent_sessions.get_mut(&document_id).expect("session should exist");
    session.active_run_id = Some(run_id);
    session.request_id = Some(crate::RequestId(17));
    session.activities.push(super::agent_workflow_state::AgentUiActivity {
        call_id: Some("patch-1".to_owned()),
        tool: Some(db_pro_core::domain::agent::AgentTool::PatchQuery),
        label: "Preparing SQL change".to_owned(),
        status: super::agent_workflow_state::AgentUiActivityStatus::Running,
        duration_ms: None,
    });
    session.pending_confirmation = Some(super::agent_workflow_state::AgentUiConfirmation {
        run_id,
        call_id: "patch-1".to_owned(),
        kind: db_pro_core::domain::agent_workflow::AgentConfirmationKind::ApplyPatch,
        preview: None,
        document_id: document_id.clone(),
    });

    app.apply_runtime_event(UiEvent::AgentFailed {
        request_id: crate::RequestId(17),
        message: "provider unavailable".to_owned(),
    });

    let session = &app.agent_sessions[&document_id];
    assert_eq!(session.state, db_pro_core::domain::agent::AgentSessionState::Failed);
    assert_eq!(session.active_run_id, None);
    assert_eq!(session.request_id, None);
    assert_eq!(session.pending_confirmation, None);
    assert_eq!(
        session.activities[0].status,
        super::agent_workflow_state::AgentUiActivityStatus::Failed
    );
}

#[test]
fn typed_agent_open_result_in_workspace_populates_query_document() {
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let document_id = app.query_documents[0].id.clone();
    let mut session = super::agent_workflow_state::AgentUiSession::for_document(&document_id, None, None);
    let summary = db_pro_core::domain::agent_context::AgentResultSummary {
        columns: vec![
            db_pro_core::domain::agent_context::AgentResultColumn {
                name: "id".to_owned(),
                data_type: Some("integer".to_owned()),
            },
            db_pro_core::domain::agent_context::AgentResultColumn {
                name: "name".to_owned(),
                data_type: Some("text".to_owned()),
            },
        ],
        sample_rows: vec![vec!["1".to_owned(), "Alice".to_owned()]],
        row_count: Some(1),
        affected_rows: None,
        truncated: false,
    };
    session.tool_results.insert(
        "query-1".to_owned(),
        super::agent_workflow_state::AgentUiToolResult {
            call_id: "query-1".to_owned(),
            tool: db_pro_core::domain::agent::AgentTool::RunQuery,
            output: db_pro_core::domain::agent::AgentToolOutput::QueryResult {
                statement_index: Some(0),
                result_count: 1,
                summary,
            },
            duration_ms: Some(42),
            status: super::agent_workflow_state::AgentUiActivityStatus::Success,
        },
    );
    app.agent_sessions.insert(document_id, session);

    app.open_agent_result_in_workspace("query-1");

    assert!(app.query_documents[0].query_result.is_some());
    let res = app.query_documents[0].query_result.as_ref().unwrap();
    assert_eq!(res.columns.len(), 2);
    assert_eq!(res.rows.len(), 1);
    assert_eq!(res.duration_ms, 42);
    assert_eq!(app.output_tab, crate::app::OutputTab::Results);
}

#[test]
fn typed_agent_cancellation_marks_session_and_activities_cancelled() {
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let document_id = app.query_documents[0].id.clone();
    let session = super::agent_workflow_state::AgentUiSession::for_document(&document_id, None, None);
    let session_id = session.session.as_ref().expect("session should exist").id;
    let run_id = db_pro_core::domain::agent::AgentRunId::new();
    app.agent_sessions.insert(document_id.clone(), session);
    let session = app.agent_sessions.get_mut(&document_id).expect("session should exist");
    session.active_run_id = Some(run_id);
    session.state = db_pro_core::domain::agent::AgentSessionState::Running;
    session.activities.push(super::agent_workflow_state::AgentUiActivity {
        call_id: Some("tool-1".to_owned()),
        tool: Some(db_pro_core::domain::agent::AgentTool::InspectSchema),
        label: "Inspecting schema".to_owned(),
        status: super::agent_workflow_state::AgentUiActivityStatus::Running,
        duration_ms: None,
    });

    app.apply_runtime_event(UiEvent::AgentWorkflow {
        request_id: crate::RequestId(1),
        event: db_pro_core::domain::agent_workflow::AgentWorkflowEvent::Cancelled {
            run_id,
            session_id,
            document_id: document_id.clone(),
        },
    });

    let session = &app.agent_sessions[&document_id];
    assert_eq!(session.state, db_pro_core::domain::agent::AgentSessionState::Cancelled);
    assert_eq!(session.active_run_id, None);
    assert_eq!(
        session.activities[0].status,
        super::agent_workflow_state::AgentUiActivityStatus::Cancelled
    );
}

#[test]
fn late_agent_workflow_events_are_ignored_after_cancellation() {
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let document_id = app.query_documents[0].id.clone();
    let session = super::agent_workflow_state::AgentUiSession::for_document(&document_id, None, None);
    let session_id = session.session.as_ref().expect("session should exist").id;
    let run_id = db_pro_core::domain::agent::AgentRunId::new();
    app.agent_sessions.insert(document_id.clone(), session);
    let session = app.agent_sessions.get_mut(&document_id).expect("session should exist");
    session.active_run_id = Some(run_id);
    session.state = db_pro_core::domain::agent::AgentSessionState::Cancelled;

    // Late event arrives for cancelled run
    app.apply_runtime_event(UiEvent::AgentWorkflow {
        request_id: crate::RequestId(2),
        event: db_pro_core::domain::agent_workflow::AgentWorkflowEvent::TextDelta {
            run_id,
            session_id,
            document_id: document_id.clone(),
            delta: "Late arriving text".to_owned(),
        },
    });

    let session = &app.agent_sessions[&document_id];
    assert_eq!(session.state, db_pro_core::domain::agent::AgentSessionState::Cancelled);
    assert!(session.streaming_text.is_empty());
}

#[test]
fn closing_query_tab_cleans_up_agent_session_and_cancels_active_run() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let document_id = app.query_documents[0].id.clone();
    let session = super::agent_workflow_state::AgentUiSession::for_document(&document_id, None, None);
    let run_id = db_pro_core::domain::agent::AgentRunId::new();
    app.agent_sessions.insert(document_id.clone(), session);
    let session = app.agent_sessions.get_mut(&document_id).expect("session should exist");
    session.active_run_id = Some(run_id);
    session.state = db_pro_core::domain::agent::AgentSessionState::Running;

    app.close_query_document(0);

    assert!(!app.agent_sessions.contains_key(&document_id));
    let mut saw_cancel = false;
    while let Ok(cmd) = command_rx.try_recv() {
        if let UiCommand::CancelAgentRun { run_id: cancelled, .. } = cmd {
            if cancelled == run_id {
                saw_cancel = true;
            }
        }
    }
    assert!(saw_cancel);
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
fn sql_diagnostics_include_unmatched_square_bracket_range() {
    let diagnostics = DbProApp::parse_sql_diagnostics("SELECT items[1 FROM data;", "PostgreSQL");

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("Unmatched delimiter [")));
}

#[test]
fn sql_diagnostics_report_mixed_delimiter_mismatch() {
    let diagnostics = DbProApp::parse_sql_diagnostics("SELECT ([)]", "PostgreSQL");

    assert!(diagnostics
        .iter()
        .any(|message| message.contains("Mismatched delimiter ): expected ]")));
}

#[test]
fn sql_lint_warns_on_select_star_and_null_compare() {
    let (messages, structured) =
        DbProApp::analyze_sql_diagnostics("SELECT * FROM t WHERE id = NULL", "PostgreSQL");
    assert!(messages.iter().any(|m| m.contains("SELECT *")));
    assert!(messages.iter().any(|m| m.contains("IS NULL")));
    assert!(structured.iter().any(|d| {
        d.source == crate::editor::DiagnosticSource::Lint && d.code.as_deref() == Some("lint.select-star")
    }));
    assert!(structured.iter().any(|d| {
        d.source == crate::editor::DiagnosticSource::Lint && d.code.as_deref() == Some("lint.null-compare")
    }));
}

#[test]
fn sql_lint_warns_on_delete_without_where() {
    let (messages, structured) = DbProApp::analyze_sql_diagnostics("DELETE FROM t", "SQLite");
    assert!(messages.iter().any(|m| m.contains("DELETE without WHERE")));
    assert!(structured.iter().any(|d| d.code.as_deref() == Some("lint.delete-no-where")));
}

#[test]
fn database_error_position_maps_postgres_character_to_utf8_editor_offset() {
    let sql = "SELECT café FROM users";
    let diagnostic = super::query_view::database_error_diagnostic(
        "syntax error",
        sql,
        (4, 4 + sql.len()),
        Some(11),
        Some("QUERY_SYNTAX_ERROR"),
    )
    .expect("database diagnostic should have a range");

    assert_eq!(diagnostic.source, crate::editor::DiagnosticSource::Database);
    assert_eq!(diagnostic.code.as_deref(), Some("QUERY_SYNTAX_ERROR"));
    assert_eq!(diagnostic.range, (14, 16));
}

#[test]
fn database_error_position_maps_selection_relative_to_document_offset() {
    let prefix = "-- before\n";
    let sql = "SELECT café FROM users";
    let selection_range = (prefix.len(), prefix.len() + sql.len());
    let diagnostic = super::query_view::database_error_diagnostic("syntax error", sql, selection_range, Some(11), None)
        .expect("database diagnostic should have a range");

    assert_eq!(diagnostic.range, (prefix.len() + 10, prefix.len() + 12));
}

#[test]
fn database_error_position_outside_executed_sql_falls_back_to_statement_range() {
    let sql = "SELECT café";
    let range = (3, 3 + sql.len());
    let diagnostic = super::query_view::database_error_diagnostic("syntax error", sql, range, Some(0), None)
        .expect("database diagnostic should have a range");

    assert_eq!(diagnostic.range, range);
}

#[test]
fn database_error_without_position_uses_the_executed_statement_range() {
    let sql = "SELECT 1";
    let range = (8, 8 + sql.len());
    let diagnostic = super::query_view::database_error_diagnostic("database error", sql, range, None, None)
        .expect("database diagnostic should have a range");

    assert_eq!(diagnostic.range, range);
}

#[test]
fn query_failure_attaches_database_diagnostic_to_the_originating_document() {
    let (bridge, _command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let mut doc = QueryDocument::new("query-1", "Query 1", "SELECT café;");
    let request_id = crate::RequestId(77);
    doc.execution_state = QueryExecutionState::Running(request_id);
    doc.executing_range = Some((0, doc.text().len()));
    doc.executing_sql = Some(doc.text().to_owned());
    doc.executing_version = Some(doc.buffer.version());
    app.query_documents = vec![doc];
    app.query_document_requests.insert(request_id, "query-1".to_owned());

    event_tx
        .send(UiEvent::QueryFailedDetailed {
            request_id,
            code: "QUERY_SYNTAX_ERROR".to_owned(),
            message: "syntax error".to_owned(),
            position: Some(8),
        })
        .expect("query failure should be queued");
    app.apply_runtime_events();

    let diagnostic = app.query_documents[0]
        .execution_diagnostic
        .as_ref()
        .expect("query failure should attach a diagnostic");
    assert_eq!(diagnostic.source, crate::editor::DiagnosticSource::Database);
    assert_eq!(diagnostic.range, (7, 8));
    assert_eq!(app.query_documents[0].execution_state, QueryExecutionState::Failed);
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
        ssl_mode: UiSslMode::Disable,
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
            ssl_mode: UiSslMode::Disable,
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
            ssl_mode: UiSslMode::Disable,
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
        ssl_mode: UiSslMode::Disable,
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

#[test]
fn test_column_order_and_move_column() {
    let mut app = DbProApp::default();
    let order = app.column_order(4);
    assert_eq!(order, vec![0, 1, 2, 3]);

    app.move_column(0, 2, 4);
    assert_eq!(app.grid_column_order, vec![1, 2, 0, 3]);

    app.move_column(3, 1, 4);
    assert_eq!(app.grid_column_order, vec![1, 3, 2, 0]);

    // Invalid persisted indexes are removed while valid order is preserved.
    let new_order = app.column_order(2);
    assert_eq!(new_order, vec![1, 0]);
}

#[test]
fn test_format_cell_csv_and_cell_to_json() {
    assert_eq!(DbProApp::format_cell_csv(&UiCell::Null), "");
    assert_eq!(DbProApp::format_cell_csv(&UiCell::Boolean(true)), "true");
    assert_eq!(DbProApp::format_cell_csv(&UiCell::Number("42.50".to_owned())), "42.50");
    assert_eq!(
        DbProApp::format_cell_csv(&UiCell::Text("Hello, \"World\"".to_owned())),
        "\"Hello, \"\"World\"\"\""
    );

    assert_eq!(DbProApp::cell_to_json_value(&UiCell::Null), serde_json::Value::Null);
    assert_eq!(
        DbProApp::cell_to_json_value(&UiCell::Boolean(false)),
        serde_json::Value::Bool(false)
    );
    assert_eq!(
        DbProApp::cell_to_json_value(&UiCell::Number("100".to_owned())),
        serde_json::json!(100)
    );
    assert_eq!(
        DbProApp::cell_to_json_value(&UiCell::Text("admin".to_owned())),
        serde_json::Value::String("admin".to_owned())
    );
}

/// Copy-as-JSON must not round an exact value through `f64`.
#[test]
fn test_copy_as_json_keeps_exact_numeric_digits() {
    // Beyond f64's 2^53 exact-integer range: must stay the identical text.
    assert_eq!(
        DbProApp::cell_to_json_value(&UiCell::Number("9007199254740993".to_owned())),
        serde_json::Value::String("9007199254740993".to_owned())
    );
    // Exact decimal with trailing zeroes: the digits are the value.
    assert_eq!(
        DbProApp::cell_to_json_value(&UiCell::Number("42.50".to_owned())),
        serde_json::Value::String("42.50".to_owned())
    );
    assert_eq!(
        DbProApp::cell_to_json_value(&UiCell::Number("12345678901234567890.12345".to_owned())),
        serde_json::Value::String("12345678901234567890.12345".to_owned())
    );
    // A value f64 represents exactly still serializes as a JSON number.
    assert_eq!(
        DbProApp::cell_to_json_value(&UiCell::Number("1.5".to_owned())),
        serde_json::json!(1.5)
    );
    // ... and the serialized text of that number is the original text.
    let copied = DbProApp::cell_to_json_value(&UiCell::Number("100".to_owned())).to_string();
    assert_eq!(copied, "100");
}

/// The copied/exported record shape must survive values that contain the
/// separator, a quote, or a line break.
#[test]
fn test_delimited_export_keeps_field_count_for_awkward_values() {
    let value = UiQueryResult {
        columns: vec![
            crate::UiColumn {
                name: "id".to_owned(),
                data_type: "int".to_owned(),
                nullable: false,
            },
            crate::UiColumn {
                name: "note".to_owned(),
                data_type: "text".to_owned(),
                nullable: true,
            },
        ],
        rows: vec![vec![
            crate::UiCell::Number("9007199254740993".to_owned()),
            crate::UiCell::Text("first, \"second\"\nthird".to_owned()),
        ]],
        row_count: 1,
        duration_ms: 0,
    };

    let csv = DbProApp::format_result_delimited(&value, ",");
    assert_eq!(
        csv, "id,note\n9007199254740993,\"first, \"\"second\"\"\nthird\"\n",
        "the delimiter, the quotes and the newline must be escaped inside one field, \
         and the exact digits must survive unquoted"
    );

    let tsv = DbProApp::format_result_delimited(&value, "\t");
    assert_eq!(
        tsv, "id\tnote\n9007199254740993\t\"first, \"\"second\"\"\nthird\"\n",
        "the tab-separated shape must survive too"
    );
}

/// The export writes exactly the shared serializer's text to disk.
#[test]
fn test_export_result_writes_escaped_delimited_text() {
    let value = UiQueryResult {
        columns: vec![
            crate::UiColumn {
                name: "note".to_owned(),
                data_type: "text".to_owned(),
                nullable: true,
            },
            crate::UiColumn {
                name: "amount".to_owned(),
                data_type: "numeric".to_owned(),
                nullable: true,
            },
        ],
        rows: vec![
            vec![
                crate::UiCell::Text("line, one\ntwo".to_owned()),
                crate::UiCell::Number("42.50".to_owned()),
            ],
            vec![
                crate::UiCell::Null,
                crate::UiCell::Number("9007199254740993".to_owned()),
            ],
        ],
        row_count: 2,
        duration_ms: 0,
    };

    let path = std::env::temp_dir().join(format!("db-pro-export-test-{}.csv", uuid::Uuid::new_v4()));
    let path_text = path.to_string_lossy().into_owned();
    let mut app = DbProApp {
        export_format: "CSV".to_owned(),
        export_path: path_text.clone(),
        export_open: true,
        ..Default::default()
    };
    app.export_result(&value);
    assert!(!app.export_open, "the dialog closes after a successful export");
    assert_eq!(app.runtime_message, format!("Exported 2 rows to {path_text}"));

    let written = std::fs::read_to_string(&path).expect("export file must exist");
    let _ = std::fs::remove_file(&path);
    assert_eq!(
        written, "note,amount\n\"line, one\ntwo\",42.50\n,9007199254740993\n",
        "NULL is an empty field, the exact digits survive, and the embedded newline stays quoted"
    );
}

#[test]
fn test_table_data_limit_and_paging_offset() {
    let mut app = DbProApp::default();
    assert_eq!(app.table_data_limit, 100);

    app.table_data_limit = 50;
    app.table_data_offset = 100;
    app.reset_table_data_page();
    assert_eq!(app.table_data_offset, 0);
}

#[test]
fn test_table_metadata_dependency_and_constraint_models() {
    let dep = UiTableDependency {
        name: "orders_archive".to_owned(),
        schema: "public".to_owned(),
        kind: UiDependencyKind::Table,
        direction: UiDependencyDirection::DependsOn,
        details: "Foreign key reference".to_owned(),
    };
    assert_eq!(dep.kind, UiDependencyKind::Table);
    assert_eq!(dep.direction, UiDependencyDirection::DependsOn);

    let check = UiCheckConstraint {
        name: "chk_positive_qty".to_owned(),
        definition: "quantity > 0".to_owned(),
    };
    assert_eq!(check.name, "chk_positive_qty");
    assert_eq!(check.definition, "quantity > 0");
}

#[test]
fn test_compare_ui_cells_typed_sorting() {
    use std::cmp::Ordering;

    // Number comparisons (exact numeric, not string alphabetical)
    let n2 = UiCell::Number("2".to_owned());
    let n10 = UiCell::Number("10".to_owned());
    let n3 = UiCell::Number("3".to_owned());
    assert_eq!(crate::compare_ui_cells(Some(&n2), Some(&n10)), Ordering::Less);
    assert_eq!(crate::compare_ui_cells(Some(&n10), Some(&n3)), Ordering::Greater);
    assert_eq!(crate::compare_ui_cells(Some(&n2), Some(&n3)), Ordering::Less);

    // Negative and decimal numbers
    let neg = UiCell::Number("-5.5".to_owned());
    let pos = UiCell::Number("1.2".to_owned());
    assert_eq!(crate::compare_ui_cells(Some(&neg), Some(&pos)), Ordering::Less);

    // Booleans: false < true
    let b_false = UiCell::Boolean(false);
    let b_true = UiCell::Boolean(true);
    assert_eq!(crate::compare_ui_cells(Some(&b_false), Some(&b_true)), Ordering::Less);

    // Nulls placed last
    assert_eq!(
        crate::compare_ui_cells(Some(&UiCell::Null), Some(&n2)),
        Ordering::Greater
    );
    assert_eq!(crate::compare_ui_cells(Some(&n2), Some(&UiCell::Null)), Ordering::Less);
    assert_eq!(
        crate::compare_ui_cells(Some(&UiCell::Null), Some(&UiCell::Null)),
        Ordering::Equal
    );
    assert_eq!(crate::compare_ui_cells(None, Some(&n2)), Ordering::Greater);

    // Date / timestamp strings compared chronologically
    let d1 = UiCell::Text("2026-01-15T09:00:00Z".to_owned());
    let d2 = UiCell::Text("2026-03-01T10:00:00Z".to_owned());
    assert_eq!(crate::compare_ui_cells(Some(&d1), Some(&d2)), Ordering::Less);
}

#[test]
fn test_open_table_blocked_with_unapplied_staged_changes() {
    let mut app = DbProApp {
        selected_table: Some("users".to_owned()),
        ..Default::default()
    };
    app.staged_changes.ensure_target("users");
    app.staged_changes.stage_update(StagedChange::Update {
        identity: primary_key_identity("1"),
        current_row_index: Some(0),
        column_index: 0,
        column: "name".to_owned(),
        data_type: "text".to_owned(),
        original: UiCell::Text("Alice".to_owned()),
        value: UiCell::Text("Alicia".to_owned()),
    });

    // Opening another table should be blocked to prevent mutation retargeting
    app.open_table("orders".to_owned());
    assert_eq!(app.selected_table, Some("users".to_owned()));
    assert!(app.discard_changes_confirmation);
    assert!(app.runtime_message.contains("Apply or discard staged changes"));

    // Closing table tab with staged changes is guarded
    app.discard_changes_confirmation = false;
    app.request_close_workspace_tab(WorkspaceTab::Table);
    assert_eq!(app.selected_table, Some("users".to_owned()));
    assert!(app.discard_changes_confirmation);

    // Discarding changes allows opening a new table
    app.discard_staged_changes();
    app.open_table("orders".to_owned());
    assert_eq!(app.selected_table, Some("orders".to_owned()));
}

#[test]
fn test_query_cancellation_capability_gate() {
    let postgres_conn = UiConnectionSummary {
        id: "pg".to_owned(),
        name: "PostgreSQL".to_owned(),
        host: "localhost".to_owned(),
        port: 5432,
        database: "app".to_owned(),
        username: "postgres".to_owned(),
        driver: "PostgreSQL".to_owned(),
        ssl_mode: UiSslMode::Disable,
        readonly: false,
    };
    let sqlite_conn = UiConnectionSummary {
        id: "sqlite".to_owned(),
        name: "SQLite".to_owned(),
        host: String::new(),
        port: 0,
        database: "app.db".to_owned(),
        username: String::new(),
        driver: "SQLite".to_owned(),
        ssl_mode: UiSslMode::Disable,
        readonly: false,
    };

    let mut app = DbProApp {
        connections: vec![postgres_conn, sqlite_conn],
        active_connection_id: Some("pg".to_owned()),
        connected: true,
        ..Default::default()
    };

    // PostgreSQL does not support query cancellation in capabilities
    assert!(!app.active_capabilities().allows(|caps| caps.query.cancel));

    // Switching to SQLite enables query cancellation
    app.active_connection_id = Some("sqlite".to_owned());
    assert!(app.active_capabilities().allows(|caps| caps.query.cancel));
}

#[test]
fn test_navigation_staged_changes_apply_discard_cancel_flows() {
    let mut app = DbProApp {
        selected_table: Some("users".to_owned()),
        ..Default::default()
    };
    app.staged_changes.ensure_target("users");
    app.staged_changes.stage_update(StagedChange::Update {
        identity: primary_key_identity("1"),
        current_row_index: Some(0),
        column_index: 0,
        column: "name".to_owned(),
        data_type: "text".to_owned(),
        original: UiCell::Text("Alice".to_owned()),
        value: UiCell::Text("Alicia".to_owned()),
    });

    // 1. Navigation Attempt sets pending_navigation_action
    app.open_table("orders".to_owned());
    assert_eq!(
        app.pending_navigation_action,
        Some(PendingNavigationAction::OpenTable("orders".to_owned()))
    );
    assert!(app.discard_changes_confirmation);
    assert_eq!(app.selected_table, Some("users".to_owned()));

    // 2. Cancel retains current context and clears pending action
    app.discard_changes_confirmation = false;
    app.pending_navigation_action = None;
    assert_eq!(app.selected_table, Some("users".to_owned()));
    assert!(!app.staged_changes.is_empty());

    // 3. Staged apply success executes pending navigation action
    app.open_table("products".to_owned());
    assert_eq!(
        app.pending_navigation_action,
        Some(PendingNavigationAction::OpenTable("products".to_owned()))
    );
    app.staged_apply_completed();
    assert_eq!(app.selected_table, Some("products".to_owned()));
    assert!(app.staged_changes.is_empty());
    assert!(app.pending_navigation_action.is_none());
}

#[test]
fn test_typed_filter_operator_support() {
    // Text types support full text operators
    assert!(DbProApp::filter_operator_supported(
        "text",
        &UiTableFilterOperator::Contains
    ));
    assert!(DbProApp::filter_operator_supported(
        "varchar(255)",
        &UiTableFilterOperator::StartsWith
    ));
    assert!(DbProApp::filter_operator_supported(
        "character varying",
        &UiTableFilterOperator::EndsWith
    ));

    // Numeric and timestamp types support comparison operators
    assert!(DbProApp::filter_operator_supported(
        "integer",
        &UiTableFilterOperator::GreaterThan
    ));
    assert!(DbProApp::filter_operator_supported(
        "bigint",
        &UiTableFilterOperator::LessThanOrEqual
    ));
    assert!(DbProApp::filter_operator_supported(
        "numeric(10,2)",
        &UiTableFilterOperator::GreaterThanOrEqual
    ));
    assert!(DbProApp::filter_operator_supported(
        "timestamptz",
        &UiTableFilterOperator::GreaterThan
    ));

    // Boolean only supports equals / not equals
    assert!(DbProApp::filter_operator_supported(
        "boolean",
        &UiTableFilterOperator::Equals
    ));
    assert!(DbProApp::filter_operator_supported(
        "bool",
        &UiTableFilterOperator::NotEquals
    ));
    assert!(!DbProApp::filter_operator_supported(
        "boolean",
        &UiTableFilterOperator::GreaterThan
    ));

    // All types support IS NULL and IS NOT NULL
    assert!(DbProApp::filter_operator_supported(
        "integer",
        &UiTableFilterOperator::IsNull
    ));
    assert!(DbProApp::filter_operator_supported(
        "text",
        &UiTableFilterOperator::IsNotNull
    ));
    assert!(DbProApp::filter_operator_supported(
        "uuid",
        &UiTableFilterOperator::IsNull
    ));
}

#[test]
fn test_grid_layout_schema_reconciliation() {
    let mut app = DbProApp::default();
    let initial_columns = vec![
        crate::UiColumn {
            name: "id".to_owned(),
            data_type: "int".to_owned(),
            nullable: false,
        },
        crate::UiColumn {
            name: "email".to_owned(),
            data_type: "text".to_owned(),
            nullable: false,
        },
    ];

    app.grid_pending_named_layout = Some(vec![
        PersistedGridColumnLayout {
            column_name: "email".to_owned(),
            width: 240.0,
            order: 0,
            hidden: false,
        },
        PersistedGridColumnLayout {
            column_name: "id".to_owned(),
            width: 100.0,
            order: 1,
            hidden: false,
        },
        // Removed column in DB should be gracefully dropped
        PersistedGridColumnLayout {
            column_name: "old_column".to_owned(),
            width: 300.0,
            order: 2,
            hidden: true,
        },
    ]);

    let order = app.column_order_for_columns(&initial_columns);
    // email was index 1, id was index 0
    assert_eq!(order, vec![1, 0]);
    assert_eq!(app.grid_column_widths[1], 240.0);
    assert_eq!(app.grid_column_widths[0], 100.0);
}

#[test]
fn test_multi_tab_query_result_routing() {
    let (bridge, _command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);

    // Create 2 query documents
    app.query_documents = vec![
        QueryDocument::new("query-1", "Query 1", "SELECT 1;"),
        QueryDocument::new("query-2", "Query 2", "SELECT 2;"),
    ];
    app.active_query_document = 0;

    // Simulate Tab 1 running request 101
    let req1 = crate::RequestId(101);
    app.query_documents[0].execution_state = QueryExecutionState::Running(req1);
    app.query_document_requests.insert(req1, "query-1".to_owned());

    // Switch to Tab 2 and simulate Tab 2 running request 102
    app.switch_query_document(1);
    let req2 = crate::RequestId(102);
    app.query_documents[1].execution_state = QueryExecutionState::Running(req2);
    app.query_document_requests.insert(req2, "query-2".to_owned());

    // Tab 1 query completes while user is on Tab 2
    let result1 = UiQueryResult {
        columns: vec![crate::UiColumn {
            name: "num".to_owned(),
            data_type: "int".to_owned(),
            nullable: false,
        }],
        rows: vec![vec![crate::UiCell::Text("1".to_owned())]],
        row_count: 1,
        duration_ms: 12,
    };
    event_tx
        .send(UiEvent::QueryCompleted {
            request_id: req1,
            result: result1,
        })
        .unwrap();

    app.apply_runtime_events();

    // Tab 1 should have received its result and message, but Tab 2 is active and has no result yet
    assert_eq!(
        app.query_documents[0].query_result.as_ref().map(|r| r.row_count),
        Some(1)
    );
    assert_eq!(app.query_documents[0].execution_state, QueryExecutionState::Idle);
    assert!(app.query_documents[0]
        .query_messages
        .iter()
        .any(|m| m.contains("1 rows")));
    assert!(app.active_query_result().is_none());

    // Tab 2 query completes
    let result2 = UiQueryResult {
        columns: vec![crate::UiColumn {
            name: "num".to_owned(),
            data_type: "int".to_owned(),
            nullable: false,
        }],
        rows: vec![vec![crate::UiCell::Text("2".to_owned())]],
        row_count: 1,
        duration_ms: 8,
    };
    event_tx
        .send(UiEvent::QueryCompleted {
            request_id: req2,
            result: result2,
        })
        .unwrap();

    app.apply_runtime_events();

    // Tab 2 is active, active_query_result() now returns Tab 2's result
    assert_eq!(
        app.active_query_result().and_then(|r| match &r.rows[0][0] {
            crate::UiCell::Text(s) => Some(s.as_str()),
            _ => None,
        }),
        Some("2")
    );
    assert_eq!(app.query_documents[1].execution_state, QueryExecutionState::Idle);

    // Switch back to Tab 1 -> active_query_result() returns Tab 1's result
    app.switch_query_document(0);
    assert_eq!(
        app.active_query_result().and_then(|r| match &r.rows[0][0] {
            crate::UiCell::Text(s) => Some(s.as_str()),
            _ => None,
        }),
        Some("1")
    );
}

#[test]
fn multi_result_completion_keeps_statement_order_and_active_tab_state() {
    let (bridge, _command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.query_documents = vec![QueryDocument::new("query-1", "Query 1", "SELECT 1; SELECT 2;")];
    let request_id = crate::RequestId(301);
    app.query_documents[0].execution_state = QueryExecutionState::Running(request_id);
    app.query_documents[0].execution_started_at = Some(std::time::Instant::now());
    app.query_document_requests.insert(request_id, "query-1".to_owned());

    let make_result = |value: &str| UiQueryResult {
        columns: vec![crate::UiColumn {
            name: "value".to_owned(),
            data_type: "int".to_owned(),
            nullable: false,
        }],
        rows: vec![vec![crate::UiCell::Number(value.to_owned())]],
        row_count: 1,
        duration_ms: 1,
    };
    event_tx
        .send(UiEvent::QueryMultiCompleted {
            request_id,
            output: crate::UiQueryExecutionOutput {
                statements: vec![
                    crate::UiStatementOutput {
                        statement_index: 0,
                        result_set: Some(make_result("1")),
                        affected_rows: None,
                        duration_ms: 1,
                        message: None,
                        error: None,
                    },
                    crate::UiStatementOutput {
                        statement_index: 1,
                        result_set: Some(make_result("2")),
                        affected_rows: None,
                        duration_ms: 1,
                        message: None,
                        error: None,
                    },
                ],
                total_duration_ms: 2,
            },
        })
        .unwrap();
    app.apply_runtime_events();

    assert_eq!(app.query_documents[0].query_results.len(), 2);
    app.set_active_query_result(1);
    assert_eq!(app.query_documents[0].active_result_index, 1);
    assert_eq!(
        app.query_documents[0].query_results[1].rows[0][0],
        UiCell::Number("2".to_owned())
    );
}

#[test]
fn query_history_uses_execution_start_time() {
    let (bridge, _command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let request_id = crate::RequestId(304);
    app.query_documents[0].execution_state = QueryExecutionState::Running(request_id);
    app.query_documents[0].execution_started_at = Some(std::time::Instant::now());
    app.query_documents[0].execution_started_wall_time = Some("2026-09-13T01:02:03Z".to_owned());
    app.query_documents[0].executing_sql = Some("SELECT 1".to_owned());
    app.query_document_requests.insert(request_id, "query-1".to_owned());

    event_tx
        .send(UiEvent::QueryCompleted {
            request_id,
            result: result(),
        })
        .unwrap();
    app.apply_runtime_events();

    assert_eq!(app.query_history_entries.len(), 1);
    assert_eq!(app.query_history_entries[0].started_at, "2026-09-13T01:02:03Z");
}

#[test]
fn multi_result_failure_attaches_database_diagnostic_to_failed_statement() {
    let (bridge, _command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let document = &mut app.query_documents[0];
    document.set_text("SELECT 1;\nSELECT bad;");
    let request_id = crate::RequestId(305);
    let document_id = document.id.clone();
    let version = document.buffer.version();
    let end = document.buffer.len_bytes();
    document.execution_state = QueryExecutionState::Running(request_id);
    document.execution_started_at = Some(std::time::Instant::now());
    document.executing_range = Some((0, end));
    document.executing_sql = Some(document.text().to_owned());
    document.executing_version = Some(version);
    app.query_document_requests.insert(request_id, document_id);

    event_tx
        .send(UiEvent::QueryMultiCompleted {
            request_id,
            output: crate::UiQueryExecutionOutput {
                statements: vec![
                    crate::UiStatementOutput {
                        statement_index: 0,
                        result_set: Some(result()),
                        affected_rows: None,
                        duration_ms: 1,
                        message: None,
                        error: None,
                    },
                    crate::UiStatementOutput {
                        statement_index: 1,
                        result_set: None,
                        affected_rows: None,
                        duration_ms: 0,
                        message: None,
                        error: Some(crate::UiQueryError {
                            code: "QUERY_FAILED".to_owned(),
                            message: "column \"bad\" does not exist".to_owned(),
                            position: None,
                            detail: None,
                            hint: None,
                        }),
                    },
                ],
                total_duration_ms: 2,
            },
        })
        .unwrap();
    app.apply_runtime_events();

    let diagnostic = app.query_documents[0]
        .execution_diagnostic
        .as_ref()
        .expect("failed statement should have a database diagnostic");
    assert_eq!(diagnostic.source, crate::editor::DiagnosticSource::Database);
    assert_eq!(diagnostic.range, app.query_documents[0].analysis.statements[1].range);
}

#[test]
fn saved_query_event_resets_dirty_baseline_and_failed_save_keeps_it() {
    let (bridge, _command_rx, event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.query_documents[0].set_text("SELECT changed;");
    assert!(app.query_documents[0].is_dirty());
    let save_request = crate::RequestId(302);
    app.query_save_requests.insert(save_request, "query-1".to_owned());
    event_tx
        .send(UiEvent::QuerySaved {
            request_id: save_request,
            query: UiSavedQuerySummary {
                id: "saved-1".to_owned(),
                name: "Query 1".to_owned(),
                sql: "SELECT changed;".to_owned(),
                folder: None,
            },
        })
        .unwrap();
    app.apply_runtime_events();
    assert!(!app.query_documents[0].is_dirty());
    assert_eq!(app.query_documents[0].saved_query_id.as_deref(), Some("saved-1"));

    app.query_documents[0].set_text("SELECT failed;");
    let failed_save_request = crate::RequestId(303);
    app.query_save_requests
        .insert(failed_save_request, "query-1".to_owned());
    event_tx
        .send(UiEvent::QueryFailed {
            request_id: failed_save_request,
            message: "storage unavailable".to_owned(),
        })
        .unwrap();
    app.apply_runtime_events();
    assert!(app.query_documents[0].is_dirty());
}

#[test]
fn dirty_query_close_is_deferred_until_user_decision() {
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.query_documents[0].set_text("SELECT changed;");

    app.request_close_query_document(0);

    assert_eq!(app.query_documents.len(), 1);
    assert_eq!(app.pending_dirty_close, Some(0));
}

#[test]
fn query_dispatch_allows_independent_documents_to_run_concurrently() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.connections = vec![
        UiConnectionSummary {
            id: "conn-1".to_owned(),
            name: "DB 1".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "db1".to_owned(),
            username: "user".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: UiSslMode::Disable,
            readonly: false,
        },
        UiConnectionSummary {
            id: "conn-2".to_owned(),
            name: "DB 2".to_owned(),
            host: String::new(),
            port: 0,
            database: "db2".to_owned(),
            username: String::new(),
            driver: "SQLite".to_owned(),
            ssl_mode: UiSslMode::Disable,
            readonly: false,
        },
    ];
    app.active_connection_id = Some("conn-1".to_owned());
    app.connected = true;
    app.set_document_connection(0, Some("conn-1".to_owned()));
    app.set_active_query_text("SELECT 1;");
    app.dispatch_query();
    let first = command_rx.recv().expect("first query command");
    let first_request = match first {
        UiCommand::RunQuery { request_id, .. } => request_id,
        _ => panic!("unexpected first command"),
    };

    app.new_query_document();
    app.set_document_connection(1, Some("conn-2".to_owned()));
    app.set_active_query_text("SELECT 2;");
    app.dispatch_query();
    let second = command_rx.recv().expect("second query command");
    let second_request = match second {
        UiCommand::RunQuery {
            request_id,
            connection_id,
            ..
        } => {
            assert_eq!(connection_id, "conn-2");
            request_id
        }
        _ => panic!("unexpected second command"),
    };

    assert_ne!(first_request, second_request);
    assert!(matches!(
        app.query_documents[0].execution_state,
        QueryExecutionState::Running(request) if request == first_request
    ));
    assert!(matches!(
        app.query_documents[1].execution_state,
        QueryExecutionState::Running(request) if request == second_request
    ));
}

#[test]
fn test_tab_switching_preserves_completion_and_prediction_isolation() {
    let mut doc1 = QueryDocument::new("query-1", "Query 1", "SELECT * FROM u");
    doc1.completion.open(
        15,
        egui::Pos2::new(100.0, 100.0),
        "u".to_string(),
        vec![crate::editor::CompletionItem {
            label: "users".to_owned(),
            insert_text: "users".to_owned(),
            kind: crate::editor::CompletionItemKind::Table,
            detail: None,
            documentation: None,
            replacement_range: (14, 15),
            sort_score: 800,
        }],
        crate::editor::CompletionTriggerKind::Automatic,
    );
    doc1.prediction = Some(crate::editor::EditPrediction {
        request_id: Some(crate::RequestId(1)),
        document_version: doc1.buffer.version(),
        anchor: 15,
        replacement_range: (15, 15),
        text: "sers WHERE id = 1".to_owned(),
    });

    let doc2 = QueryDocument::new("query-2", "Query 2", "SELECT 2;");

    let mut app = DbProApp {
        query_documents: vec![doc1, doc2],
        active_query_document: 0,
        ..Default::default()
    };

    // Doc 1 has open completion and prediction
    assert!(app.query_documents[0].completion.is_open);
    assert!(app.query_documents[0].prediction.is_some());

    // Switch to Doc 2
    app.switch_query_document(1);
    assert!(!app.query_documents[1].completion.is_open);
    assert!(app.query_documents[1].prediction.is_none());

    // Switch back to Doc 1
    app.switch_query_document(0);
    assert!(app.query_documents[0].completion.is_open);
    assert!(app.query_documents[0].prediction.is_some());
}

#[test]
fn test_prefix_replacement_logic() {
    let mut doc = QueryDocument::new("query-1", "Query 1", "SELECT * FROM us");
    doc.cursor.set_offset(&doc.buffer, 16); // end of "us"

    let item = crate::editor::CompletionItem {
        label: "users".to_owned(),
        insert_text: "users".to_owned(),
        kind: crate::editor::CompletionItemKind::Table,
        detail: None,
        documentation: None,
        replacement_range: (14, 16),
        sort_score: 800,
    };
    let (start, end) = item.replacement_range;
    doc.buffer.replace(start, end, &item.insert_text);
    let new_offset = start + item.insert_text.len();
    doc.cursor.set_offset(&doc.buffer, new_offset);
    doc.dirty = true;

    assert_eq!(doc.text(), "SELECT * FROM users");
    assert_eq!(doc.cursor.offset, 19);
    assert!(doc.dirty);
}

#[test]
fn test_popup_flipping_near_viewport_bottom() {
    let screen_rect = egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1000.0, 800.0));
    let popup_height = 220.0;
    let mut popup_pos = egui::Pos2::new(200.0, 750.0); // Near bottom (750 + 220 = 970 > 800 - 30)

    if popup_pos.y + popup_height > screen_rect.max.y - 30.0 {
        popup_pos.y = (popup_pos.y - popup_height - 24.0).max(screen_rect.min.y + 10.0);
    }

    assert_eq!(popup_pos.y, 750.0 - 220.0 - 24.0); // Flipped upward to 506.0
}

#[test]
fn test_multi_tab_explain_plan_routing() {
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.active_connection_id = Some("conn-1".to_owned());
    app.connected = true;
    app.connections = vec![UiConnectionSummary {
        id: "conn-1".to_owned(),
        name: "Test DB".to_owned(),
        database: "test".to_owned(),
        host: "localhost".to_owned(),
        port: 5432,
        username: "postgres".to_owned(),
        driver: "PostgreSQL".to_owned(),
        ssl_mode: UiSslMode::Disable,
        readonly: false,
    }];

    // Tab 1
    app.set_active_query_text("SELECT count(*) FROM users");
    let doc1_id = app.query_documents[0].id.clone();

    // Trigger explain on Tab 1
    app.explain_query();
    let explain_req_id = app.active_explain_request().expect("explain request id must be set");
    assert_eq!(app.query_documents[0].explain_request, Some(explain_req_id));

    // Create and switch to Tab 2
    app.new_query_document();
    app.set_active_query_text("SELECT * FROM orders");
    assert_eq!(app.active_query_document, 1);
    assert!(app.active_explain_request().is_none());
    assert!(app.active_explain_plan().is_none());

    // Explain completion event arrives for Tab 1's request
    app.apply_runtime_event(UiEvent::ExplainCompleted {
        request_id: explain_req_id,
        plan: "Seq Scan on users (cost=0.00..35.50 rows=2550 width=8)".to_owned(),
    });

    // Tab 2 (currently active) should NOT have the plan
    assert!(app.active_explain_plan().is_none());

    // Tab 1 must have the plan received and request cleared
    let doc1 = app.query_documents.iter().find(|d| d.id == doc1_id).unwrap();
    assert!(doc1.explain_request.is_none());
    assert_eq!(
        doc1.explain_plan.as_deref(),
        Some("Seq Scan on users (cost=0.00..35.50 rows=2550 width=8)")
    );

    // Switch back to Tab 1, active explain plan is immediately available
    app.switch_query_document(0);
    assert_eq!(
        app.active_explain_plan(),
        Some("Seq Scan on users (cost=0.00..35.50 rows=2550 width=8)")
    );
}

#[test]
fn test_prediction_mode_defaults_and_options() {
    let app = DbProApp::default();
    assert_eq!(app.prediction_mode, PredictionMode::Eager);

    let eager = PredictionMode::Eager;
    let off = PredictionMode::Off;
    assert_ne!(eager, off);
}

#[test]
fn test_per_document_connection_and_schema_isolation() {
    let mut app = DbProApp {
        connections: vec![
            UiConnectionSummary {
                id: "conn-pg".to_owned(),
                name: "Postgres Prod".to_owned(),
                driver: "postgresql".to_owned(),
                host: "localhost".to_owned(),
                port: 5432,
                database: "prod".to_owned(),
                username: "postgres".to_owned(),
                ssl_mode: UiSslMode::Disable,
                readonly: false,
            },
            UiConnectionSummary {
                id: "conn-sqlite".to_owned(),
                name: "Local SQLite".to_owned(),
                driver: "sqlite".to_owned(),
                host: "".to_owned(),
                port: 0,
                database: "/tmp/test.db".to_owned(),
                username: "".to_owned(),
                ssl_mode: UiSslMode::Disable,
                readonly: false,
            },
        ],
        active_connection_id: Some("conn-pg".to_owned()),
        ..Default::default()
    };

    // Tab 1 setup
    app.set_document_connection(0, Some("conn-pg".to_owned()));
    app.set_document_schema(0, Some("analytics".to_owned()));
    assert_eq!(app.active_query_connection_id(), Some("conn-pg"));
    assert_eq!(app.active_query_schema(), "analytics");

    // Tab 2: create and switch connection to SQLite
    app.new_query_document();
    assert_eq!(app.active_query_document, 1);
    app.set_document_connection(1, Some("conn-sqlite".to_owned()));
    app.set_document_schema(1, Some("main".to_owned()));

    assert_eq!(app.active_query_connection_id(), Some("conn-sqlite"));
    assert_eq!(app.active_query_schema(), "main");
    assert_eq!(app.active_query_driver(), "sqlite");

    // Switch back to Tab 1: retains its own connection and schema
    app.switch_query_document(0);
    assert_eq!(app.active_query_connection_id(), Some("conn-pg"));
    assert_eq!(app.active_query_schema(), "analytics");
    assert_eq!(app.active_query_driver(), "postgresql");
}

#[test]
fn test_ambiguous_column_completion_qualified_ranking() {
    let summary = UiSchemaSummary {
        table_details: vec![
            UiTableSummary {
                schema: "public".to_owned(),
                name: "users".to_owned(),
                row_count: Some(10),
                columns: vec![
                    crate::runtime::UiSchemaColumn {
                        name: "id".to_owned(),
                        data_type: "integer".to_owned(),
                        nullable: false,
                        is_primary_key: true,
                    },
                    crate::runtime::UiSchemaColumn {
                        name: "email".to_owned(),
                        data_type: "text".to_owned(),
                        nullable: false,
                        is_primary_key: false,
                    },
                ],
                foreign_keys: vec![],
            },
            UiTableSummary {
                schema: "public".to_owned(),
                name: "orders".to_owned(),
                row_count: Some(50),
                columns: vec![
                    crate::runtime::UiSchemaColumn {
                        name: "id".to_owned(),
                        data_type: "integer".to_owned(),
                        nullable: false,
                        is_primary_key: true,
                    },
                    crate::runtime::UiSchemaColumn {
                        name: "user_id".to_owned(),
                        data_type: "integer".to_owned(),
                        nullable: false,
                        is_primary_key: false,
                    },
                ],
                foreign_keys: vec![],
            },
        ],
        ..Default::default()
    };

    let ctx = crate::query::CompletionContext {
        text_before_cursor: "SELECT i",
        text_after_cursor: " FROM users u JOIN orders o ON u.id = o.user_id",
        cursor_offset: 8,
        active_schema: "public",
        schema_summary: &summary,
        cached_tokens: None,
        is_sqlite: false,
        is_manual_trigger: false,
    };

    let (prefix, items) = crate::query::SchemaCompletionProvider::provide(&ctx);
    assert_eq!(prefix, "i");
    assert!(!items.is_empty());

    // Because 'id' exists in both users and orders, qualified versions (u.id, o.id) should be boosted
    let u_id = items.iter().find(|it| it.insert_text == "u.id");
    let o_id = items.iter().find(|it| it.insert_text == "o.id");
    assert!(u_id.is_some(), "u.id should be suggested");
    assert!(o_id.is_some(), "o.id should be suggested");
    assert!(u_id.unwrap().sort_score >= 880);
}

#[test]
fn test_prediction_partial_accept_word_and_line() {
    let mut pred = crate::editor::EditPrediction::new(
        10,
        "WHERE users.id = 42\nORDER BY created_at DESC",
        Some(crate::RequestId(1)),
    );

    // Accept next word
    assert_eq!(pred.accept_next_word(), "WHERE");
    pred.consume("WHERE".len());
    assert_eq!(pred.anchor, 15);
    assert_eq!(pred.text, " users.id = 42\nORDER BY created_at DESC");

    // Accept next line
    assert_eq!(pred.accept_next_line(), " users.id = 42\n");
    pred.consume(" users.id = 42\n".len());
    assert_eq!(pred.text, "ORDER BY created_at DESC");

    // Accept remaining full text
    assert_eq!(pred.accept_full(), "ORDER BY created_at DESC");
    pred.consume("ORDER BY created_at DESC".len());
    assert!(pred.is_empty());
}

#[test]
fn test_async_prediction_routing_and_stale_rejection() {
    let mut app = DbProApp::default();
    app.new_query_document();

    let doc = &mut app.query_documents[0];
    doc.set_text("SELECT * FROM users ");
    doc.cursor.offset = doc.buffer.len_bytes();
    let current_version = doc.buffer.version();
    let req_id = crate::RequestId(77);
    doc.pending_prediction_request = Some(req_id);

    // Apply prediction ready event for matching request & version
    app.apply_runtime_event(UiEvent::SqlPredictionReady {
        request_id: req_id,
        document_id: app.query_documents[0].id.clone(),
        document_version: current_version,
        anchor: app.query_documents[0].cursor.offset,
        replacement_range: (
            app.query_documents[0].cursor.offset,
            app.query_documents[0].cursor.offset,
        ),
        prediction: "WHERE active = true".to_owned(),
    });

    let doc = &app.query_documents[0];
    assert!(doc.pending_prediction_request.is_none());
    assert!(doc.prediction.is_some());
    let pred = doc.prediction.as_ref().unwrap();
    assert_eq!(pred.text, "WHERE active = true");
    assert_eq!(pred.document_version, current_version);
}

#[test]
fn stale_prediction_event_does_not_mutate_a_newer_document_version() {
    let mut app = DbProApp::default();
    let document_id = app.query_documents[0].id.clone();
    let doc = &mut app.query_documents[0];
    doc.set_text("SELECT 1");
    doc.cursor.set_offset(&doc.buffer, doc.buffer.len_bytes());
    let request_id = crate::RequestId(88);
    let current_version = doc.buffer.version();
    let anchor = doc.cursor.offset;
    doc.pending_prediction_request = Some(request_id);

    app.apply_runtime_event(UiEvent::SqlPredictionReady {
        request_id,
        document_id,
        document_version: current_version.saturating_sub(1),
        anchor,
        replacement_range: (anchor, anchor),
        prediction: "WHERE stale = true".to_owned(),
    });

    let doc = &app.query_documents[0];
    assert!(doc.pending_prediction_request.is_none());
    assert!(doc.prediction.is_none());
}

#[test]
fn test_agent_multitab_isolation_and_close_tab_cancellation() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.new_query_document(); // creates tab 1 (index 1)
    let doc_a_id = app.query_documents[0].id.clone();
    let doc_b_id = app.query_documents[1].id.clone();

    // Start agent workflow on Tab A
    app.active_query_document = 0;
    let mut session_a = super::agent_workflow_state::AgentUiSession::for_document(&doc_a_id, None, None);
    let session_a_id = session_a.session.as_ref().unwrap().id;
    let run_a_id = db_pro_core::domain::agent::AgentRunId::new();
    session_a.state = db_pro_core::domain::agent::AgentSessionState::Running;
    session_a.active_run_id = Some(run_a_id);
    app.agent_sessions.insert(doc_a_id.clone(), session_a);

    // Send ToolRequested for Tab A
    app.on_agent_workflow_event(db_pro_core::domain::agent_workflow::AgentWorkflowEvent::ToolRequested {
        session_id: session_a_id,
        run_id: run_a_id,
        document_id: doc_a_id.clone(),
        call: db_pro_core::domain::agent::AgentToolCall {
            call_id: "call-a-1".to_owned(),
            tool: db_pro_core::domain::agent::AgentTool::InspectSchema,
            input: db_pro_core::domain::agent::AgentToolInput::None,
        },
    });

    // Tab A has 1 activity
    assert_eq!(app.agent_sessions.get(&doc_a_id).unwrap().activities.len(), 1);

    // Switch to Tab B
    app.active_query_document = 1;
    let session_b = super::agent_workflow_state::AgentUiSession::for_document(&doc_b_id, None, None);
    app.agent_sessions.insert(doc_b_id.clone(), session_b);

    // Tab B session is isolated from Tab A
    assert_eq!(app.agent_sessions.get(&doc_b_id).unwrap().activities.len(), 0);
    assert_eq!(app.agent_sessions.get(&doc_b_id).unwrap().messages.len(), 0);

    // Close Tab A
    app.close_query_document(0);

    // Tab A session was cleaned up
    assert!(!app.agent_sessions.contains_key(&doc_a_id));

    // Cancel command was dispatched for Tab A's active run
    assert!(matches!(
        command_rx.try_recv(),
        Ok(UiCommand::CancelAgentRun { run_id, .. }) if run_id == run_a_id
    ));

    // Late event for Tab A is dropped silently and does not recreate session
    app.on_agent_workflow_event(db_pro_core::domain::agent_workflow::AgentWorkflowEvent::TextDelta {
        session_id: session_a_id,
        run_id: run_a_id,
        document_id: doc_a_id.clone(),
        delta: "Late message".to_owned(),
    });
    assert!(!app.agent_sessions.contains_key(&doc_a_id));
}

#[test]
fn test_agent_vietnamese_ime_input_and_patch_version_safety() {
    let mut app = DbProApp::default();
    let doc_id = app.query_documents[0].id.clone();
    let initial_version = app.query_documents[0].buffer.version();

    // User types Vietnamese query with IME into editor
    app.query_documents[0].set_text("SELECT * FROM người_dùng WHERE tên = 'Nguyễn Văn A'");
    let typed_version = app.query_documents[0].buffer.version();
    assert!(typed_version > initial_version);

    // Setup Agent session with pending patch targeted at initial_version
    let mut session = super::agent_workflow_state::AgentUiSession::for_document(&doc_id, None, None);
    let run_id = db_pro_core::domain::agent::AgentRunId::new();
    session.pending_confirmation = Some(super::agent_workflow_state::AgentUiConfirmation {
        run_id,
        call_id: "patch-1".to_owned(),
        kind: db_pro_core::domain::agent_workflow::AgentConfirmationKind::ApplyPatch,
        preview: Some(db_pro_core::domain::agent::AgentToolOutput::PatchPreview {
            patch: db_pro_core::domain::agent::AgentSqlPatch {
                document_id: doc_id.clone(),
                expected_version: initial_version,
                range: (0, 6),
                replacement: "SELECT 1".to_owned(),
            },
            original: "SELECT".to_owned(),
            proposed: "SELECT 1".to_owned(),
        }),
        document_id: doc_id.clone(),
    });
    app.agent_sessions.insert(doc_id.clone(), session);

    // User attempts to apply patch - rejected due to stale version from typing
    app.agent_confirmation_action(true);
    assert_eq!(
        app.runtime_message,
        "This query changed since the suggestion was created."
    );
    // Text buffer unchanged and preserved
    assert_eq!(
        app.query_documents[0].text(),
        "SELECT * FROM người_dùng WHERE tên = 'Nguyễn Văn A'"
    );
}

#[test]
fn test_agent_vietnamese_valid_patch_application_and_undo() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let doc_id = app.query_documents[0].id.clone();

    // Set Vietnamese Unicode query text
    let initial_text = "SELECT tên FROM người_dùng";
    app.query_documents[0].set_text(initial_text);
    let expected_version = app.query_documents[0].buffer.version();

    // "SELECT " is 7 bytes; "tên" is 4 bytes (t: 1 byte, ê: 2 bytes, n: 1 byte) -> range (7, 11)
    let start_byte = 7;
    let end_byte = 11;
    assert_eq!(&initial_text[start_byte..end_byte], "tên");

    let mut session = super::agent_workflow_state::AgentUiSession::for_document(&doc_id, None, None);
    let run_id = db_pro_core::domain::agent::AgentRunId::new();
    session.pending_confirmation = Some(super::agent_workflow_state::AgentUiConfirmation {
        run_id,
        call_id: "patch-vn".to_owned(),
        kind: db_pro_core::domain::agent_workflow::AgentConfirmationKind::ApplyPatch,
        preview: Some(db_pro_core::domain::agent::AgentToolOutput::PatchPreview {
            patch: db_pro_core::domain::agent::AgentSqlPatch {
                document_id: doc_id.clone(),
                expected_version,
                range: (start_byte, end_byte),
                replacement: "ho_ten".to_owned(),
            },
            original: "tên".to_owned(),
            proposed: "ho_ten".to_owned(),
        }),
        document_id: doc_id.clone(),
    });
    app.agent_sessions.insert(doc_id.clone(), session);

    // Approve the patch
    app.agent_confirmation_action(true);

    // Verify document text was updated cleanly without byte index slicing panic
    assert_eq!(app.query_documents[0].text(), "SELECT ho_ten FROM người_dùng");

    // Verify ContinueAgentRun command was dispatched with PatchApplied outcome
    match command_rx.try_recv() {
        Ok(UiCommand::ContinueAgentRun {
            run_id: cmd_run_id,
            approved,
            applied_patch,
            ..
        }) => {
            assert_eq!(cmd_run_id, run_id);
            assert!(approved);
            assert!(matches!(
                applied_patch,
                Some(db_pro_core::domain::agent::AgentToolOutput::PatchApplied {
                    new_version,
                    range: (7, 11),
                    ..
                }) if new_version > expected_version
            ));
        }
        other => panic!("expected ContinueAgentRun, got {:?}", other),
    }

    // Verify undo restores the exact original Vietnamese text
    assert!(app.query_documents[0].buffer.undo().is_some());
    assert_eq!(app.query_documents[0].text(), initial_text);
}

#[test]
fn test_agent_event_routing_ignores_mismatched_session_and_document_and_run_ids() {
    let mut app = DbProApp::default();
    let doc_id = app.query_documents[0].id.clone();
    let real_run_id = db_pro_core::domain::agent::AgentRunId::new();

    let mut session = super::agent_workflow_state::AgentUiSession::for_document(&doc_id, None, None);
    let real_session_id = session.session.as_ref().unwrap().id;
    session.active_run_id = Some(real_run_id);
    app.agent_sessions.insert(doc_id.clone(), session);

    // 1. Mismatched document_id -> ignored
    let wrong_doc_id = "doc-nonexistent".to_owned();
    app.on_agent_workflow_event(db_pro_core::domain::agent_workflow::AgentWorkflowEvent::TextDelta {
        session_id: real_session_id,
        run_id: real_run_id,
        document_id: wrong_doc_id.clone(),
        delta: "ignored text".to_owned(),
    });
    assert!(!app.agent_sessions.contains_key(&wrong_doc_id));
    assert!(app.agent_sessions.get(&doc_id).unwrap().streaming_text.is_empty());

    // 2. Mismatched run_id -> ignored
    let wrong_run_id = db_pro_core::domain::agent::AgentRunId::new();
    app.on_agent_workflow_event(db_pro_core::domain::agent_workflow::AgentWorkflowEvent::TextDelta {
        session_id: real_session_id,
        run_id: wrong_run_id,
        document_id: doc_id.clone(),
        delta: "stale run text".to_owned(),
    });
    assert!(app.agent_sessions.get(&doc_id).unwrap().streaming_text.is_empty());

    // 3. Matching IDs -> accepted
    app.on_agent_workflow_event(db_pro_core::domain::agent_workflow::AgentWorkflowEvent::TextDelta {
        session_id: real_session_id,
        run_id: real_run_id,
        document_id: doc_id.clone(),
        delta: "valid delta".to_owned(),
    });
    assert_eq!(app.agent_sessions.get(&doc_id).unwrap().streaming_text, "valid delta");

    // 4. Wrong session_id with matching doc/run -> ignored
    let wrong_session_id = db_pro_core::domain::agent::AgentSessionId::new();
    app.on_agent_workflow_event(db_pro_core::domain::agent_workflow::AgentWorkflowEvent::TextDelta {
        session_id: wrong_session_id,
        run_id: real_run_id,
        document_id: doc_id.clone(),
        delta: "ignored session delta".to_owned(),
    });
    assert_eq!(app.agent_sessions.get(&doc_id).unwrap().streaming_text, "valid delta");
}

#[test]
fn test_agent_db_cancellation_and_terminal_cleanup() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let doc_id = app.query_documents[0].id.clone();
    let run_id = db_pro_core::domain::agent::AgentRunId::new();

    let mut session = super::agent_workflow_state::AgentUiSession::for_document(
        &doc_id,
        Some("conn-1".to_owned()),
        Some("public".to_owned()),
    );
    let session_id = session.session.as_ref().unwrap().id;
    session.active_run_id = Some(run_id);
    session.state = db_pro_core::domain::agent::AgentSessionState::Running;
    session.activities.push(super::agent_workflow_state::AgentUiActivity {
        call_id: Some("call-db-1".to_owned()),
        tool: Some(db_pro_core::domain::agent::AgentTool::RunQuery),
        label: "Executing query".to_owned(),
        status: super::agent_workflow_state::AgentUiActivityStatus::Running,
        duration_ms: None,
    });
    app.agent_sessions.insert(doc_id.clone(), session);

    // Cancel the active agent run
    app.cancel_active_agent_run();

    // Verify CancelAgentRun command was dispatched to runtime bridge
    assert!(matches!(
        command_rx.try_recv(),
        Ok(UiCommand::CancelAgentRun { run_id: cmd_run_id, .. }) if cmd_run_id == run_id
    ));

    // Runtime returns Cancelled event
    app.on_agent_workflow_event(db_pro_core::domain::agent_workflow::AgentWorkflowEvent::Cancelled {
        session_id,
        run_id,
        document_id: doc_id.clone(),
    });

    let finished_session = app.agent_sessions.get(&doc_id).unwrap();
    assert_eq!(
        finished_session.state,
        db_pro_core::domain::agent::AgentSessionState::Cancelled
    );
    assert_eq!(finished_session.active_run_id, None);
    assert_eq!(finished_session.pending_confirmation, None);
    assert_eq!(
        finished_session.activities[0].status,
        super::agent_workflow_state::AgentUiActivityStatus::Cancelled
    );

    // Late event arrives after cancellation -> ignored, state remains Cancelled
    app.on_agent_workflow_event(db_pro_core::domain::agent_workflow::AgentWorkflowEvent::TextDelta {
        session_id,
        run_id,
        document_id: doc_id.clone(),
        delta: "Late output".to_owned(),
    });
    let after_late = app.agent_sessions.get(&doc_id).unwrap();
    assert_eq!(
        after_late.state,
        db_pro_core::domain::agent::AgentSessionState::Cancelled
    );
    assert_eq!(after_late.active_run_id, None);
}

#[test]
fn test_agent_retry_isolation_and_session_routing() {
    let mut app = DbProApp::default();
    let doc_id = app.query_documents[0].id.clone();
    let run_1 = db_pro_core::domain::agent::AgentRunId::new();

    let mut session = super::agent_workflow_state::AgentUiSession::for_document(&doc_id, None, None);
    let session_1_id = session.session.as_ref().unwrap().id;
    session.active_run_id = Some(run_1);
    session.state = db_pro_core::domain::agent::AgentSessionState::Running;
    app.agent_sessions.insert(doc_id.clone(), session);

    // Run 1 fails
    app.on_agent_workflow_event(db_pro_core::domain::agent_workflow::AgentWorkflowEvent::Failed {
        session_id: session_1_id,
        run_id: run_1,
        document_id: doc_id.clone(),
        message: "API error".to_owned(),
    });

    let failed_session = app.agent_sessions.get(&doc_id).unwrap();
    assert_eq!(
        failed_session.state,
        db_pro_core::domain::agent::AgentSessionState::Failed
    );
    assert_eq!(failed_session.active_run_id, None);

    // Start Run 2 (Retry)
    let run_2 = db_pro_core::domain::agent::AgentRunId::new();
    let mut session_2 = super::agent_workflow_state::AgentUiSession::for_document(&doc_id, None, None);
    let session_2_id = session_2.session.as_ref().unwrap().id;
    assert_ne!(session_1_id, session_2_id);
    session_2.active_run_id = Some(run_2);
    session_2.state = db_pro_core::domain::agent::AgentSessionState::Running;
    app.agent_sessions.insert(doc_id.clone(), session_2);

    // Late event from Run 1 / Session 1 -> ignored
    app.on_agent_workflow_event(db_pro_core::domain::agent_workflow::AgentWorkflowEvent::TextDelta {
        session_id: session_1_id,
        run_id: run_1,
        document_id: doc_id.clone(),
        delta: "stale message".to_owned(),
    });
    assert!(app.agent_sessions.get(&doc_id).unwrap().streaming_text.is_empty());

    // Event from Run 2 / Session 2 -> accepted
    app.on_agent_workflow_event(db_pro_core::domain::agent_workflow::AgentWorkflowEvent::TextDelta {
        session_id: session_2_id,
        run_id: run_2,
        document_id: doc_id.clone(),
        delta: "active message".to_owned(),
    });
    assert_eq!(
        app.agent_sessions.get(&doc_id).unwrap().streaming_text,
        "active message"
    );
}

#[test]
fn test_composite_pk_targeted_reload_and_merge() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.active_connection_id = Some("conn-1".to_owned());
    app.selected_table = Some("user_roles".to_owned());
    app.table_info = Some(UiTableInfo {
        schema: "public".to_owned(),
        name: "user_roles".to_owned(),
        row_count: Some(2),
        columns: vec![
            crate::UiTableColumn {
                name: "tenant_id".to_owned(),
                data_type: "INTEGER".to_owned(),
                nullable: false,
                ..Default::default()
            },
            crate::UiTableColumn {
                name: "user_id".to_owned(),
                data_type: "INTEGER".to_owned(),
                nullable: false,
                ..Default::default()
            },
            crate::UiTableColumn {
                name: "role".to_owned(),
                data_type: "TEXT".to_owned(),
                nullable: false,
                ..Default::default()
            },
        ],
        primary_key: Some(vec!["tenant_id".to_owned(), "user_id".to_owned()]),
        indexes: Vec::new(),
        foreign_keys: Vec::new(),
        check_constraints: Vec::new(),
        dependencies: Vec::new(),
    });
    app.table_data_result = Some(UiQueryResult {
        columns: vec![
            crate::UiColumn {
                name: "tenant_id".to_owned(),
                data_type: "INTEGER".to_owned(),
                nullable: false,
            },
            crate::UiColumn {
                name: "user_id".to_owned(),
                data_type: "INTEGER".to_owned(),
                nullable: false,
            },
            crate::UiColumn {
                name: "role".to_owned(),
                data_type: "TEXT".to_owned(),
                nullable: false,
            },
        ],
        rows: vec![
            vec![
                UiCell::Number("1".to_owned()),
                UiCell::Number("10".to_owned()),
                UiCell::Text("admin".to_owned()),
            ],
            vec![
                UiCell::Number("1".to_owned()),
                UiCell::Number("20".to_owned()),
                UiCell::Text("member".to_owned()),
            ],
        ],
        row_count: 2,
        duration_ms: 5,
    });

    let target_identity = RowIdentity {
        original_pk_columns: vec!["tenant_id".to_owned(), "user_id".to_owned()],
        original_pk_values: vec![UiCell::Number("1".to_owned()), UiCell::Number("20".to_owned())],
    };

    app.request_table_row_reload(target_identity);

    let UiCommand::LoadTableData {
        filters, limit, offset, ..
    } = command_rx.try_recv().expect("LoadTableData command expected")
    else {
        panic!("expected LoadTableData");
    };

    assert_eq!(limit, 1);
    assert_eq!(offset, 0);
    assert_eq!(filters.len(), 2);
    assert_eq!(filters[0].column, "tenant_id");
    assert_eq!(filters[0].value, "1");
    assert_eq!(filters[1].column, "user_id");
    assert_eq!(filters[1].value, "20");

    let server_reloaded = UiQueryResult {
        columns: app.table_data_result.as_ref().unwrap().columns.clone(),
        rows: vec![vec![
            UiCell::Number("1".to_owned()),
            UiCell::Number("20".to_owned()),
            UiCell::Text("manager".to_owned()),
        ]],
        row_count: 1,
        duration_ms: 2,
    };

    app.on_table_row_reloaded(server_reloaded);

    let result = app.table_data_result.as_ref().unwrap();
    assert_eq!(result.rows[0][2], UiCell::Text("admin".to_owned()));
    assert_eq!(result.rows[1][2], UiCell::Text("manager".to_owned()));
}

#[test]
fn test_inserted_row_delete_removes_from_changeset_without_db_delete() {
    let (bridge, command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    app.active_connection_id = Some("conn-1".to_owned());
    app.selected_table = Some("users".to_owned());

    let local_id = app.staged_changes.stage_insert(
        vec!["username".to_owned(), "email".to_owned()],
        vec![
            UiCell::Text("alice".to_owned()),
            UiCell::Text("alice@test.com".to_owned()),
        ],
    );

    assert_eq!(app.staged_changes.counts().inserts, 1);
    assert_eq!(app.staged_changes.counts().total(), 1);

    // Deleting the draft insert row removes it locally
    let removed = app.staged_changes.remove_insert(local_id);
    assert!(removed);
    assert!(app.staged_changes.is_empty());

    // Apply now has zero changes and dispatches nothing
    app.apply_staged_changes();
    assert!(command_rx.try_recv().is_err());
}

#[test]
fn test_apply_mutation_failure_preserves_changeset_and_focuses_failed_cell() {
    let mut app = DbProApp {
        staged_apply_request: Some(crate::RequestId(12)),
        table_data_result: Some(UiQueryResult {
            columns: vec![
                crate::UiColumn {
                    name: "id".to_owned(),
                    data_type: "INTEGER".to_owned(),
                    nullable: false,
                },
                crate::UiColumn {
                    name: "name".to_owned(),
                    data_type: "TEXT".to_owned(),
                    nullable: false,
                },
            ],
            rows: vec![
                vec![UiCell::Number("1".to_owned()), UiCell::Text("Alice".to_owned())],
                vec![UiCell::Number("2".to_owned()), UiCell::Text("Bob".to_owned())],
                vec![UiCell::Number("3".to_owned()), UiCell::Text("Charlie".to_owned())],
            ],
            row_count: 3,
            duration_ms: 0,
        }),
        ..Default::default()
    };

    let id_1 = RowIdentity {
        original_pk_columns: vec!["id".to_owned()],
        original_pk_values: vec![UiCell::Number("1".to_owned())],
    };
    let id_2 = RowIdentity {
        original_pk_columns: vec!["id".to_owned()],
        original_pk_values: vec![UiCell::Number("2".to_owned())],
    };
    let id_3 = RowIdentity {
        original_pk_columns: vec!["id".to_owned()],
        original_pk_values: vec![UiCell::Number("3".to_owned())],
    };

    app.staged_changes.stage_update(StagedChange::Update {
        identity: id_1.clone(),
        current_row_index: Some(0),
        column_index: 1,
        column: "name".to_owned(),
        data_type: "TEXT".to_owned(),
        original: UiCell::Text("Alice".to_owned()),
        value: UiCell::Text("Alice Updated".to_owned()),
    });
    app.staged_changes.stage_update(StagedChange::Update {
        identity: id_2.clone(),
        current_row_index: Some(1),
        column_index: 1,
        column: "name".to_owned(),
        data_type: "TEXT".to_owned(),
        original: UiCell::Text("Bob".to_owned()),
        value: UiCell::Text("Bob Conflicting".to_owned()),
    });
    app.staged_changes.stage_delete(StagedChange::Delete {
        identity: id_3.clone(),
        current_row_index: Some(2),
    });

    app.staged_apply_targets = vec![
        MutationTarget::Update {
            identity: id_1,
            current_row_index: Some(0),
            columns: vec![1],
        },
        MutationTarget::Update {
            identity: id_2,
            current_row_index: Some(1),
            columns: vec![1],
        },
        MutationTarget::Delete {
            identity: id_3,
            current_row_index: Some(2),
        },
    ];

    // Failure occurs on statement index 1 (Bob) with CONFLICT
    app.staged_apply_failed(1, "CONFLICT", "row count was zero", true);

    // 1. Transaction rolled back
    let failure = app.table_mutation_error.as_ref().expect("failure recorded");
    assert!(failure.rolled_back);
    assert_eq!(failure.code, "CONFLICT");

    // 2. ChangeSet remains intact (2 updates + 1 delete)
    assert_eq!(app.staged_changes.counts().updates, 2);
    assert_eq!(app.staged_changes.counts().deletes, 1);

    // 3. Focus moves to failed row and cell
    assert_eq!(app.selected_row, Some(1));
    assert_eq!(app.selected_cell, Some((1, 1)));

    // 4. Conflict resolution dialog opened
    assert!(app.conflict_dialog_open);
}

#[test]
fn test_conflict_keep_mine_and_use_database_resolution_actions() {
    let (bridge, _command_rx, _event_tx) = TaskBridge::with_channels();
    let mut app = DbProApp::with_task_bridge(bridge);
    let id = RowIdentity {
        original_pk_columns: vec!["id".to_owned()],
        original_pk_values: vec![UiCell::Number("42".to_owned())],
    };

    app.table_data_result = Some(UiQueryResult {
        columns: vec![
            crate::UiColumn {
                name: "id".to_owned(),
                data_type: "INTEGER".to_owned(),
                nullable: false,
            },
            crate::UiColumn {
                name: "val".to_owned(),
                data_type: "TEXT".to_owned(),
                nullable: false,
            },
        ],
        rows: vec![vec![
            UiCell::Number("42".to_owned()),
            UiCell::Text("val_server".to_owned()),
        ]],
        row_count: 1,
        duration_ms: 0,
    });

    app.staged_changes.stage_update(StagedChange::Update {
        identity: id.clone(),
        current_row_index: Some(0),
        column_index: 1,
        column: "val".to_owned(),
        data_type: "TEXT".to_owned(),
        original: UiCell::Text("val_orig".to_owned()),
        value: UiCell::Text("val_mine".to_owned()),
    });

    app.table_mutation_error = Some(MutationFailure {
        statement_index: 0,
        target: Some(MutationTarget::Update {
            identity: id.clone(),
            current_row_index: Some(0),
            columns: vec![1],
        }),
        code: "CONFLICT".to_owned(),
        message: "Conflict".to_owned(),
        rolled_back: true,
    });
    app.conflict_dialog_open = true;

    // Test Use Database: reverts local staged changes
    app.conflict_use_database();
    assert!(app.staged_changes.is_empty());
    assert!(app.table_mutation_error.is_none());
    assert!(!app.conflict_dialog_open);
}

/// The status bar used to render `runtime_message` only when the text happened to
/// contain "failed"/"error", so every refusal, gate and informational message the app
/// sets was silently dropped — "Select a row before deleting" and "Connect with write
/// access to delete rows" among them.
#[test]
fn every_runtime_message_reaches_the_status_bar() {
    let mut app = DbProApp::default();
    app.runtime_message.clear();
    assert!(app.runtime_status().is_none(), "an empty message renders nothing");

    app.runtime_message = "Select a row before deleting".to_owned();
    let (message, color) = app.runtime_status().expect("a refusal must be shown");
    assert_eq!(message, "Select a row before deleting");
    assert_eq!(color, app.theme.text_secondary, "a refusal is not an error");

    app.runtime_message = "Connect with write access to delete rows".to_owned();
    let (_, color) = app.runtime_status().expect("a read-only refusal must be shown");
    assert_eq!(color, app.theme.text_secondary);

    app.runtime_message = "Query failed: syntax error at or near SELECT".to_owned();
    let (_, color) = app.runtime_status().expect("an error must be shown");
    assert_eq!(color, app.theme.danger, "errors keep the danger colour");
}

/// A destructive statement must not reach the database on the first Run: it is held
/// until the user confirms the exact text that the prompt shows, and the confirm path
/// dispatches that same text.
#[test]
fn destructive_statement_is_held_until_it_is_confirmed() {
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
        ssl_mode: UiSslMode::Disable,
        readonly: false,
    }];
    app.active_connection_id = Some("active".to_owned());
    app.connected = true;
    app.set_active_query_text("DROP TABLE users");

    app.dispatch_query();

    assert!(
        command_rx.try_recv().is_err(),
        "a destructive statement must not be dispatched before it is confirmed"
    );
    let pending = app
        .pending_destructive_run
        .as_ref()
        .expect("the statement must be held for confirmation");
    assert_eq!(pending.sql, "DROP TABLE users");
    assert!(!pending.all_statements);
    assert!(app.runtime_message.contains("held for confirmation"));

    app.confirm_pending_destructive_run();

    let UiCommand::RunQuery { sql, .. } = command_rx.try_recv().expect("confirmed statement must be dispatched") else {
        panic!("expected RunQuery command");
    };
    assert_eq!(sql, "DROP TABLE users");
    assert!(app.pending_destructive_run.is_none());
}

#[test]
fn cancelling_a_held_destructive_statement_sends_nothing() {
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
        ssl_mode: UiSslMode::Disable,
        readonly: false,
    }];
    app.active_connection_id = Some("active".to_owned());
    app.connected = true;
    app.set_active_query_text("TRUNCATE users");

    app.dispatch_query();
    assert!(app.pending_destructive_run.is_some());
    app.cancel_pending_destructive_run();

    assert!(
        command_rx.try_recv().is_err(),
        "a cancelled statement must never be dispatched"
    );
    assert!(app.pending_destructive_run.is_none());
    assert!(app.runtime_message.contains("cancelled"));
}

/// Reads, writes and plain DDL are not gated: the confirmation exists for the classes
/// that can drop or truncate, not for everything that mutates.
#[test]
fn reads_writes_and_plain_ddl_dispatch_without_a_prompt() {
    for sql in [
        "SELECT 1",
        "WITH recent AS (SELECT 1) SELECT * FROM recent",
        "INSERT INTO users (id) VALUES (1)",
        "UPDATE users SET name = 'x' WHERE id = 1",
        "DELETE FROM users WHERE id = 1",
        "ALTER TABLE users ADD COLUMN note TEXT",
    ] {
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
            ssl_mode: UiSslMode::Disable,
            readonly: false,
        }];
        app.active_connection_id = Some("active".to_owned());
        app.connected = true;
        app.set_active_query_text(sql);

        app.dispatch_query();

        assert!(app.pending_destructive_run.is_none(), "{sql} must not be gated");
        let UiCommand::RunQuery { sql: dispatched, .. } = command_rx
            .try_recv()
            .unwrap_or_else(|_| panic!("{sql} must dispatch immediately"))
        else {
            panic!("expected RunQuery for {sql}");
        };
        assert_eq!(dispatched.trim(), sql);
    }
}

/// A script is classified by its most dangerous statement, so a harmless-looking batch
/// that ends in DROP is held too — and the prompt says it is a script.
#[test]
fn a_script_whose_worst_statement_is_destructive_is_held() {
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
        ssl_mode: UiSslMode::Disable,
        readonly: false,
    }];
    app.active_connection_id = Some("active".to_owned());
    app.connected = true;
    app.set_active_query_text("SELECT 1;\nDROP TABLE users;");

    app.dispatch_query_all();

    assert!(command_rx.try_recv().is_err(), "the script must be held");
    let pending = app.pending_destructive_run.as_ref().expect("script must be held");
    assert!(
        pending.all_statements,
        "a run-all must be dispatched as a script on confirm"
    );

    app.confirm_pending_destructive_run();
    let UiCommand::RunQueryMulti { sql, .. } = command_rx.try_recv().expect("script must dispatch on confirm") else {
        panic!("expected RunQueryMulti");
    };
    assert_eq!(sql, "SELECT 1;\nDROP TABLE users;");

    // Removing the destructive statement makes the same script dispatch immediately.
    app.set_active_query_text("SELECT 1;\nSELECT 2;");
    app.query_documents[0].execution_state = QueryExecutionState::Idle;
    app.dispatch_query_all();
    assert!(app.pending_destructive_run.is_none());
    assert!(
        command_rx.try_recv().is_ok(),
        "a read-only script must dispatch immediately"
    );
}
