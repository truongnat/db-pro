//! Tests for `../modal_guard.rs` (moved out of the source file).
use super::*;

#[test]
fn single_modal_is_always_topmost() {
    let ctx = egui::Context::default();
    let solo = Id::new("solo");
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        assert!(register(ctx, solo));
    });
    let mut topmost = false;
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        topmost = register(ctx, solo);
    });
    assert!(topmost, "a lone dialog must stay topmost across passes");
}

#[test]
fn last_registered_modal_is_topmost() {
    let ctx = egui::Context::default();
    let lower = Id::new("lower");
    let upper = Id::new("upper");

    // Warm-up pass establishes the reference stack [lower, upper].
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        register(ctx, lower);
        register(ctx, upper);
    });

    let mut lower_topmost = true;
    let mut upper_topmost = false;
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        lower_topmost = register(ctx, lower);
        upper_topmost = register(ctx, upper);
    });

    assert!(!lower_topmost, "the lower dialog must not own Esc/backdrop");
    assert!(upper_topmost, "the last-rendered dialog must be topmost");
}

/// After the last dialog closes, `any_open` must stop reporting a modal — the
/// old untimestamped `prev` stack latched true and blocked global shortcuts.
#[test]
fn closed_modals_stop_reporting_open() {
    let ctx = egui::Context::default();
    let dialog = Id::new("closing");

    // Pass 1: the dialog paints; prepare_frame of pass 2 still sees it.
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        register(ctx, dialog);
    });
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        assert!(any_open(ctx), "a modal painted last pass still counts");
    });

    // Passes 3+: nothing registers; the stale stack must expire.
    for _ in 0..2 {
        let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
            assert!(!any_open(ctx), "no modal painted this pass or last pass");
        });
    }
}

/// A dialog that opens long after the previous one closed is topmost on its
/// first frame — the stale `prev` stack used to outrank it.
#[test]
fn reopened_modal_is_topmost_immediately() {
    let ctx = egui::Context::default();
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        register(ctx, Id::new("old"));
    });
    // Idle passes let the reference stack expire.
    for _ in 0..3 {
        let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {});
    }
    let mut topmost = false;
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        topmost = register(ctx, Id::new("new"));
    });
    assert!(topmost, "a lone dialog after idle passes must be topmost");
}

#[test]
fn stacked_modal_becomes_topmost_on_next_pass() {
    let ctx = egui::Context::default();
    let existing = Id::new("existing");
    let fresh = Id::new("fresh");

    // Pass 1: only `existing` is open.
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        register(ctx, existing);
    });

    // Pass 2: `fresh` opens on top; the previous stack is still [existing], so
    // `existing` owns this pass and exactly one dialog is topmost.
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        assert!(register(ctx, existing), "existing owns the pass it stacked on");
        assert!(!register(ctx, fresh), "a freshly stacked dialog waits one pass");
    });

    // Pass 3: previous stack is now [existing, fresh]; the newer dialog takes over.
    let mut fresh_topmost = false;
    let mut existing_topmost = true;
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        existing_topmost = register(ctx, existing);
        fresh_topmost = register(ctx, fresh);
    });

    assert!(
        fresh_topmost,
        "the newer dialog becomes topmost once it is last rendered"
    );
    assert!(!existing_topmost, "only one dialog is topmost at a time");
}

#[test]
fn focus_outside_overlay_is_redirected_to_its_anchor() {
    let ctx = egui::Context::default();
    let outside_widget = Id::new("outside_widget");
    let anchor = Id::new("overlay_focus_anchor");
    let overlay_layer = LayerId::new(egui::Order::Foreground, Id::new("overlay_layer"));

    ctx.memory_mut(|memory| memory.request_focus(outside_widget));
    trap_focus(&ctx, overlay_layer, anchor);

    assert_eq!(ctx.memory(|memory| memory.focused()), Some(anchor));
}

#[test]
fn focus_inside_overlay_layer_is_preserved() {
    let ctx = egui::Context::default();
    let overlay_id = Id::new("overlay_with_focus");
    let overlay_layer = LayerId::new(egui::Order::Foreground, overlay_id);
    let anchor = overlay_id.with("focus_anchor");
    let mut focused_widget = None;

    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::Area::new(overlay_id)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let response = ui.button("Inside overlay");
                response.request_focus();
                focused_widget = Some(response.id);
            });
    });

    let focused_widget = focused_widget.expect("overlay content should register its focusable widget");
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(focused_widget));
    trap_focus(&ctx, overlay_layer, anchor);
    assert_eq!(ctx.memory(|memory| memory.focused()), Some(focused_widget));
}

#[test]
fn tab_advances_focus_between_fields_inside_modal() {
    use egui::{Event, Key, Modifiers, Pos2, RawInput, Rect, TextEdit, Vec2};

    let ctx = egui::Context::default();
    let overlay_id = Id::new("tab_modal");
    let overlay_layer = LayerId::new(egui::Order::Foreground, overlay_id);
    let anchor = overlay_id.with("focus_anchor");
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(640.0, 240.0));
    let mut first = String::new();
    let mut second = String::new();
    let mut first_response = None;
    let mut second_response = None;
    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        },
        |ctx| {
            trap_focus(ctx, overlay_layer, anchor);
            egui::Area::new(overlay_id)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    first_response = Some(ui.add(TextEdit::singleline(&mut first).id_salt("modal.first")));
                    first_response
                        .as_ref()
                        .expect("first input should be rendered")
                        .request_focus();
                    second_response = Some(ui.add(TextEdit::singleline(&mut second).id_salt("modal.second")));
                });
        },
    );

    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(screen),
            events: vec![Event::Key {
                key: Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::default(),
            }],
            ..Default::default()
        },
        |ctx| {
            trap_focus(ctx, overlay_layer, anchor);
            egui::Area::new(overlay_id)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    first_response = Some(ui.add(TextEdit::singleline(&mut first).id_salt("modal.first")));
                    second_response = Some(ui.add(TextEdit::singleline(&mut second).id_salt("modal.second")));
                });
        },
    );

    assert!(
        second_response
            .as_ref()
            .expect("second dialog input should be rendered")
            .has_focus(),
        "Tab should advance focus within the modal instead of returning focus to the anchor"
    );

    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(screen),
            events: vec![Event::Key {
                key: Key::Tab,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: Modifiers::default(),
            }],
            ..Default::default()
        },
        |ctx| {
            trap_focus(ctx, overlay_layer, anchor);
            egui::Area::new(overlay_id)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    first_response = Some(ui.add(TextEdit::singleline(&mut first).id_salt("modal.first")));
                    second_response = Some(ui.add(TextEdit::singleline(&mut second).id_salt("modal.second")));
                });
        },
    );

    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(screen),
            events: vec![Event::Key {
                key: Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::SHIFT,
            }],
            ..Default::default()
        },
        |ctx| {
            trap_focus(ctx, overlay_layer, anchor);
            egui::Area::new(overlay_id)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    first_response = Some(ui.add(TextEdit::singleline(&mut first).id_salt("modal.first")));
                    second_response = Some(ui.add(TextEdit::singleline(&mut second).id_salt("modal.second")));
                });
        },
    );

    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        },
        |ctx| {
            trap_focus(ctx, overlay_layer, anchor);
            egui::Area::new(overlay_id)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    first_response = Some(ui.add(TextEdit::singleline(&mut first).id_salt("modal.first")));
                    second_response = Some(ui.add(TextEdit::singleline(&mut second).id_salt("modal.second")));
                });
        },
    );

    let first_response = first_response.as_ref().expect("first dialog input should be rendered");
    assert!(
        first_response.has_focus(),
        "Shift+Tab should return focus to the previous field inside the modal; focused={:?}, expected={:?}",
        ctx.memory(|memory| memory.focused()),
        first_response.id
    );
}
