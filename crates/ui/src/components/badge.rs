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

use std::borrow::Cow;

pub struct Badge<'a> {
    pub(crate) text: Cow<'a, str>,
    pub(crate) variant: BadgeVariant,
    pub(crate) dot: bool,
    pub(crate) icon: Option<Icon>,
    pub(crate) theme: DbProTheme,
    pub(crate) compact: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BadgePalette {
    pub fill: Color32,
    pub text_color: Color32,
    pub border_stroke: Stroke,
    pub dot_color: Color32,
}

impl BadgePalette {
    pub fn from_variant(variant: BadgeVariant, theme: &DbProTheme) -> Self {
        match variant {
            BadgeVariant::Default => Self {
                fill: theme.accent_soft,
                text_color: theme.accent,
                border_stroke: Stroke::NONE,
                dot_color: theme.accent,
            },
            BadgeVariant::Secondary => Self {
                fill: theme.surface_hover,
                text_color: theme.text_secondary,
                border_stroke: Stroke::NONE,
                dot_color: theme.text_secondary,
            },
            BadgeVariant::Outline => Self {
                fill: Color32::TRANSPARENT,
                text_color: theme.text_primary,
                border_stroke: Stroke::new(1.0, theme.border_default),
                dot_color: theme.text_muted,
            },
            BadgeVariant::Destructive => Self {
                fill: theme.danger.linear_multiply(0.15),
                text_color: theme.danger,
                border_stroke: Stroke::NONE,
                dot_color: theme.danger,
            },
            BadgeVariant::Success => Self {
                fill: theme.success.linear_multiply(0.15),
                text_color: theme.success,
                border_stroke: Stroke::NONE,
                dot_color: theme.success,
            },
            BadgeVariant::Warning => Self {
                fill: theme.warning.linear_multiply(0.15),
                text_color: theme.warning,
                border_stroke: Stroke::NONE,
                dot_color: theme.warning,
            },
            BadgeVariant::Info => Self {
                fill: theme.info.linear_multiply(0.15),
                text_color: theme.info,
                border_stroke: Stroke::NONE,
                dot_color: theme.info,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BadgeMetrics {
    pub font_size: f32,
    pub pad_x: f32,
    pub pad_y: f32,
    pub min_height: f32,
    pub dot_size: f32,
    pub icon_size: f32,
    pub gap: f32,
}

impl BadgeMetrics {
    pub fn from_compact(compact: bool) -> Self {
        if compact {
            Self {
                font_size: 10.5,
                pad_x: 6.0,
                pad_y: 1.5,
                min_height: 18.0,
                dot_size: 4.0,
                icon_size: 9.5,
                gap: BADGE_GAP,
            }
        } else {
            Self {
                font_size: BADGE_FONT_SIZE,
                pad_x: BADGE_PAD_X,
                pad_y: BADGE_PAD_Y,
                min_height: BADGE_MIN_HEIGHT,
                dot_size: BADGE_DOT,
                icon_size: BADGE_ICON_SIZE,
                gap: BADGE_GAP,
            }
        }
    }

    pub fn leading_size(&self, has_dot: bool, has_icon: bool) -> f32 {
        if has_dot {
            self.dot_size
        } else if has_icon {
            self.icon_size
        } else {
            0.0
        }
    }

    pub fn calculate_size(&self, text_size: Vec2, has_dot: bool, has_icon: bool) -> Vec2 {
        let leading = self.leading_size(has_dot, has_icon);
        let gap = if leading > 0.0 { self.gap } else { 0.0 };
        let width = self.pad_x * 2.0 + leading + gap + text_size.x;
        let height = (text_size.y + self.pad_y * 2.0).max(self.min_height);
        Vec2::new(width, height)
    }
}

impl<'a> Badge<'a> {
    pub fn new(text: impl Into<Cow<'a, str>>, theme: DbProTheme) -> Self {
        Self {
            text: text.into(),
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
        let palette = BadgePalette::from_variant(self.variant, &self.theme);
        let metrics = BadgeMetrics::from_compact(self.compact);

        let font_id = DbProTheme::ui_medium_font(metrics.font_size);
        let text_galley = ui
            .painter()
            .layout_no_wrap(self.text.into_owned(), font_id, palette.text_color);

        let badge_size = metrics.calculate_size(text_galley.size(), self.dot, self.icon.is_some());
        let (rect, response) = ui.allocate_exact_size(badge_size, Sense::hover());

        paint_badge_background(ui, rect, &palette);
        paint_badge_content(ui, rect, &metrics, &palette, self.dot, self.icon, text_galley);

        response
    }
}

fn paint_badge_background(ui: &Ui, rect: egui::Rect, palette: &BadgePalette) {
    let rounding = Rounding::same(rect.height() * 0.5);
    ui.painter().rect_filled(rect, rounding, palette.fill);
    if palette.border_stroke != Stroke::NONE {
        ui.painter().rect_stroke(rect, rounding, palette.border_stroke);
    }
}

fn paint_badge_content(
    ui: &Ui,
    rect: egui::Rect,
    metrics: &BadgeMetrics,
    palette: &BadgePalette,
    dot: bool,
    icon: Option<Icon>,
    text_galley: std::sync::Arc<egui::Galley>,
) {
    let mut cursor_x = rect.left() + metrics.pad_x;
    let gap = if dot || icon.is_some() { metrics.gap } else { 0.0 };

    if dot {
        ui.painter().circle_filled(
            Pos2::new(cursor_x + metrics.dot_size * 0.5, rect.center().y),
            metrics.dot_size * 0.5,
            palette.dot_color,
        );
        cursor_x += metrics.dot_size + gap;
    } else if let Some(icon_glyph) = icon {
        ui.painter().text(
            Pos2::new(cursor_x, rect.center().y),
            egui::Align2::LEFT_CENTER,
            char::from(icon_glyph).to_string(),
            FontId::new(metrics.icon_size, FontFamily::Name("lucide".into())),
            palette.text_color,
        );
        cursor_x += metrics.icon_size + gap;
    }

    let text_pos = Pos2::new(cursor_x, rect.center().y - text_galley.size().y * 0.5);
    ui.painter().galley(text_pos, text_galley, palette.text_color);
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
