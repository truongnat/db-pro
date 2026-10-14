//! Tests for `../ui.rs` (moved out of the source file).
use egui::{Button, CentralPanel, Context, Event, Key, Pos2, RawInput, Rect, Vec2};

use super::*;

const SCREEN_SIZE: Vec2 = Vec2::new(800.0, 600.0);

fn trigger_rect() -> Rect {
    Rect::from_min_size(Pos2::new(100.0, 120.0), Vec2::new(120.0, 32.0))
}

fn run_frame(ctx: &Context, time: f64, events: Vec<Event>, content_height: f32) -> bool {
    let mut content_was_rendered = false;
    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN_SIZE)),
            time: Some(time),
            events,
            ..Default::default()
        },
        |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                let (_, content_result) = HoverCard::new("escape_test", DbProTheme::light())
                    .open_delay(0.0)
                    .close_delay(0.0)
                    .show(
                        ui,
                        |ui| ui.put(trigger_rect(), Button::new("Trigger")),
                        |ui| ui.add_space(content_height),
                    );
                content_was_rendered = content_result.is_some();
            });
        },
    );
    content_was_rendered
}

#[test]
fn escape_closes_immediately_and_stays_closed_while_trigger_is_hovered() {
    let ctx = Context::default();
    DbProTheme::install_fonts(&ctx);
    let pointer = egui::Event::PointerMoved(trigger_rect().center());

    assert!(!run_frame(&ctx, -0.1, Vec::new(), 24.0));
    assert!(run_frame(&ctx, 0.0, vec![pointer], 24.0));
    assert!(!run_frame(
        &ctx,
        0.1,
        vec![
            egui::Event::PointerMoved(trigger_rect().center()),
            Event::Key {
                key: Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            },
        ],
        24.0,
    ));
    assert!(!run_frame(
        &ctx,
        0.2,
        vec![egui::Event::PointerMoved(trigger_rect().center())],
        24.0,
    ));
}

#[test]
fn area_keeps_the_calculated_position_after_its_size_is_known() {
    let ctx = Context::default();
    DbProTheme::install_fonts(&ctx);
    let trigger_rect = Rect::from_min_size(Pos2::new(100.0, 300.0), Vec2::new(120.0, 32.0));
    let mut area_id = None;

    let run_sized_frame = |ctx: &Context, time: f64, content_height: f32, area_id: &mut Option<egui::Id>| {
        let mut card_area_rect = None;
        let _ = crate::test_frame::frame(
            &ctx,
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN_SIZE)),
                time: Some(time),
                events: vec![Event::PointerMoved(trigger_rect.center())],
                ..Default::default()
            },
            |ctx| {
                CentralPanel::default().show(ctx, |ui| {
                    *area_id = Some(ui.id().with(("hover_card", "height_test")).with("area"));
                    let _ = HoverCard::new("height_test", DbProTheme::light()).open_delay(0.0).show(
                        ui,
                        |ui| ui.put(trigger_rect, Button::new("Trigger")),
                        |ui| {
                            ui.allocate_space(Vec2::new(ui.available_width(), content_height));
                        },
                    );
                });
            },
        );
        if let Some(id) = *area_id {
            card_area_rect = ctx.memory(|memory| memory.area_rect(id));
        }
        card_area_rect
    };

    let _ = run_sized_frame(&ctx, 0.0, 20.0, &mut area_id);
    let card_rect = run_sized_frame(&ctx, 0.1, 20.0, &mut area_id).expect("card area");
    let expected_y = trigger_rect.bottom() + super::super::config::TRIGGER_GAP;

    assert!(
        (card_rect.top() - expected_y).abs() <= 1.0,
        "card {card_rect:?} should stay at calculated y={expected_y} below trigger {trigger_rect:?}"
    );
}
