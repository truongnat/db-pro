use crate::DbProTheme;
use egui::{Align, Color32, Frame, Layout, Margin, Pos2, Rect, RichText, Rounding, Stroke, Ui, Vec2};
use lucide_icons::Icon;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableColumnAlign {
    Left,
    Center,
    Right,
}

pub struct TableColumn<'a> {
    pub title: &'a str,
    pub width: Option<f32>,
    pub align: TableColumnAlign,
    pub sortable: bool,
}

impl<'a> TableColumn<'a> {
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

pub(crate) fn draw_crisp_checkmark(painter: &egui::Painter, center: Pos2, color: Color32) {
    let p1 = Pos2::new(center.x - 3.8, center.y - 0.2);
    let p2 = Pos2::new(center.x - 0.9, center.y + 2.8);
    let p3 = Pos2::new(center.x + 3.8, center.y - 2.8);
    painter.add(egui::epaint::PathShape::line(vec![p1, p2, p3], Stroke::new(1.8, color)));
}

pub(crate) fn draw_crisp_minus(painter: &egui::Painter, center: Pos2, color: Color32) {
    let p1 = Pos2::new(center.x - 3.5, center.y);
    let p2 = Pos2::new(center.x + 3.5, center.y);
    painter.line_segment([p1, p2], Stroke::new(1.8, color));
}

pub struct Table<'a> {
    columns: &'a [TableColumn<'a>],
    theme: DbProTheme,
    selectable: bool,
    all_selected: bool,
    indeterminate: bool,
    sort_column: Option<usize>,
    sort_desc: bool,
    row_height: f32,
    show_vertical_grid: bool,
}

impl<'a> Table<'a> {
    pub fn new(columns: &'a [TableColumn<'a>], theme: DbProTheme) -> Self {
        Self {
            columns,
            theme,
            selectable: false,
            all_selected: false,
            indeterminate: false,
            sort_column: None,
            sort_desc: false,
            row_height: 44.0,
            show_vertical_grid: false,
        }
    }

    pub fn selectable(mut self, selectable: bool, all_selected: bool) -> Self {
        self.selectable = selectable;
        self.all_selected = all_selected;
        self
    }

    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
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

    pub fn vertical_grid(mut self, show: bool) -> Self {
        self.show_vertical_grid = show;
        self
    }

    /// Table layout algorithm: distributes widths cleanly across available space
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

        // Precompute absolute column x offsets relative to the start of column 0
        let mut col_x_offsets = Vec::with_capacity(self.columns.len());
        let mut current_offset = 0.0;
        for &w in &widths {
            col_x_offsets.push(current_offset);
            current_offset += w;
        }

        let header_h = 36.0;
        let total_rows_h = if row_count == 0 {
            80.0
        } else {
            row_count as f32 * self.row_height
        };
        let total_table_h = header_h + 1.0 + total_rows_h;

        // Allocate entire table frame rect to guarantee rigid coordinate space
        Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_default),
            rounding: Rounding::same(8.0),
            inner_margin: Margin::ZERO,
            ..Default::default()
        }
        .show(ui, |ui| {
            let table_min = ui.cursor().min;
            let table_w = ui.available_width();
            let col_start_x = table_min.x + checkbox_w;

            // ── 1. Header Background ──────────────────────────────────
            let header_rect = Rect::from_min_size(table_min, Vec2::new(table_w, header_h));
            ui.painter().rect_filled(
                header_rect,
                Rounding {
                    nw: 7.0,
                    ne: 7.0,
                    sw: 0.0,
                    se: 0.0,
                },
                self.theme.surface_hover.linear_multiply(0.4),
            );

            // Select All Checkbox
            if self.selectable {
                let cb_rect = Rect::from_min_size(table_min, Vec2::new(checkbox_w, header_h));
                let resp = ui
                    .interact(cb_rect, ui.id().with("select_all"), egui::Sense::click())
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                let center = cb_rect.center();
                let box_rect = Rect::from_center_size(center, Vec2::splat(15.0));

                if self.all_selected {
                    let fill = if resp.hovered() {
                        self.theme.accent.linear_multiply(0.9)
                    } else {
                        self.theme.accent
                    };
                    ui.painter().rect_filled(box_rect, Rounding::same(3.5), fill);
                    draw_crisp_checkmark(ui.painter(), center, Color32::WHITE);
                } else if self.indeterminate {
                    let fill = if resp.hovered() {
                        self.theme.accent.linear_multiply(0.9)
                    } else {
                        self.theme.accent
                    };
                    ui.painter().rect_filled(box_rect, Rounding::same(3.5), fill);
                    draw_crisp_minus(ui.painter(), center, Color32::WHITE);
                } else {
                    let border_color = if resp.hovered() {
                        self.theme.accent
                    } else {
                        self.theme.border_default
                    };
                    let fill = if resp.hovered() {
                        self.theme.surface_hover
                    } else {
                        self.theme.surface_editor
                    };
                    ui.painter().rect_filled(box_rect, Rounding::same(3.5), fill);
                    ui.painter()
                        .rect_stroke(box_rect, Rounding::same(3.5), Stroke::new(1.2, border_color));
                }

                if resp.clicked() {
                    on_toggle_all(!self.all_selected);
                }
            }

            // Header Column Cells (rigorously placed at exact X coordinates)
            for (col_idx, col) in self.columns.iter().enumerate() {
                let col_x = col_start_x + col_x_offsets[col_idx];
                let col_w = widths[col_idx];
                let col_rect = Rect::from_min_size(Pos2::new(col_x, table_min.y), Vec2::new(col_w, header_h));
                let resp = ui.interact(
                    col_rect,
                    ui.id().with(("header_col", col_idx)),
                    if col.sortable {
                        egui::Sense::click()
                    } else {
                        egui::Sense::hover()
                    },
                );

                if col.sortable && resp.hovered() {
                    ui.painter()
                        .rect_filled(col_rect, Rounding::ZERO, self.theme.surface_hover);
                }

                let is_sorted = self.sort_column == Some(col_idx);
                let text_color = if is_sorted {
                    self.theme.accent
                } else if resp.hovered() && col.sortable {
                    self.theme.text_primary
                } else {
                    self.theme.text_secondary
                };

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

                let inner_rect = col_rect.shrink2(Vec2::new(12.0, 0.0));
                let align_layout = match col.align {
                    TableColumnAlign::Left => Layout::left_to_right(Align::Center),
                    TableColumnAlign::Center => Layout::centered_and_justified(egui::Direction::LeftToRight),
                    TableColumnAlign::Right => Layout::right_to_left(Align::Center),
                };

                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                    ui.set_clip_rect(ui.clip_rect().intersect(inner_rect));
                    ui.with_layout(align_layout, |ui| {
                        // For right-aligned column headers, add sort icon first in right-to-left layout so it sits at far right
                        let show_sort = col.sortable && (is_sorted || resp.hovered());
                        if show_sort && col.align == TableColumnAlign::Right {
                            ui.label(
                                RichText::new(char::from(icon).to_string())
                                    .font(egui::FontId::new(10.0, egui::FontFamily::Name("lucide".into())))
                                    .color(icon_color),
                            );
                            ui.add_space(4.0);
                        }

                        ui.label(
                            RichText::new(col.title)
                                .font(crate::DbProTheme::ui_medium_font(13.0))
                                .color(text_color),
                        );

                        if show_sort && col.align != TableColumnAlign::Right {
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

            // ── Header Bottom Divider ─────────────────────────────────
            let header_divider_y = table_min.y + header_h;
            ui.painter().hline(
                table_min.x..=table_min.x + table_w,
                header_divider_y,
                Stroke::new(1.0, self.theme.border_default),
            );

            // ── 2. Data Rows (Rigorously placed at exact X coordinates) ─
            let body_start_y = header_divider_y + 1.0;

            if row_count == 0 {
                let empty_rect = Rect::from_min_size(Pos2::new(table_min.x, body_start_y), Vec2::new(table_w, 90.0));
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(empty_rect), |ui| {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        ui.add_space(20.0);
                        ui.label(
                            RichText::new(char::from(Icon::Inbox).to_string())
                                .font(egui::FontId::new(20.0, egui::FontFamily::Name("lucide".into())))
                                .color(self.theme.text_muted),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("No matching rows found.")
                                .size(13.0)
                                .color(self.theme.text_secondary),
                        );
                    });
                });
            } else {
                for row_idx in 0..row_count {
                    let row_y = body_start_y + (row_idx as f32 * self.row_height);
                    let row_rect =
                        Rect::from_min_size(Pos2::new(table_min.x, row_y), Vec2::new(table_w, self.row_height));

                    // Viewport culling (virtualization): skip offscreen rows to maintain 60/120 FPS at scale
                    if !ui.is_rect_visible(row_rect) {
                        continue;
                    }

                    let is_selected = is_row_selected(row_idx);
                    let row_resp = ui
                        .interact(row_rect, ui.id().with(("row", row_idx)), egui::Sense::click())
                        .on_hover_cursor(egui::CursorIcon::PointingHand);

                    let hover_t = crate::components::animation::hover_t(
                        ui.ctx(),
                        row_resp.id.with("row_hover"),
                        row_resp.hovered() && !is_selected,
                    );
                    let select_t =
                        crate::components::animation::hover_t(ui.ctx(), row_resp.id.with("row_sel"), is_selected);

                    // Alternating row stripes (Ant Design style)
                    if row_idx % 2 == 1 {
                        ui.painter().rect_filled(
                            row_rect,
                            Rounding::ZERO,
                            self.theme.surface_hover.linear_multiply(0.25),
                        );
                    }

                    if select_t > 0.001 {
                        ui.painter().rect_filled(
                            row_rect,
                            Rounding::ZERO,
                            crate::components::animation::lerp_color(
                                Color32::TRANSPARENT,
                                self.theme.accent_soft,
                                select_t,
                            ),
                        );
                    } else if hover_t > 0.001 {
                        ui.painter().rect_filled(
                            row_rect,
                            Rounding::ZERO,
                            crate::components::animation::lerp_color(
                                Color32::TRANSPARENT,
                                self.theme.surface_hover,
                                hover_t,
                            ),
                        );
                    }

                    // Checkbox cell
                    if self.selectable {
                        let cb_rect =
                            Rect::from_min_size(Pos2::new(table_min.x, row_y), Vec2::new(checkbox_w, self.row_height));
                        let cb_resp = ui
                            .interact(cb_rect, ui.id().with(("row_cb", row_idx)), egui::Sense::click())
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                        let center = cb_rect.center();
                        let box_rect = Rect::from_center_size(center, Vec2::splat(15.0));

                        if is_selected {
                            let fill = if cb_resp.hovered() {
                                self.theme.accent.linear_multiply(0.9)
                            } else {
                                self.theme.accent
                            };
                            ui.painter().rect_filled(box_rect, Rounding::same(3.5), fill);
                            draw_crisp_checkmark(ui.painter(), center, Color32::WHITE);
                        } else {
                            let border_color = if cb_resp.hovered() {
                                self.theme.accent
                            } else {
                                self.theme.border_default
                            };
                            let fill = if cb_resp.hovered() {
                                self.theme.surface_hover
                            } else {
                                self.theme.surface_editor
                            };
                            ui.painter().rect_filled(box_rect, Rounding::same(3.5), fill);
                            ui.painter()
                                .rect_stroke(box_rect, Rounding::same(3.5), Stroke::new(1.2, border_color));
                        }

                        if cb_resp.clicked() {
                            on_toggle_row(row_idx);
                        }
                    }

                    // Data cells (EXACT same X coordinates and padding as Header!)
                    for (col_idx, col) in self.columns.iter().enumerate() {
                        let col_x = col_start_x + col_x_offsets[col_idx];
                        let col_w = widths[col_idx];
                        let cell_rect = Rect::from_min_size(Pos2::new(col_x, row_y), Vec2::new(col_w, self.row_height));
                        let inner_rect = cell_rect.shrink2(Vec2::new(12.0, 0.0));

                        let align_layout = match col.align {
                            TableColumnAlign::Left => Layout::left_to_right(Align::Center),
                            TableColumnAlign::Center => Layout::centered_and_justified(egui::Direction::LeftToRight),
                            TableColumnAlign::Right => Layout::right_to_left(Align::Center),
                        };

                        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                            ui.set_clip_rect(ui.clip_rect().intersect(inner_rect));
                            ui.with_layout(align_layout, |ui| {
                                render_cell(ui, row_idx, col_idx);
                            });
                        });
                    }

                    // Row horizontal divider
                    if row_idx < row_count - 1 {
                        let div_y = row_y + self.row_height;
                        ui.painter().hline(
                            table_min.x..=table_min.x + table_w,
                            div_y,
                            Stroke::new(1.0, self.theme.border_subtle),
                        );
                    }
                }
            }

            // ── 3. Vertical Column Gridlines (Crisp vertical alignment) ──
            if self.show_vertical_grid {
                let grid_bottom = body_start_y + total_rows_h;

                // Line between checkbox and first column
                if self.selectable {
                    ui.painter().vline(
                        col_start_x,
                        table_min.y..=grid_bottom,
                        Stroke::new(1.0, self.theme.border_subtle),
                    );
                }

                // Lines between each column
                for &offset in col_x_offsets.iter().skip(1).take(self.columns.len().saturating_sub(1)) {
                    let col_x = col_start_x + offset;
                    ui.painter().vline(
                        col_x,
                        table_min.y..=grid_bottom,
                        Stroke::new(1.0, self.theme.border_subtle),
                    );
                }
            }

            // Advance cursor past the entire table height
            ui.advance_cursor_after_rect(Rect::from_min_size(table_min, Vec2::new(table_w, total_table_h)));
        });
    }
}
