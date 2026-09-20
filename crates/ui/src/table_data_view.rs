use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use egui::{FontFamily, FontId, Frame, Margin, Rounding, Stroke};
use lucide_icons::Icon;

struct TableDataPaging {
    page_range: String,
    total_rows: u64,
    total_known: bool,
    has_next: bool,
    has_previous: bool,
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

                    // Unified Search & Filter Bar
                    Frame {
                        fill: self.theme.surface_elevated,
                        stroke: Stroke::new(1.0, self.theme.border_subtle),
                        rounding: Rounding::same(6.0),
                        inner_margin: Margin::symmetric(8.0, 3.0),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Search icon
                            ui.label(
                                RichText::new(char::from(Icon::Search).to_string())
                                    .font(FontId::new(12.0, FontFamily::Name("lucide".into())))
                                    .color(self.theme.text_muted),
                            );

                            // Column Scope Dropdown
                            let col_label = if self.table.data_query.filter_column.is_empty() {
                                "All columns".to_owned()
                            } else {
                                self.table.data_query.filter_column.clone()
                            };
                            let col_color = if self.table.data_query.filter_column.is_empty() {
                                self.theme.text_secondary
                            } else {
                                self.theme.accent
                            };

                            egui::ComboBox::from_id_salt(("table-unified-filter-col", table_name))
                                .selected_text(RichText::new(col_label).size(12.0).color(col_color))
                                .width(95.0)
                                .show_ui(ui, |ui| {
                                    if ui
                                        .selectable_value(
                                            &mut self.table.data_query.filter_column,
                                            String::new(),
                                            "All columns (Instant)",
                                        )
                                        .clicked()
                                    {
                                        self.table.data_query.filter_operator = UiTableFilterOperator::default();
                                        self.table.data.grid_filter = self.table.data_query.filter_value.clone();
                                    }
                                    for col in &column_names {
                                        ui.selectable_value(
                                            &mut self.table.data_query.filter_column,
                                            col.clone(),
                                            col.as_str(),
                                        );
                                    }
                                });

                            ui.add(egui::Separator::default().vertical());

                            let filter_data_type = self
                                .table
                                .state
                                .table_info
                                .as_ref()
                                .and_then(|info| {
                                    info.columns
                                        .iter()
                                        .find(|column| column.name == self.table.data_query.filter_column)
                                })
                                .map(|column| column.data_type.clone())
                                .or_else(|| {
                                    result
                                        .columns
                                        .iter()
                                        .find(|column| column.name == self.table.data_query.filter_column)
                                        .map(|column| column.data_type.clone())
                                })
                                .unwrap_or_else(|| "text".to_owned());
                            let filter_operator_options =
                                TableDataQueryState::filter_operator_options(&filter_data_type);
                            if !filter_operator_options
                                .iter()
                                .any(|(operator, _)| operator == &self.table.data_query.filter_operator)
                            {
                                self.table.data_query.filter_operator = filter_operator_options
                                    .first()
                                    .map(|(operator, _)| operator.clone())
                                    .unwrap_or_default();
                            }
                            let operator_label = match self.table.data_query.filter_operator {
                                UiTableFilterOperator::Equals => "equals",
                                UiTableFilterOperator::NotEquals => "not equals",
                                UiTableFilterOperator::Contains => "contains",
                                UiTableFilterOperator::StartsWith => "starts with",
                                UiTableFilterOperator::EndsWith => "ends with",
                                UiTableFilterOperator::GreaterThan => ">",
                                UiTableFilterOperator::GreaterThanOrEqual => ">=",
                                UiTableFilterOperator::LessThan => "<",
                                UiTableFilterOperator::LessThanOrEqual => "<=",
                                UiTableFilterOperator::IsNull => "IS NULL",
                                UiTableFilterOperator::IsNotNull => "IS NOT NULL",
                            };
                            let mut operator_changed = false;
                            egui::ComboBox::from_id_salt(("table-unified-filter-op", table_name))
                                .selected_text(
                                    RichText::new(operator_label)
                                        .size(12.0)
                                        .color(self.theme.text_secondary),
                                )
                                .width(86.0)
                                .show_ui(ui, |ui| {
                                    for (operator, label) in &filter_operator_options {
                                        if ui
                                            .selectable_value(
                                                &mut self.table.data_query.filter_operator,
                                                operator.clone(),
                                                *label,
                                            )
                                            .clicked()
                                        {
                                            operator_changed = true;
                                        }
                                    }
                                });

                            // Search / Filter Input
                            let is_all_cols = self.table.data_query.filter_column.is_empty();
                            let is_null_operator = matches!(
                                self.table.data_query.filter_operator,
                                UiTableFilterOperator::IsNull | UiTableFilterOperator::IsNotNull
                            );
                            let placeholder = if is_all_cols {
                                "Search rows instantly…"
                            } else if is_null_operator {
                                "No value required"
                            } else {
                                "Filter value (Enter to query DB)…"
                            };

                            let edit_target = if is_all_cols {
                                &mut self.table.data.grid_filter
                            } else {
                                &mut self.table.data_query.filter_value
                            };

                            let edit = egui::TextEdit::singleline(edit_target)
                                .hint_text(RichText::new(placeholder).size(12.0).color(self.theme.text_muted))
                                .font(FontId::proportional(12.0))
                                .frame(false)
                                .interactive(is_all_cols || !is_null_operator)
                                .desired_width(210.0);

                            let resp = ui.add(edit);
                            if (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) || operator_changed)
                                && !is_all_cols
                            {
                                if is_null_operator {
                                    self.table.data_query.filter_value.clear();
                                }
                                self.commit_table_filter_draft();
                            }

                            // Trailing clear button
                            let has_text = if is_all_cols {
                                !self.table.data.grid_filter.is_empty()
                            } else {
                                !self.table.data_query.filter_value.is_empty()
                            };

                            if has_text
                                && Button::new(self.theme)
                                    .icon(Icon::X)
                                    .size(ButtonSize::IconSm)
                                    .variant(ButtonVariant::Ghost)
                                    .show(ui)
                                    .on_hover_text("Clear filter")
                                    .clicked()
                            {
                                if is_all_cols {
                                    self.table.data.grid_filter.clear();
                                } else {
                                    let filter_column = self.table.data_query.filter_column.clone();
                                    self.table
                                        .data_query
                                        .filters
                                        .retain(|filter| filter.column != filter_column);
                                    self.table.data_query.filter_value.clear();
                                    self.table.data_query.filter_operator = UiTableFilterOperator::default();
                                    self.table.data_query.offset = 0;
                                    self.request_table_data();
                                }
                            }
                        });
                    });

                    if !self.table.data_query.filters.is_empty() {
                        let mut remove_filter = None;
                        let mut edit_filter = None;
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("Filters:").small().color(self.theme.text_muted));
                            for (index, filter) in self.table.data_query.filters.iter().enumerate() {
                                let operator = match filter.operator {
                                    UiTableFilterOperator::Equals => "=",
                                    UiTableFilterOperator::NotEquals => "!=",
                                    UiTableFilterOperator::Contains => "contains",
                                    UiTableFilterOperator::StartsWith => "starts",
                                    UiTableFilterOperator::EndsWith => "ends",
                                    UiTableFilterOperator::GreaterThan => ">",
                                    UiTableFilterOperator::GreaterThanOrEqual => ">=",
                                    UiTableFilterOperator::LessThan => "<",
                                    UiTableFilterOperator::LessThanOrEqual => "<=",
                                    UiTableFilterOperator::IsNull => "IS NULL",
                                    UiTableFilterOperator::IsNotNull => "IS NOT NULL",
                                };
                                let value = if filter.value.is_empty() {
                                    String::new()
                                } else {
                                    format!(" {}", filter.value)
                                };
                                if ui
                                    .small_button(format!("{} {}{}", filter.column, operator, value))
                                    .on_hover_text("Edit filter")
                                    .clicked()
                                {
                                    edit_filter = Some(index);
                                }
                                if ui.small_button("×").on_hover_text("Remove filter").clicked() {
                                    remove_filter = Some(index);
                                }
                            }
                            if ui.small_button("Clear all").clicked() {
                                remove_filter = Some(usize::MAX);
                            }
                        });
                        if let Some(index) = edit_filter {
                            if let Some(filter) = self.table.data_query.filters.get(index).cloned() {
                                self.table.data_query.filter_column = filter.column;
                                self.table.data_query.filter_operator = filter.operator;
                                self.table.data_query.filter_value = filter.value;
                                self.table.data_query.filter_editing = Some(index);
                            }
                        }
                        if let Some(index) = remove_filter {
                            if index == usize::MAX {
                                self.clear_table_filters();
                            } else {
                                self.remove_table_filter(index);
                            }
                        }
                    }

                    // Compact Sort Selector
                    let sort_active =
                        !self.table.data_query.sorts.is_empty() || self.table.data.grid_sort_column.is_some();
                    let sort_label = if !self.table.data_query.sorts.is_empty() {
                        let clauses = self
                            .table
                            .data_query
                            .sorts
                            .iter()
                            .enumerate()
                            .map(|(priority, sort)| {
                                format!(
                                    "{} {}{}",
                                    sort.column,
                                    if sort.descending { "↓" } else { "↑" },
                                    priority + 1
                                )
                            })
                            .collect::<Vec<_>>();
                        format!("Sort: {}", clauses.join(", "))
                    } else if let Some(idx) = self.table.data.grid_sort_column {
                        if let Some(col) = column_names.get(idx) {
                            format!("Sort: {col} {}", if self.table.data.grid_sort_desc { "↓" } else { "↑" })
                        } else {
                            "Sort".to_owned()
                        }
                    } else {
                        "Sort".to_owned()
                    };

                    egui::ComboBox::from_id_salt(("table-unified-sort-col", table_name))
                        .selected_text(RichText::new(&sort_label).size(11.5).color(if sort_active {
                            self.theme.accent
                        } else {
                            self.theme.text_secondary
                        }))
                        .width(100.0)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(!sort_active, "Default (None)").clicked() {
                                if self.table.mutation.staged_changes.is_empty() {
                                    self.table.data_query.sorts.clear();
                                    self.table.data.grid_sort_column = None;
                                    self.reload_table_data_from_start();
                                } else {
                                    self.feedback.runtime_message =
                                        "Apply or discard staged changes before changing sort".to_owned();
                                }
                            }
                            for col in &column_names {
                                let is_sel = self.table.data_query.sorts.first().map(|sort| sort.column.as_str())
                                    == Some(col.as_str());
                                if ui.selectable_label(is_sel, col.as_str()).clicked() {
                                    if !self.table.mutation.staged_changes.is_empty() {
                                        self.feedback.runtime_message =
                                            "Apply or discard staged changes before changing sort".to_owned();
                                    } else if is_sel {
                                        if let Some(sort) = self.table.data_query.sorts.first_mut() {
                                            sort.descending = !sort.descending;
                                        }
                                    } else {
                                        self.table.data_query.sorts = vec![UiTableDataSort {
                                            column: col.clone(),
                                            descending: false,
                                        }];
                                    }
                                    self.reload_table_data_from_start();
                                }
                            }
                        });
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if paging.total_known
                        && paging.total_rows > 0
                        && Button::new(self.theme)
                            .text("Last")
                            .icon(Icon::ChevronsRight)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .enabled(paging.has_next && self.table.mutation.staged_changes.is_empty())
                            .tooltip("Last page")
                            .show(ui)
                            .clicked()
                        && paging.has_next
                        && self.table.mutation.staged_changes.is_empty()
                    {
                        let last_page = paging.total_rows.saturating_sub(1) / self.table.data_query.limit;
                        self.table.data_query.offset = last_page.saturating_mul(self.table.data_query.limit);
                        self.request_table_data();
                    }

                    if Button::new(self.theme)
                        .icon(Icon::ChevronRight)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .enabled(paging.has_next && self.table.mutation.staged_changes.is_empty())
                        .tooltip("Next page")
                        .show(ui)
                        .clicked()
                        && self.table.mutation.staged_changes.is_empty()
                    {
                        self.table.data_query.offset =
                            self.table.data_query.offset.saturating_add(self.table.data_query.limit);
                        self.request_table_data();
                    }

                    ui.label(
                        RichText::new(&paging.page_range)
                            .font(font_caption())
                            .color(self.theme.text_secondary),
                    );

                    if Button::new(self.theme)
                        .icon(Icon::ChevronLeft)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .enabled(paging.has_previous && self.table.mutation.staged_changes.is_empty())
                        .tooltip("Previous page")
                        .show(ui)
                        .clicked()
                        && self.table.mutation.staged_changes.is_empty()
                    {
                        self.table.data_query.offset =
                            self.table.data_query.offset.saturating_sub(self.table.data_query.limit);
                        self.request_table_data();
                    }

                    if paging.total_known
                        && paging.total_rows > 0
                        && Button::new(self.theme)
                            .text("First")
                            .icon(Icon::ChevronsLeft)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .enabled(self.table.data_query.offset > 0 && self.table.mutation.staged_changes.is_empty())
                            .tooltip("First page")
                            .show(ui)
                            .clicked()
                        && self.table.data_query.offset > 0
                        && self.table.mutation.staged_changes.is_empty()
                    {
                        self.table.data_query.offset = 0;
                        self.request_table_data();
                    }

                    ui.separator();

                    let prev_limit = self.table.data_query.limit;
                    let limit_label = format!("{} / page", self.table.data_query.limit);
                    egui::ComboBox::from_id_salt(("table-data-limit-select", table_name))
                        .selected_text(RichText::new(&limit_label).size(11.0).color(self.theme.text_secondary))
                        .width(90.0)
                        .show_ui(ui, |ui| {
                            for limit_opt in [50, 100, 250, 500, 1000] {
                                ui.selectable_value(
                                    &mut self.table.data_query.limit,
                                    limit_opt,
                                    format!("{limit_opt} / page"),
                                );
                            }
                        });
                    if self.table.data_query.limit != prev_limit {
                        self.reset_table_data_page();
                    }
                });
            });
        });
    }
}
