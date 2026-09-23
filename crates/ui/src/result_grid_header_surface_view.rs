use super::*;
use egui::{Align2, Pos2, Rect, Rounding, Stroke, Vec2};

#[derive(Debug, Clone, PartialEq)]
pub(super) enum GridHeaderAction {
    Sort {
        column_index: usize,
        descending: Option<bool>,
    },
    CycleTableSort {
        column_index: usize,
        shift: bool,
    },
    StartResize {
        widths: Vec<f32>,
    },
    Resize {
        column_index: usize,
        delta: f32,
    },
    CopyColumnName(usize),
    CopyColumnValues(usize),
    MoveLeft(usize),
    MoveRight(usize),
    ResetOrder,
    ResetWidths,
    HideColumn(usize),
    AddFilter(usize),
    ShowColumns,
    ResetLayout,
    AutoSize(usize),
}

pub(super) struct GridHeaderViewContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) workspace: &'a WorkspaceFeatureState,
    pub(super) table: &'a TableEditorState,
}

pub(super) struct GridHeaderInput<'a> {
    pub(super) result: &'a UiQueryResult,
    pub(super) widths: &'a [f32],
    pub(super) order: &'a [usize],
}

impl<'a> GridHeaderViewContext<'a> {
    pub(super) fn draw_header(&self, ui: &mut egui::Ui, input: GridHeaderInput<'_>) -> Vec<GridHeaderAction> {
        let result = input.result;
        let widths = input.widths;
        let order = input.order;

        let mut actions = Vec::new();
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

                let content_context = result_grid_header_content_view::GridHeaderContentContext {
                    column,
                    col_rect,
                    is_primary_key: is_pk,
                    is_foreign_key: is_fk,
                    sort_marker: &sort_marker,
                    sort_active,
                    theme: self.theme,
                };
                result_grid_header_content_view::draw_header_content(&content_context, ui);

                let menu_context = result_grid_header_menu_view::GridHeaderMenuContext {
                    theme: self.theme,
                    visual_index: visual_idx,
                    column_count: order.len(),
                    column_index: col_idx,
                    is_sorted: sort_active,
                    table_data_active: self.workspace.active_tab == WorkspaceTab::Table
                        && self.table.state.table_view == TableView::Data,
                    has_hidden_columns: !self.table.data.grid_hidden_columns.is_empty(),
                };
                if let Some(action) = result_grid_header_menu_view::draw_menu(&menu_context, ui, &col_resp) {
                    use result_grid_header_menu_view::GridHeaderMenuAction as MenuAction;
                    match action {
                        MenuAction::Sort(direction) => actions.push(GridHeaderAction::Sort {
                            column_index: col_idx,
                            descending: direction,
                        }),
                        MenuAction::AddFilter => add_filter_req = Some(col_idx),
                        MenuAction::CopyColumnName(index) => actions.push(GridHeaderAction::CopyColumnName(index)),
                        MenuAction::CopyColumnValues(index) => actions.push(GridHeaderAction::CopyColumnValues(index)),
                        MenuAction::MoveLeft(index) => move_left_req = Some(index),
                        MenuAction::MoveRight(index) => move_right_req = Some(index),
                        MenuAction::ResetOrder => reset_order_req = true,
                        MenuAction::ResetWidths => reset_widths_req = true,
                        MenuAction::HideColumn(index) => hide_column_req = Some(index),
                        MenuAction::ShowColumns => show_columns_req = true,
                        MenuAction::ResetLayout => reset_layout_req = true,
                        MenuAction::AutoSize(index) => auto_size_req = Some(index),
                    }
                }

                if col_resp.clicked() && !divider.dragged() {
                    if self.workspace.active_tab == WorkspaceTab::Table
                        && self.table.state.table_view == TableView::Data
                    {
                        actions.push(GridHeaderAction::CycleTableSort {
                            column_index: col_idx,
                            shift: ui.input(|input| input.modifiers.shift),
                        });
                    } else if self.table.data.grid_sort_column == Some(col_idx) {
                        actions.push(GridHeaderAction::Sort {
                            column_index: col_idx,
                            descending: if self.table.data.grid_sort_desc {
                                None
                            } else {
                                Some(true)
                            },
                        });
                    } else {
                        actions.push(GridHeaderAction::Sort {
                            column_index: col_idx,
                            descending: Some(false),
                        });
                    }
                }

                if divider.drag_started() {
                    actions.push(GridHeaderAction::StartResize {
                        widths: widths.to_vec(),
                    });
                }
                if divider.dragged() {
                    actions.push(GridHeaderAction::Resize {
                        column_index: col_idx,
                        delta: divider.drag_delta().x,
                    });
                }
                if divider.double_clicked() {
                    auto_size_req = Some(col_idx);
                }
            }
        });

        if let Some(idx) = move_left_req {
            actions.push(GridHeaderAction::MoveLeft(idx));
        }
        if let Some(idx) = move_right_req {
            actions.push(GridHeaderAction::MoveRight(idx));
        }
        if reset_order_req {
            actions.push(GridHeaderAction::ResetOrder);
        }
        if reset_widths_req {
            actions.push(GridHeaderAction::ResetWidths);
        }
        if let Some(column_index) = hide_column_req {
            actions.push(GridHeaderAction::HideColumn(column_index));
        }
        if let Some(column_index) = add_filter_req {
            actions.push(GridHeaderAction::AddFilter(column_index));
        }
        if show_columns_req {
            actions.push(GridHeaderAction::ShowColumns);
        }
        if reset_layout_req {
            actions.push(GridHeaderAction::ResetLayout);
        }
        if let Some(column_index) = auto_size_req {
            actions.push(GridHeaderAction::AutoSize(column_index));
        }

        actions
    }
}
