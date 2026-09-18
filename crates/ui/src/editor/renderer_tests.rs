use super::*;

#[test]
fn test_caret_geometry_calculation() {
    let text = "SELECT 1;\nSELECT 2;\nSELECT 3;";
    let buf = TextBuffer::from_string(text);
    let mut buf_clone = buf.clone();
    let mut cursor = CursorPosition::default();
    let mut selection = SelectionRange::default();
    let theme = DbProTheme::dark();
    let editor = SqlEditor::new(
        &mut buf_clone,
        &mut cursor,
        &mut selection,
        SqlDialect::Postgres,
        &theme,
        &[],
        None,
        "test",
    );

    let origin = Pos2::new(100.0, 50.0);
    let gutter_w = 40.0;
    let line_height = 20.0;
    let char_width = 8.0;

    // Line 1 ("SELECT 2;"), col 3 ("E") -> offset = 10 + 3 = 13
    let screen_pos = editor.offset_to_screen_pos(&buf, 13, origin, gutter_w, line_height, char_width);
    assert_eq!(screen_pos.x, 100.0 + 40.0 + PADDING_LEFT + 3.0 * 8.0);
    assert_eq!(screen_pos.y, 50.0 + PADDING_TOP + 1.0 * 20.0);

    let resolved_offset = editor.screen_pos_to_offset(&buf, screen_pos, origin, gutter_w, line_height, char_width);
    assert_eq!(resolved_offset, 13);
}

#[test]
fn test_ime_text_input_multilingual() {
    let mut buf = TextBuffer::from_string("SELECT ");
    let mut cursor = CursorPosition::from_offset(&buf, 7);
    let mut selection = SelectionRange::default();
    let theme = DbProTheme::dark();
    let mut editor = SqlEditor::new(
        &mut buf,
        &mut cursor,
        &mut selection,
        SqlDialect::Postgres,
        &theme,
        &[],
        None,
        "test",
    );

    // Simulate IME committing Vietnamese text
    editor.type_text("'tiếng Việt có dấu'");
    assert_eq!(editor.buffer.text(), "SELECT 'tiếng Việt có dấu'");
    assert_eq!(editor.cursor.offset, "SELECT 'tiếng Việt có dấu'".len());

    // Test undo
    assert!(editor.buffer.undo().is_some());
    assert_eq!(editor.buffer.text(), "SELECT ");
}

#[test]
fn editor_keeps_focus_after_click_across_frames() {
    let ctx = egui::Context::default();
    let mut buf = TextBuffer::from_string("select 1");
    let mut cursor = CursorPosition::default();
    let mut selection = SelectionRange::default();
    let theme = DbProTheme::light();
    let editor_size = egui::vec2(400.0, 200.0);

    // Frame 1: show the editor and request focus on its interactive id.
    let _ = ctx.run(
        egui::RawInput {
            focused: true,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let editor_id = ui.make_persistent_id("focus-survive");
                let editor = SqlEditor::new(
                    &mut buf,
                    &mut cursor,
                    &mut selection,
                    SqlDialect::Postgres,
                    &theme,
                    &[],
                    None,
                    "focus-survive",
                );
                let _ = editor.show(ui, editor_size);
                // Mimic a click: focus the same id `show` registers as interactive.
                ui.memory_mut(|m| m.request_focus(editor_id));
            });
        },
    );

    // Frame 2: editor must still report focused — the old bug cleared focus here
    // because focus was requested on an id that never registered as a widget.
    let mut focused = false;
    let _ = ctx.run(
        egui::RawInput {
            focused: true,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let editor = SqlEditor::new(
                    &mut buf,
                    &mut cursor,
                    &mut selection,
                    SqlDialect::Postgres,
                    &theme,
                    &[],
                    None,
                    "focus-survive",
                );
                focused = editor.show(ui, editor_size).focused;
            });
        },
    );
    assert!(
        focused,
        "SQL editor must keep keyboard focus after click; otherwise typing is impossible"
    );
}

#[test]
fn test_ime_commit_event_in_editor_widget() {
    let ctx = egui::Context::default();
    let mut buf = TextBuffer::from_string("SELECT ");
    let mut cursor = CursorPosition::from_offset(&buf, 7);
    let mut selection = SelectionRange::default();
    let theme = DbProTheme::dark();

    let raw_input = egui::RawInput {
        focused: true,
        events: vec![egui::Event::Ime(egui::ImeEvent::Commit("tên_cột".to_owned()))],
        ..Default::default()
    };

    let _ = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Focus is acquired via the editor's interactive widget id inside `show`.
            let editor = SqlEditor::new(
                &mut buf,
                &mut cursor,
                &mut selection,
                SqlDialect::Postgres,
                &theme,
                &[],
                None,
                "test-ime",
            );
            // Pre-focus by interacting once so IME events are consumed.
            let editor_id = ui.make_persistent_id("test-ime");
            ui.memory_mut(|m| m.request_focus(editor_id));
            let resp = editor.show(ui, egui::vec2(600.0, 400.0));
            assert!(resp.changed, "IME commit should insert text when editor is focused");
            assert!(resp.wants_completion);
        });
    });

    assert_eq!(buf.text(), "SELECT tên_cột");
    assert_eq!(cursor.offset, "SELECT tên_cột".len());
}

#[test]
fn long_buffer_scrolls_to_keep_end_caret_visible() {
    let ctx = egui::Context::default();
    let text = (0..80).map(|i| format!("SELECT {i};")).collect::<Vec<_>>().join("\n");
    let mut buf = TextBuffer::from_string(&text);
    let end = buf.len_bytes();
    let mut cursor = CursorPosition::from_offset(&buf, end);
    let mut selection = SelectionRange::new(end, end);
    let theme = DbProTheme::dark();
    let viewport = egui::vec2(420.0, 160.0);

    assert!(cursor.line > 40, "fixture caret should sit far below the fold");

    let _ = ctx.run(
        egui::RawInput {
            focused: true,
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let editor = SqlEditor::new(
                    &mut buf,
                    &mut cursor,
                    &mut selection,
                    SqlDialect::Postgres,
                    &theme,
                    &[],
                    None,
                    "scroll-long",
                );
                let _ = editor.show(ui, viewport);
                let editor_id = ui.make_persistent_id("scroll-long");
                let scroll = ui
                    .ctx()
                    .data(|d| d.get_temp::<egui::Vec2>(editor_id.with("scroll")))
                    .unwrap_or(egui::Vec2::ZERO);
                assert!(
                    scroll.y > 0.0,
                    "end caret must pull the viewport down, got scroll.y={}",
                    scroll.y
                );
            });
        },
    );
}
