//! Tests for `../component_gallery_rendering.rs` (moved out of the source file).
    use super::{linear_blend_reference, snap_pixel_center};
    use egui::Color32;

    #[test]
    fn linear_reference_encodes_half_white_over_black_as_srgb_188() {
        assert_eq!(linear_blend_reference([0, 0, 0], [255, 255, 255], 128), Color32::from_gray(188));
    }

    #[test]
    fn pixel_center_snapping_tracks_physical_scale() {
        assert_eq!(snap_pixel_center(10.0, 2.0, true), 10.25);
        assert_eq!(snap_pixel_center(10.0, 1.0, false), 10.0);
    }