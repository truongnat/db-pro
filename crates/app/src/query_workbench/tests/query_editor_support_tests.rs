//! Tests for `../query_editor_support.rs` (moved out of the source file).
    use super::*;
    use crate::editor::{CompletionItemKind, CompletionTriggerKind, EditorSnapshot};

    fn column_item(insert_text: &str, range: (usize, usize)) -> CompletionItem {
        CompletionItem {
            label: insert_text.to_owned(),
            insert_text: insert_text.to_owned(),
            kind: CompletionItemKind::Column,
            detail: None,
            documentation: None,
            replacement_range: range,
            sort_score: 1,
        }
    }

    fn table_hover_help(column_count: usize) -> RichHoverHelp {
        let columns = (0..column_count)
            .map(|i| HoverColumn {
                name: format!("col_{i}"),
                data_type: "TEXT".to_owned(),
                nullable: true,
                is_primary_key: i == 0,
                is_foreign_key: false,
            })
            .collect();
        RichHoverHelp::Table {
            qualified_name: "main.customers".to_owned(),
            kind: "Table".to_owned(),
            row_count: None,
            columns,
            foreign_keys: Vec::new(),
        }
    }

    fn run_hover_pass(ctx: &egui::Context, help: &RichHoverHelp) -> egui::FullOutput {
        crate::test_frame::frame(&ctx, egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0))),
            ..Default::default()
        }, |ctx| {
            let anchor = egui::Rect::from_min_size(egui::pos2(100.0, 100.0), egui::vec2(60.0, 18.0));
            draw_rich_hover_popup(ctx, anchor, (0, 9), help, &DbProTheme::default());
        })
    }

    fn painted_texts(output: &egui::FullOutput) -> Vec<String> {
        output
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(text) => Some(text.galley.text().to_owned()),
                _ => None,
            })
            .collect()
    }

    /// Regression test for the hover popup "shrink lock": egui `Area` confines its child to
    /// the size measured on the previous frame. If the popup first renders short content for
    /// a token (e.g. a `Symbol` card while introspection is still streaming) and later grows
    /// to a tall `Table` card for the same token range, every row must still be painted.
    #[test]
    fn rich_hover_popup_grows_and_paints_every_column_row() {
        let ctx = egui::Context::default();
        let short_help = RichHoverHelp::Symbol(SqlSymbolHelp {
            title: "customers".to_owned(),
            kind: "Symbol".to_owned(),
            detail: "unresolved".to_owned(),
            documentation: "Resolving schema…".to_owned(),
        });
        let _ = run_hover_pass(&ctx, &short_help);
        let texts = painted_texts(&run_hover_pass(&ctx, &table_hover_help(5)));
        for i in 0..5 {
            assert!(texts.iter().any(|t| t.contains(&format!("col_{i}"))), "missing col_{i}");
        }
    }

    #[test]
    fn completion_undo_restores_replaced_prefix() {
        let mut doc = QueryDocument::new("q", "Query", "SELECT ");
        let prefix_start = doc.buffer.len_bytes();
        doc.buffer.type_text(
            prefix_start,
            "p",
            EditorSnapshot {
                cursor_offset: prefix_start,
                anchor_offset: prefix_start,
            },
            EditorSnapshot {
                cursor_offset: prefix_start + 1,
                anchor_offset: prefix_start + 1,
            },
        );
        doc.buffer.type_text(
            prefix_start + 1,
            "u",
            EditorSnapshot {
                cursor_offset: prefix_start + 1,
                anchor_offset: prefix_start + 1,
            },
            EditorSnapshot {
                cursor_offset: prefix_start + 2,
                anchor_offset: prefix_start + 2,
            },
        );
        let end = doc.buffer.len_bytes();
        let version = doc.buffer.version();
        let item = column_item("purchase_order_id", (prefix_start, end));
        doc.completion.open(
            end,
            version,
            egui::Pos2::ZERO,
            "pu".to_owned(),
            vec![item.clone()],
            CompletionTriggerKind::Automatic,
        );

        assert!(apply_completion_item(&mut doc, &item, SqlDialect::Postgres));
        assert_eq!(doc.buffer.text(), "SELECT purchase_order_id");
        assert!(doc.buffer.undo().is_some());
        assert_eq!(doc.buffer.text(), "SELECT pu");
    }

    #[test]
    fn completion_undo_dot_reopens_completion() {
        let mut doc = QueryDocument::new("q", "Query", "SELECT ");
        let end = doc.buffer.len_bytes();
        let version = doc.buffer.version();
        let item = column_item("public.", (end, end));
        doc.completion.open(
            end,
            version,
            egui::Pos2::ZERO,
            String::new(),
            vec![item.clone()],
            CompletionTriggerKind::Automatic,
        );

        assert!(apply_completion_item(&mut doc, &item, SqlDialect::Postgres));
        assert_eq!(doc.buffer.text(), "SELECT public.");
        assert_eq!(doc.cursor.offset, doc.buffer.len_bytes());
        assert!(doc.completion_due_at.is_some());
    }

    #[test]
    fn completion_apply_syncs_selection_anchor_to_new_cursor() {
        // #488: collapse_to_active alone left `active` at the pre-edit offset —
        // anchor and active must both land on the post-insert cursor.
        let mut doc = QueryDocument::new("q", "Query", "SELECT pu");
        let version = doc.buffer.version();
        // Simulate a stale selection left over from the prefix range.
        doc.selection = crate::editor::SelectionRange::new(7, 9);
        let item = column_item("purchase_order_id", (7, 9));
        doc.completion.open(
            9,
            version,
            egui::Pos2::ZERO,
            "pu".to_owned(),
            vec![item.clone()],
            CompletionTriggerKind::Automatic,
        );

        assert!(apply_completion_item(&mut doc, &item, SqlDialect::Postgres));
        assert_eq!(doc.buffer.text(), "SELECT purchase_order_id");
        let cursor = 7 + "purchase_order_id".len();
        assert_eq!(doc.selection.anchor, cursor);
        assert_eq!(doc.selection.active, cursor);
        assert!(doc.selection.is_empty());
    }