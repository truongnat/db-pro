//! Tests for `../palette.rs` (moved out of the source file).
use super::*;

#[test]
fn filled_button_text_keeps_normal_text_contrast_through_hover() {
    for theme in [DbProTheme::light(), DbProTheme::dark()] {
        for variant in [ButtonVariant::Default, ButtonVariant::Destructive] {
            let palette = ButtonPalette::from_variant(variant, theme);
            for step in 0..=100 {
                let (fill, _) = palette.resolve_state(step as f32 / 100.0);
                let text = palette.text_on_fill(variant, fill, theme);
                assert!(contrast_ratio(text, fill) >= 4.5, "{variant:?} step {step}");
            }
        }
    }
}
