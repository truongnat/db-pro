use crate::components::table::config::{
    CHECKBOX_SIZE, DIVIDER_STROKE_WIDTH, EMPTY_BODY_RECT_HEIGHT, EMPTY_BODY_TOP_SPACE, EMPTY_ICON_GAP, EMPTY_ICON_SIZE,
    EMPTY_TEXT_SIZE, HEADER_INNER_RADIUS, HEADER_TEXT_SIZE, INNER_PADDING_X, SORT_ICON_GAP, SORT_ICON_SIZE,
    TABLE_CORNER_RADIUS,
};
use crate::components::table::handler::{build_column_layout, select_all_state, SelectAllState, TableGeometry};
use crate::components::table::{draw_crisp_checkmark, draw_crisp_minus, Table, TableColumnAlign};
use egui::{Align, Color32, Frame, Layout, Margin, Pos2, Rect, RichText, Rounding, Stroke, Ui, Vec2};
use lucide_icons::Icon;

impl<'a> Table<'a> {
    // allow: table widget receives distinct generic callbacks for each interaction (row selection,
    // sort, toggle) — bundling into an options struct would complicate generic lifetimes across all egui call sites.
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
        let layout = build_column_layout(self.columns, available_w, self.selectable);
        let geometry = TableGeometry::new(row_count, self.row_height);
        let widths = &layout.widths;
        let col_x_offsets = &layout.offsets;
        let checkbox_w = layout.checkbox_width;
        let total_cols_w = layout.total_width;
        let header_h = geometry.header_height;
        let total_rows_h = geometry.rows_height;
        let total_table_h = geometry.total_height;

        // Allocate entire table frame rect to guarantee rigid coordinate space
        Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(
                crate::components::table::config::DIVIDER_STROKE_WIDTH,
                self.theme.border_default,
            ),
            rounding: Rounding::same(TABLE_CORNER_RADIUS),
            inner_margin: Margin::ZERO,
            ..Default::default()
        }
        .show(ui, |ui| {
            let table_min = ui.cursor().min;
            let table_w = ui.available_width().max(total_cols_w);
            let col_start_x = table_min.x + checkbox_w;

            // ── 1. Header Background ──────────────────────────────────
            let header_rect = Rect::from_min_size(table_min, Vec2::new(table_w, header_h));
            ui.painter().rect_filled(
                header_rect,
                Rounding {
                    nw: HEADER_INNER_RADIUS,
                    ne: HEADER_INNER_RADIUS,
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
                let box_rect = Rect::from_center_size(center, Vec2::splat(CHECKBOX_SIZE));

                if matches!(
                    select_all_state(self.all_selected, self.indeterminate),
                    SelectAllState::Checked
                ) {
                    let fill = if resp.hovered() {
                        self.theme.accent.linear_multiply(0.9)
                    } else {
                        self.theme.accent
                    };
                    ui.painter().rect_filled(
                        box_rect,
                        Rounding::same(crate::components::table::config::CHECKBOX_CORNER_RADIUS),
                        fill,
                    );
                    draw_crisp_checkmark(ui.painter(), center, Color32::WHITE);
                } else if matches!(
                    select_all_state(self.all_selected, self.indeterminate),
                    SelectAllState::Indeterminate
                ) {
                    let fill = if resp.hovered() {
                        self.theme.accent.linear_multiply(0.9)
                    } else {
                        self.theme.accent
                    };
                    ui.painter().rect_filled(
                        box_rect,
                        Rounding::same(crate::components::table::config::CHECKBOX_CORNER_RADIUS),
                        fill,
                    );
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
                    ui.painter().rect_filled(
                        box_rect,
                        Rounding::same(crate::components::table::config::CHECKBOX_CORNER_RADIUS),
                        fill,
                    );
                    ui.painter().rect_stroke(
                        box_rect,
                        Rounding::same(crate::components::table::config::CHECKBOX_CORNER_RADIUS),
                        Stroke::new(crate::components::table::config::CHECKBOX_STROKE_WIDTH, border_color),
                    );
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
                    self.theme
                        .text_muted
                        .linear_multiply(crate::components::table::config::SORT_ICON_FADE)
                };

                let inner_rect = col_rect.shrink2(Vec2::new(INNER_PADDING_X, 0.0));
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
                                    .font(egui::FontId::new(
                                        SORT_ICON_SIZE,
                                        egui::FontFamily::Name("lucide".into()),
                                    ))
                                    .color(icon_color),
                            );
                            ui.add_space(SORT_ICON_GAP);
                        }

                        ui.label(
                            RichText::new(col.title)
                                .font(crate::DbProTheme::ui_medium_font(HEADER_TEXT_SIZE))
                                .color(text_color),
                        );

                        if show_sort && col.align != TableColumnAlign::Right {
                            ui.add_space(SORT_ICON_GAP);
                            ui.label(
                                RichText::new(char::from(icon).to_string())
                                    .font(egui::FontId::new(
                                        SORT_ICON_SIZE,
                                        egui::FontFamily::Name("lucide".into()),
                                    ))
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
                Stroke::new(DIVIDER_STROKE_WIDTH, self.theme.border_default),
            );

            // ── 2. Data Rows (Rigorously placed at exact X coordinates) ─
            let body_start_y = header_divider_y + DIVIDER_STROKE_WIDTH;

            if row_count == 0 {
                let empty_rect = Rect::from_min_size(
                    Pos2::new(table_min.x, body_start_y),
                    Vec2::new(table_w, EMPTY_BODY_RECT_HEIGHT),
                );
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(empty_rect), |ui| {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        ui.add_space(EMPTY_BODY_TOP_SPACE);
                        ui.label(
                            RichText::new(char::from(Icon::Inbox).to_string())
                                .font(egui::FontId::new(
                                    EMPTY_ICON_SIZE,
                                    egui::FontFamily::Name("lucide".into()),
                                ))
                                .color(self.theme.text_muted),
                        );
                        ui.add_space(EMPTY_ICON_GAP);
                        ui.label(
                            RichText::new("No matching rows found.")
                                .size(EMPTY_TEXT_SIZE)
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
                        let box_rect = Rect::from_center_size(center, Vec2::splat(CHECKBOX_SIZE));

                        if is_selected {
                            let fill = if cb_resp.hovered() {
                                self.theme.accent.linear_multiply(0.9)
                            } else {
                                self.theme.accent
                            };
                            ui.painter().rect_filled(
                                box_rect,
                                Rounding::same(crate::components::table::config::CHECKBOX_CORNER_RADIUS),
                                fill,
                            );
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
                            ui.painter().rect_filled(
                                box_rect,
                                Rounding::same(crate::components::table::config::CHECKBOX_CORNER_RADIUS),
                                fill,
                            );
                            ui.painter().rect_stroke(
                                box_rect,
                                Rounding::same(crate::components::table::config::CHECKBOX_CORNER_RADIUS),
                                Stroke::new(crate::components::table::config::CHECKBOX_STROKE_WIDTH, border_color),
                            );
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
                        let inner_rect = cell_rect.shrink2(Vec2::new(INNER_PADDING_X, 0.0));

                        let align_layout = match col.align {
                            TableColumnAlign::Left => Layout::left_to_right(Align::Center),
                            TableColumnAlign::Center => Layout::centered_and_justified(egui::Direction::LeftToRight),
                            TableColumnAlign::Right => Layout::right_to_left(Align::Center),
                        };

                        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(inner_rect), |ui| {
                            // Allow widget strokes into cell padding without leaking into neighboring cells.
                            ui.set_clip_rect(ui.clip_rect().intersect(inner_rect.expand(1.0).intersect(cell_rect)));
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
                            Stroke::new(DIVIDER_STROKE_WIDTH, self.theme.border_subtle),
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
                        Stroke::new(DIVIDER_STROKE_WIDTH, self.theme.border_subtle),
                    );
                }

                // Lines between each column
                for &offset in col_x_offsets.iter().skip(1).take(self.columns.len().saturating_sub(1)) {
                    let col_x = col_start_x + offset;
                    ui.painter().vline(
                        col_x,
                        table_min.y..=grid_bottom,
                        Stroke::new(DIVIDER_STROKE_WIDTH, self.theme.border_subtle),
                    );
                }
            }

            // Advance cursor past the entire table height
            ui.advance_cursor_after_rect(Rect::from_min_size(table_min, Vec2::new(table_w, total_table_h)));
        });
    }
}
