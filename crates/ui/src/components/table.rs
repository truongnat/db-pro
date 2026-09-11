use crate::DbProTheme;
use egui::{Frame, Margin, Rect, RichText, Rounding, Stroke, Ui, Vec2};

pub struct ShadcnTableColumn<'a> {
    pub title: &'a str,
    pub width: f32,
}

pub struct ShadcnTable<'a> {
    columns: &'a [ShadcnTableColumn<'a>],
    theme: DbProTheme,
}

impl<'a> ShadcnTable<'a> {
    pub fn new(columns: &'a [ShadcnTableColumn<'a>], theme: DbProTheme) -> Self {
        Self { columns, theme }
    }

    pub fn show<F>(self, ui: &mut Ui, row_count: usize, mut render_cell: F)
    where
        F: FnMut(&mut Ui, usize, usize),
    {
        Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_default),
            rounding: Rounding::same(8.0),
            inner_margin: Margin::ZERO,
            ..Default::default()
        }
        .show(ui, |ui| {
            // Header
            Frame {
                fill: self.theme.surface_hover,
                inner_margin: Margin::symmetric(12.0, 8.0),
                rounding: Rounding {
                    nw: 8.0,
                    ne: 8.0,
                    sw: 0.0,
                    se: 0.0,
                },
                stroke: Stroke::NONE,
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for col in self.columns {
                        ui.allocate_ui_with_layout(
                            Vec2::new(col.width, 18.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.label(
                                    RichText::new(col.title)
                                        .size(11.5)
                                        .strong()
                                        .color(self.theme.text_secondary),
                                );
                            },
                        );
                    }
                });
            });

            // Divider
            let (divider_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
            ui.painter().hline(
                divider_rect.x_range(),
                divider_rect.center().y,
                Stroke::new(1.0, self.theme.border_subtle),
            );

            // Rows
            for row_idx in 0..row_count {
                let row_bg = if row_idx % 2 == 1 {
                    self.theme.surface_hover.linear_multiply(0.4)
                } else {
                    egui::Color32::TRANSPARENT
                };

                let row_frame = Frame {
                    fill: row_bg,
                    inner_margin: Margin::symmetric(12.0, 8.0),
                    stroke: Stroke::NONE,
                    rounding: if row_idx == row_count - 1 {
                        Rounding {
                            nw: 0.0,
                            ne: 0.0,
                            sw: 8.0,
                            se: 8.0,
                        }
                    } else {
                        Rounding::ZERO
                    },
                    ..Default::default()
                };

                let row_resp = row_frame
                    .show(ui, |ui| {
                        ui.horizontal_centered(|ui| {
                            for (col_idx, col) in self.columns.iter().enumerate() {
                                ui.allocate_ui_with_layout(
                                    Vec2::new(col.width, 20.0),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        render_cell(ui, row_idx, col_idx);
                                    },
                                );
                            }
                        });
                    })
                    .response;

                if row_resp.hovered() {
                    ui.painter().rect_filled(
                        Rect::from_min_size(row_resp.rect.min, row_resp.rect.size()),
                        Rounding::ZERO,
                        self.theme.surface_hover.linear_multiply(0.5),
                    );
                }

                if row_idx < row_count - 1 {
                    let (line_rect, _) =
                        ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
                    ui.painter().hline(
                        line_rect.x_range(),
                        line_rect.center().y,
                        Stroke::new(1.0, self.theme.border_subtle),
                    );
                }
            }
        });
    }
}
