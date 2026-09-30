use egui::{Pos2, Rect, Vec2};

use super::{config, AgentComposerAction};

/// Converts the text-edit signals into the composer's keyboard submit decision.
///
/// The multiline editor deliberately submits only after focus is lost and Enter is
/// pressed without Shift. Keeping this predicate independent of egui lets the UI
/// layer preserve multiline Shift+Enter input without embedding policy in paint code.
pub fn keyboard_action(
    lost_focus: bool,
    enter_pressed: bool,
    shift_held: bool,
    prompt: &str,
) -> Option<AgentComposerAction> {
    if !lost_focus || !enter_pressed || shift_held || !has_prompt_text(prompt) {
        return None;
    }
    Some(AgentComposerAction::Submit)
}

/// Reports whether a prompt contains content that can be submitted.
pub fn has_prompt_text(prompt: &str) -> bool {
    !prompt.trim().is_empty()
}

/// Maps the send/stop button signal to an action while enforcing prompt validation.
///
/// A generating composer always exposes Stop, whereas an idle composer ignores a
/// click for a blank prompt. The UI can therefore use this same decision for both
/// the button's enabled state and the returned action without duplicating rules.
pub fn button_action(is_generating: bool, prompt: &str, clicked: bool) -> Option<AgentComposerAction> {
    if !clicked {
        return None;
    }
    if is_generating {
        return Some(AgentComposerAction::Stop);
    }
    has_prompt_text(prompt).then_some(AgentComposerAction::Submit)
}

/// Calculates a badge background from the egui cursor and measured text width.
///
/// Text measurement belongs to `ui.rs`, but the component-owned padding and
/// baseline offsets stay in this pure geometry helper so they are easy to verify.
pub fn badge_rect(cursor: Pos2, text_width: f32) -> Rect {
    Rect::from_min_size(
        Pos2::new(cursor.x, cursor.y + config::BADGE_TOP_OFFSET),
        Vec2::new(text_width + config::BADGE_HORIZONTAL_PADDING, config::BADGE_HEIGHT),
    )
}

/// Returns the paint position for text inside a badge rectangle.
pub fn badge_text_pos(rect: Rect) -> Pos2 {
    Pos2::new(
        rect.left() + config::BADGE_TEXT_OFFSET_X,
        rect.top() + config::BADGE_TEXT_OFFSET_Y,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_submit_requires_focus_loss_enter_and_nonblank_prompt() {
        assert_eq!(
            keyboard_action(true, true, false, "SELECT 1"),
            Some(AgentComposerAction::Submit)
        );
        assert_eq!(keyboard_action(false, true, false, "SELECT 1"), None);
        assert_eq!(keyboard_action(true, false, false, "SELECT 1"), None);
        assert_eq!(keyboard_action(true, true, true, "SELECT 1"), None);
        assert_eq!(keyboard_action(true, true, false, "  \n\t"), None);
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
}
