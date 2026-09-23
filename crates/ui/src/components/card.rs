use crate::tokens::{SPACE_MD, SPACE_SM, SPACE_XXS};
use crate::DbProTheme;
use egui::{FontFamily, FontId, Frame, Margin, Pos2, RichText, Rounding, Stroke, Ui, Vec2};
use lucide_icons::Icon;

pub struct Card {
    theme: DbProTheme,
}

impl Card {
    pub fn new(theme: DbProTheme) -> Self {
        Self { theme }
    }

    pub fn frame(&self) -> Frame {
        Frame {
            fill: self.theme.surface_elevated,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            inner_margin: Margin::same(16.0),
            rounding: Rounding::same(8.0),
            shadow: egui::epaint::Shadow {
                offset: egui::vec2(0.0, 2.0),
                blur: 8.0,
                spread: 0.0,
                color: egui::Color32::from_black_alpha(10),
            },
            ..Default::default()
        }
    }

    pub fn show<R>(&self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        self.frame().show(ui, add_contents).inner
    }
}

use std::borrow::Cow;

pub fn card_header<'a>(
    ui: &mut Ui,
    title: impl Into<Cow<'a, str>>,
    description: Option<impl Into<Cow<'a, str>>>,
    theme: DbProTheme,
) {
    let title = title.into();
    let description = description.map(|d| d.into());
    ui.vertical(|ui| {
        ui.label(
            RichText::new(title.as_ref())
                .size(15.0)
                .strong()
                .color(theme.text_primary),
        );
        if let Some(desc) = description {
            ui.add_space(SPACE_XXS);
            ui.label(RichText::new(desc.as_ref()).size(12.0).color(theme.text_secondary));
        }
    });
    ui.add_space(SPACE_SM);
}

pub fn card_content<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    ui.vertical(|ui| add_contents(ui)).inner
}

pub fn card_footer<R>(ui: &mut Ui, theme: DbProTheme, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    ui.add_space(SPACE_MD);
    let avail_w = ui.available_width();
    let (sep_rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, 1.0), egui::Sense::hover());
    ui.painter().line_segment(
        [
            Pos2::new(sep_rect.left(), sep_rect.center().y),
            Pos2::new(sep_rect.right(), sep_rect.center().y),
        ],
        Stroke::new(1.0, theme.border_subtle),
    );
    ui.add_space(SPACE_SM);

    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| add_contents(ui))
            .inner
    })
    .inner
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetricTrend<'a> {
    pub text: Cow<'a, str>,
    pub is_positive: bool,
}

impl<'a> MetricTrend<'a> {
    pub fn new(text: impl Into<Cow<'a, str>>, is_positive: bool) -> Self {
        Self {
            text: text.into(),
            is_positive,
        }
    }

    pub fn style(&self, theme: &DbProTheme) -> (egui::Color32, Icon) {
        if self.is_positive {
            (theme.success, Icon::TrendingUp)
        } else {
            (theme.danger, Icon::TrendingDown)
        }
    }
}

pub struct MetricCard<'a> {
    title: Cow<'a, str>,
    value: Cow<'a, str>,
    trend: Option<MetricTrend<'a>>,
    icon: Option<Icon>,
    theme: DbProTheme,
}

impl<'a> MetricCard<'a> {
    pub fn new(title: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>, theme: DbProTheme) -> Self {
        Self {
            title: title.into(),
            value: value.into(),
            trend: None,
            icon: None,
            theme,
        }
    }

    pub fn change(mut self, text: impl Into<Cow<'a, str>>, is_positive: bool) -> Self {
        self.trend = Some(MetricTrend::new(text, is_positive));
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn show(self, ui: &mut Ui) {
        Card::new(self.theme).show(ui, |ui| {
            ui.set_min_height(96.0);
            self.draw_metric_header(ui);

            ui.add_space(12.0);
            ui.label(
                RichText::new(self.value.as_ref())
                    .font(crate::DbProTheme::ui_medium_font(24.0))
                    .color(self.theme.text_primary),
            );

            if let Some(trend) = &self.trend {
                trend.draw(ui, &self.theme);
            }
        });
    }

    fn draw_metric_header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(self.title.as_ref())
                    .font(crate::DbProTheme::ui_medium_font(12.5))
                    .color(self.theme.text_tertiary),
            );
            if let Some(icon_glyph) = self.icon {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
                    ui.painter()
                        .rect_filled(rect, Rounding::same(8.0), self.theme.surface_2);
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        char::from(icon_glyph).to_string(),
                        FontId::new(14.0, FontFamily::Name("lucide".into())),
                        self.theme.text_secondary,
                    );
                });
            }
        });
    }
}

impl<'a> MetricTrend<'a> {
    pub fn draw(&self, ui: &mut Ui, theme: &DbProTheme) {
        ui.add_space(8.0);
        let (color, icon) = self.style(theme);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label(
                RichText::new(char::from(icon).to_string())
                    .font(FontId::new(14.0, FontFamily::Name("lucide".into())))
                    .color(color),
            );
            ui.label(RichText::new(format!(" {}", self.text)).size(12.0).color(color));
        });
    }
}
