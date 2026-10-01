use super::config::{
    CARD_FRAME_STROKE_WIDTH, CARD_HEADER_DESCRIPTION_SIZE, CARD_HEADER_TITLE_SIZE, FOOTER_SEPARATOR_SIZE,
    METRIC_CARD_MIN_HEIGHT, METRIC_ICON_BOX_RADIUS, METRIC_ICON_BOX_SIZE, METRIC_ICON_FONT_SIZE, METRIC_TITLE_SIZE,
    METRIC_TREND_TEXT_SIZE, METRIC_TREND_TOP_GAP, METRIC_VALUE_SIZE, METRIC_VALUE_TOP_GAP,
};
use super::handler::{footer_separator_segment, metric_trend_visual, trend_label_text};
use crate::tokens::{CARD_INNER_PAD, RADIUS_CARD, SPACE_MD, SPACE_SM, SPACE_XXS};
use crate::DbProTheme;
use egui::{FontFamily, FontId, Frame, Margin, RichText, Rounding, Stroke, Ui, Vec2, WidgetInfo, WidgetType};
use lucide_icons::Icon;
use std::borrow::Cow;

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
            stroke: Stroke::new(CARD_FRAME_STROKE_WIDTH, self.theme.border_subtle),
            inner_margin: Margin::same(CARD_INNER_PAD),
            rounding: Rounding::same(RADIUS_CARD),
            shadow: Default::default(),
            ..Default::default()
        }
    }

    pub fn show<R>(&self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        self.frame().show(ui, add_contents).inner
    }
}

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
                .size(CARD_HEADER_TITLE_SIZE)
                .strong()
                .color(theme.text_primary),
        );
        if let Some(desc) = description {
            ui.add_space(SPACE_XXS);
            ui.label(
                RichText::new(desc.as_ref())
                    .size(CARD_HEADER_DESCRIPTION_SIZE)
                    .color(theme.text_secondary),
            );
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
    let (sep_rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, FOOTER_SEPARATOR_SIZE), egui::Sense::hover());
    ui.painter().line_segment(
        footer_separator_segment(sep_rect),
        Stroke::new(FOOTER_SEPARATOR_SIZE, theme.border_subtle),
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
    pub direction: super::handler::MetricTrendDirection,
    pub tone: super::handler::MetricTrendTone,
}

impl<'a> MetricTrend<'a> {
    /// Creates a trend using the legacy boolean mapping retained by `MetricCard::change`.
    pub fn new(text: impl Into<Cow<'a, str>>, is_positive: bool) -> Self {
        let (direction, tone) = if is_positive {
            (
                super::handler::MetricTrendDirection::Up,
                super::handler::MetricTrendTone::Positive,
            )
        } else {
            (
                super::handler::MetricTrendDirection::Down,
                super::handler::MetricTrendTone::Negative,
            )
        };

        Self {
            text: text.into(),
            direction,
            tone,
        }
    }

    pub fn with_semantics(
        text: impl Into<Cow<'a, str>>,
        direction: super::handler::MetricTrendDirection,
        tone: super::handler::MetricTrendTone,
    ) -> Self {
        Self {
            text: text.into(),
            direction,
            tone,
        }
    }

    /// Preserves the legacy style tuple for existing callers; rendered rows use `visual` so
    /// `Unspecified` direction can omit its icon entirely.
    pub fn style(&self, theme: &DbProTheme) -> (egui::Color32, Icon) {
        let visual = metric_trend_visual(self.direction, self.tone, theme);
        (visual.color, visual.icon.unwrap_or(Icon::Minus))
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

    /// Legacy shorthand: `true` maps to `Up + Positive`, `false` to `Down + Negative`.
    pub fn change(mut self, text: impl Into<Cow<'a, str>>, is_positive: bool) -> Self {
        self.trend = Some(MetricTrend::new(text, is_positive));
        self
    }

    pub fn trend(
        mut self,
        text: impl Into<Cow<'a, str>>,
        direction: super::handler::MetricTrendDirection,
        tone: super::handler::MetricTrendTone,
    ) -> Self {
        self.trend = Some(MetricTrend::with_semantics(text, direction, tone));
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn show(self, ui: &mut Ui) {
        Card::new(self.theme).show(ui, |ui| {
            ui.set_min_height(METRIC_CARD_MIN_HEIGHT);
            self.draw_metric_header(ui);

            ui.add_space(METRIC_VALUE_TOP_GAP);
            ui.label(
                RichText::new(self.value.as_ref())
                    .font(crate::DbProTheme::ui_medium_font(METRIC_VALUE_SIZE))
                    .color(self.theme.text_primary),
            );

            if let Some(trend) = &self.trend {
                trend.draw(ui, &self.theme);
            }
        });
    }

    fn draw_metric_header(&self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(self.title.as_ref())
                    .font(crate::DbProTheme::ui_medium_font(METRIC_TITLE_SIZE))
                    .color(self.theme.text_secondary),
            );
            if let Some(icon_glyph) = self.icon {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(METRIC_ICON_BOX_SIZE, METRIC_ICON_BOX_SIZE),
                        egui::Sense::hover(),
                    );
                    ui.painter()
                        .rect_filled(rect, Rounding::same(METRIC_ICON_BOX_RADIUS), self.theme.surface_2);
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        char::from(icon_glyph).to_string(),
                        FontId::new(METRIC_ICON_FONT_SIZE, FontFamily::Name("lucide".into())),
                        self.theme.text_secondary,
                    );
                });
            }
        });
    }
}

impl<'a> MetricTrend<'a> {
    pub fn draw(&self, ui: &mut Ui, theme: &DbProTheme) {
        ui.add_space(METRIC_TREND_TOP_GAP);
        let visual = metric_trend_visual(self.direction, self.tone, theme);
        let row = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            if let Some(icon) = visual.icon {
                ui.label(
                    RichText::new(char::from(icon).to_string())
                        .font(FontId::new(METRIC_ICON_FONT_SIZE, FontFamily::Name("lucide".into())))
                        .color(visual.color),
                );
                ui.label(
                    RichText::new(trend_label_text(self.text.as_ref()))
                        .size(METRIC_TREND_TEXT_SIZE)
                        .color(visual.color),
                );
            } else {
                ui.label(
                    RichText::new(self.text.as_ref())
                        .size(METRIC_TREND_TEXT_SIZE)
                        .color(visual.color),
                );
            }
        });
        row.response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, self.text.as_ref()));
    }
}
