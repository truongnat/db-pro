//! Tests for query context picker sizing, bounds, and actions.
use super::*;
use egui::{Context, Pos2, RawInput, Rect, Vec2};

#[test]
fn query_context_picker_has_compact_bounds_for_short_lists() {
    let ctx = Context::default();
    DbProTheme::install_fonts(&ctx);
    let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0));
    let anchor = Rect::from_min_size(Pos2::new(100.0, 50.0), Vec2::new(180.0, 24.0));

    let connections = vec![
        ("sqlite-1".to_string(), "Sample E-Commerce (SQLite)".to_string(), "Development".to_string()),
        ("pg-1".to_string(), "Xe Lạc Hồng (PostgreSQL)".to_string(), "Production".to_string()),
    ];
    let schemas = vec!["main".to_string()];

    let context = QueryContextPickerContext {
        theme: DbProTheme::dark(),
        current_connection_id: Some("sqlite-1"),
        current_schema: "main",
        available_schemas: &schemas,
        connections: &connections,
    };

    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(screen_rect),
            ..Default::default()
        },
        |ctx| {
            let _ = draw_picker(&context, ctx, anchor);
        },
    );

    let popup_rect = ctx
        .memory(|mem| mem.area_rect(egui::Id::new("query_context_picker")))
        .expect("popup area rect should be recorded in egui memory");

    // The popup should be comfortably compact:
    // - Width should be pinned/bounded around 280-360px, NOT stretched across the window (> 500px).
    // - Height for 2 connections + 1 schema should shrink to content (~180-260px), NOT inflate to 400px+.
    assert!(
        popup_rect.width() <= PICKER_MAX_WIDTH + SURFACE_INSET,
        "query context picker width was {}, expected <= {}",
        popup_rect.width(),
        PICKER_MAX_WIDTH + SURFACE_INSET
    );
    assert!(
        popup_rect.width() >= PICKER_MIN_WIDTH,
        "query context picker width was {}, expected >= {PICKER_MIN_WIDTH}",
        popup_rect.width()
    );
    assert!(
        popup_rect.height() <= 280.0,
        "query context picker height was {}, expected <= 280px for 3 items total",
        popup_rect.height()
    );
}

#[test]
fn query_context_picker_caps_to_a_narrow_viewport() {
    let ctx = Context::default();
    DbProTheme::install_fonts(&ctx);
    // Viewport narrower than the picker's preferred width: the popup must not exceed it.
    let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(240.0, 500.0));
    let anchor = Rect::from_min_size(Pos2::new(8.0, 8.0), Vec2::new(120.0, 24.0));

    let connections = vec![
        ("sqlite-1".to_string(), "Sample E-Commerce (SQLite)".to_string(), "Development".to_string()),
        ("pg-1".to_string(), "Xe Lạc Hồng (PostgreSQL)".to_string(), "Production".to_string()),
    ];
    let schemas = vec!["main".to_string()];

    let context = QueryContextPickerContext {
        theme: DbProTheme::dark(),
        current_connection_id: Some("sqlite-1"),
        current_schema: "main",
        available_schemas: &schemas,
        connections: &connections,
    };

    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(screen_rect),
            ..Default::default()
        },
        |ctx| {
            let _ = draw_picker(&context, ctx, anchor);
        },
    );

    let popup_rect = ctx
        .memory(|mem| mem.area_rect(egui::Id::new("query_context_picker")))
        .expect("popup area rect should be recorded in egui memory");

    assert!(
        popup_rect.width() <= screen_rect.width() - VIEWPORT_GUTTER + 0.5,
        "narrow-viewport popup width was {}, viewport width {}",
        popup_rect.width(),
        screen_rect.width()
    );
    assert!(popup_rect.width() > 0.0);
}

#[test]
fn query_context_picker_action_on_escape_closes() {
    let ctx = Context::default();
    DbProTheme::install_fonts(&ctx);
    let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0));
    let anchor = Rect::from_min_size(Pos2::new(100.0, 50.0), Vec2::new(180.0, 24.0));

    let connections = vec![("sqlite-1".to_string(), "Sample".to_string(), "".to_string())];
    let schemas = vec!["main".to_string()];

    let context = QueryContextPickerContext {
        theme: DbProTheme::dark(),
        current_connection_id: Some("sqlite-1"),
        current_schema: "main",
        available_schemas: &schemas,
        connections: &connections,
    };

    let mut action = None;
    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(screen_rect),
            events: vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: Some(egui::Key::Escape),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ctx| {
            action = draw_picker(&context, ctx, anchor);
        },
    );

    assert_eq!(action, Some(QueryContextPickerAction::Close));
}
