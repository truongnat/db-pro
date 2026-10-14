//! Tests for `../query_output_dock_surface_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn dock_body_is_the_dock_height_minus_measured_chrome() {
        // Body fills whatever the chrome (grip + tab strip) leaves of the dock budget.
        assert_eq!(QueryOutputDockContext::body_height(180.0, 34.0), 146.0);
        // A chrome taller than the dock never yields a negative allocation.
        assert_eq!(QueryOutputDockContext::body_height(20.0, 34.0), 0.0);
    }