//! Tests for `../query_output_actions_view.rs` (moved out of the source file).
    use super::bounded_explain_metric;

    #[test]
    fn explain_display_metrics_are_finite_and_bounded() {
        assert_eq!(bounded_explain_metric(f64::MAX), f32::MAX);
        assert_eq!(bounded_explain_metric(f64::NAN), 0.0);
        assert_eq!(bounded_explain_metric(-1.0), 0.0);
    }