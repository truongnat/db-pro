use super::config::INPUT_MIN_WIDTH;
use super::layout::resolve_field_width;

#[test]
fn fills_available_width_when_unrequested() {
    assert_eq!(resolve_field_width(None, 320.0), 320.0);
}

#[test]
fn requested_width_never_exceeds_the_container() {
    assert_eq!(resolve_field_width(Some(420.0), 260.0), 260.0);
}

#[test]
fn narrow_containers_keep_a_usable_floor() {
    assert_eq!(resolve_field_width(None, 40.0), INPUT_MIN_WIDTH);
    assert_eq!(resolve_field_width(Some(200.0), 40.0), INPUT_MIN_WIDTH);
}

#[test]
fn requested_width_below_floor_is_raised() {
    assert_eq!(resolve_field_width(Some(60.0), 320.0), INPUT_MIN_WIDTH);
}

#[test]
fn input_field_receives_click_when_clicked() {
    // Regression guard for the frame-click-steal bug: a surrounding `Frame` must not register a
    // `Sense::click()` region on top of the inner `TextEdit`. When it does, egui reports the click
    // as consumed by the frame, so the `TextEdit` never receives it — the caret never lands where
    // you click and a double-click never selects text. We assert the `TextEdit`'s own `clicked()`
    // (not just focus, which the buggy block faked via `request_focus()`), so the test fails when
    // the frame swallows the pointer.
    use crate::components::Input;
    use crate::DbProTheme;
    use egui::{CentralPanel, Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};

    let ctx = Context::default();
    DbProTheme::install_fonts(&ctx);
    let mut value = String::from("seed");
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(420.0, 200.0));

    let mut edit_rect = None;
    let _ = ctx.run(
        RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        },
        |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                edit_rect = Some(Input::new(&mut value, "Host", DbProTheme::dark()).show(ui).rect);
            });
        },
    );
    let edit_rect = edit_rect.expect("the input field is laid out");

    let pos = edit_rect.center();
    let mut edit_clicked = false;
    for pressed in [true, false] {
        let _ = ctx.run(
            RawInput {
                screen_rect: Some(screen),
                events: vec![Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::default(),
                }],
                ..Default::default()
            },
            |ctx| {
                CentralPanel::default().show(ctx, |ui| {
                    // `clicked()` is true only on the release frame; assigning every iteration
                    // (not just on release) avoids a clippy unused-assignment on `edit_clicked`.
                    edit_clicked = Input::new(&mut value, "Host", DbProTheme::dark()).show(ui).clicked();
                });
            },
        );
    }

    assert!(
        edit_clicked,
        "the text field must receive the click (caret placement and double-click selection depend \
         on the inner widget seeing the pointer), not have it swallowed by the surrounding frame"
    );
}

#[test]
fn password_eye_toggles_when_clicked() {
    // Regression guard for the same frame-click-steal bug, but through `PasswordInput`. The eye
    // toggle button lives inside the field's `Frame`, immediately right of the text edit. If the
    // surrounding frame registers a `Sense::click()` region on top, it eats the click and the eye
    // button never fires — the password stays masked even when toggled. We sweep a few candidate
    // points over the eye region: under the fix exactly the eye point flips `show_password`; under
    // the bug the frame swallows every click in the frame and nothing toggles.
    use super::PasswordInput;
    use crate::DbProTheme;
    use egui::{CentralPanel, Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};

    let ctx = Context::default();
    DbProTheme::install_fonts(&ctx);
    let mut value = String::from("hunter2");
    let mut show_password = false;
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(420.0, 200.0));

    let mut edit_rect = None;
    let _ = ctx.run(
        RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        },
        |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                edit_rect = Some(
                    PasswordInput::new(&mut value, "Password", &mut show_password, DbProTheme::dark())
                        .show(ui)
                        .rect,
                );
            });
        },
    );
    let edit_rect = edit_rect.expect("the password field is laid out");

    // The eye button sits immediately right of the text edit, inside the same frame.
    let mut eye_toggled = false;
    for dx in [2.0, 5.0, 8.0, 11.0] {
        let pos = Pos2::new(edit_rect.right() + dx, edit_rect.center().y);
        let before = show_password;
        for pressed in [true, false] {
            let _ = ctx.run(
                RawInput {
                    screen_rect: Some(screen),
                    events: vec![Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::default(),
                    }],
                    ..Default::default()
                },
                |ctx| {
                    CentralPanel::default().show(ctx, |ui| {
                        let _ =
                            PasswordInput::new(&mut value, "Password", &mut show_password, DbProTheme::dark()).show(ui);
                    });
                },
            );
        }
        if show_password != before {
            eye_toggled = true;
            break;
        }
    }

    assert!(
        eye_toggled,
        "clicking the eye button must toggle `show_password`; if the surrounding frame swallows the \
         click, the eye never fires and the password stays masked"
    );
}
