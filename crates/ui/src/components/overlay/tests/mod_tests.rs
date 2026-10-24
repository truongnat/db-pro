//! Tests for `../mod.rs` (moved out of the source file).
use super::{
    handler::should_close_on_outside_click,
    ui::{context_action_menu, popup_dismiss_scrim},
};
use crate::DbProTheme;

#[test]
fn outside_click_requires_an_interaction_position() {
    assert!(!should_close_on_outside_click(true, true, false));
    assert!(should_close_on_outside_click(true, true, true));
    assert!(!should_close_on_outside_click(false, true, true));
}

fn frame_with_row(ctx: &egui::Context, input: egui::RawInput, theme: DbProTheme, menu_rendered: &mut bool) {
    let _ = crate::test_frame::frame(&ctx, input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (_rect, response) = ui.allocate_exact_size(egui::vec2(300.0, 26.0), egui::Sense::click());
            context_action_menu(ui, &response, theme, |ui, _close_menu| {
                *menu_rendered = true;
                let _ = ui.label("Menu item");
            });
        });
    });
}

fn button_event(pos: egui::Pos2, button: egui::PointerButton, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button,
        pressed,
        modifiers: egui::Modifiers::default(),
    }
}

#[test]
fn context_menu_opens_on_secondary_click() {
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let theme = DbProTheme::dark();
    let click_pos = egui::pos2(60.0, 13.0);
    let mut menu_rendered = false;

    frame_with_row(&ctx, Default::default(), theme, &mut menu_rendered);
    assert!(!menu_rendered);

    for events in [
        vec![
            egui::Event::PointerMoved(click_pos),
            button_event(click_pos, egui::PointerButton::Secondary, true),
        ],
        vec![button_event(click_pos, egui::PointerButton::Secondary, false)],
        vec![],
    ] {
        let input = egui::RawInput {
            events,
            ..Default::default()
        };
        frame_with_row(&ctx, input, theme, &mut menu_rendered);
    }

    assert!(menu_rendered, "menu body should render after right-click");
}
struct PopoverFixture {
    victim_rect: egui::Rect,
    inner_rect: egui::Rect,
    victim_clicks: u32,
    inner_clicks: u32,
}

/// A clickable victim row with a toggle-button `egui::Popup` anchored above it,
/// guarded by [`popup_dismiss_scrim`].
fn popover_fixture(ctx: &egui::Context, input: egui::RawInput, fixture: &mut PopoverFixture) {
    let popup_id = egui::Id::new("fixture_filter_popover");
    let _ = crate::test_frame::frame(&ctx, input, |ui| {
        let button = ui.add(egui::Button::new("Filter"));
        if button.clicked() {
            egui::Popup::toggle_id(ui.ctx(), popup_id);
        }
        ui.add_space(8.0);
        let (victim_rect, victim) = ui.allocate_exact_size(egui::vec2(300.0, 26.0), egui::Sense::click());
        fixture.victim_rect = victim_rect;
        if victim.clicked() {
            fixture.victim_clicks += 1;
        }
        popup_dismiss_scrim(ui.ctx(), popup_id, egui::Popup::is_id_open(ui.ctx(), popup_id));
        egui::Popup::new(popup_id, ui.ctx().clone(), &button, ui.layer_id())
            .kind(egui::PopupKind::Tooltip)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .open_memory(None)
            .show(|ui| {
                let (inner_rect, inner) = ui.allocate_exact_size(egui::vec2(120.0, 24.0), egui::Sense::click());
                fixture.inner_rect = inner_rect;
                if inner.clicked() {
                    fixture.inner_clicks += 1;
                }
            });
    });
}

#[test]
fn dismiss_scrim_consumes_the_outside_click() {
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let popup_id = egui::Id::new("fixture_filter_popover");
    let mut fx = PopoverFixture {
        victim_rect: egui::Rect::NOTHING,
        inner_rect: egui::Rect::NOTHING,
        victim_clicks: 0,
        inner_clicks: 0,
    };

    // Settle layout, then open the popover via the button.
    popover_fixture(&ctx, Default::default(), &mut fx);
    let button_center = egui::pos2(fx.victim_rect.left() + 20.0, 10.0);
    for events in [
        vec![
            egui::Event::PointerMoved(button_center),
            button_event(button_center, egui::PointerButton::Primary, true),
        ],
        vec![button_event(button_center, egui::PointerButton::Primary, false)],
        vec![],
    ] {
        popover_fixture(
            &ctx,
            egui::RawInput {
                events,
                ..Default::default()
            },
            &mut fx,
        );
    }
    assert!(
        egui::Popup::is_id_open(&ctx, popup_id),
        "popover should be open after toggling"
    );

    // Clicking the row under the pointer must dismiss the popover without
    // activating the row — the click is swallowed by the scrim.
    let outside = fx.victim_rect.center();
    for events in [
        vec![
            egui::Event::PointerMoved(outside),
            button_event(outside, egui::PointerButton::Primary, true),
        ],
        vec![button_event(outside, egui::PointerButton::Primary, false)],
        vec![],
    ] {
        popover_fixture(
            &ctx,
            egui::RawInput {
                events,
                ..Default::default()
            },
            &mut fx,
        );
    }
    assert_eq!(
        fx.victim_clicks, 0,
        "dismiss click must not reach the widget underneath"
    );
    assert!(
        !egui::Popup::is_id_open(&ctx, popup_id),
        "outside click should still close the popover"
    );

    // Re-open and click inside the popover body: it stays interactive and open.
    for events in [
        vec![
            egui::Event::PointerMoved(button_center),
            button_event(button_center, egui::PointerButton::Primary, true),
        ],
        vec![button_event(button_center, egui::PointerButton::Primary, false)],
        vec![],
    ] {
        popover_fixture(
            &ctx,
            egui::RawInput {
                events,
                ..Default::default()
            },
            &mut fx,
        );
    }
    assert!(egui::Popup::is_id_open(&ctx, popup_id));
    let inside = fx.inner_rect.center();
    for events in [
        vec![
            egui::Event::PointerMoved(inside),
            button_event(inside, egui::PointerButton::Primary, true),
        ],
        vec![button_event(inside, egui::PointerButton::Primary, false)],
        vec![],
    ] {
        popover_fixture(
            &ctx,
            egui::RawInput {
                events,
                ..Default::default()
            },
            &mut fx,
        );
    }
    assert_eq!(fx.inner_clicks, 1, "clicks inside the popover must keep working");
    assert!(
        egui::Popup::is_id_open(&ctx, popup_id),
        "inside clicks must not dismiss"
    );
}
