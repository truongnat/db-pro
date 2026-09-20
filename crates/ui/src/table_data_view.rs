use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use lucide_icons::Icon;

pub(super) struct TableDataPaging {
    pub(super) page_range: String,
    pub(super) total_rows: u64,
    pub(super) total_known: bool,
    pub(super) has_next: bool,
    pub(super) has_previous: bool,
}

impl DbProApp {
    pub(crate) fn draw_table_data(&mut self, ui: &mut egui::Ui, table_name: &str) {
        // Temporarily move the result out while rendering. The grid mutates
        // selection/edit state, so borrowing it directly from `self` would
        // conflict with those updates; moving avoids cloning every frame.
        let Some(result) = self.table.data_query.result.take() else {
            self.draw_table_data_placeholder(ui, table_name);
            return;
        };

        let can_mutate = self.can_mutate_active_connection();
        let total_known = self.table.data_query.total_rows.is_some();
        let total_rows = self.table.data_query.total_rows.unwrap_or(result.row_count);
        let paging = TableDataPaging {
            page_range: crate::components::common_utils::format_page_range(
                self.table.data_query.offset,
                result.row_count,
                self.table.data_query.total_rows,
            ),
            total_rows,
            total_known,
            has_next: if total_known {
                self.table.data_query.offset.saturating_add(self.table.data_query.limit) < total_rows
            } else {
                result.row_count >= self.table.data_query.limit
            },
            has_previous: self.table.data_query.offset > 0,
        };

        self.draw_table_data_unified_toolbar(ui, table_name, &result, can_mutate, &paging);
        ui.add_space(4.0);

        let data_width = ui.max_rect().width();
        grid_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(data_width.max(0.0));
            self.draw_result_grid(ui, &result);
        });
        self.draw_pending_changes_dialog(ui);
        self.draw_conflict_dialog(ui, &result);
        if self.table.data_query.result.is_none() {
            self.table.data_query.result = Some(result);
        }
    }

    /// Loading / failed placeholder shown while table data is not available.
    fn draw_table_data_placeholder(&mut self, ui: &mut egui::Ui, table_name: &str) {
        let context = table_data_placeholder_view::TableDataPlaceholderContext {
            theme: self.theme,
            error: self.table.data_query.error.as_deref(),
        };
        if matches!(
            table_data_placeholder_view::draw_placeholder(&context, ui, table_name),
            Some(table_data_placeholder_view::TableDataPlaceholderAction::Retry)
        ) {
            self.table.data_query.error = None;
            self.request_table_data();
        }
    }

    /// Unified DBeaver-style header bar: refresh, add row, staged changes, inline WHERE/filter input, ORDER BY, and pagination.
    fn draw_table_data_unified_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        table_name: &str,
        result: &UiQueryResult,
        can_mutate: bool,
        paging: &TableDataPaging,
    ) {
        let column_names: Vec<String> = result.columns.iter().map(|c| c.name.clone()).collect();

        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                if compact_button_with_icon(ui, Icon::RotateCcw, "Refresh", self.theme)
                    .on_hover_text("Reload table data (F5)")
                    .clicked()
                {
                    if self.table.mutation.staged_changes.is_empty() {
                        self.request_table_data();
                    } else {
                        self.feedback.runtime_message = "Apply or discard staged changes before refreshing".to_owned();
                    }
                }

                if can_mutate {
                    ui.separator();
                    if Button::new(self.theme)
                        .text("Add Row")
                        .icon(Icon::Plus)
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .tooltip("Insert new row")
                        .show(ui)
                        .clicked()
                    {
                        self.open_insert_row();
                    }

                    if !self.table.state.has_primary_key() {
                        ui.label(
                            RichText::new("Table has no primary key; safe row editing is unavailable.")
                                .font(font_caption())
                                .color(self.theme.warning),
                        )
                        .on_hover_text(
                            "This table has no primary key; safe row editing is unavailable. Inserts remain available.",
                        );
                    }

                    if !self.table.mutation.staged_changes.is_empty() {
                        ui.separator();
                        let counts = self.table.mutation.staged_changes.counts();
                        if ui
                            .small_button(format!(
                                "{} pending · +{} ~{} -{}",
                                counts.total(),
                                counts.inserts,
                                counts.updates,
                                counts.deletes
                            ))
                            .on_hover_text("Open pending changes")
                            .clicked()
                        {
                            self.table.mutation.pending_changes_open = true;
                        }
                        let apply_enabled = self.table.mutation.staged_apply_request.is_none()
                            && self.table.editing.data_edit_error.is_none();
                        if Button::new(self.theme)
                            .text("Apply")
                            .icon(Icon::Check)
                            .variant(ButtonVariant::Default)
                            .size(ButtonSize::Sm)
                            .enabled(apply_enabled)
                            .tooltip(format!(
                                "Apply all staged changes ({}S)",
                                Self::primary_modifier_label()
                            ))
                            .show(ui)
                            .clicked()
                        {
                            self.apply_staged_changes();
                        }
                        if Button::new(self.theme)
                            .text("Discard")
                            .icon(Icon::Undo2)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .tooltip(format!(
                                "Discard all staged changes ({}Z)",
                                Self::primary_modifier_label()
                            ))
                            .show(ui)
                            .clicked()
                        {
                            if self.table.mutation.staged_changes.counts().total() > 1 {
                                self.table.editing.discard_changes_confirmation = true;
                            } else {
                                self.discard_staged_changes();
                            }
                        }
                    }
                } else if self.connection.lifecycle.is_connected() {
                    ui.separator();
                    ui.label(
                        RichText::new(if self.table.state.has_primary_key() {
                            "Read-only"
                        } else {
                            "Table has no primary key; safe row editing is unavailable."
                        })
                        .font(font_caption())
                        .color(if self.table.state.has_primary_key() {
                            self.theme.warning
                        } else {
                            self.theme.danger
                        }),
                    );
                }

                if let Some(failure) = self.table.mutation.table_mutation_error.as_ref() {
                    let is_conflict = failure.code == "CONFLICT";
                    ui.separator();
                    ui.label(
                        RichText::new(format!(
                            "{} · {}",
                            failure.code,
                            failure.target.as_ref().map_or_else(
                                || "Transaction failed".to_owned(),
                                |_| format!("Mutation #{} failed", failure.statement_index.saturating_add(1)),
                            )
                        ))
                        .font(font_caption())
                        .color(self.theme.danger),
                    )
                    .on_hover_text(failure.message.as_str());
                    if failure.target.is_some() {
                        if Button::new(self.theme)
                            .text("Reload Row")
                            .icon(Icon::RotateCcw)
                            .variant(ButtonVariant::Secondary)
                            .size(ButtonSize::Sm)
                            .tooltip("Reload database values while keeping the local staged mutation")
                            .show(ui)
                            .clicked()
                        {
                            self.reload_failed_mutation();
                        }
                        if Button::new(self.theme)
                            .text("Discard Local Change")
                            .icon(Icon::Undo2)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .tooltip("Revert only the failed staged mutation")
                            .show(ui)
                            .clicked()
                        {
                            self.discard_failed_mutation(false);
                        }
                        if is_conflict {
                            if Button::new(self.theme)
                                .text("Resolve Conflict")
                                .icon(Icon::GitCompare)
                                .variant(ButtonVariant::Secondary)
                                .size(ButtonSize::Sm)
                                .tooltip("Open 3-way conflict resolution panel")
                                .show(ui)
                                .clicked()
                            {
                                self.table.mutation.conflict_dialog_open = true;
                            }
                            if Button::new(self.theme)
                                .text("Retry")
                                .icon(Icon::RotateCcw)
                                .variant(ButtonVariant::Default)
                                .size(ButtonSize::Sm)
                                .tooltip("Reload the row, then retry the staged mutation")
                                .show(ui)
                                .clicked()
                            {
                                self.retry_failed_mutation_after_reload();
                            }
                        }
                    }
                }

                if self.table.data.selected_rows.len() > 1 {
                    crate::components::badge::Badge::new(
                        format!("{} rows selected", self.table.data.selected_rows.len()),
                        self.theme,
                    )
                    .variant(crate::components::badge::BadgeVariant::Secondary)
                    .compact(true)
                    .show(ui);
                }

                if !column_names.is_empty() {
                    ui.separator();
                    let mut filter_context = table_data_filter_view::TableDataFilterContext {
                        theme: self.theme,
                        table_name,
                        result,
                        column_names: &column_names,
                        data_query: &mut self.table.data_query,
                        data: &mut self.table.data,
                        table_info: self.table.state.table_info.as_ref(),
                    };
                    if let Some(action) = table_data_filter_view::draw_filter(&mut filter_context, ui) {
                        match action {
                            table_data_filter_view::TableDataFilterAction::CommitDraft => {
                                self.commit_table_filter_draft();
                            }
                            table_data_filter_view::TableDataFilterAction::ReloadData => {
                                self.request_table_data();
                            }
                            table_data_filter_view::TableDataFilterAction::Remove(index) => {
                                self.remove_table_filter(index);
                            }
                            table_data_filter_view::TableDataFilterAction::ClearAll => {
                                self.clear_table_filters();
                            }
                        }
                    }

                    let mut sort_context = table_data_sort_view::TableDataSortContext {
                        theme: self.theme,
                        table_name,
                        column_names: &column_names,
                        data_query: &mut self.table.data_query,
                        data: &mut self.table.data,
                        has_staged_changes: !self.table.mutation.staged_changes.is_empty(),
                    };
                    if let Some(action) = table_data_sort_view::draw_sort(&mut sort_context, ui) {
                        match action {
                            table_data_sort_view::TableDataSortAction::ReloadFromStart => {
                                self.reload_table_data_from_start();
                            }
                            table_data_sort_view::TableDataSortAction::BlockedByStagedChanges => {
                                self.feedback.runtime_message =
                                    "Apply or discard staged changes before changing sort".to_owned();
                            }
                        }
                    }
                }

                let mut pagination_context = table_data_pagination_view::TableDataPaginationContext {
                    theme: self.theme,
                    table_name,
                    paging,
                    query: &mut self.table.data_query,
                    has_staged_changes: !self.table.mutation.staged_changes.is_empty(),
                };
                if let Some(action) = table_data_pagination_view::draw_pagination(&mut pagination_context, ui) {
                    match action {
                        table_data_pagination_view::TableDataPaginationAction::RequestData => {
                            self.request_table_data();
                        }
                        table_data_pagination_view::TableDataPaginationAction::ResetPage => {
                            self.reset_table_data_page();
                        }
                    }
                }
            });
        });
    }
}
