use crate::DbProTheme;
use egui::{FontFamily, FontId, Pos2, Response, Rounding, Sense, Stroke, Ui, WidgetInfo, WidgetType};
use lucide_icons::Icon;
use std::borrow::Cow;

use super::config::BADGE_RADIUS;
use super::handler::{leading_gap, text_position, BadgeMetrics, BadgePalette, BadgeVariant};

pub struct Badge<'a> {
    pub(crate) text: Cow<'a, str>,
    pub(crate) variant: BadgeVariant,
    pub(crate) dot: bool,
    pub(crate) icon: Option<Icon>,
    pub(crate) theme: DbProTheme,
    pub(crate) compact: bool,
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

        let accessible_label = self.text.to_string();
        let font_id = DbProTheme::ui_medium_font(metrics.font_size);
        let text_galley = ui
            .painter()
            .layout_no_wrap(self.text.into_owned(), font_id, palette.text_color);

        let badge_size = metrics.calculate_size(text_galley.size(), self.dot, self.icon.is_some());
        let (rect, response) = ui.allocate_exact_size(badge_size, Sense::hover());
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &accessible_label));

        paint_badge_background(ui, rect, &palette);
        paint_badge_content(ui, rect, &metrics, &palette, self.dot, self.icon, text_galley);

        response
    }
}

fn paint_badge_background(ui: &Ui, rect: egui::Rect, palette: &BadgePalette) {
    let rounding = Rounding::same(BADGE_RADIUS);
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
    let gap = leading_gap(dot, icon.is_some(), metrics.gap);

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

    ui.painter().galley(
        text_position(cursor_x, rect.center().y, text_galley.size().y),
        text_galley,
        palette.text_color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbProTheme;

    #[test]
    fn badge_uses_compact_workstation_metrics() {
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
            (super::super::config::BADGE_MIN_HEIGHT..=22.0).contains(&height),
            "badge height {height} should stay near 20px"
        );
        assert!(width > height, "badge should be wider than tall");
    }

    #[test]
    fn semantic_badges_keep_a_visible_boundary() {
        let theme = DbProTheme::dark();
        for variant in [
            BadgeVariant::Default,
            BadgeVariant::Secondary,
            BadgeVariant::Outline,
            BadgeVariant::Destructive,
            BadgeVariant::Success,
            BadgeVariant::Warning,
            BadgeVariant::Info,
        ] {
            let palette = BadgePalette::from_variant(variant, &theme);
            assert_ne!(
                palette.border_stroke,
                Stroke::NONE,
                "{variant:?} lost its status boundary"
            );
        }
    }
}
