//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn keyboard_submit_needs_the_enter_signal_and_a_nonblank_prompt() {
    assert_eq!(keyboard_action(true, "SELECT 1"), Some(AgentComposerAction::Submit));
    assert_eq!(keyboard_action(false, "SELECT 1"), None);
    assert_eq!(keyboard_action(true, "  \n\t"), None);
    assert_eq!(keyboard_action(true, ""), None);
}

#[test]
fn blank_prompts_are_not_submittable() {
    assert!(!has_prompt_text(""));
    assert!(!has_prompt_text("  \n\t"));
    assert!(has_prompt_text("SELECT 1"));
}

#[test]
fn button_actions_distinguish_generating_and_idle_states() {
    assert_eq!(button_action(true, "", true), Some(AgentComposerAction::Stop));
    assert_eq!(
        button_action(false, "SELECT 1", true),
        Some(AgentComposerAction::Submit)
    );
    assert_eq!(button_action(false, "  ", true), None);
    assert_eq!(button_action(false, "SELECT 1", false), None);
}

#[test]
fn badge_geometry_preserves_local_padding_and_offsets() {
    let rect = badge_rect(Pos2::new(10.0, 20.0), 42.0);
    assert_eq!(rect.min, Pos2::new(10.0, 22.0));
    assert_eq!(rect.size(), Vec2::new(54.0, 20.0));
    assert_eq!(badge_text_pos(rect), Pos2::new(16.0, 24.0));
}
