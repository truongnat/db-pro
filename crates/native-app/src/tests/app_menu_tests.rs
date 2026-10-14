//! Tests for `../app_menu.rs` (moved out of the source file).
use super::*;

#[test]
fn select_all_pushes_command_a_key_event() {
    let ctx = egui::Context::default();
    push_edit_event(&ctx, AppMenuAction::SelectAll);
    let pushed = ctx.input(|input| {
        input.events.iter().any(|event| {
            matches!(
                event,
                egui::Event::Key {
                    key: egui::Key::A,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.command
            )
        })
    });
    assert!(pushed, "Select All must inject a command+A key event");
}

#[test]
fn copy_pushes_egui_copy_event() {
    let ctx = egui::Context::default();
    push_edit_event(&ctx, AppMenuAction::Copy);
    assert!(ctx.input(|input| input.events.iter().any(|event| matches!(event, egui::Event::Copy))));
}

#[test]
fn redo_pushes_shift_command_z() {
    let ctx = egui::Context::default();
    push_edit_event(&ctx, AppMenuAction::Redo);
    let pushed = ctx.input(|input| {
        input.events.iter().any(|event| {
            matches!(
                event,
                egui::Event::Key {
                    key: egui::Key::Z,
                    pressed: true,
                    modifiers,
                    ..
                } if modifiers.command && modifiers.shift
            )
        })
    });
    assert!(pushed, "Redo must inject shift+command+Z");
}
