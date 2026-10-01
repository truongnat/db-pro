use super::ButtonSize;
use crate::tokens::component::button as core;
use egui::Vec2;

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
        // The public size choice maps straight to core tokens. No local config
        // aliases can drift from the dimensions used by other button surfaces.
        match size {
            ButtonSize::Sm => Self {
                min_height: core::BUTTON_HEIGHT_SM,
                font_size: core::BUTTON_FONT_SIZE_SM,
                icon_size: core::BUTTON_ICON_SIZE_SM_GLYPH,
                padding: core::BUTTON_SM_PADDING,
                default_width: core::BUTTON_HEIGHT_SM,
            },
            ButtonSize::Default => Self {
                min_height: core::BUTTON_HEIGHT_DEFAULT,
                font_size: core::BUTTON_FONT_SIZE_DEFAULT,
                icon_size: core::BUTTON_ICON_SIZE_DEFAULT_GLYPH,
                padding: core::BUTTON_DEFAULT_PADDING,
                default_width: core::BUTTON_DEFAULT_WIDTH,
            },
            ButtonSize::Lg => Self {
                min_height: core::BUTTON_HEIGHT_LG,
                font_size: core::BUTTON_FONT_SIZE_LG,
                icon_size: core::BUTTON_ICON_SIZE_LG_GLYPH,
                padding: core::BUTTON_LG_PADDING,
                default_width: core::BUTTON_LG_WIDTH,
            },
            ButtonSize::Icon => Self {
                min_height: core::BUTTON_HEIGHT_DEFAULT,
                font_size: core::BUTTON_FONT_SIZE_DEFAULT,
                icon_size: core::BUTTON_ICON_SIZE_GLYPH,
                padding: core::BUTTON_ICON_PADDING,
                default_width: core::BUTTON_DEFAULT_WIDTH,
            },
            ButtonSize::IconSm => Self {
                min_height: core::BUTTON_ICON_SIZE_SM.x,
                font_size: core::BUTTON_ICON_SM_FONT_SIZE,
                icon_size: core::BUTTON_ICON_SIZE_SM_GLYPH,
                padding: core::BUTTON_ICON_SM_PADDING,
                default_width: core::BUTTON_ICON_SIZE_SM.x,
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
