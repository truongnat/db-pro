use super::ButtonVariant;
use crate::components::animation::lerp_color;
use crate::tokens::STROKE_THIN;
use crate::DbProTheme;
use egui::{Color32, Stroke};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonPalette {
    pub fill_rest: Color32,
    pub fill_hover: Color32,
    pub stroke_rest: Stroke,
    pub stroke_hover: Stroke,
    pub text_color: Color32,
}

impl ButtonPalette {
    pub fn from_variant(variant: ButtonVariant, theme: DbProTheme) -> Self {
        // A variant changes semantic roles, while the supplied theme determines
        // their light/dark values. UI consumes this palette without new colors.
        match variant {
            ButtonVariant::Default => Self {
                fill_rest: theme.accent,
                fill_hover: theme.accent_hover,
                stroke_rest: Stroke::NONE,
                stroke_hover: Stroke::NONE,
                text_color: theme.text_on_solid(theme.accent),
            },
            ButtonVariant::Secondary => Self {
                fill_rest: theme.surface_hover,
                fill_hover: theme.surface_active,
                stroke_rest: Stroke::NONE,
                stroke_hover: Stroke::NONE,
                text_color: theme.text_primary,
            },
            ButtonVariant::Outline => Self {
                fill_rest: Color32::TRANSPARENT,
                fill_hover: theme.surface_hover,
                stroke_rest: Stroke::new(STROKE_THIN, theme.border_default),
                stroke_hover: Stroke::new(STROKE_THIN, theme.border_strong),
                text_color: theme.text_primary,
            },
            ButtonVariant::Ghost => Self {
                fill_rest: Color32::TRANSPARENT,
                fill_hover: theme.surface_hover,
                stroke_rest: Stroke::NONE,
                stroke_hover: Stroke::NONE,
                text_color: theme.text_primary,
            },
            ButtonVariant::Destructive => Self {
                fill_rest: theme.danger,
                fill_hover: theme.danger.linear_multiply(0.85),
                stroke_rest: Stroke::NONE,
                stroke_hover: Stroke::NONE,
                text_color: theme.text_on_solid(theme.danger),
            },
            ButtonVariant::Link => Self {
                fill_rest: Color32::TRANSPARENT,
                fill_hover: Color32::TRANSPARENT,
                stroke_rest: Stroke::NONE,
                stroke_hover: Stroke::NONE,
                text_color: theme.accent,
            },
        }
    }

    pub fn disabled(theme: DbProTheme) -> Self {
        Self {
            fill_rest: theme.surface_2,
            fill_hover: theme.surface_2,
            stroke_rest: Stroke::new(STROKE_THIN, theme.border_subtle),
            stroke_hover: Stroke::new(STROKE_THIN, theme.border_subtle),
            text_color: theme.text_disabled,
        }
    }

    pub fn text_on_fill(&self, variant: ButtonVariant, fill: Color32, theme: DbProTheme) -> Color32 {
        if !matches!(variant, ButtonVariant::Default | ButtonVariant::Destructive) {
            return self.text_color;
        }
        // Hover blends can cross the contrast threshold; core chooses the
        // readable on-solid foreground from the actual frame's fill.
        theme.text_on_solid(fill)
    }

    pub fn loading_colors(&self, variant: ButtonVariant, theme: DbProTheme) -> (Color32, Color32, Stroke) {
        match variant {
            ButtonVariant::Default => (self.text_color, self.fill_rest, Stroke::NONE),
            ButtonVariant::Secondary => (theme.text_secondary, theme.surface_hover, Stroke::NONE),
            ButtonVariant::Outline => (
                theme.text_secondary,
                Color32::TRANSPARENT,
                Stroke::new(STROKE_THIN, theme.border_default),
            ),
            ButtonVariant::Ghost => (theme.text_secondary, Color32::TRANSPARENT, Stroke::NONE),
            ButtonVariant::Destructive => (self.text_color, self.fill_rest, Stroke::NONE),
            ButtonVariant::Link => (theme.text_secondary, Color32::TRANSPARENT, Stroke::NONE),
        }
    }

    pub fn resolve_state(&self, hover: f32) -> (Color32, Stroke) {
        // egui's hover progress becomes the final surface colors and border;
        // preserving the stroke width during the blend avoids a border jump.
        let fill = lerp_color(self.fill_rest, self.fill_hover, hover);
        let stroke_color = lerp_color(self.stroke_rest.color, self.stroke_hover.color, hover);
        let stroke_width = self.stroke_rest.width + (self.stroke_hover.width - self.stroke_rest.width) * hover;
        let stroke = if stroke_width > 0.0 && stroke_color != Color32::TRANSPARENT {
            Stroke::new(stroke_width, stroke_color)
        } else {
            Stroke::NONE
        };
        (fill, stroke)
    }
}

#[cfg(test)]
fn contrast_ratio(foreground: Color32, background: Color32) -> f32 {
    let a = crate::theme::relative_luminance(foreground);
    let b = crate::theme::relative_luminance(background);
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filled_button_text_keeps_normal_text_contrast_through_hover() {
        let baseline_ratio = contrast_ratio(Color32::WHITE, DbProTheme::light().accent);
        assert!((baseline_ratio - 3.62).abs() < 0.03);
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
}
