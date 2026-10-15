use egui::{Pos2, Rect, Vec2};

use super::{config, AgentComposerAction};

/// Converts the composer's submit signal into an action.
///
/// The UI consumes Enter (or Ctrl/Cmd+Enter) before the multiline editor can
/// turn it into a newline — egui's `TextEdit::multiline` never reports
/// `lost_focus` on Enter, so a focus-based predicate could never submit
/// (#358). Shift+Enter is left untouched and still inserts a newline.
pub fn keyboard_action(enter_submitted: bool, prompt: &str) -> Option<AgentComposerAction> {
    if !enter_submitted || !has_prompt_text(prompt) {
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
#[path = "tests/handler_tests.rs"]
mod tests;
