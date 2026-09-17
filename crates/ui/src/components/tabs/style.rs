//! Label typography for segmented and underline tab items.

use super::config::{SEGMENTED_FONT_SIZE, UNDERLINE_FONT_SIZE};
use crate::DbProTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TabKind {
    Segmented,
    Underline,
}

pub(super) struct TabItemStyle {
    pub font_id: egui::FontId,
    pub text_color: egui::Color32,
}

impl TabItemStyle {
    pub fn new(kind: TabKind, is_active: bool, is_hovered: bool, theme: &DbProTheme) -> Self {
        let size = match kind {
            TabKind::Segmented => SEGMENTED_FONT_SIZE,
            TabKind::Underline => UNDERLINE_FONT_SIZE,
        };
        let font_id = if is_active {
            DbProTheme::ui_medium_font(size)
        } else {
            egui::FontId::proportional(size)
        };
        let text_color = if is_active || is_hovered {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        Self { font_id, text_color }
    }
}
