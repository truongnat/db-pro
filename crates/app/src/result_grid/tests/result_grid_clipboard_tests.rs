//! Tests for `../result_grid_clipboard.rs`.

use super::*;

fn insert_test_result() -> UiQueryResult {
    UiQueryResult {
        columns: vec![crate::UiColumn {
            name: "id".to_owned(),
            data_type: "int".to_owned(),
            nullable: false,
        }],
        rows: vec![vec![UiCell::Number("1".to_owned())]],
        row_count: 1,
        duration_ms: 0,
    }
}

fn copied_text(app: &mut DbProApp, mut run: impl FnMut(&mut DbProApp, &mut egui::Ui)) -> String {
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let output = crate::test_frame::frame(
        &ctx,
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1024.0, 640.0))),
            ..Default::default()
        },
        |ui| {
            egui::CentralPanel::default().show(ui, |ui| run(app, ui));
        },
    );
    output
        .platform_output
        .commands
        .iter()
        .find_map(|command| match command {
            egui::OutputCommand::CopyText(text) => Some(text.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn copy_as_insert_uses_placeholder_for_query_results() {
    // A query result has no discoverable source table: copying INSERTs must never target the
    // table that happens to be selected in the schema explorer.
    let mut app = DbProApp::default();
    app.schema.explorer.selected_table = Some("orders".to_owned());
    app.workspace.active_tab = WorkspaceTab::Query;
    let result = insert_test_result();

    let text = copied_text(&mut app, |app, ui| app.copy_all_as_insert(ui, &result, &[0]));

    assert!(text.contains("INSERT INTO \"table_name\""), "{text}");
    assert!(!text.contains("orders"), "{text}");
}

#[test]
fn copy_as_insert_uses_selected_table_in_table_data_view() {
    let mut app = DbProApp::default();
    app.schema.explorer.selected_table = Some("orders".to_owned());
    app.schema.explorer.selected_schema = Some("public".to_owned());
    app.workspace.active_tab = WorkspaceTab::Table;
    app.table.state.table_view = TableView::Data;
    let result = insert_test_result();

    let text = copied_text(&mut app, |app, ui| app.copy_all_as_insert(ui, &result, &[0]));

    assert!(text.contains("INSERT INTO \"public\".\"orders\""), "{text}");
}

// #487: an empty string is a real value — copying it must not report "nothing to copy" —
// and a NULL cell copies the Postgres `\copy` sentinel so it cannot be mistaken for `''`.
#[test]
fn copy_cell_copies_empty_text_and_null_sentinel() {
    let mut app = DbProApp::default();
    let result = UiQueryResult {
        columns: vec![
            crate::UiColumn { name: "a".to_owned(), data_type: "text".to_owned(), nullable: true },
            crate::UiColumn { name: "b".to_owned(), data_type: "text".to_owned(), nullable: true },
        ],
        rows: vec![vec![UiCell::Text(String::new()), UiCell::Null]],
        row_count: 1,
        duration_ms: 0,
    };

    assert_eq!(app.copy_cell_value(&result, 0, 0), Some(UiCell::Text(String::new())));
    let text = copied_text(&mut app, |app, ui| app.copy_cell_at(ui, &result, 0, 0));
    assert_eq!(app.feedback.copy_status, "Cell copied");
    // egui emits an empty CopyText; whether the OS clipboard stores it is platform-defined.
    assert_eq!(text, "");

    let text = copied_text(&mut app, |app, ui| app.copy_cell_at(ui, &result, 0, 1));
    assert_eq!(text, "\\N");
    assert_eq!(app.feedback.copy_status, "Cell copied");
}

// #487: row copy uses the same `\N` sentinel so a pasted row round-trips NULL vs ''.
#[test]
fn copied_row_text_distinguishes_null_from_empty_text() {
    let app = DbProApp::default();
    let result = UiQueryResult {
        columns: vec![
            crate::UiColumn { name: "a".to_owned(), data_type: "text".to_owned(), nullable: true },
            crate::UiColumn { name: "b".to_owned(), data_type: "text".to_owned(), nullable: true },
        ],
        rows: vec![vec![UiCell::Null, UiCell::Text(String::new())]],
        row_count: 1,
        duration_ms: 0,
    };
    assert_eq!(app.copied_row_text(&result, 0, &result.rows[0]), "\\N\t");
}
