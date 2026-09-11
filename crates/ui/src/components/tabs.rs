use crate::DbProTheme;
use egui::{Button, Frame, Margin, Rect, RichText, Rounding, Stroke, Ui, Vec2};

pub struct SegmentedTabs<'a> {
    selected: &'a mut usize,
    tabs: &'a [&'a str],
    theme: DbProTheme,
}

impl<'a> SegmentedTabs<'a> {
    pub fn new(selected: &'a mut usize, tabs: &'a [&'a str], theme: DbProTheme) -> Self {
        Self { selected, tabs, theme }
    }

    pub fn show(self, ui: &mut Ui) {
        Frame {
            fill: self.theme.surface_hover,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            inner_margin: Margin::symmetric(3.0, 3.0),
            rounding: Rounding::same(7.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for (idx, tab_name) in self.tabs.iter().enumerate() {
                    let is_active = idx == *self.selected;

                    let (fill, text_color, shadow) = if is_active {
                        (
                            self.theme.surface_panel,
                            self.theme.text_primary,
                            egui::epaint::Shadow {
                                offset: egui::vec2(0.0, 1.0),
                                blur: 3.0,
                                spread: 0.0,
                                color: egui::Color32::from_black_alpha(15),
                            },
                        )
                    } else {
                        (
                            egui::Color32::TRANSPARENT,
                            self.theme.text_secondary,
                            egui::epaint::Shadow::NONE,
                        )
                    };

                    let frame = Frame {
                        fill,
                        inner_margin: Margin::symmetric(10.0, 4.0),
                        rounding: Rounding::same(5.0),
                        shadow,
                        ..Default::default()
                    };

                    let resp = frame
                        .show(ui, |ui| {
                            let text = RichText::new(*tab_name).size(12.5).color(text_color);
                            ui.add(
                                Button::new(if is_active { text.strong() } else { text })
                                    .frame(false)
                                    .min_size(Vec2::new(0.0, 20.0)),
                            )
                        })
                        .inner;

                    if resp.clicked() {
                        *self.selected = idx;
                    }
                }
            });
        });
    }
}

pub struct UnderlineTabs<'a> {
    selected: &'a mut usize,
    tabs: &'a [&'a str],
    theme: DbProTheme,
}

impl<'a> UnderlineTabs<'a> {
    pub fn new(selected: &'a mut usize, tabs: &'a [&'a str], theme: DbProTheme) -> Self {
        Self { selected, tabs, theme }
    }

    pub fn show(self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for (idx, tab_name) in self.tabs.iter().enumerate() {
                let is_active = idx == *self.selected;
                let text_color = if is_active {
                    self.theme.text_primary
                } else {
                    self.theme.text_secondary
                };

                let text = RichText::new(*tab_name).size(13.0).color(text_color);
                let resp = ui.add(
                    Button::new(if is_active { text.strong() } else { text })
                        .frame(false)
                        .min_size(Vec2::new(0.0, 26.0)),
                );

                if is_active {
                    let underline_rect = Rect::from_min_max(
                        egui::pos2(resp.rect.left(), resp.rect.bottom() - 2.0),
                        egui::pos2(resp.rect.right(), resp.rect.bottom()),
                    );
                    ui.painter()
                        .rect_filled(underline_rect, Rounding::same(1.0), self.theme.accent);
                }

                if resp.clicked() {
                    *self.selected = idx;
                }

                ui.add_space(8.0);
            }
        });
    }
}
