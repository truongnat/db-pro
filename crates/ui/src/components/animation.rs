use egui::{epaint::PathShape, Color32, Id, Painter, Pos2, Stroke, Ui};
use std::f32::consts::PI;

/// Hover / control transitions. Spec §26: 120–160ms.
pub const HOVER_DURATION_SECS: f32 = 0.140;
/// Press / active squash. Same band as hover, slightly faster.
pub const PRESS_DURATION_SECS: f32 = 0.120;
/// Overlay / modal open-close. Spec §26: 160–220ms.
pub const OVERLAY_DURATION_SECS: f32 = 0.180;
/// Determinate progress uses hover timing, not a slow ease.
pub const PROGRESS_LERP_SECS: f32 = HOVER_DURATION_SECS;
const PRESS_SCALE_DELTA: f32 = 0.03;

/// One full spinner revolution. Loop period, not a transition duration.
pub const SPINNER_PERIOD_SECS: f32 = 0.80;
const SPINNER_ARC_SWEEP_RAD: f32 = PI * 1.35;
const SPINNER_ARC_SEGMENTS: usize = 24;

/// Indeterminate beam sweep. Loop period, not a transition duration.
pub const BEAM_PERIOD_SECS: f64 = 1.4;
/// Skeleton / pulse breathing. Loop period, not a transition duration.
pub const PULSE_PERIOD_SECS: f64 = 1.2;

const SLIGHT_SCALE_MIN: f32 = 0.98;

/// Transition durations consumed by hover, overlay, and progress helpers.
pub fn basic_ui_durations_secs() -> &'static [f32] {
    &[
        HOVER_DURATION_SECS,
        PRESS_DURATION_SECS,
        OVERLAY_DURATION_SECS,
        PROGRESS_LERP_SECS,
    ]
}

pub fn hover_t(ctx: &egui::Context, id: Id, on: bool) -> f32 {
    ctx.animate_bool_with_time(id, on, HOVER_DURATION_SECS)
}

pub fn press_t(ctx: &egui::Context, id: Id, on: bool) -> f32 {
    ctx.animate_bool_with_time(id, on, PRESS_DURATION_SECS)
}

pub fn overlay_t(ctx: &egui::Context, id: Id, on: bool) -> f32 {
    ctx.animate_bool_with_time(id, on, OVERLAY_DURATION_SECS)
}

pub fn press_scale(t: f32) -> f32 {
    1.0 - PRESS_SCALE_DELTA * fade_alpha(t)
}

pub fn lerp_color(from: Color32, to: Color32, t: f32) -> Color32 {
    from.lerp_to_gamma(to, fade_alpha(t))
}

pub fn fade_alpha(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}

pub fn slight_scale(t: f32) -> f32 {
    SLIGHT_SCALE_MIN + (1.0 - SLIGHT_SCALE_MIN) * fade_alpha(t)
}

pub fn small_translate(t: f32, distance_px: f32) -> f32 {
    (1.0 - fade_alpha(t)) * distance_px
}

pub fn spinner_angle_at(time_secs: f64) -> f32 {
    let period = f64::from(SPINNER_PERIOD_SECS);
    let phase = time_secs.rem_euclid(period) / period;
    (phase as f32) * 2.0 * PI
}

/// Continuous rotation angle in radians; requests a repaint each frame.
pub fn spinner_angle(ui: &Ui) -> f32 {
    ui.ctx().request_repaint();
    spinner_angle_at(ui.input(|i| i.time))
}

/// Vector spinner: quiet track plus a rotating arc.
pub fn paint_spinner(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    stroke_width: f32,
    color: Color32,
    track_color: Color32,
    angle: f32,
) {
    painter.circle_stroke(center, radius, Stroke::new(stroke_width, track_color));

    let mut arc_points = Vec::with_capacity(SPINNER_ARC_SEGMENTS);
    let last = (SPINNER_ARC_SEGMENTS - 1) as f32;
    for i in 0..SPINNER_ARC_SEGMENTS {
        let t = i as f32 / last;
        let a = angle + t * SPINNER_ARC_SWEEP_RAD;
        arc_points.push(Pos2::new(center.x + a.cos() * radius, center.y + a.sin() * radius));
    }

    painter.add(PathShape::line(arc_points, Stroke::new(stroke_width, color)));
}

/// Breathing opacity between `min_alpha` and `max_alpha`.
pub fn pulse_alpha(ui: &Ui, min_alpha: f32, max_alpha: f32) -> f32 {
    let min_alpha = min_alpha.clamp(0.0, 1.0);
    let max_alpha = max_alpha.clamp(min_alpha, 1.0);
    ui.ctx().request_repaint();
    let time = ui.input(|i| i.time);
    let phase = ((time % PULSE_PERIOD_SECS) / PULSE_PERIOD_SECS * (2.0 * std::f64::consts::PI)) as f32;
    let norm = (phase.sin() + 1.0) * 0.5;
    min_alpha + norm * (max_alpha - min_alpha)
}

/// `(tail, head)` fractions for an indeterminate beam sweeping `[0, 1]`.
pub fn indeterminate_beam(ui: &Ui) -> (f32, f32) {
    ui.ctx().request_repaint();
    let time = ui.input(|i| i.time);
    beam_fractions_at(time, BEAM_PERIOD_SECS)
}

pub fn beam_fractions_at(time_secs: f64, period_secs: f64) -> (f32, f32) {
    let period = period_secs.max(f64::EPSILON);
    let t = ((time_secs.rem_euclid(period)) / period) as f32;
    let head = (t * 1.4 - 0.2).clamp(0.0, 1.0);
    let tail = (t * 1.4 - 0.5).clamp(0.0, 1.0);
    let head_eased = (head * PI * 0.5).sin();
    let tail_eased = (tail * tail).min(head_eased);
    (tail_eased, head_eased)
}

pub fn faded_overlay(overlay: Color32, t: f32) -> Color32 {
    Color32::from_black_alpha((f32::from(overlay.a()) * fade_alpha(t)) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_ui(mut on_ui: impl FnMut(&mut Ui)) {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
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
        let _ = ctx.run(Default::default(), |ctx| {
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
}
