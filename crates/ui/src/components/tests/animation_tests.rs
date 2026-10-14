//! Tests for `../animation.rs` (moved out of the source file).
use super::*;

fn run_ui(mut on_ui: impl FnMut(&mut Ui)) {
    let ctx = egui::Context::default();
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| on_ui(ui));
    });
}

#[test]
fn spinner_angle_is_finite_and_periodic() {
    let a0 = spinner_angle_at(0.1);
    let a1 = spinner_angle_at(0.1 + f64::from(SPINNER_PERIOD_SECS));
    let a2 = spinner_angle_at(0.1 + f64::from(SPINNER_PERIOD_SECS) * 2.0);
    assert!(a0.is_finite() && a1.is_finite() && a2.is_finite());
    assert!((a0 - a1).abs() < 1e-4);
    assert!((a0 - a2).abs() < 1e-4);

    run_ui(|ui| {
        let angle = spinner_angle(ui);
        assert!(angle.is_finite());
        assert!((0.0..=2.0 * PI + 1e-3).contains(&angle));
    });
}

#[test]
fn reduce_motion_disables_requested_animation() {
    assert!(!should_animate(true, true));
    assert!(!should_animate(false, false));
    assert!(should_animate(true, false));
}

#[test]
fn pulse_alpha_stays_inside_requested_range() {
    run_ui(|ui| {
        let alpha = pulse_alpha(ui, 0.2, 0.8);
        assert!((0.2..=0.8).contains(&alpha));
    });
}

#[test]
fn indeterminate_beam_fractions_stay_ordered_in_unit_interval() {
    for step in 0..32 {
        let time = f64::from(step) * (BEAM_PERIOD_SECS / 16.0);
        let (tail, head) = beam_fractions_at(time, BEAM_PERIOD_SECS);
        assert!((0.0..=1.0).contains(&tail), "tail={tail} time={time}");
        assert!((0.0..=1.0).contains(&head), "head={head} time={time}");
        assert!(tail <= head, "tail={tail} head={head} time={time}");
    }

    run_ui(|ui| {
        let (tail, head) = indeterminate_beam(ui);
        assert!((0.0..=1.0).contains(&tail));
        assert!((0.0..=1.0).contains(&head));
        assert!(tail <= head);
    });
}

#[test]
fn named_durations_match_spec_and_helpers_consume_them() {
    assert!((0.120..=0.160).contains(&HOVER_DURATION_SECS));
    assert!((0.160..=0.220).contains(&OVERLAY_DURATION_SECS));
    for duration in basic_ui_durations_secs() {
        assert!(*duration < 0.400, "basic UI duration {duration}s must stay under 400ms");
    }

    let ctx = egui::Context::default();
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        let hover = hover_t(ctx, Id::new("hover"), true);
        let overlay = overlay_t(ctx, Id::new("overlay"), true);
        assert!((0.0..=1.0).contains(&hover));
        assert!((0.0..=1.0).contains(&overlay));
        assert!((0.0..=1.0).contains(&fade_alpha(hover)));
        let scale = slight_scale(overlay);
        assert!((SLIGHT_SCALE_MIN..=1.0).contains(&scale));
        assert_eq!(small_translate(1.0, 8.0), 0.0);
        assert_eq!(small_translate(0.0, 8.0), 8.0);
        assert!((0.120..=0.160).contains(&PRESS_DURATION_SECS));
        let press = press_t(ctx, Id::new("press"), true);
        assert!((0.0..=1.0).contains(&press));
        assert_eq!(press_scale(0.0), 1.0);
        assert!((press_scale(1.0) - (1.0 - PRESS_SCALE_DELTA)).abs() < 1e-5);
        let mixed = lerp_color(Color32::BLACK, Color32::WHITE, 0.5);
        assert_ne!(mixed, Color32::BLACK);
        assert_ne!(mixed, Color32::WHITE);
    });
}
