//! Result-grid column header, menus, and resize/reorder.
use super::*;
use egui::{Align2, Pos2, Rect, Rounding, Stroke, Vec2};

impl DbProApp {
    pub(super) fn draw_grid_header(
        &mut self,
        ui: &mut egui::Ui,
        result: &UiQueryResult,
        indexes: &[usize],
        widths: &[f32],
        order: &[usize],
    ) {
        let (mut move_left_req, mut move_right_req, mut reset_order_req, mut reset_widths_req) =
            (None, None, false, false);
        let (mut hide_column_req, mut show_columns_req, mut reset_layout_req, mut auto_size_req) =
            (None, false, false, None);
        let mut add_filter_req = None;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let (gutter_rect, _) = ui.allocate_exact_size(egui::vec2(GRID_ROW_NUMBER_WIDTH, 34.0), Sense::hover());
            ui.painter()
                .rect_filled(gutter_rect, Rounding::ZERO, self.theme.surface_panel);
            ui.painter().hline(
                gutter_rect.x_range(),
                gutter_rect.bottom(),
                Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.7)),
            );
            ui.painter().vline(
                gutter_rect.right(),
                gutter_rect.y_range(),
                Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.7)),
            );
            ui.painter().text(
                gutter_rect.center(),
                Align2::CENTER_CENTER,
                "#",
                FontId::monospace(11.5),
                self.theme.text_muted,
            );

            for (visual_idx, &col_idx) in order.iter().enumerate() {
                let Some(column) = result.columns.get(col_idx) else {
                    continue;
                };
                let width = widths.get(col_idx).copied().unwrap_or(180.0);
                let (col_rect, col_resp) = ui.allocate_exact_size(egui::vec2(width, 34.0), Sense::click());

                // Check PK / FK indicators
                let is_pk = self
                    .table
                    .state
                    .table_info
                    .as_ref()
                    .is_some_and(|info| info.columns.iter().any(|c| c.name == column.name && c.is_primary_key));
                let is_fk = self.table.state.table_info.as_ref().is_some_and(|info| {
                    info.foreign_keys
                        .iter()
                        .any(|fk| fk.from_columns.iter().any(|col| col == &column.name))
                });

                // Resize divider on the right edge (4px grab target)
                let divider_rect = Rect::from_min_max(
                    Pos2::new(col_rect.right() - 3.0, col_rect.top()),
                    Pos2::new(col_rect.right() + 3.0, col_rect.bottom()),
                );
                let divider_id = ui.id().with(("grid_col_resize", col_idx));
                let divider = ui.interact(divider_rect, divider_id, Sense::drag());
                let is_resizing = divider.hovered() || divider.dragged();
                if is_resizing {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }

                let col_hovered = col_resp.hovered() && !is_resizing;
                let bg_fill = if col_hovered {
                    self.theme.surface_hover.linear_multiply(0.4)
                } else {
                    self.theme.surface_panel
                };
                ui.painter().rect_filled(col_rect, Rounding::ZERO, bg_fill);

                // Bottom and right border lines
                ui.painter().hline(
                    col_rect.x_range(),
                    col_rect.bottom(),
                    Stroke::new(1.0, self.theme.border_subtle.linear_multiply(0.7)),
                );
                ui.painter().vline(
                    col_rect.right(),
                    col_rect.y_range(),
                    Stroke::new(
                        if divider.dragged() { 2.0 } else { 1.0 },
                        if is_resizing {
                            self.theme.accent
                        } else {
                            self.theme.border_subtle.linear_multiply(0.7)
                        },
                    ),
                );

                // Column title + data type + sort icon
                let table_sort = (self.workspace.active_tab == WorkspaceTab::Table
                    && self.table.state.table_view == TableView::Data)
                    .then(|| {
                        self.table
                            .data_query
                            .sorts
                            .iter()
                            .find(|sort| sort.column == column.name)
                    })
                    .flatten();
                let table_sort_priority = table_sort.and_then(|_| {
                    self.table
                        .data_query
                        .sorts
                        .iter()
                        .position(|sort| sort.column == column.name)
                        .map(|position| position + 1)
                });
                let table_sort_active = table_sort.is_some();
                let sort_active = table_sort_active || self.table.data.grid_sort_column == Some(col_idx);
                let sort_desc = if let Some(sort) = table_sort {
                    sort.descending
                } else {
                    self.table.data.grid_sort_desc
                };
                let sort_marker = if let Some(priority) = table_sort_priority {
                    format!(" {}{}", if sort_desc { "↓" } else { "↑" }, priority)
                } else if sort_active {
                    format!(" {}", if sort_desc { "↓" } else { "↑" })
                } else {
                    String::new()
                };

                let header_text_rect = col_rect.shrink2(egui::vec2(8.0, 4.0));
                let painter = ui.painter().with_clip_rect(header_text_rect);

                let mut text_x = header_text_rect.left();

                // Draw PK / FK badges
                if is_pk {
                    let pk_galley = painter.layout_no_wrap(
                        "PK".to_owned(),
                        FontId::new(9.5, egui::FontFamily::Proportional),
                        self.theme.warning,
                    );
                    let badge_rect = Rect::from_min_size(
                        Pos2::new(text_x, header_text_rect.center().y - 7.0),
                        Vec2::new(pk_galley.size().x + 6.0, 14.0),
                    );
                    painter.rect_filled(
                        badge_rect,
                        Rounding::same(3.0),
                        self.theme.warning.linear_multiply(0.18),
                    );
                    painter.galley(
                        Pos2::new(
                            badge_rect.left() + 3.0,
                            badge_rect.center().y - pk_galley.size().y * 0.5,
                        ),
                        pk_galley,
                        self.theme.warning,
                    );
                    text_x += badge_rect.width() + 4.0;
                } else if is_fk {
                    let fk_galley = painter.layout_no_wrap(
                        "FK".to_owned(),
                        FontId::new(9.5, egui::FontFamily::Proportional),
                        self.theme.accent,
                    );
                    let badge_rect = Rect::from_min_size(
                        Pos2::new(text_x, header_text_rect.center().y - 7.0),
                        Vec2::new(fk_galley.size().x + 6.0, 14.0),
                    );
                    painter.rect_filled(badge_rect, Rounding::same(3.0), self.theme.accent.linear_multiply(0.18));
                    painter.galley(
                        Pos2::new(
                            badge_rect.left() + 3.0,
                            badge_rect.center().y - fk_galley.size().y * 0.5,
                        ),
                        fk_galley,
                        self.theme.accent,
                    );
                    text_x += badge_rect.width() + 4.0;
                }

                // Draw column name
                let col_name_galley = painter.layout_no_wrap(
                    column.name.clone(),
                    DbProTheme::ui_medium_font(12.5),
                    self.theme.text_primary,
                );
                let name_width = col_name_galley.size().x;
                painter.galley(
                    Pos2::new(text_x, header_text_rect.center().y - col_name_galley.size().y * 0.5),
                    col_name_galley,
                    self.theme.text_primary,
                );

                // Draw column type
                let type_text = format!(" {}{}", column.data_type, sort_marker);
                let type_galley = painter.layout_no_wrap(
                    type_text,
                    FontId::monospace(10.5),
                    if sort_active {
                        self.theme.accent
                    } else {
                        self.theme.text_muted
                    },
                );
                painter.galley(
                    Pos2::new(
                        text_x + name_width + 4.0,
                        header_text_rect.center().y - type_galley.size().y * 0.5,
                    ),
                    type_galley,
                    if sort_active {
                        self.theme.accent
                    } else {
                        self.theme.text_muted
                    },
                );

                // Header right-click context menu
                let theme = self.theme;
                let is_sorted = sort_active;
                context_action_menu(ui, &col_resp, theme, |ui, close_menu| {
                    if ctx_menu_item(
                        ui,
                        Some(Icon::ArrowUp),
                        "Sort Ascending (A → Z)",
                        None,
                        theme.text_primary,
                        theme,
                    )
                    .clicked()
                    {
                        self.set_table_or_grid_sort(result, col_idx, Some(false));
                        *close_menu = true;
                    }
                    if ctx_menu_item(
                        ui,
                        Some(Icon::ArrowDown),
                        "Sort Descending (Z → A)",
                        None,
                        theme.text_primary,
                        theme,
                    )
                    .clicked()
                    {
                        self.set_table_or_grid_sort(result, col_idx, Some(true));
                        *close_menu = true;
                    }
                    if is_sorted
                        && ctx_menu_item(ui, Some(Icon::X), "Clear Sort", None, theme.text_secondary, theme).clicked()
                    {
                        self.set_table_or_grid_sort(result, col_idx, None);
                        *close_menu = true;
                    }
                    if self.workspace.active_tab == WorkspaceTab::Table
                        && self.table.state.table_view == TableView::Data
                        && ctx_menu_item(ui, Some(Icon::Filter), "Add Filter", None, theme.text_primary, theme)
                            .clicked()
                    {
                        add_filter_req = Some(col_idx);
                        *close_menu = true;
                    }
                    ui.separator();
                    if visual_idx > 0
                        && ctx_menu_item(
                            ui,
                            Some(Icon::ArrowLeft),
                            "Move Column Left",
                            None,
                            theme.text_primary,
                            theme,
                        )
                        .clicked()
                    {
                        move_left_req = Some(visual_idx);
                        *close_menu = true;
                    }
                    if visual_idx + 1 < order.len()
                        && ctx_menu_item(
                            ui,
                            Some(Icon::ArrowRight),
                            "Move Column Right",
                            None,
                            theme.text_primary,
                            theme,
                        )
                        .clicked()
                    {
                        move_right_req = Some(visual_idx);
                        *close_menu = true;
                    }
                    ui.separator();
                    if ctx_menu_item(
                        ui,
                        Some(Icon::RotateCcw),
                        "Reset Column Order",
                        None,
                        theme.text_secondary,
                        theme,
                    )
                    .clicked()
                    {
                        reset_order_req = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(
                        ui,
                        Some(Icon::Maximize2),
                        "Reset Column Widths",
                        None,
                        theme.text_secondary,
                        theme,
                    )
                    .clicked()
                    {
                        reset_widths_req = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(ui, Some(Icon::EyeOff), "Hide Column", None, theme.text_secondary, theme).clicked()
                    {
                        hide_column_req = Some(col_idx);
                        *close_menu = true;
                    }
                    if !self.table.data.grid_hidden_columns.is_empty()
                        && ctx_menu_item(ui, Some(Icon::Eye), "Show Columns", None, theme.text_secondary, theme)
                            .clicked()
                    {
                        show_columns_req = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(
                        ui,
                        Some(Icon::RotateCcw),
                        "Reset Layout",
                        None,
                        theme.text_secondary,
                        theme,
                    )
                    .clicked()
                    {
                        reset_layout_req = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(ui, Some(Icon::Ruler), "Auto Size", None, theme.text_secondary, theme).clicked() {
                        auto_size_req = Some(col_idx);
                        *close_menu = true;
                    }
                });

                if col_resp.clicked() && !divider.dragged() {
                    if self.workspace.active_tab == WorkspaceTab::Table
                        && self.table.state.table_view == TableView::Data
                    {
                        self.cycle_table_data_sort(result, col_idx, ui.input(|input| input.modifiers.shift));
                    } else if self.table.data.grid_sort_column == Some(col_idx) {
                        self.set_table_or_grid_sort(
                            result,
                            col_idx,
                            if self.table.data.grid_sort_desc {
                                None
                            } else {
                                Some(true)
                            },
                        );
                    } else {
                        self.set_table_or_grid_sort(result, col_idx, Some(false));
                    }
                }

                if divider.drag_started() {
                    self.table.data.grid_column_widths = widths.to_vec();
                    self.table.data.grid_columns_user_resized = true;
                }
                if divider.dragged() {
                    self.table.data.grid_column_widths[col_idx] =
                        (self.table.data.grid_column_widths[col_idx] + divider.drag_delta().x).clamp(60.0, 1000.0);
                }
                if divider.double_clicked() {
                    auto_size_req = Some(col_idx);
                }
            }
        });

        if let Some(idx) = move_left_req {
            if idx > 0 {
                self.table.data.move_column(idx, idx - 1, order.len());
            }
        }
        if let Some(idx) = move_right_req {
            if idx + 1 < order.len() {
                self.table.data.move_column(idx, idx + 1, order.len());
            }
        }
        if reset_order_req {
            self.table.data.grid_column_order = (0..result.columns.len()).collect();
        }
        if reset_widths_req {
            self.table.data.grid_columns_user_resized = false;
        }
        if let Some(column_index) = hide_column_req {
            self.table.data.hide_column(column_index, order.len());
        }
        if let Some(column_index) = add_filter_req {
            if let Some(column) = result.columns.get(column_index) {
                self.table.data_query.filter_column = column.name.clone();
                self.table.data_query.filter_operator = UiTableFilterOperator::Equals;
                self.table.data_query.filter_value.clear();
                self.table.data_query.filter_editing = None;
                self.feedback.runtime_message = format!("Filter draft ready for {}", column.name);
            }
        }
        if show_columns_req {
            self.table.data.show_all_columns();
        }
        if reset_layout_req {
            self.table.data.reset_grid_layout(result.columns.len());
        }
        if let Some(column_index) = auto_size_req {
            // `indexes` is the projection this frame already holds; recomputing it here would repeat
            // the whole filter/sort pass for one column-width change.
            self.table.data.auto_size_column(result, indexes, column_index);
        }
    }
}
