use crate::DbProTheme;
use egui::{Align, Color32, Frame, Layout, Margin, Rect, RichText, Rounding, Stroke, Ui, Vec2};
use lucide_icons::Icon;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableColumnAlign {
    Left,
    Center,
    Right,
}

pub struct ShadcnTableColumn<'a> {
    pub title: &'a str,
    pub width: Option<f32>,
    pub align: TableColumnAlign,
    pub sortable: bool,
}

impl<'a> ShadcnTableColumn<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            width: None,
            align: TableColumnAlign::Left,
            sortable: false,
        }
    }

    pub fn fixed(title: &'a str, width: f32) -> Self {
        Self {
            title,
            width: Some(width),
            align: TableColumnAlign::Left,
            sortable: false,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn align(mut self, align: TableColumnAlign) -> Self {
        self.align = align;
        self
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }
}

pub struct ShadcnTable<'a> {
    columns: &'a [ShadcnTableColumn<'a>],
    theme: DbProTheme,
    selectable: bool,
    all_selected: bool,
    sort_column: Option<usize>,
    sort_desc: bool,
    row_height: f32,
}

impl<'a> ShadcnTable<'a> {
    pub fn new(columns: &'a [ShadcnTableColumn<'a>], theme: DbProTheme) -> Self {
        Self {
            columns,
            theme,
            selectable: false,
            all_selected: false,
            sort_column: None,
            sort_desc: false,
            row_height: 38.0,
        }
    }

    pub fn selectable(mut self, selectable: bool, all_selected: bool) -> Self {
        self.selectable = selectable;
        self.all_selected = all_selected;
        self
    }

    pub fn sort(mut self, sort_column: Option<usize>, sort_desc: bool) -> Self {
        self.sort_column = sort_column;
        self.sort_desc = sort_desc;
        self
    }

    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height;
        self
    }

    /// Calculate concrete column widths to fill available width smoothly
    fn compute_column_widths(&self, available_width: f32) -> Vec<f32> {
        let checkbox_width = if self.selectable { 42.0 } else { 0.0 };
        let net_width = (available_width - checkbox_width - 2.0).max(100.0);

        let mut fixed_sum = 0.0;
        let mut flex_count = 0;

        for col in self.columns {
            if let Some(w) = col.width {
                fixed_sum += w;
            } else {
                flex_count += 1;
            }
        }

        if flex_count > 0 {
            let flex_width = ((net_width - fixed_sum) / flex_count as f32).max(80.0);
            self.columns.iter().map(|col| col.width.unwrap_or(flex_width)).collect()
        } else if fixed_sum > 0.0 && fixed_sum < net_width {
            let ratio = net_width / fixed_sum;
            self.columns
                .iter()
                .map(|col| col.width.unwrap_or(100.0) * ratio)
                .collect()
        } else {
            self.columns.iter().map(|col| col.width.unwrap_or(120.0)).collect()
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn show<F>(
        self,
        ui: &mut Ui,
        row_count: usize,
        is_row_selected: impl Fn(usize) -> bool,
        mut on_toggle_all: impl FnMut(bool),
        mut on_toggle_row: impl FnMut(usize),
        mut on_sort: impl FnMut(usize),
        mut render_cell: F,
    ) where
        F: FnMut(&mut Ui, usize, usize),
    {
        let available_w = ui.available_width();
        let widths = self.compute_column_widths(available_w);
        let checkbox_w = if self.selectable { 42.0 } else { 0.0 };

        Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_default),
            rounding: Rounding::same(8.0),
            inner_margin: Margin::ZERO,
            ..Default::default()
        }
        .show(ui, |ui| {
            // ── Table Header ──────────────────────────────────────────
            Frame {
                fill: self.theme.surface_hover.linear_multiply(0.6),
                inner_margin: Margin::symmetric(0.0, 0.0),
                rounding: Rounding {
                    nw: 7.0,
                    ne: 7.0,
                    sw: 0.0,
                    se: 0.0,
                },
                stroke: Stroke::NONE,
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;

                    // Select All Checkbox
                    if self.selectable {
                        let (rect, resp) = ui.allocate_exact_size(Vec2::new(checkbox_w, 36.0), egui::Sense::click());
                        let center = rect.center();
                        let box_rect = Rect::from_center_size(center, Vec2::splat(15.0));

                        if self.all_selected {
                            ui.painter()
                                .rect_filled(box_rect, Rounding::same(3.0), self.theme.accent);
                            ui.painter().text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                "✓",
                                egui::FontId::proportional(11.0),
                                Color32::WHITE,
                            );
                        } else {
                            let border_color = if resp.hovered() {
                                self.theme.accent
                            } else {
                                self.theme.border_default
                            };
                            ui.painter()
                                .rect_stroke(box_rect, Rounding::same(3.0), Stroke::new(1.0, border_color));
                            ui.painter().rect_filled(
                                box_rect.shrink(1.0),
                                Rounding::same(2.0),
                                self.theme.surface_editor,
                            );
                        }

                        if resp.clicked() {
                            on_toggle_all(!self.all_selected);
                        }
                    }

                    // Column Headers
                    for (col_idx, col) in self.columns.iter().enumerate() {
                        let col_w = widths[col_idx];
                        let (rect, resp) = ui.allocate_exact_size(Vec2::new(col_w, 36.0), egui::Sense::click());

                        if col.sortable && resp.hovered() {
                            ui.painter().rect_filled(rect, Rounding::ZERO, self.theme.surface_hover);
                        }

                        let align_layout = match col.align {
                            TableColumnAlign::Left => Layout::left_to_right(Align::Center),
                            TableColumnAlign::Center => Layout::centered_and_justified(egui::Direction::LeftToRight),
                            TableColumnAlign::Right => Layout::right_to_left(Align::Center),
                        };

                        let header_rect = rect.shrink2(Vec2::new(12.0, 0.0));
                        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(header_rect), |ui| {
                            ui.with_layout(align_layout, |ui| {
                                let is_sorted = self.sort_column == Some(col_idx);
                                let icon = if is_sorted {
                                    if self.sort_desc {
                                        Icon::ArrowDown
                                    } else {
                                        Icon::ArrowUp
                                    }
                                } else {
                                    Icon::ArrowUpDown
                                };

                                let icon_color = if is_sorted {
                                    self.theme.accent
                                } else {
                                    self.theme.text_muted.linear_multiply(0.5)
                                };

                                let text_color = if is_sorted {
                                    self.theme.accent
                                } else if resp.hovered() && col.sortable {
                                    self.theme.text_primary
                                } else {
                                    self.theme.text_secondary
                                };

                                if col.sortable && col.align == TableColumnAlign::Right {
                                    ui.label(
                                        RichText::new(char::from(icon).to_string())
                                            .font(egui::FontId::new(10.0, egui::FontFamily::Name("lucide".into())))
                                            .color(icon_color),
                                    );
                                    ui.add_space(4.0);
                                }

                                ui.label(
                                    RichText::new(col.title.to_uppercase())
                                        .size(11.0)
                                        .strong()
                                        .color(text_color),
                                );

                                if col.sortable && col.align != TableColumnAlign::Right {
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(char::from(icon).to_string())
                                            .font(egui::FontId::new(10.0, egui::FontFamily::Name("lucide".into())))
                                            .color(icon_color),
                                    );
                                }
                            });
                        });

                        if col.sortable && resp.clicked() {
                            on_sort(col_idx);
                        }
                    }
                });
            });

            // ── Header Divider ──────────────────────────────────────────
            let (divider_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
            ui.painter().hline(
                divider_rect.x_range(),
                divider_rect.center().y,
                Stroke::new(1.0, self.theme.border_default),
            );

            // ── Rows ────────────────────────────────────────────────────
            if row_count == 0 {
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), 80.0),
                    Layout::centered_and_justified(egui::Direction::TopDown),
                    |ui| {
                        ui.label(
                            RichText::new("No matching rows found.")
                                .size(13.0)
                                .color(self.theme.text_muted),
                        );
                    },
                );
            } else {
                for row_idx in 0..row_count {
                    let is_selected = is_row_selected(row_idx);
                    let row_h = self.row_height;

                    let bg_color = if is_selected {
                        self.theme.accent_soft
                    } else if row_idx % 2 == 1 {
                        self.theme.surface_hover.linear_multiply(0.25)
                    } else {
                        Color32::TRANSPARENT
                    };

                    let (row_rect, row_resp) =
                        ui.allocate_exact_size(Vec2::new(ui.available_width(), row_h), egui::Sense::click());

                    // Row background paint
                    if bg_color != Color32::TRANSPARENT {
                        ui.painter().rect_filled(row_rect, Rounding::ZERO, bg_color);
                    }

                    // Hover feedback
                    if row_resp.hovered() && !is_selected {
                        ui.painter().rect_filled(
                            row_rect,
                            Rounding::ZERO,
                            self.theme.surface_hover.linear_multiply(0.6),
                        );
                    }

                    // Active left indicator when selected
                    if is_selected {
                        let ind_rect = Rect::from_min_size(row_rect.min, Vec2::new(3.0, row_rect.height()));
                        ui.painter().rect_filled(ind_rect, Rounding::ZERO, self.theme.accent);
                    }

                    // Paint cell content within row_rect
                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(row_rect), |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::ZERO;

                            // Checkbox cell
                            if self.selectable {
                                let (cb_rect, cb_resp) =
                                    ui.allocate_exact_size(Vec2::new(checkbox_w, row_h), egui::Sense::click());
                                let center = cb_rect.center();
                                let box_rect = Rect::from_center_size(center, Vec2::splat(15.0));

                                if is_selected {
                                    ui.painter()
                                        .rect_filled(box_rect, Rounding::same(3.0), self.theme.accent);
                                    ui.painter().text(
                                        center,
                                        egui::Align2::CENTER_CENTER,
                                        "✓",
                                        egui::FontId::proportional(11.0),
                                        Color32::WHITE,
                                    );
                                } else {
                                    let border_color = if cb_resp.hovered() {
                                        self.theme.accent
                                    } else {
                                        self.theme.border_default
                                    };
                                    ui.painter().rect_stroke(
                                        box_rect,
                                        Rounding::same(3.0),
                                        Stroke::new(1.0, border_color),
                                    );
                                    ui.painter().rect_filled(
                                        box_rect.shrink(1.0),
                                        Rounding::same(2.0),
                                        self.theme.surface_editor,
                                    );
                                }

                                if cb_resp.clicked() {
                                    on_toggle_row(row_idx);
                                }
                            }

                            // Data cells
                            for (col_idx, col) in self.columns.iter().enumerate() {
                                let col_w = widths[col_idx];
                                let (cell_rect, _) =
                                    ui.allocate_exact_size(Vec2::new(col_w, row_h), egui::Sense::hover());

                                let cell_inner = cell_rect.shrink2(Vec2::new(12.0, 0.0));
                                let align_layout = match col.align {
                                    TableColumnAlign::Left => Layout::left_to_right(Align::Center),
                                    TableColumnAlign::Center => {
                                        Layout::centered_and_justified(egui::Direction::LeftToRight)
                                    }
                                    TableColumnAlign::Right => Layout::right_to_left(Align::Center),
                                };

                                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(cell_inner), |ui| {
                                    ui.with_layout(align_layout, |ui| {
                                        render_cell(ui, row_idx, col_idx);
                                    });
                                });
                            }
                        });
                    });

                    // Row divider (except last row)
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
            }
        });
    }
}
