use super::{focus_visible, update_focus_visible};

fn ctx() -> egui::Context {
    egui::Context::default()
}

fn run_frame(ctx: &egui::Context, events: Vec<egui::Event>) {
    let input = egui::RawInput {
        events,
        ..Default::default()
    };
    let mut out = ctx.run_ui(input, |ui| update_focus_visible(ui.ctx()));
    out.textures_delta.clear();
}

fn key_down(key: egui::Key) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}

#[test]
fn focus_visible_starts_off_and_stays_off_on_idle_frames() {
    let ctx = ctx();
    assert!(!focus_visible(&ctx));
    run_frame(&ctx, vec![]);
    assert!(!focus_visible(&ctx));
}

#[test]
fn pointer_press_suppresses_ring_after_keyboard_nav() {
    let ctx = ctx();
    run_frame(&ctx, vec![key_down(egui::Key::Tab)]);
    assert!(focus_visible(&ctx));
    run_frame(
        &ctx,
        vec![egui::Event::PointerButton {
            pos: egui::pos2(10.0, 10.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert!(!focus_visible(&ctx));
}

#[test]
fn keyboard_nav_turns_ring_on_and_it_sticks_across_idle_frames() {
    let ctx = ctx();
    run_frame(&ctx, vec![key_down(egui::Key::Tab)]);
    assert!(focus_visible(&ctx));
    run_frame(&ctx, vec![]);
    assert!(focus_visible(&ctx), "ring stays until a pointer event");
}

#[test]
fn pointer_release_alone_also_clears_keyboard_modality() {
    let ctx = ctx();
    run_frame(&ctx, vec![key_down(egui::Key::Tab)]);
    run_frame(
        &ctx,
        vec![egui::Event::PointerButton {
            pos: egui::pos2(10.0, 10.0),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert!(!focus_visible(&ctx));
}

#[test]
fn arrow_and_enter_count_as_focus_keys() {
    for key in [
        egui::Key::ArrowUp,
        egui::Key::ArrowDown,
        egui::Key::ArrowLeft,
        egui::Key::ArrowRight,
        egui::Key::Enter,
        egui::Key::Space,
    ] {
        let ctx = ctx();
        run_frame(&ctx, vec![key_down(key)]);
        assert!(focus_visible(&ctx), "{key:?} should enable the ring");
    }
}
