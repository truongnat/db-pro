use super::config::{MIN_VISIBLE_HEIGHT, MIN_WIDTH, SCREEN_EDGE_INSET, TRIGGER_GAP};
use egui::{Pos2, Rect};

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
    let mut next_hover_start = hover_start;
    let mut next_leave_start = leave_start;

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
mod tests {
    use super::*;
    use crate::components::hover_card::config::{DEFAULT_CLOSE_DELAY, DEFAULT_OPEN_DELAY};
    use egui::Vec2;

    #[test]
    fn hover_timer_opens_after_delay() {
        let (open_immediate, hover_start, _) =
            compute_open_state(true, false, None, None, 10.0, DEFAULT_OPEN_DELAY, DEFAULT_CLOSE_DELAY);
        assert!(!open_immediate);
        assert_eq!(hover_start, Some(10.0));

        let (open_before_delay, _, _) = compute_open_state(
            true,
            false,
            hover_start,
            None,
            10.10,
            DEFAULT_OPEN_DELAY,
            DEFAULT_CLOSE_DELAY,
        );
        assert!(!open_before_delay);

        let (open_after_delay, _, _) = compute_open_state(
            true,
            false,
            hover_start,
            None,
            10.25,
            DEFAULT_OPEN_DELAY,
            DEFAULT_CLOSE_DELAY,
        );
        assert!(open_after_delay);
    }

    #[test]
    fn leave_timer_closes_after_delay() {
        let (stay_open_immediate, _, leave_start) =
            compute_open_state(false, true, None, None, 20.0, DEFAULT_OPEN_DELAY, DEFAULT_CLOSE_DELAY);
        assert!(stay_open_immediate);
        assert_eq!(leave_start, Some(20.0));

        let (stay_open_during_grace, _, _) = compute_open_state(
            false,
            true,
            None,
            leave_start,
            20.10,
            DEFAULT_OPEN_DELAY,
            DEFAULT_CLOSE_DELAY,
        );
        assert!(stay_open_during_grace);

        let (closed_after_grace, _, _) = compute_open_state(
            false,
            true,
            None,
            leave_start,
            20.20,
            DEFAULT_OPEN_DELAY,
            DEFAULT_CLOSE_DELAY,
        );
        assert!(!closed_after_grace);
    }

    #[test]
    fn active_card_cancels_leave_timer() {
        let (is_open, hover_start, leave_start) = compute_open_state(
            true,
            true,
            None,
            Some(10.0),
            10.05,
            DEFAULT_OPEN_DELAY,
            DEFAULT_CLOSE_DELAY,
        );
        assert!(is_open);
        assert_eq!(leave_start, None);
        assert_eq!(hover_start, Some(10.05));
    }

    #[test]
    fn calculate_card_position_normal_placement() {
        let screen = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(1280.0, 800.0));
        let trigger = Rect::from_min_size(Pos2::new(100.0, 100.0), Vec2::new(80.0, 30.0));
        let pos = calculate_card_position(trigger, 300.0, 150.0, screen);

        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 100.0 + 30.0 + TRIGGER_GAP);
    }

    #[test]
    fn calculate_card_position_clamps_right_overflow() {
        let screen = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(1000.0, 800.0));
        let trigger = Rect::from_min_size(Pos2::new(850.0, 100.0), Vec2::new(80.0, 30.0));
        let pos = calculate_card_position(trigger, 300.0, 150.0, screen);

        // 1000 - 300 - 10 = 690
        assert_eq!(pos.x, 690.0);
        assert_eq!(pos.y, 136.0);
    }

    #[test]
    fn calculate_card_position_flips_above_when_bottom_overflows() {
        let screen = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(1000.0, 500.0));
        let trigger = Rect::from_min_size(Pos2::new(100.0, 450.0), Vec2::new(80.0, 30.0));
        let pos = calculate_card_position(trigger, 300.0, 150.0, screen);

        assert_eq!(pos.x, 100.0);
        // Position above: trigger.top (450) - TRIGGER_GAP (6) - estimated_height (150) = 294
        assert_eq!(pos.y, 294.0);
    }

    #[test]
    fn clamp_card_width_respects_bounds() {
        assert_eq!(clamp_card_width(300.0, 1280.0), 300.0);
        assert_eq!(clamp_card_width(50.0, 1280.0), MIN_WIDTH);
        assert_eq!(clamp_card_width(500.0, 400.0), 380.0);
        assert_eq!(clamp_card_width(300.0, 100.0), 80.0);
        assert_eq!(clamp_card_width(f32::NAN, 1280.0), MIN_WIDTH);
    }

    #[test]
    fn clamp_card_width_handles_viewport_narrower_than_insets() {
        assert_eq!(clamp_card_width(300.0, 12.0), 0.0);
    }
}
