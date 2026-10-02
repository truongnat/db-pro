mod palette;
mod size;

pub use palette::ButtonPalette;
pub use size::SizeTokens;

use egui::Pos2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Default,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Sm,
    Default,
    Lg,
    Icon,
    IconSm,
}

pub fn leading_content_x(
    left_aligned: bool,
    rect_left: f32,
    rect_center_x: f32,
    content_width: f32,
    padding_x: f32,
) -> f32 {
    if left_aligned {
        rect_left + padding_x
    } else {
        rect_center_x - content_width * 0.5
    }
}

pub fn centered_content_pos(start_x: f32, center_y: f32, content_height: f32) -> Pos2 {
    Pos2::new(start_x, center_y - content_height * 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbProTheme;
    use egui::{Color32, Stroke, Vec2};

    #[test]
    fn size_tokens_preserve_existing_dimensions() {
        assert_eq!(SizeTokens::from_size(ButtonSize::Sm).min_height, 28.0);
        assert_eq!(SizeTokens::from_size(ButtonSize::Default).padding, Vec2::new(10.0, 5.0));
        assert_eq!(SizeTokens::from_size(ButtonSize::IconSm).default_width, 26.0);
    }

    #[test]
    fn width_uses_full_width_or_content_plus_padding() {
        let tokens = SizeTokens::from_size(ButtonSize::Default);
        assert_eq!(tokens.calculate_width(4.0, false, 200.0), 32.0);
        assert_eq!(tokens.calculate_width(20.0, false, 200.0), 40.0);
        assert_eq!(tokens.calculate_width(20.0, true, 200.0), 200.0);
    }

    #[test]
    fn leading_content_x_supports_centered_and_left_aligned_layouts() {
        assert_eq!(leading_content_x(false, 10.0, 50.0, 20.0, 8.0), 40.0);
        assert_eq!(leading_content_x(true, 10.0, 50.0, 20.0, 8.0), 18.0);
    }

    #[test]
    fn action_variants_stay_borderless_through_hover_loading_and_disabled_states() {
        let theme = DbProTheme::dark();
        let outline = ButtonPalette::from_variant(ButtonVariant::Outline, theme);
        assert_eq!(outline.fill_rest, Color32::TRANSPARENT);

        for variant in [
            ButtonVariant::Default,
            ButtonVariant::Secondary,
            ButtonVariant::Outline,
            ButtonVariant::Ghost,
            ButtonVariant::Destructive,
            ButtonVariant::Link,
        ] {
            let palette = ButtonPalette::from_variant(variant, theme);
            assert_eq!(palette.stroke_rest, Stroke::NONE, "{variant:?} rest");
            assert_eq!(palette.stroke_hover, Stroke::NONE, "{variant:?} hover");
            for step in 0..=10 {
                let (_, stroke) = palette.resolve_state(step as f32 / 10.0);
                assert_eq!(stroke, Stroke::NONE, "{variant:?} step {step}");
            }
            assert_eq!(
                palette.loading_colors(variant, theme).2,
                Stroke::NONE,
                "{variant:?} loading"
            );
        }

        let disabled = ButtonPalette::disabled(theme);
        assert_eq!(disabled.text_color, theme.text_disabled);
        assert_eq!(disabled.stroke_rest, Stroke::NONE);
        assert_eq!(disabled.stroke_hover, Stroke::NONE);
    }
}
