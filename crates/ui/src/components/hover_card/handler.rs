use std::time::Duration;

use egui::{Pos2, Rect};

use super::config::{MAX_TIMER_DELAY_SECS, MIN_VISIBLE_HEIGHT, MIN_WIDTH, SCREEN_EDGE_INSET, TRIGGER_GAP};

/// Returns a safe timer delay in seconds.
///
/// Invalid public inputs mean no delay. Large finite values are bounded so the
/// same sanitized value is safe to pass to `Duration::from_secs_f64`.
pub fn sanitize_delay(delay: f64) -> f64 {
    if !delay.is_finite() || delay < 0.0 {
        return 0.0;
    }

    delay.min(MAX_TIMER_DELAY_SECS)
}

/// Converts a public timer delay into a panic-free repaint duration.
pub fn timer_duration(delay: f64) -> Duration {
    Duration::from_secs_f64(sanitize_delay(delay))
}

/// Whether an Escape press should dismiss the card and suppress reopening.
pub fn escape_dismissed(escape_pressed: bool, is_open: bool) -> bool {
    escape_pressed && is_open
}

/// Computes the open/closed state and updated timer state for the hover card.
///
/// Returns `(is_open, new_hover_start, new_leave_start)`.
pub fn compute_open_state(
    is_active: bool,
    was_open: bool,
    hover_start: Option<f64>,
    leave_start: Option<f64>,
    now: f64,
    open_delay: f64,
    close_delay: f64,
) -> (bool, Option<f64>, Option<f64>) {
    let open_delay = sanitize_delay(open_delay);
    let close_delay = sanitize_delay(close_delay);
    let now = if now.is_finite() { now } else { 0.0 };
    let mut next_hover_start = hover_start.filter(|start| start.is_finite());
    let mut next_leave_start = leave_start.filter(|start| start.is_finite());

    if is_active {
        next_leave_start = None;
        if next_hover_start.is_none() {
            next_hover_start = Some(now);
        }
    } else {
        next_hover_start = None;
        if was_open && next_leave_start.is_none() {
            next_leave_start = Some(now);
        }
    }

    let is_open = if is_active {
        let elapsed = next_hover_start.map(|t| now - t).unwrap_or(0.0);
        elapsed >= open_delay || was_open
    } else if was_open {
        let elapsed = next_leave_start.map(|t| now - t).unwrap_or(0.0);
        elapsed < close_delay
    } else {
        false
    };

    (is_open, next_hover_start, next_leave_start)
}

/// Calculates the screen position for the floating card, taking viewport bounds
/// and collision/overflow into account.
///
/// Flips above the trigger if there is insufficient space below and more space above.
pub fn calculate_card_position(trigger_rect: Rect, card_width: f32, estimated_height: f32, screen_rect: Rect) -> Pos2 {
    let min_x = screen_rect.left() + SCREEN_EDGE_INSET;
    let max_width = (screen_rect.width() - 2.0 * SCREEN_EDGE_INSET).max(0.0);
    let visible_width = card_width.min(max_width).max(0.0);
    let max_x = (screen_rect.right() - SCREEN_EDGE_INSET - visible_width).max(min_x);
    let x = trigger_rect.left().clamp(min_x.min(max_x), max_x);

    let min_y = screen_rect.top() + SCREEN_EDGE_INSET;
    let max_height = (screen_rect.height() - 2.0 * SCREEN_EDGE_INSET).max(0.0);
    let requested_height = if estimated_height.is_finite() {
        estimated_height.max(MIN_VISIBLE_HEIGHT)
    } else {
        MIN_VISIBLE_HEIGHT
    };
    let visible_height = requested_height.min(max_height);
    let max_y = (screen_rect.bottom() - SCREEN_EDGE_INSET - visible_height).max(min_y);
    let space_below = screen_rect.bottom() - (trigger_rect.bottom() + TRIGGER_GAP);
    let space_above = (trigger_rect.top() - TRIGGER_GAP) - screen_rect.top();
    let requested_y = if space_below < visible_height && space_above > space_below {
        trigger_rect.top() - TRIGGER_GAP - visible_height
    } else {
        trigger_rect.bottom() + TRIGGER_GAP
    };

    Pos2::new(x, requested_y.clamp(min_y.min(max_y), max_y))
}

/// Clamps the card width to fit safely within the screen bounds.
pub fn clamp_card_width(requested_width: f32, screen_width: f32) -> f32 {
    let available_width = (screen_width - 2.0 * SCREEN_EDGE_INSET).max(0.0);
    if available_width < MIN_WIDTH {
        return available_width;
    }

    let requested_width = if requested_width.is_finite() {
        requested_width.max(MIN_WIDTH)
    } else {
        MIN_WIDTH
    };
    requested_width.min(available_width)
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
