//! Tests for `../handler.rs` (moved out of the source file).
use egui::Vec2;

use super::*;
use crate::components::hover_card::config::{DEFAULT_CLOSE_DELAY, DEFAULT_OPEN_DELAY};

#[test]
fn invalid_delays_are_sanitized_to_immediate() {
    let (open, _, _) = compute_open_state(true, false, None, None, 1.0, f64::NAN, f64::INFINITY);
    assert!(open);
    assert_eq!(sanitize_delay(-1.0), 0.0);
    assert_eq!(sanitize_delay(f64::INFINITY), 0.0);
    assert_eq!(sanitize_delay(f64::NEG_INFINITY), 0.0);
}

#[test]
fn finite_delays_are_clamped_to_safe_timer_bound() {
    assert_eq!(sanitize_delay(MAX_TIMER_DELAY_SECS), MAX_TIMER_DELAY_SECS);
    assert_eq!(sanitize_delay(f64::MAX), MAX_TIMER_DELAY_SECS);
    assert_eq!(timer_duration(f64::MAX), Duration::from_secs(60));
    assert_eq!(timer_duration(f64::NAN), Duration::ZERO);
}

#[test]
fn escape_only_dismisses_an_open_card() {
    assert!(escape_dismissed(true, true));
    assert!(!escape_dismissed(true, false));
    assert!(!escape_dismissed(false, true));
}

#[test]
fn measured_height_controls_flip_decision() {
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 500.0));
    let trigger = Rect::from_min_size(Pos2::new(100.0, 300.0), Vec2::new(80.0, 30.0));
    let pos = calculate_card_position(trigger, 300.0, 100.0, screen);
    assert_eq!(pos.y, 336.0);
}

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
