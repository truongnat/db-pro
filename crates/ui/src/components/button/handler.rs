use crate::components::animation::lerp_color;
use crate::DbProTheme;
use egui::{Color32, Pos2, Stroke, Vec2};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeTokens {
    pub min_height: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub padding: Vec2,
    pub default_width: f32,
}

impl SizeTokens {
    pub fn from_size(size: ButtonSize) -> Self {
        use super::config::*;
        match size {
            ButtonSize::Sm => Self {
                min_height: SM_MIN_HEIGHT,
                font_size: SM_FONT_SIZE,
                icon_size: SM_ICON_SIZE,
                padding: SM_PADDING,
                default_width: SM_DEFAULT_WIDTH,
            },
            ButtonSize::Default => Self {
                min_height: DEFAULT_MIN_HEIGHT,
                font_size: DEFAULT_FONT_SIZE,
                icon_size: DEFAULT_ICON_SIZE,
                padding: DEFAULT_PADDING,
                default_width: DEFAULT_WIDTH,
            },
            ButtonSize::Lg => Self {
                min_height: LG_MIN_HEIGHT,
                font_size: LG_FONT_SIZE,
                icon_size: LG_ICON_SIZE,
                padding: LG_PADDING,
                default_width: LG_DEFAULT_WIDTH,
            },
            ButtonSize::Icon => Self {
                min_height: DEFAULT_MIN_HEIGHT,
                font_size: DEFAULT_FONT_SIZE,
                icon_size: ICON_SIZE,
                padding: ICON_PADDING,
                default_width: DEFAULT_WIDTH,
            },
            ButtonSize::IconSm => Self {
                min_height: ICON_SM_DEFAULT_WIDTH,
                font_size: ICON_SM_FONT_SIZE,
                icon_size: ICON_SM_SIZE,
                padding: ICON_SM_PADDING,
                default_width: ICON_SM_DEFAULT_WIDTH,
            },
        }
    }

    pub fn calculate_width(&self, content_width: f32, full_width: bool, available_width: f32) -> f32 {
        if full_width {
            available_width
        } else {
            self.default_width.max(content_width + self.padding.x * 2.0)
        }
    }
}

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
        match variant {
            ButtonVariant::Default => Self {
                fill_rest: theme.accent,
                fill_hover: theme.accent_hover,
                stroke_rest: Stroke::NONE,
                stroke_hover: Stroke::NONE,
                text_color: theme.accent_foreground,
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
                stroke_rest: Stroke::new(1.0, theme.border_default),
                stroke_hover: Stroke::new(1.0, theme.border_strong),
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
                text_color: theme.accent_foreground,
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
            stroke_rest: Stroke::new(1.0, theme.border_subtle),
            stroke_hover: Stroke::new(1.0, theme.border_subtle),
            text_color: theme.text_disabled,
        }
    }

    pub fn loading_colors(&self, variant: ButtonVariant, theme: DbProTheme) -> (Color32, Color32, Stroke) {
        match variant {
            ButtonVariant::Default => (self.text_color, self.fill_rest, Stroke::NONE),
            ButtonVariant::Secondary => (theme.text_secondary, theme.surface_hover, Stroke::NONE),
            ButtonVariant::Outline => (
                theme.text_secondary,
                Color32::TRANSPARENT,
                Stroke::new(1.0, theme.border_default),
            ),
            ButtonVariant::Ghost => (theme.text_secondary, Color32::TRANSPARENT, Stroke::NONE),
            ButtonVariant::Destructive => (self.text_color, self.fill_rest, Stroke::NONE),
            ButtonVariant::Link => (theme.text_secondary, Color32::TRANSPARENT, Stroke::NONE),
        }
    }

    pub fn resolve_state(&self, hover: f32) -> (Color32, Stroke) {
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
    fn palette_preserves_outline_border_and_disabled_text() {
        let theme = DbProTheme::dark();
        let outline = ButtonPalette::from_variant(ButtonVariant::Outline, theme);
        assert_eq!(outline.fill_rest, Color32::TRANSPARENT);
        assert_eq!(outline.stroke_rest, Stroke::new(1.0, theme.border_default));

        let disabled = ButtonPalette::disabled(theme);
        assert_eq!(disabled.text_color, theme.text_disabled);
        assert_eq!(disabled.stroke_rest, Stroke::new(1.0, theme.border_subtle));
    }
}
