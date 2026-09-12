use crate::DbProTheme;
use egui::{Color32, FontFamily, FontId, Pos2, Response, Rounding, Sense, Stroke, Ui, Vec2};
use lucide_icons::Icon;

/// Codex/ChatGPT status badge: 12px / 500, padding 2×8, radius-full.
const BADGE_FONT_SIZE: f32 = 12.0;
const BADGE_ICON_SIZE: f32 = 11.0;
const BADGE_PAD_X: f32 = 8.0;
const BADGE_PAD_Y: f32 = 2.0;
const BADGE_GAP: f32 = 4.0;
const BADGE_DOT: f32 = 5.0;
const BADGE_MIN_HEIGHT: f32 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Outline,
    Destructive,
    Success,
    Warning,
    Info,
}

pub struct Badge<'a> {
    pub(crate) text: &'a str,
    pub(crate) variant: BadgeVariant,
    pub(crate) dot: bool,
    pub(crate) icon: Option<Icon>,
    pub(crate) theme: DbProTheme,
    pub(crate) compact: bool,
}

impl<'a> Badge<'a> {
    pub fn new(text: &'a str, theme: DbProTheme) -> Self {
        Self {
            text,
            variant: BadgeVariant::Default,
            dot: false,
            icon: None,
            theme,
            compact: false,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn dot(mut self, dot: bool) -> Self {
        self.dot = dot;
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (fill, text_color, border_stroke, dot_color) = match self.variant {
            BadgeVariant::Default => (
                self.theme.accent_soft,
                self.theme.accent,
                Stroke::NONE,
                self.theme.accent,
            ),
            BadgeVariant::Secondary => (
                self.theme.surface_hover,
                self.theme.text_secondary,
                Stroke::NONE,
                self.theme.text_secondary,
            ),
            BadgeVariant::Outline => (
                Color32::TRANSPARENT,
                self.theme.text_primary,
                Stroke::new(1.0, self.theme.border_default),
                self.theme.text_muted,
            ),
            BadgeVariant::Destructive => (
                self.theme.danger.linear_multiply(0.15),
                self.theme.danger,
                Stroke::NONE,
                self.theme.danger,
            ),
            BadgeVariant::Success => (
                self.theme.success.linear_multiply(0.15),
                self.theme.success,
                Stroke::NONE,
                self.theme.success,
            ),
            BadgeVariant::Warning => (
                self.theme.warning.linear_multiply(0.15),
                self.theme.warning,
                Stroke::NONE,
                self.theme.warning,
            ),
            BadgeVariant::Info => (
                self.theme.info.linear_multiply(0.15),
                self.theme.info,
                Stroke::NONE,
                self.theme.info,
            ),
        };

        let (font_size, pad_x, pad_y, min_height, dot_size, icon_size) = if self.compact {
            (10.5, 6.0, 1.5, 18.0, 4.0, 9.5)
        } else {
            (
                BADGE_FONT_SIZE,
                BADGE_PAD_X,
                BADGE_PAD_Y,
                BADGE_MIN_HEIGHT,
                BADGE_DOT,
                BADGE_ICON_SIZE,
            )
        };

        let font_id = DbProTheme::ui_medium_font(font_size);
        let text_galley = ui.painter().layout_no_wrap(self.text.to_owned(), font_id, text_color);
        let leading = if self.dot {
            dot_size
        } else if self.icon.is_some() {
            icon_size
        } else {
            0.0
        };
        let gap = if leading > 0.0 { BADGE_GAP } else { 0.0 };
        let width = pad_x * 2.0 + leading + gap + text_galley.size().x;
        let height = (text_galley.size().y + pad_y * 2.0).max(min_height);

        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let rounding = Rounding::same(height * 0.5);
        ui.painter().rect_filled(rect, rounding, fill);
        if border_stroke != Stroke::NONE {
            ui.painter().rect_stroke(rect, rounding, border_stroke);
        }

        let mut cursor_x = rect.left() + pad_x;
        if self.dot {
            ui.painter().circle_filled(
                Pos2::new(cursor_x + dot_size * 0.5, rect.center().y),
                dot_size * 0.5,
                dot_color,
            );
            cursor_x += dot_size + gap;
        } else if let Some(icon) = self.icon {
            ui.painter().text(
                Pos2::new(cursor_x, rect.center().y),
                egui::Align2::LEFT_CENTER,
                char::from(icon).to_string(),
                FontId::new(icon_size, FontFamily::Name("lucide".into())),
                text_color,
            );
            cursor_x += icon_size + gap;
        }

        let text_pos = Pos2::new(cursor_x, rect.center().y - text_galley.size().y * 0.5);
        ui.painter().galley(text_pos, text_galley, text_color);
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbProTheme;

    #[test]
    fn badge_pill_is_compact_like_codex_status_chip() {
        let theme = DbProTheme::light();
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let mut height = 0.0;
        let mut width = 0.0;
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let response = Badge::new("Active", theme)
                    .variant(BadgeVariant::Success)
                    .dot(true)
                    .show(ui);
                height = response.rect.height();
                width = response.rect.width();
            });
        });
        assert!(
            (BADGE_MIN_HEIGHT..=22.0).contains(&height),
            "badge height {height} should stay near 20px"
        );
        assert!(width > height, "pill should be wider than tall");
    }
}
