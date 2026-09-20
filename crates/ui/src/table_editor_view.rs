use super::*;
use crate::components::alert::{Alert, AlertVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use egui::{FontFamily, FontId, Frame, Margin, Rounding, Stroke};
use lucide_icons::Icon;

/// Whether the table editor executes DDL itself.
///
/// It does not: v0.1 ships schema/DDL *inspection* from the table editor and DDL
/// *execution* through the query editor (`docs/release/known-limitations.md`, "DDL via
/// query editor"). The confirmation card and `submit_ddl` stay compiled behind this
/// flag so the capability can be enabled by a deliberate change (with its own
/// qualification) instead of by a stray click.
const DDL_APPLY_ENABLED: bool = false;

/// Paging state for the table data editor toolbar.
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
        let Some(result) = self.table_state.table_data_result.take() else {
            self.draw_table_data_placeholder(ui, table_name);
            return;
        };

        let can_mutate = self.can_mutate_active_connection();
        let total_known = self.table_state.table_data_total_rows.is_some();
        let total_rows = self.table_state.table_data_total_rows.unwrap_or(result.row_count);
        let paging = TableDataPaging {
            page_range: crate::components::common_utils::format_page_range(
                self.table_state.table_data_offset,
                result.row_count,
                self.table_state.table_data_total_rows,
            ),
            total_rows,
            total_known,
            has_next: if total_known {
                self.table_state
                    .table_data_offset
                    .saturating_add(self.table_state.table_data_limit)
                    < total_rows
            } else {
                result.row_count >= self.table_state.table_data_limit
            },
            has_previous: self.table_state.table_data_offset > 0,
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
        if self.table_state.table_data_result.is_none() {
            self.table_state.table_data_result = Some(result);
        }
    }

    /// Loading / failed placeholder shown while table data is not available.
    fn draw_table_data_placeholder(&mut self, ui: &mut egui::Ui, table_name: &str) {
        grid_frame(self.theme).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(28.0);
                let failed = self.table_state.table_data_error.as_deref();
                ui.label(icon_text(
                    if failed.is_some() {
                        Icon::TriangleAlert
                    } else {
                        Icon::LoaderCircle
                    },
                    "",
                    if failed.is_some() {
                        self.theme.warning
                    } else {
                        self.theme.accent
                    },
                ));
                ui.add_space(8.0);
                ui.label(
                    RichText::new(if failed.is_some() {
                        format!("Data for {table_name} could not be loaded")
                    } else {
                        format!("Loading data for {table_name}…")
                    })
                    .strong()
                    .color(self.theme.text_primary),
                );
                if let Some(error) = failed {
                    ui.label(RichText::new(error).small().color(self.theme.text_secondary));
                    ui.add_space(12.0);
                    if secondary_button_with_icon(ui, Icon::RotateCcw, "Retry", self.theme).clicked() {
                        self.table_state.table_data_error = None;
                        self.request_table_data();
                    }
                } else {
                    ui.label(
                        RichText::new("Rows will appear here with the shared result-grid controls.")
                            .small()
                            .color(self.theme.text_secondary),
                    );
                }
                ui.add_space(28.0);
            });
        });
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
                    if self.table_mutation.staged_changes.is_empty() {
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

                    if !self.table_state.has_primary_key() {
                        ui.label(
                            RichText::new("Table has no primary key; safe row editing is unavailable.")
                                .font(font_caption())
                                .color(self.theme.warning),
                        )
                        .on_hover_text(
                            "This table has no primary key; safe row editing is unavailable. Inserts remain available.",
                        );
                    }

                    if !self.table_mutation.staged_changes.is_empty() {
                        ui.separator();
                        let counts = self.table_mutation.staged_changes.counts();
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
                            self.table_mutation.pending_changes_open = true;
                        }
                        let apply_enabled = self.table_mutation.staged_apply_request.is_none()
                            && self.table_data.data_edit_error.is_none();
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
                            if self.table_mutation.staged_changes.counts().total() > 1 {
                                self.table_data.discard_changes_confirmation = true;
                            } else {
                                self.discard_staged_changes();
                            }
                        }
                    }
                } else if self.connection.lifecycle.is_connected() {
                    ui.separator();
                    ui.label(
                        RichText::new(if self.table_state.has_primary_key() {
                            "Read-only"
                        } else {
                            "Table has no primary key; safe row editing is unavailable."
                        })
                        .font(font_caption())
                        .color(if self.table_state.has_primary_key() {
                            self.theme.warning
                        } else {
                            self.theme.danger
                        }),
                    );
                }

                if let Some(failure) = self.table_mutation.table_mutation_error.as_ref() {
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
                                self.table_mutation.conflict_dialog_open = true;
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

                if self.table_data.selected_rows.len() > 1 {
                    crate::components::badge::Badge::new(
                        format!("{} rows selected", self.table_data.selected_rows.len()),
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
                            let col_label = if self.table_state.table_data_filter_column.is_empty() {
                                "All columns".to_owned()
                            } else {
                                self.table_state.table_data_filter_column.clone()
                            };
                            let col_color = if self.table_state.table_data_filter_column.is_empty() {
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
                                            &mut self.table_state.table_data_filter_column,
                                            String::new(),
                                            "All columns (Instant)",
                                        )
                                        .clicked()
                                    {
                                        self.table_state.table_data_filter_operator = UiTableFilterOperator::default();
                                        self.table_data.grid_filter = self.table_state.table_data_filter_value.clone();
                                    }
                                    for col in &column_names {
                                        ui.selectable_value(
                                            &mut self.table_state.table_data_filter_column,
                                            col.clone(),
                                            col.as_str(),
                                        );
                                    }
                                });

                            ui.add(egui::Separator::default().vertical());

                            let filter_data_type = self
                                .table_state
                                .table_info
                                .as_ref()
                                .and_then(|info| {
                                    info.columns
                                        .iter()
                                        .find(|column| column.name == self.table_state.table_data_filter_column)
                                })
                                .map(|column| column.data_type.clone())
                                .or_else(|| {
                                    result
                                        .columns
                                        .iter()
                                        .find(|column| column.name == self.table_state.table_data_filter_column)
                                        .map(|column| column.data_type.clone())
                                })
                                .unwrap_or_else(|| "text".to_owned());
                            let filter_operator_options = Self::filter_operator_options(&filter_data_type);
                            if !filter_operator_options
                                .iter()
                                .any(|(operator, _)| operator == &self.table_state.table_data_filter_operator)
                            {
                                self.table_state.table_data_filter_operator = filter_operator_options
                                    .first()
                                    .map(|(operator, _)| operator.clone())
                                    .unwrap_or_default();
                            }
                            let operator_label = match self.table_state.table_data_filter_operator {
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
                                                &mut self.table_state.table_data_filter_operator,
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
                            let is_all_cols = self.table_state.table_data_filter_column.is_empty();
                            let is_null_operator = matches!(
                                self.table_state.table_data_filter_operator,
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
                                &mut self.table_data.grid_filter
                            } else {
                                &mut self.table_state.table_data_filter_value
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
                                    self.table_state.table_data_filter_value.clear();
                                }
                                self.commit_table_filter_draft();
                            }

                            // Trailing clear button
                            let has_text = if is_all_cols {
                                !self.table_data.grid_filter.is_empty()
                            } else {
                                !self.table_state.table_data_filter_value.is_empty()
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
                                    self.table_data.grid_filter.clear();
                                } else {
                                    let filter_column = self.table_state.table_data_filter_column.clone();
                                    self.table_state
                                        .table_data_filters
                                        .retain(|filter| filter.column != filter_column);
                                    self.table_state.table_data_filter_value.clear();
                                    self.table_state.table_data_filter_operator = UiTableFilterOperator::default();
                                    self.table_state.table_data_offset = 0;
                                    self.request_table_data();
                                }
                            }
                        });
                    });

                    if !self.table_state.table_data_filters.is_empty() {
                        let mut remove_filter = None;
                        let mut edit_filter = None;
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("Filters:").small().color(self.theme.text_muted));
                            for (index, filter) in self.table_state.table_data_filters.iter().enumerate() {
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
                            if let Some(filter) = self.table_state.table_data_filters.get(index).cloned() {
                                self.table_state.table_data_filter_column = filter.column;
                                self.table_state.table_data_filter_operator = filter.operator;
                                self.table_state.table_data_filter_value = filter.value;
                                self.table_state.table_data_filter_editing = Some(index);
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
                        !self.table_state.table_data_sorts.is_empty() || self.table_data.grid_sort_column.is_some();
                    let sort_label = if !self.table_state.table_data_sorts.is_empty() {
                        let clauses = self
                            .table_state
                            .table_data_sorts
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
                    } else if let Some(idx) = self.table_data.grid_sort_column {
                        if let Some(col) = column_names.get(idx) {
                            format!("Sort: {col} {}", if self.table_data.grid_sort_desc { "↓" } else { "↑" })
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
                                if self.table_mutation.staged_changes.is_empty() {
                                    self.table_state.table_data_sorts.clear();
                                    self.table_data.grid_sort_column = None;
                                    self.reload_table_data_from_start();
                                } else {
                                    self.feedback.runtime_message =
                                        "Apply or discard staged changes before changing sort".to_owned();
                                }
                            }
                            for col in &column_names {
                                let is_sel = self
                                    .table_state
                                    .table_data_sorts
                                    .first()
                                    .map(|sort| sort.column.as_str())
                                    == Some(col.as_str());
                                if ui.selectable_label(is_sel, col.as_str()).clicked() {
                                    if !self.table_mutation.staged_changes.is_empty() {
                                        self.feedback.runtime_message =
                                            "Apply or discard staged changes before changing sort".to_owned();
                                    } else if is_sel {
                                        if let Some(sort) = self.table_state.table_data_sorts.first_mut() {
                                            sort.descending = !sort.descending;
                                        }
                                    } else {
                                        self.table_state.table_data_sorts = vec![UiTableDataSort {
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
                            .enabled(paging.has_next && self.table_mutation.staged_changes.is_empty())
                            .tooltip("Last page")
                            .show(ui)
                            .clicked()
                        && paging.has_next
                        && self.table_mutation.staged_changes.is_empty()
                    {
                        let last_page = paging.total_rows.saturating_sub(1) / self.table_state.table_data_limit;
                        self.table_state.table_data_offset =
                            last_page.saturating_mul(self.table_state.table_data_limit);
                        self.request_table_data();
                    }

                    if Button::new(self.theme)
                        .icon(Icon::ChevronRight)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .enabled(paging.has_next && self.table_mutation.staged_changes.is_empty())
                        .tooltip("Next page")
                        .show(ui)
                        .clicked()
                        && self.table_mutation.staged_changes.is_empty()
                    {
                        self.table_state.table_data_offset = self
                            .table_state
                            .table_data_offset
                            .saturating_add(self.table_state.table_data_limit);
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
                        .enabled(paging.has_previous && self.table_mutation.staged_changes.is_empty())
                        .tooltip("Previous page")
                        .show(ui)
                        .clicked()
                        && self.table_mutation.staged_changes.is_empty()
                    {
                        self.table_state.table_data_offset = self
                            .table_state
                            .table_data_offset
                            .saturating_sub(self.table_state.table_data_limit);
                        self.request_table_data();
                    }

                    if paging.total_known
                        && paging.total_rows > 0
                        && Button::new(self.theme)
                            .text("First")
                            .icon(Icon::ChevronsLeft)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .enabled(
                                self.table_state.table_data_offset > 0 && self.table_mutation.staged_changes.is_empty(),
                            )
                            .tooltip("First page")
                            .show(ui)
                            .clicked()
                        && self.table_state.table_data_offset > 0
                        && self.table_mutation.staged_changes.is_empty()
                    {
                        self.table_state.table_data_offset = 0;
                        self.request_table_data();
                    }

                    ui.separator();

                    let prev_limit = self.table_state.table_data_limit;
                    let limit_label = format!("{} / page", self.table_state.table_data_limit);
                    egui::ComboBox::from_id_salt(("table-data-limit-select", table_name))
                        .selected_text(RichText::new(&limit_label).size(11.0).color(self.theme.text_secondary))
                        .width(90.0)
                        .show_ui(ui, |ui| {
                            for limit_opt in [50, 100, 250, 500, 1000] {
                                ui.selectable_value(
                                    &mut self.table_state.table_data_limit,
                                    limit_opt,
                                    format!("{limit_opt} / page"),
                                );
                            }
                        });
                    if self.table_state.table_data_limit != prev_limit {
                        self.reset_table_data_page();
                    }
                });
            });
        });
    }

    pub(crate) fn open_duplicate_row(&mut self, result: &UiQueryResult, row_index: usize) {
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to insert rows".to_owned();
            return;
        }
        let Some(info) = self.table_state.table_info.clone() else {
            self.feedback.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        let Some(row) = result.rows.get(row_index) else {
            return;
        };
        self.table_data.insert_row_values = table_editor_values::duplicate_row_values(&info, row);
        self.table_data.insert_row_error.clear();
        self.table_data.insert_row_open = true;
    }

    fn open_insert_row(&mut self) {
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to insert rows".to_owned();
            return;
        }
        let Some(info) = self.table_state.table_info.clone() else {
            self.feedback.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        self.table_data.insert_row_values = vec![String::new(); info.columns.len()];
        self.table_data.insert_row_error.clear();
        self.table_data.insert_row_open = true;
    }

    pub(crate) fn submit_insert_row(&mut self) {
        let Some(table) = self.schema_explorer.selected_table.clone() else {
            self.table_data.insert_row_error = "Select a table before inserting a row".to_owned();
            return;
        };
        let Some(info) = self.table_state.table_info.clone() else {
            self.table_data.insert_row_error = "Table structure is still loading".to_owned();
            return;
        };
        let (columns, values) =
            match table_editor_values::parse_insert_row_values(&info, &self.table_data.insert_row_values) {
                Ok(parsed) => parsed,
                Err(error) => {
                    self.table_data.insert_row_error = error;
                    return;
                }
            };
        self.table_mutation.staged_changes.ensure_target(&table);
        self.table_mutation.staged_changes.stage_insert(columns, values);
        self.table_data.insert_row_open = false;
        self.table_data.insert_row_error.clear();
        self.feedback.runtime_message = format!("Row staged for {}", table);
    }

    pub(super) fn draw_insert_row_dialog(&mut self, ctx: &egui::Context) {
        let Some(info) = self.table_state.table_info.clone() else {
            self.table_data.insert_row_open = false;
            return;
        };
        if self.table_data.insert_row_values.len() != info.columns.len() {
            self.table_data.insert_row_values = vec![String::new(); info.columns.len()];
        }
        let mut open = self.table_data.insert_row_open;
        let mut submit = false;
        let mut cancel = false;
        let title = format!("Insert Row · {}", info.name);
        let description = format!("Schema: {} · {} columns", info.schema, info.columns.len());

        Dialog::new(&mut open, &title, self.theme)
            .description(&description)
            .width(620.0)
            .id_salt("insert_row_modal_dialog")
            .show_ctx(ctx, |ui| {
                // Quick batch generation bar
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Fill values or generate sample mock data:")
                            .font(font_caption())
                            .color(self.theme.text_secondary),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if Button::new(self.theme)
                            .icon(Icon::RotateCcw)
                            .text("Clear All")
                            .size(ButtonSize::Sm)
                            .variant(ButtonVariant::Ghost)
                            .show(ui)
                            .on_hover_text("Clear all field inputs")
                            .clicked()
                        {
                            for val in &mut self.table_data.insert_row_values {
                                val.clear();
                            }
                        }

                        if Button::new(self.theme)
                            .icon(Icon::Sparkles)
                            .text("Fill Required")
                            .size(ButtonSize::Sm)
                            .variant(ButtonVariant::Secondary)
                            .show(ui)
                            .on_hover_text("Auto-generate sample values for empty required fields")
                            .clicked()
                        {
                            for (index, column) in info.columns.iter().enumerate() {
                                if !column.nullable
                                    && column.default.is_none()
                                    && self.table_data.insert_row_values[index].trim().is_empty()
                                {
                                    self.table_data.insert_row_values[index] =
                                        table_editor_values::generate_sample_value(&column.name, &column.data_type);
                                }
                            }
                        }

                        if Button::new(self.theme)
                            .icon(Icon::Wand2)
                            .text("Generate All")
                            .size(ButtonSize::Sm)
                            .variant(ButtonVariant::Secondary)
                            .show(ui)
                            .on_hover_text("Auto-generate sample mock data for all empty fields")
                            .clicked()
                        {
                            for (index, column) in info.columns.iter().enumerate() {
                                if self.table_data.insert_row_values[index].trim().is_empty() {
                                    self.table_data.insert_row_values[index] =
                                        table_editor_values::generate_sample_value(&column.name, &column.data_type);
                                }
                            }
                        }
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Columns list
                egui::ScrollArea::vertical()
                    .max_height(380.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (index, column) in info.columns.iter().enumerate() {
                            let is_fk = info
                                .foreign_keys
                                .iter()
                                .any(|fk| fk.from_columns.contains(&column.name));

                            Frame {
                                fill: self.theme.surface_panel,
                                stroke: Stroke::new(1.0, self.theme.border_subtle),
                                rounding: Rounding::same(8.0),
                                inner_margin: Margin::symmetric(12.0, 8.0),
                                ..Default::default()
                            }
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if column.is_primary_key {
                                        badge(ui, "PK", self.theme.accent_soft, self.theme.warning);
                                    } else if is_fk {
                                        badge(ui, "FK", self.theme.surface_active, self.theme.accent);
                                    }
                                    ui.label(
                                        RichText::new(&column.name)
                                            .font(font_ui_label())
                                            .strong()
                                            .color(self.theme.text_primary),
                                    );
                                    ui.label(
                                        RichText::new(&column.data_type)
                                            .monospace()
                                            .size(11.5)
                                            .color(self.theme.text_secondary),
                                    );
                                    if !column.nullable && column.default.is_none() {
                                        ui.label(
                                            RichText::new("*required").font(font_caption()).color(self.theme.danger),
                                        );
                                    } else if column.nullable {
                                        ui.label(
                                            RichText::new("nullable")
                                                .font(font_caption())
                                                .color(self.theme.text_muted),
                                        );
                                    }
                                    if let Some(ref def) = column.default {
                                        ui.label(
                                            RichText::new(format!("default: {def}"))
                                                .font(font_caption())
                                                .color(self.theme.text_muted),
                                        );
                                    }

                                    // Per-field actions
                                    if ColumnWritePolicy::read(column).is_writable() {
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if !self.table_data.insert_row_values[index].is_empty() {
                                                let clear_btn = Button::new(self.theme)
                                                    .icon(Icon::X)
                                                    .size(ButtonSize::IconSm)
                                                    .variant(ButtonVariant::Ghost)
                                                    .show(ui);
                                                if clear_btn.on_hover_text("Clear this field").clicked() {
                                                    self.table_data.insert_row_values[index].clear();
                                                }
                                            }

                                            if column.nullable && self.table_data.insert_row_values[index] != "NULL" {
                                                let null_btn = Button::new(self.theme)
                                                    .text("NULL")
                                                    .size(ButtonSize::Sm)
                                                    .variant(ButtonVariant::Ghost)
                                                    .show(ui);
                                                if null_btn.on_hover_text("Set value to literal NULL").clicked() {
                                                    self.table_data.insert_row_values[index] = "NULL".to_owned();
                                                }
                                            }

                                            let lower_dt = column.data_type.to_ascii_lowercase();
                                            let gen_text = if lower_dt.contains("uuid") {
                                                "UUID"
                                            } else if lower_dt.contains("time") || lower_dt.contains("date") {
                                                "Now"
                                            } else {
                                                "Gen"
                                            };
                                            let gen_btn = Button::new(self.theme)
                                                .icon(Icon::Wand2)
                                                .text(gen_text)
                                                .size(ButtonSize::Sm)
                                                .variant(ButtonVariant::Secondary)
                                                .show(ui);
                                            let tooltip =
                                                format!("Generate sample {} for {}", column.data_type, column.name);
                                            if gen_btn.on_hover_text(tooltip).clicked() {
                                                self.table_data.insert_row_values[index] =
                                                    table_editor_values::generate_sample_value(
                                                        &column.name,
                                                        &column.data_type,
                                                    );
                                            }
                                        });
                                    }
                                });

                                ui.add_space(4.0);

                                let is_null_val = self.table_data.insert_row_values[index]
                                    .trim()
                                    .eq_ignore_ascii_case("null");
                                let write_block = ColumnWritePolicy::read(column).write_block();
                                if let Some(block) = write_block {
                                    ui.label(
                                        RichText::new(format!("Read-only — {}", block.reason()))
                                            .font(font_caption())
                                            .color(self.theme.text_muted),
                                    );
                                } else {
                                    let placeholder = if column.default.is_some() {
                                        "Leave empty for DEFAULT, or enter value / click Gen..."
                                    } else if column.nullable {
                                        "Enter value, click NULL, or click Gen..."
                                    } else {
                                        "Enter value or click Gen..."
                                    };

                                    let val_ref = &mut self.table_data.insert_row_values[index];
                                    let edit = egui::TextEdit::singleline(val_ref)
                                        .hint_text(
                                            RichText::new(placeholder)
                                                .font(font_caption())
                                                .color(self.theme.text_muted),
                                        )
                                        .text_color(if is_null_val {
                                            self.theme.warning
                                        } else {
                                            self.theme.text_primary
                                        })
                                        .font(font_ui_label())
                                        .margin(Margin::symmetric(8.0, 6.0))
                                        .desired_width(ui.available_width());

                                    Frame {
                                        fill: self.theme.surface_elevated,
                                        stroke: Stroke::new(
                                            1.0,
                                            if is_null_val {
                                                self.theme.warning.linear_multiply(0.6)
                                            } else {
                                                self.theme.border_subtle
                                            },
                                        ),
                                        rounding: Rounding::same(6.0),
                                        ..Default::default()
                                    }
                                    .show(ui, |ui| {
                                        ui.add(edit);
                                    });
                                }
                            });
                            ui.add_space(6.0);
                        }
                    });

                if !self.table_data.insert_row_error.is_empty() {
                    ui.add_space(8.0);
                    Alert::new("Cannot Stage Insert", &self.table_data.insert_row_error, self.theme)
                        .variant(AlertVariant::Destructive)
                        .icon(Icon::AlertCircle)
                        .show(ui);
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .icon(Icon::Plus)
                        .text("Stage Insert")
                        .show(ui)
                        .clicked()
                    {
                        submit = true;
                    }

                    if Button::new(self.theme)
                        .text("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .show(ui)
                        .clicked()
                    {
                        cancel = true;
                    }
                });
            });

        if submit {
            self.submit_insert_row();
        }
        if cancel || !open {
            self.table_data.insert_row_open = false;
            self.table_data.insert_row_error.clear();
        }
    }

    pub(crate) fn submit_ddl(&mut self) {
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to execute DDL".to_owned();
            return;
        }
        if self.table_state.ddl_execution_request.is_some() {
            return;
        }
        let Some(connection) = self.active_connection().cloned() else {
            self.feedback.runtime_message = "Connect to a database before executing DDL".to_owned();
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let command = match self.table_state.execute_ddl_command(request_id, connection.id) {
            Ok(command) => command,
            Err(error) => {
                self.feedback.runtime_message = error;
                return;
            }
        };
        self.dispatch_command(command);
        self.table_state.ddl_execution_request = Some(request_id);
        self.table_state.ddl_execute_confirmation = false;
        self.feedback.runtime_message = "Executing DDL…".to_owned();
    }

    /// Loading / failed placeholder shown while the DDL is not available.
    pub(super) fn draw_table_ddl_placeholder(&mut self, ui: &mut egui::Ui, table_name: &str) {
        grid_frame(self.theme).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(28.0);
                let failed = self.table_state.table_ddl_error.as_deref();
                ui.label(icon_text(
                    if failed.is_some() {
                        Icon::TriangleAlert
                    } else {
                        Icon::Code2
                    },
                    "",
                    if failed.is_some() {
                        self.theme.warning
                    } else {
                        self.theme.accent
                    },
                ));
                ui.add_space(8.0);
                ui.label(
                    RichText::new(if failed.is_some() {
                        format!("DDL for {table_name} could not be loaded")
                    } else {
                        format!("Loading DDL for {table_name}…")
                    })
                    .strong()
                    .color(self.theme.text_primary),
                );
                if let Some(error) = failed {
                    ui.label(RichText::new(error).small().color(self.theme.text_secondary));
                }
                ui.add_space(28.0);
            });
        });
    }

    /// Editable CREATE SCRIPT card. Returns true when "Apply DDL" was pressed.
    pub(super) fn draw_ddl_script_card(&mut self, ui: &mut egui::Ui, writable: bool, ddl: &mut String) -> bool {
        let mut request_execution = false;
        card_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                section_label(ui, "CREATE SCRIPT", self.theme);
                ui.label(
                    RichText::new(if writable {
                        "Editable preview · execute DDL in the query editor (Open in Query)"
                    } else {
                        "Read-only preview"
                    })
                    .small()
                    .color(self.theme.text_muted),
                );
                // DDL execution from the table editor is not enabled in v0.1: the shipped
                // path is the query editor (`docs/release/known-limitations.md`). The
                // control stays visible and says why rather than looking live and doing
                // nothing — pressing it used to store the buffer back and stop there.
                if writable
                    && self.table_state.ddl_execution_request.is_none()
                    && Button::new(self.theme)
                        .icon(Icon::Play)
                        .text("Apply DDL")
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .enabled(DDL_APPLY_ENABLED)
                        .tooltip(if DDL_APPLY_ENABLED {
                            "Execute the DDL script"
                        } else {
                            "Not enabled in v0.1 — run DDL with Open in Query"
                        })
                        .show(ui)
                        .clicked()
                {
                    request_execution = true;
                }
            });
            ui.add_space(8.0);
            if let Some(error) = self.table_state.table_ddl_error.as_deref() {
                ui.label(
                    RichText::new(format!("DDL execution failed · {error}"))
                        .small()
                        .color(self.theme.danger),
                );
                ui.add_space(6.0);
            }
            editor_frame(self.theme).show(ui, |ui| {
                let response = ui.add(
                    TextEdit::multiline(&mut *ddl)
                        .font(FontId::monospace(13.0))
                        .desired_width(ui.available_width())
                        .desired_rows(18)
                        .interactive(writable),
                );
                if response.changed() {
                    self.table_state.table_ddl_error = None;
                }
            });
        });
        request_execution
    }

    /// Confirmation gate shown before the DDL is executed against the database.
    pub(super) fn draw_ddl_confirmation_card(&mut self, ui: &mut egui::Ui, impact: &str) {
        let Some(ddl) = self.table_state.table_ddl.as_deref() else {
            return;
        };
        let risk = if impact.contains("destructive") || impact.contains("drop") {
            RiskLevel::Destructive
        } else {
            RiskLevel::Medium
        };
        let approval = ExecutionApproval::new("Review DDL Migration", impact, ddl, risk, self.theme);
        if let Some(action) = approval.show(ui) {
            match action {
                ExecutionApprovalAction::Run => {
                    self.submit_ddl();
                }
                ExecutionApprovalAction::Cancel => {
                    self.table_state.ddl_execute_confirmation = false;
                }
                _ => {}
            }
        }
    }
    pub(crate) fn request_table_info(&mut self) {
        let (Some(connection_id), Some(table)) = (
            self.connection.lifecycle.active_connection_id().map(str::to_owned),
            self.schema_explorer.selected_table.clone(),
        ) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.table_state.table_info_request = Some(request_id);
        self.dispatch_command(self.table_state.load_info_command(
            request_id,
            connection_id,
            self.active_schema().to_owned(),
            table,
        ));
        self.feedback.runtime_message = "Loading table structure…".to_owned();
    }

    pub(crate) fn can_mutate_active_connection(&self) -> bool {
        table_editor_context::can_mutate_active_connection(&self.connection.catalog, &self.connection.lifecycle)
    }

    pub(crate) fn can_edit_table_rows(&self) -> bool {
        table_editor_context::can_edit_table_rows(
            &self.table_state,
            &self.connection.catalog,
            &self.connection.lifecycle,
        )
    }

    pub(crate) fn begin_data_cell_edit(
        &mut self,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
        cell: &UiCell,
    ) {
        if !self.can_edit_table_rows() {
            if self.can_mutate_active_connection() && !self.table_state.has_primary_key() {
                self.feedback.runtime_message = "Table has no primary key; safe row editing is unavailable.".to_owned();
            } else {
                self.feedback.runtime_message = "Connect with write access to edit rows".to_owned();
            }
            return;
        }
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to edit rows".to_owned();
            return;
        }
        if self.staged_row_deleted(result, row_index) {
            self.feedback.runtime_message = "Discard the staged delete before editing this row".to_owned();
            return;
        }
        if let Some(block) = result
            .columns
            .get(column_index)
            .and_then(|column| self.table_state.column_write_policy(&column.name))
            .and_then(|policy| policy.write_block())
        {
            // Still allow the advanced inspector for binary / blocked columns (#228).
            self.open_cell_inspector(result, row_index, column_index);
            self.feedback.runtime_message = block.reason().to_owned();
            return;
        }
        self.table_data.selected_cell = Some((row_index, column_index));
        self.table_data.selected_row = Some(row_index);
        self.table_data.selected_rows.clear();
        self.table_data.selected_rows.insert(row_index);
        self.table_data.selection_anchor_row = Some(row_index);
        self.table_data.selection_anchor_cell = Some((row_index, column_index));
        if let Some(identity) =
            self.table_data
                .row_identity_for_result(result, self.table_state.table_info.as_ref(), row_index)
        {
            self.table_mutation
                .clear_error_for_identity(&identity, Some(column_index));
        }
        self.table_data.data_editing_cell = Some((row_index, column_index));
        let should_expand = result.columns.get(column_index).is_some_and(|column| {
            let data_type = column.data_type.to_ascii_lowercase();
            data_type.contains("json")
                || matches!(cell, UiCell::Json(_))
                || matches!(cell, UiCell::Bytes(_))
                || matches!(cell, UiCell::Text(value) if value.chars().count() > 120)
        });
        if should_expand {
            self.open_cell_inspector(result, row_index, column_index);
        } else {
            self.table_data.expanded_data_editor = None;
            self.table_data.data_edit_error = None;
            self.table_data.data_edit_value = match cell {
                UiCell::Null => "NULL".to_owned(),
                _ => crate::cell_text(cell),
            };
        }
        self.feedback.copy_status.clear();
    }

    pub(crate) fn submit_data_cell_edit(
        &mut self,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
    ) -> bool {
        let Some(info) = self.table_state.table_info.clone() else {
            self.feedback.runtime_message = "Table structure is still loading".to_owned();
            return false;
        };
        let Some(column) = result.columns.get(column_index).map(|column| column.name.clone()) else {
            self.table_data.data_editing_cell = None;
            return false;
        };
        let Some(column_info) = info.columns.iter().find(|item| item.name == column) else {
            self.feedback.runtime_message = "The selected column is not present in the table metadata".to_owned();
            self.table_data.data_editing_cell = None;
            return false;
        };
        if let Some(block) = ColumnWritePolicy::read(column_info).write_block() {
            let error = block.reason().to_owned();
            self.table_data.data_edit_error = Some(error.clone());
            self.feedback.runtime_message = format!("{}: {error}", column_info.name);
            return false;
        }
        let value =
            match table_editor_values::parse_update_value(&self.table_data.data_edit_value, &column_info.data_type) {
                Ok(value) => value,
                Err(error) => {
                    self.feedback.runtime_message = format!("{}: {error}", column_info.name);
                    self.table_data.data_edit_error = Some(error);
                    return false;
                }
            };
        if matches!(value, UiCell::Null) && !column_info.nullable {
            let error = format!("{} is NOT NULL; enter a value instead", column_info.name);
            self.feedback.runtime_message = error.clone();
            self.table_data.data_edit_error = Some(error);
            return false;
        }
        let identity = match TableDataState::row_identity(result, &info, row_index) {
            Ok(identity) => identity,
            Err(error) => {
                self.feedback.runtime_message = error;
                self.table_data.data_edit_error = Some(self.feedback.runtime_message.clone());
                return false;
            }
        };
        let original = result
            .rows
            .get(row_index)
            .and_then(|row| row.get(column_index))
            .cloned()
            .ok_or_else(|| "The selected cell is no longer available".to_owned());
        let Ok(original) = original else {
            self.table_data.data_editing_cell = None;
            self.feedback.runtime_message = "The selected cell is no longer available".to_owned();
            self.table_data.data_edit_error = Some(self.feedback.runtime_message.clone());
            return false;
        };
        if let Some(table) = self.schema_explorer.selected_table.as_deref() {
            self.table_mutation.staged_changes.ensure_target(table);
        }
        self.table_mutation.staged_changes.stage_update(StagedChange::Update {
            identity,
            current_row_index: Some(row_index),
            column_index,
            column,
            data_type: column_info.data_type.clone(),
            original,
            value,
        });
        self.table_mutation.table_mutation_error = None;
        self.table_mutation.staged_apply_targets.clear();
        self.table_data.data_editing_cell = None;
        self.table_data.expanded_data_editor = None;
        self.table_data.data_edit_error = None;
        let counts = self.table_mutation.staged_changes.counts();
        self.feedback.runtime_message = format!(
            "Staged edit · {} pending (+{} ~{} -{})",
            counts.total(),
            counts.inserts,
            counts.updates,
            counts.deletes
        );
        true
    }

    pub(crate) fn request_delete_selected_data_rows(&mut self, result: &UiQueryResult) {
        if !self.can_edit_table_rows() {
            if self.can_mutate_active_connection() && !self.table_state.has_primary_key() {
                self.feedback.runtime_message = "Table has no primary key; safe row editing is unavailable.".to_owned();
            } else {
                self.feedback.runtime_message = "Connect with write access to delete rows".to_owned();
            }
            return;
        }
        if !self.can_mutate_active_connection() {
            self.feedback.runtime_message = "Connect with write access to delete rows".to_owned();
            return;
        }
        let row_indexes: Vec<usize> = if self.table_data.selected_rows.is_empty() {
            self.table_data.selected_row.into_iter().collect()
        } else {
            self.table_data.selected_rows.iter().copied().collect()
        };
        if row_indexes.is_empty() {
            self.feedback.runtime_message = "Select a row before deleting".to_owned();
            return;
        }
        let Some(info) = self.table_state.table_info.clone() else {
            self.feedback.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        if let Some(table) = self.schema_explorer.selected_table.as_deref() {
            self.table_mutation.staged_changes.ensure_target(table);
        }
        self.table_data.data_editing_cell = None;
        self.table_data.expanded_data_editor = None;
        self.table_data.data_edit_value.clear();
        self.table_data.data_edit_error = None;
        self.table_data.data_delete_confirmation = false;
        for row_index in row_indexes {
            if self.staged_row_deleted(result, row_index) {
                continue;
            }
            let identity = match TableDataState::row_identity(result, &info, row_index) {
                Ok(identity) => identity,
                Err(error) => {
                    self.feedback.runtime_message = error;
                    return;
                }
            };
            self.table_mutation.staged_changes.stage_delete(StagedChange::Delete {
                identity,
                current_row_index: Some(row_index),
            });
        }
        self.table_mutation.table_mutation_error = None;
        self.table_mutation.staged_apply_targets.clear();
        self.feedback.runtime_message = format!(
            "{} row(s) marked for deletion · {} staged change(s)",
            self.table_mutation
                .staged_changes
                .iter()
                .filter(|change| matches!(change, StagedChange::Delete { .. }))
                .count(),
            self.table_mutation.staged_changes.counts().total()
        );
    }

    pub(crate) fn staged_cell_value(
        &self,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
    ) -> Option<UiCell> {
        table_editor_context::staged_cell_value(
            &self.table_data,
            &self.table_state,
            &self.table_mutation,
            result,
            row_index,
            column_index,
        )
    }

    pub(crate) fn staged_row_deleted(&self, result: &UiQueryResult, row_index: usize) -> bool {
        table_editor_context::staged_row_deleted(
            &self.table_data,
            &self.table_state,
            &self.table_mutation,
            result,
            row_index,
        )
    }

    pub(crate) fn revert_staged_cell(&mut self, result: &UiQueryResult, row_index: usize, column_index: usize) {
        self.table_mutation_context()
            .revert_staged_cell(result, row_index, column_index);
    }

    pub(crate) fn revert_staged_row(&mut self, result: &UiQueryResult, row_index: usize) {
        self.table_mutation_context().revert_staged_row(result, row_index);
    }

    pub(crate) fn discard_staged_changes(&mut self) {
        if self.table_mutation_context().discard_staged_changes() {
            self.request_table_data();
        }
    }

    fn table_mutation_context(&mut self) -> table_editor_context::TableMutationContext<'_> {
        table_editor_context::TableMutationContext::new(
            &mut self.table_state,
            &mut self.table_data,
            &mut self.table_mutation,
            &mut self.feedback,
        )
    }

    fn discard_failed_mutation(&mut self, reload: bool) {
        let Some(target) = self
            .table_mutation
            .table_mutation_error
            .as_ref()
            .and_then(|failure| failure.target.clone())
        else {
            return;
        };
        match target {
            MutationTarget::Update { identity, columns, .. } => {
                for column_index in columns {
                    self.table_mutation.staged_changes.revert_cell(&identity, column_index);
                }
            }
            MutationTarget::Delete { identity, .. } => {
                self.table_mutation.staged_changes.revert_row(&identity);
            }
            MutationTarget::Insert => {}
        }
        self.table_mutation.table_mutation_error = None;
        self.table_mutation.table_mutation_retry_after_reload = false;
        self.table_mutation.table_mutation_retry_target = None;
        if reload {
            self.table_state.table_data_result = None;
            self.table_state.table_data_total_rows = None;
            self.table_state.table_data_error = None;
            self.request_table_data();
        }
    }

    fn reload_failed_mutation(&mut self) {
        let target = self
            .table_mutation
            .table_mutation_error
            .as_ref()
            .and_then(|failure| failure.target.clone());
        if let Some(MutationTarget::Update { identity, .. } | MutationTarget::Delete { identity, .. }) = target {
            self.request_table_row_reload(identity);
            return;
        }
        self.table_mutation.table_mutation_error = None;
        self.table_state.table_data_result = None;
        self.table_state.table_data_total_rows = None;
        self.table_state.table_data_error = None;
        self.request_table_data();
    }

    fn retry_failed_mutation_after_reload(&mut self) {
        let Some(target) = self
            .table_mutation
            .table_mutation_error
            .as_ref()
            .and_then(|failure| failure.target.clone())
        else {
            self.feedback.runtime_message = "This failure has no retryable mutation target".to_owned();
            return;
        };
        self.table_mutation.table_mutation_retry_target = Some(target);
        self.table_mutation.table_mutation_retry_after_reload = true;
        self.reload_failed_mutation();
    }

    pub(crate) fn request_table_row_reload(&mut self, identity: RowIdentity) {
        let (Some(connection_id), Some(table)) = (
            self.connection.lifecycle.active_connection_id().map(str::to_owned),
            self.schema_explorer.selected_table.clone(),
        ) else {
            self.feedback.runtime_message = "Connect to a database before reloading the row".to_owned();
            return;
        };
        let Some(info) = self.table_state.table_info.as_ref() else {
            self.feedback.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        let filters = match TableMutationState::row_reload_filters(info, &identity) {
            Ok(filters) => filters,
            Err(error) => {
                self.feedback.runtime_message = error;
                return;
            }
        };
        let request_id = self.task_bridge.next_request_id();
        self.table_state.table_row_reload_request = Some(request_id);
        self.table_state.table_row_reload_identity = Some(identity);
        self.dispatch_command(self.table_state.load_row_command(
            request_id,
            connection_id,
            self.active_schema().to_owned(),
            table,
            filters,
        ));
        self.feedback.runtime_message = "Reloading row from database…".to_owned();
    }

    pub(crate) fn draw_discard_changes_confirmation(&mut self, ui: &mut egui::Ui) {
        if !self.table_data.discard_changes_confirmation {
            return;
        }
        let counts = self.table_mutation.staged_changes.counts();
        let description = format!(
            "You have {} unapplied staged change(s) (+{} inserts, {} updates, {} deletes). Apply changes to database, discard them, or cancel navigation?",
            counts.total(),
            counts.inserts,
            counts.updates,
            counts.deletes
        );
        let mut open = true;
        let mut apply = false;
        let mut discard = false;
        let mut cancel = false;
        let theme = self.theme;
        Dialog::new(&mut open, "Unapplied Changes", theme)
            .description(&description)
            .id_salt("discard-table-changes")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if Button::new(theme)
                        .text("Apply Changes")
                        .variant(ButtonVariant::Default)
                        .show(ui)
                        .clicked()
                    {
                        apply = true;
                    }
                    if Button::new(theme)
                        .text("Discard Changes")
                        .variant(ButtonVariant::Destructive)
                        .show(ui)
                        .clicked()
                    {
                        discard = true;
                    }
                    if Button::new(theme)
                        .text("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .show(ui)
                        .clicked()
                    {
                        cancel = true;
                    }
                });
            });
        if apply {
            self.table_data.discard_changes_confirmation = false;
            self.apply_staged_changes();
        } else if discard {
            self.table_data.discard_changes_confirmation = false;
            let pending = self.workspace.pending_navigation_action.take();
            self.discard_staged_changes();
            if let Some(action) = pending {
                self.execute_pending_navigation(action);
            }
        } else if cancel || !open {
            self.table_data.discard_changes_confirmation = false;
            self.workspace.pending_navigation_action = None;
        }
    }

    fn draw_pending_changes_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.table_mutation.pending_changes_open {
            return;
        }
        let entries: Vec<StagedChange> = self.table_mutation.staged_changes.iter().cloned().collect();
        let mut groups: Vec<(Option<RowIdentity>, Option<u64>, Vec<StagedChange>)> = Vec::new();
        for entry in entries {
            let (identity, local_id) = match &entry {
                StagedChange::Update { identity, .. } | StagedChange::Delete { identity, .. } => {
                    (Some(identity.clone()), None)
                }
                StagedChange::Insert { local_id, .. } => (None, Some(*local_id)),
            };
            if let Some(group) = groups
                .iter_mut()
                .find(|(group_identity, group_local_id, _)| *group_identity == identity && *group_local_id == local_id)
            {
                group.2.push(entry);
            } else {
                groups.push((identity, local_id, vec![entry]));
            }
        }
        let mut open = true;
        enum PendingAction {
            Cell(RowIdentity, usize),
            Row(RowIdentity),
            Insert(u64),
        }
        let mut action = None;
        egui::Window::new("Pending changes")
            .open(&mut open)
            .resizable(true)
            .default_width(620.0)
            .show(ui.ctx(), |ui| {
                ui.label(
                    RichText::new("Review staged changes before Apply")
                        .small()
                        .color(self.theme.text_muted),
                );
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("{} row group(s)", groups.len()));
                    if ui.small_button("Discard All").clicked() {
                        self.table_data.discard_changes_confirmation = true;
                    }
                });
                egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                    for (identity, local_id, changes) in &groups {
                        let group_label = if let Some(identity) = identity {
                            format!(
                                "Row PK: [{}]",
                                identity
                                    .original_pk_values
                                    .iter()
                                    .map(crate::cell_text)
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        } else {
                            format!("New row · temporary identity #{}", local_id.unwrap_or_default())
                        };
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(group_label).strong());
                            if let Some(identity) = identity {
                                if ui.small_button("Revert Row").clicked() {
                                    action = Some(PendingAction::Row(identity.clone()));
                                }
                            } else if let Some(local_id) = local_id {
                                if ui.small_button("Remove Insert").clicked() {
                                    action = Some(PendingAction::Insert(*local_id));
                                }
                            }
                        });
                        for change in changes {
                            match change {
                                StagedChange::Update {
                                    identity,
                                    column_index,
                                    column,
                                    original,
                                    value,
                                    ..
                                } => {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(format!(
                                            "{column}: {} → {}",
                                            crate::cell_text(original),
                                            crate::cell_text(value)
                                        ));
                                        if ui.small_button("Revert Cell").clicked() {
                                            action = Some(PendingAction::Cell(identity.clone(), *column_index));
                                        }
                                    });
                                }
                                StagedChange::Delete { .. } => {
                                    ui.label("Delete row");
                                }
                                StagedChange::Insert { columns, .. } => {
                                    ui.label(format!("Insert ({})", columns.join(", ")));
                                }
                            }
                        }
                        ui.separator();
                    }
                });
            });
        if let Some(action) = action {
            match action {
                PendingAction::Cell(identity, column_index) => {
                    self.table_mutation.staged_changes.revert_cell(&identity, column_index);
                    self.table_mutation
                        .clear_error_for_identity(&identity, Some(column_index));
                }
                PendingAction::Row(identity) => {
                    self.table_mutation.staged_changes.revert_row(&identity);
                    self.table_mutation.clear_error_for_identity(&identity, None);
                }
                PendingAction::Insert(local_id) => {
                    self.table_mutation.staged_changes.remove_insert(local_id);
                    self.table_mutation.table_mutation_error = None;
                }
            }
        }
        if !open {
            self.table_mutation.pending_changes_open = false;
        }
    }

    pub(crate) fn conflict_keep_mine(&mut self) {
        let Some(failure) = self.table_mutation.table_mutation_error.as_ref() else {
            return;
        };
        let Some(target) = failure.target.clone() else {
            return;
        };
        match &target {
            MutationTarget::Update { identity, .. } => {
                if let Some(result) = self.table_state.table_data_result.as_ref() {
                    let col_map: std::collections::HashMap<&str, usize> = result
                        .columns
                        .iter()
                        .enumerate()
                        .map(|(i, col)| (col.name.as_str(), i))
                        .collect();
                    if let Some(row_index) = result
                        .rows
                        .iter()
                        .position(|row| table_events::row_matches_identity(row, &col_map, identity))
                    {
                        let current_db_row = &result.rows[row_index];
                        for (col_idx, _col) in result.columns.iter().enumerate() {
                            if let Some(val) = current_db_row.get(col_idx) {
                                self.table_mutation.staged_changes.update_original_baseline(
                                    identity,
                                    col_idx,
                                    val.clone(),
                                );
                            }
                        }
                    }
                }
                self.table_mutation.table_mutation_retry_target = Some(target);
                self.table_mutation.table_mutation_error = None;
                self.table_mutation.conflict_dialog_open = false;
                self.apply_staged_changes();
            }
            MutationTarget::Delete { identity: _, .. } => {
                self.table_mutation.table_mutation_retry_target = Some(target);
                self.table_mutation.table_mutation_error = None;
                self.table_mutation.conflict_dialog_open = false;
                self.apply_staged_changes();
            }
            MutationTarget::Insert => {
                self.table_mutation.table_mutation_error = None;
                self.table_mutation.conflict_dialog_open = false;
                self.apply_staged_changes();
            }
        }
    }

    pub(crate) fn conflict_use_database(&mut self) {
        let Some(failure) = self.table_mutation.table_mutation_error.as_ref() else {
            return;
        };
        let Some(target) = failure.target.clone() else {
            return;
        };
        match &target {
            MutationTarget::Update { identity, .. } => {
                self.table_mutation.staged_changes.revert_row(identity);
                self.table_mutation.table_mutation_error = None;
                self.table_mutation.conflict_dialog_open = false;
                self.feedback
                    .show_info_toast("Reverted local changes; adopted database values");
            }
            MutationTarget::Delete { identity, .. } => {
                self.table_mutation.staged_changes.revert_row(identity);
                self.table_mutation.table_mutation_error = None;
                self.table_mutation.conflict_dialog_open = false;
                self.feedback.show_info_toast("Reverted staged delete");
            }
            MutationTarget::Insert => {
                self.table_mutation.table_mutation_error = None;
                self.table_mutation.conflict_dialog_open = false;
            }
        }
    }

    pub(crate) fn draw_conflict_dialog(&mut self, ui: &mut egui::Ui, result: &UiQueryResult) {
        if !self.table_mutation.conflict_dialog_open {
            return;
        }
        let Some(failure) = self.table_mutation.table_mutation_error.as_ref() else {
            self.table_mutation.conflict_dialog_open = false;
            return;
        };
        let Some(target) = failure.target.clone() else {
            self.table_mutation.conflict_dialog_open = false;
            return;
        };

        let mut open = true;
        let mut close_dialog = false;
        let mut keep_mine = false;
        let mut use_database = false;
        let mut retry = false;
        let mut discard = false;
        let theme = self.theme;

        egui::Window::new("Row Conflict Resolution")
            .open(&mut open)
            .resizable(true)
            .default_width(680.0)
            .show(ui.ctx(), |ui| {
                ui.label(
                    RichText::new("A concurrent modification or deletion was detected for this row.")
                        .small()
                        .color(theme.text_secondary),
                );
                ui.add_space(4.0);

                match &target {
                    MutationTarget::Update {
                        identity,
                        current_row_index,
                        columns: changed_col_indices,
                    } => {
                        let pk_str = identity
                            .original_pk_columns
                            .iter()
                            .zip(&identity.original_pk_values)
                            .map(|(c, v)| format!("{c}={}", crate::cell_text(v)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        ui.label(
                            RichText::new(format!("Primary Key: [{pk_str}]"))
                                .strong()
                                .color(theme.text_primary),
                        );
                        ui.add_space(8.0);

                        let current_row = self
                            .current_row_index_for_identity(identity, *current_row_index)
                            .and_then(|idx| result.rows.get(idx));

                        egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                            egui::Grid::new("conflict_diff_grid")
                                .num_columns(5)
                                .spacing([12.0, 6.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Column").strong().color(theme.text_muted));
                                    ui.label(RichText::new("Original Baseline").strong().color(theme.text_muted));
                                    ui.label(RichText::new("Local Staged (Mine)").strong().color(theme.text_muted));
                                    ui.label(RichText::new("Database Current").strong().color(theme.text_muted));
                                    ui.label(RichText::new("Status").strong().color(theme.text_muted));
                                    ui.end_row();

                                    for &col_idx in changed_col_indices {
                                        let col_name = result.columns.get(col_idx).map_or("?", |c| c.name.as_str());
                                        let local_val =
                                            self.table_mutation.staged_changes.cell_value(identity, col_idx);
                                        let staged_entry =
                                            self.table_mutation.staged_changes.iter().find(|e| match e {
                                                StagedChange::Update {
                                                    identity: id,
                                                    column_index: idx,
                                                    ..
                                                } => id == identity && *idx == col_idx,
                                                _ => false,
                                            });
                                        let orig_val = match staged_entry {
                                            Some(StagedChange::Update { original, .. }) => Some(original),
                                            _ => None,
                                        };
                                        let db_val = current_row.and_then(|r| r.get(col_idx));

                                        let orig_text = orig_val.map_or("—".to_owned(), crate::cell_text);
                                        let local_text = local_val.as_ref().map_or("—".to_owned(), crate::cell_text);
                                        let db_text = db_val.map_or("—".to_owned(), crate::cell_text);

                                        let is_conflict = local_text != db_text && orig_text != db_text;

                                        ui.label(RichText::new(col_name).strong().color(theme.text_primary));
                                        ui.label(RichText::new(orig_text).color(theme.text_secondary));
                                        ui.label(RichText::new(local_text).color(theme.accent).strong());
                                        ui.label(
                                            RichText::new(db_text)
                                                .color(if is_conflict { theme.warning } else { theme.text_primary })
                                                .strong(),
                                        );
                                        if is_conflict {
                                            ui.label(RichText::new("Conflict").color(theme.warning).strong());
                                        } else {
                                            ui.label(RichText::new("Match / Clean").color(theme.text_muted));
                                        }
                                        ui.end_row();
                                    }
                                });
                        });

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.horizontal_wrapped(|ui| {
                            if Button::new(theme)
                                .text("Keep Mine (Overwrite DB)")
                                .variant(ButtonVariant::Default)
                                .show(ui)
                                .on_hover_text(
                                    "Keep local staged changes, refresh the row baseline, and retry applying",
                                )
                                .clicked()
                            {
                                keep_mine = true;
                            }
                            if Button::new(theme)
                                .text("Use Database (Discard Mine)")
                                .variant(ButtonVariant::Secondary)
                                .show(ui)
                                .on_hover_text(
                                    "Discard local staged changes for this row and adopt current database values",
                                )
                                .clicked()
                            {
                                use_database = true;
                            }
                            if Button::new(theme)
                                .text("Reload & Retry")
                                .variant(ButtonVariant::Secondary)
                                .show(ui)
                                .on_hover_text("Reload the latest row from database, then retry the mutation")
                                .clicked()
                            {
                                retry = true;
                            }
                            if Button::new(theme)
                                .text("Discard Local Change")
                                .variant(ButtonVariant::Ghost)
                                .show(ui)
                                .clicked()
                            {
                                discard = true;
                            }
                        });
                    }
                    MutationTarget::Delete { identity, .. } => {
                        let pk_str = identity
                            .original_pk_columns
                            .iter()
                            .zip(&identity.original_pk_values)
                            .map(|(c, v)| format!("{c}={}", crate::cell_text(v)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        ui.label(
                            RichText::new(format!("Primary Key: [{pk_str}]"))
                                .strong()
                                .color(theme.text_primary),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(
                                "Could not delete row because it was modified or already removed in the database.",
                            )
                            .color(theme.warning),
                        );
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if Button::new(theme)
                                .text("Discard Staged Delete")
                                .variant(ButtonVariant::Default)
                                .show(ui)
                                .clicked()
                            {
                                discard = true;
                            }
                            if Button::new(theme)
                                .text("Reload & Retry Delete")
                                .variant(ButtonVariant::Secondary)
                                .show(ui)
                                .clicked()
                            {
                                retry = true;
                            }
                        });
                    }
                    MutationTarget::Insert => {
                        ui.label(RichText::new("Insert conflict or constraint failure.").color(theme.warning));
                        ui.add_space(12.0);
                        if Button::new(theme)
                            .text("Close")
                            .variant(ButtonVariant::Default)
                            .show(ui)
                            .clicked()
                        {
                            close_dialog = true;
                        }
                    }
                }
            });

        if !open || close_dialog {
            self.table_mutation.conflict_dialog_open = false;
        } else if keep_mine {
            self.conflict_keep_mine();
        } else if use_database {
            self.conflict_use_database();
        } else if retry {
            self.table_mutation.conflict_dialog_open = false;
            self.retry_failed_mutation_after_reload();
        } else if discard {
            self.table_mutation.conflict_dialog_open = false;
            self.discard_failed_mutation(false);
        }
    }

    pub(crate) fn apply_staged_changes(&mut self) {
        if self.table_mutation.staged_apply_request.is_some() {
            return;
        }
        if self.table_mutation.staged_changes.is_empty() {
            return;
        }
        if self.table_data.data_edit_error.is_some() {
            self.feedback.runtime_message = "Fix the validation error before applying changes".to_owned();
            return;
        }
        let Some(connection) = self.active_connection().cloned() else {
            self.feedback.runtime_message = "Connect to a database before applying changes".to_owned();
            return;
        };
        let Some(table) = self.schema_explorer.selected_table.clone() else {
            self.feedback.runtime_message = "Select a table before applying changes".to_owned();
            return;
        };
        if let Some(target) = self.table_mutation.staged_changes.target_table() {
            if target != table {
                self.feedback.runtime_message = format!("Staged changes belong to table `{target}`, not `{table}`");
                return;
            }
        }
        let plan = self.table_mutation.build_apply_plan();
        if plan.changes.is_empty() {
            self.table_mutation.table_mutation_retry_after_reload = false;
            self.feedback.runtime_message = "The related staged change is no longer available".to_owned();
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        let command = UiCommand::ApplyTableChanges {
            request_id,
            connection_id: connection.id,
            schema: self.active_schema().to_owned(),
            table,
            changes: plan.changes,
        };
        if self.dispatch_command(command) {
            self.table_mutation.staged_apply_request = Some(request_id);
            self.table_mutation.table_mutation_request = Some(request_id);
            self.table_mutation.staged_apply_targets = plan.targets;
            self.table_mutation.table_mutation_error = None;
            let counts = self.table_mutation.staged_changes.counts();
            self.feedback.runtime_message = format!(
                "Applying {} changes in one transaction (+{} ~{} -{})…",
                counts.total(),
                counts.inserts,
                counts.updates,
                counts.deletes
            );
        } else {
            self.feedback.runtime_message = "Could not send staged change to runtime".to_owned();
        }
    }

    pub(crate) fn staged_apply_completed(&mut self) {
        self.table_mutation.staged_apply_request = None;
        self.table_mutation.table_mutation_request = None;
        self.table_mutation.table_mutation_retry_after_reload = false;
        self.table_mutation.table_mutation_retry_target = None;
        self.table_mutation.staged_changes.clear();
        self.table_mutation.staged_apply_targets.clear();
        self.table_mutation.table_mutation_error = None;
        self.feedback.runtime_message = "All staged changes applied".to_owned();
        self.feedback
            .show_success_toast("All staged changes applied successfully");
        if let Some(action) = self.workspace.pending_navigation_action.take() {
            self.execute_pending_navigation(action);
            return;
        }
        self.table_state.table_data_result = None;
        self.table_state.table_data_total_rows = None;
        self.table_state.table_data_error = None;
        self.request_table_data();
    }

    pub(crate) fn staged_apply_failed(&mut self, statement_index: usize, code: &str, message: &str, rolled_back: bool) {
        self.workspace.pending_navigation_action = None;
        self.table_mutation.staged_apply_request = None;
        self.table_mutation.table_mutation_request = None;
        self.table_mutation.table_mutation_retry_after_reload = false;
        self.table_mutation.table_mutation_retry_target = None;
        let target = self.table_mutation.staged_apply_targets.get(statement_index).cloned();
        let has_target = target.is_some();
        if let Some(target) = target.as_ref() {
            match target {
                MutationTarget::Update {
                    identity,
                    current_row_index,
                    columns,
                } => {
                    let row_index = self.current_row_index_for_identity(identity, *current_row_index);
                    self.table_data.selected_row = row_index;
                    self.table_data.selected_rows.clear();
                    if let Some(row_index) = row_index {
                        self.table_data.selected_rows.insert(row_index);
                        if let Some(column_index) = columns.first().copied() {
                            self.table_data.selected_cell = Some((row_index, column_index));
                            self.table_data.selection_anchor_cell = Some((row_index, column_index));
                        }
                    }
                }
                MutationTarget::Delete {
                    identity,
                    current_row_index,
                } => {
                    let row_index = self.current_row_index_for_identity(identity, *current_row_index);
                    self.table_data.selected_row = row_index;
                    self.table_data.selected_rows.clear();
                    if let Some(row_index) = row_index {
                        self.table_data.selected_rows.insert(row_index);
                    }
                    self.table_data.selected_cell = None;
                    self.table_data.selection_anchor_cell = None;
                }
                MutationTarget::Insert => {}
            }
        }
        let normalized_code = match code {
            "INTERNAL_ERROR" => "INTERNAL",
            "CONSTRAINT_VIOLATION" => "CONSTRAINT_VIOLATION",
            "VALIDATION_ERROR" => "VALIDATION_ERROR",
            "CONFLICT" => "CONFLICT",
            _ => "INTERNAL",
        };
        let display_message = if normalized_code == "CONFLICT" {
            format!("This row changed or was deleted in the database. Database: {message}")
        } else {
            message.to_owned()
        };
        let mutation_failure = MutationFailure {
            statement_index,
            target: target.clone(),
            code: normalized_code.to_owned(),
            message: display_message.clone(),
            rolled_back,
        };
        self.table_mutation.table_mutation_error = Some(mutation_failure);
        if normalized_code == "CONFLICT" {
            self.table_mutation.conflict_dialog_open = true;
            if let Some(MutationTarget::Update { identity, .. } | MutationTarget::Delete { identity, .. }) =
                target.as_ref()
            {
                self.request_table_row_reload(identity.clone());
            }
        }
        let outcome = if rolled_back {
            "transaction rolled back"
        } else {
            "transaction outcome is unknown"
        };
        let formatted = if has_target {
            format!(
                "Staged change #{} failed · {outcome} · {display_message}",
                statement_index.saturating_add(1)
            )
        } else {
            format!("Staged changes failed · {outcome} · {display_message}")
        };
        self.feedback.runtime_message = formatted.clone();
        self.feedback.show_error_toast(formatted);
    }

    fn current_row_index_for_identity(&self, identity: &RowIdentity, fallback: Option<usize>) -> Option<usize> {
        self.table_state
            .table_data_result
            .as_ref()
            .and_then(|result| {
                result.rows.iter().enumerate().find_map(|(row_index, _)| {
                    (self
                        .table_data
                        .row_identity_for_result(result, self.table_state.table_info.as_ref(), row_index)
                        .as_ref()
                        == Some(identity))
                    .then_some(row_index)
                })
            })
            .or(fallback)
    }

    pub(crate) fn request_table_ddl(&mut self) {
        let (Some(connection_id), Some(table)) = (
            self.connection.lifecycle.active_connection_id().map(str::to_owned),
            self.schema_explorer.selected_table.clone(),
        ) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.table_state.table_ddl_request = Some(request_id);
        self.dispatch_command(self.table_state.load_ddl_command(
            request_id,
            connection_id,
            self.active_schema().to_owned(),
            table,
        ));
        self.feedback.runtime_message = "Loading table DDL…".to_owned();
    }

    pub(crate) fn request_table_data(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let table = self.schema_explorer.selected_table.clone().or_else(|| {
            match self.schema_explorer.selected_schema_object.as_ref() {
                Some(SchemaObjectSelection::View(name)) => Some(name.clone()),
                _ => None,
            }
        });
        let Some(table) = table else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.table_state.table_data_request = Some(request_id);
        self.dispatch_command(self.table_state.load_data_command(
            request_id,
            connection_id,
            self.active_schema().to_owned(),
            table,
            self.table_state.table_data_filters.clone(),
            self.table_state.table_data_sorts.clone(),
        ));
        self.feedback.runtime_message = "Loading table data…".to_owned();
    }

    pub(crate) fn commit_table_filter_draft(&mut self) {
        if !self.table_mutation.staged_changes.is_empty() {
            self.feedback.runtime_message = "Apply or discard staged changes before changing filters".to_owned();
            return;
        }
        let column = self.table_state.table_data_filter_column.trim();
        if column.is_empty() {
            return;
        }
        let is_null_operator = matches!(
            self.table_state.table_data_filter_operator,
            UiTableFilterOperator::IsNull | UiTableFilterOperator::IsNotNull
        );
        let data_type = self
            .table_state
            .table_info
            .as_ref()
            .and_then(|info| info.columns.iter().find(|item| item.name == column))
            .map(|item| item.data_type.clone())
            .unwrap_or_else(|| "text".to_owned());
        if !Self::filter_operator_supported(&data_type, &self.table_state.table_data_filter_operator) {
            self.feedback.runtime_message = format!("That filter operator is not supported for {data_type}");
            return;
        }
        if !is_null_operator
            && self.table_state.table_data_filter_value.is_empty()
            && !table_editor_values::is_text_type(&data_type.to_ascii_lowercase())
        {
            self.feedback.runtime_message = "Enter a filter value first".to_owned();
            return;
        }
        if !is_null_operator {
            if let Err(error) =
                table_editor_values::parse_update_value(&self.table_state.table_data_filter_value, &data_type)
            {
                self.feedback.runtime_message = format!("Invalid filter for {column}: {error}");
                return;
            }
        }
        let filter = UiTableDataFilter {
            column: column.to_owned(),
            data_type,
            operator: self.table_state.table_data_filter_operator.clone(),
            value: if is_null_operator {
                String::new()
            } else {
                self.table_state.table_data_filter_value.clone()
            },
        };
        if let Some(index) = self.table_state.table_data_filter_editing.take() {
            if let Some(existing) = self.table_state.table_data_filters.get_mut(index) {
                *existing = filter;
            } else {
                self.table_state.table_data_filters.push(filter);
            }
        } else {
            self.table_state.table_data_filters.push(filter);
        }
        self.table_state.table_data_offset = 0;
        self.request_table_data();
    }

    pub(crate) fn remove_table_filter(&mut self, index: usize) {
        if !self.table_mutation.staged_changes.is_empty() {
            self.feedback.runtime_message = "Apply or discard staged changes before changing filters".to_owned();
            return;
        }
        if index < self.table_state.table_data_filters.len() {
            self.table_state.table_data_filters.remove(index);
            self.table_state.table_data_filter_editing = match self.table_state.table_data_filter_editing {
                Some(editing) if editing == index => None,
                Some(editing) if editing > index => Some(editing - 1),
                other => other,
            };
            self.table_state.table_data_offset = 0;
            self.request_table_data();
        }
    }

    pub(crate) fn clear_table_filters(&mut self) {
        if !self.table_mutation.staged_changes.is_empty() {
            self.feedback.runtime_message = "Apply or discard staged changes before changing filters".to_owned();
            return;
        }
        self.table_state.table_data_filters.clear();
        self.table_state.table_data_filter_editing = None;
        self.table_state.table_data_offset = 0;
        self.request_table_data();
    }

    pub(crate) fn reset_table_data_page(&mut self) {
        self.table_state.table_data_offset = 0;
        self.request_table_data();
    }

    pub(crate) fn filter_operator_supported(data_type: &str, operator: &UiTableFilterOperator) -> bool {
        Self::filter_operator_options(data_type)
            .iter()
            .any(|(candidate, _)| candidate == operator)
    }

    fn filter_operator_options(data_type: &str) -> Vec<(UiTableFilterOperator, &'static str)> {
        let normalized = data_type.to_ascii_lowercase();
        let mut operators = if table_editor_values::is_text_type(&normalized) {
            vec![
                (UiTableFilterOperator::Equals, "equals"),
                (UiTableFilterOperator::NotEquals, "not equals"),
                (UiTableFilterOperator::Contains, "contains"),
                (UiTableFilterOperator::StartsWith, "starts with"),
                (UiTableFilterOperator::EndsWith, "ends with"),
            ]
        } else if normalized.contains("bool") {
            vec![
                (UiTableFilterOperator::Equals, "equals"),
                (UiTableFilterOperator::NotEquals, "not equals"),
            ]
        } else if normalized.contains("int")
            || normalized.contains("serial")
            || normalized.contains("real")
            || normalized.contains("float")
            || normalized.contains("double")
            || table_editor_values::is_decimal_type(&normalized)
            || normalized == "date"
            || normalized.starts_with("time")
            || normalized.contains("timestamp")
        {
            vec![
                (UiTableFilterOperator::Equals, "equals"),
                (UiTableFilterOperator::NotEquals, "not equals"),
                (UiTableFilterOperator::GreaterThan, ">"),
                (UiTableFilterOperator::GreaterThanOrEqual, ">="),
                (UiTableFilterOperator::LessThan, "<"),
                (UiTableFilterOperator::LessThanOrEqual, "<="),
            ]
        } else {
            vec![
                (UiTableFilterOperator::Equals, "equals"),
                (UiTableFilterOperator::NotEquals, "not equals"),
            ]
        };
        operators.push((UiTableFilterOperator::IsNull, "IS NULL"));
        operators.push((UiTableFilterOperator::IsNotNull, "IS NOT NULL"));
        operators
    }

    pub(crate) fn reload_table_data_from_start(&mut self) {
        if !self.table_mutation.staged_changes.is_empty() {
            self.feedback.runtime_message = "Apply or discard staged changes before reloading".to_owned();
            return;
        }
        self.table_state.table_data_offset = 0;
        self.table_state.table_data_result = None;
        self.table_state.table_data_total_rows = None;
        self.table_state.table_data_error = None;
        self.request_table_data();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sample_value_types() {
        let uuid_val = table_editor_values::generate_sample_value("id", "uuid");
        assert!(uuid::Uuid::parse_str(&uuid_val).is_ok());

        let time_val = table_editor_values::generate_sample_value("created_at", "timestamptz");
        assert!(chrono::DateTime::parse_from_rfc3339(&time_val).is_ok());

        let date_val = table_editor_values::generate_sample_value("birth_date", "date");
        assert_eq!(date_val.len(), 10);

        let email_val = table_editor_values::generate_sample_value("user_email", "varchar");
        assert!(email_val.contains('@'));

        let bool_val = table_editor_values::generate_sample_value("is_active", "boolean");
        assert!(bool_val == "true" || bool_val == "false");
    }

    #[test]
    fn test_parse_insert_value_uuid() {
        let valid_uuid = "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11";
        let parsed = table_editor_values::parse_insert_value(valid_uuid, "uuid");
        assert_eq!(parsed, Ok(Some(UiCell::Text(valid_uuid.to_owned()))));

        let invalid_uuid = "not-a-uuid";
        assert!(table_editor_values::parse_insert_value(invalid_uuid, "uuid").is_err());
    }

    #[test]
    fn update_parser_preserves_null_empty_and_whitespace_text() {
        assert_eq!(
            table_editor_values::parse_update_value("NULL", "text"),
            Ok(UiCell::Null)
        );
        assert_eq!(
            table_editor_values::parse_update_value("", "text"),
            Ok(UiCell::Text(String::new()))
        );
        assert_eq!(
            table_editor_values::parse_update_value("   ", "text"),
            Ok(UiCell::Text("   ".to_owned()))
        );
        assert!(table_editor_values::parse_update_value("", "integer").is_err());
    }

    #[test]
    fn update_parser_validates_temporal_json_and_binary_values() {
        assert!(table_editor_values::parse_update_value("2026-09-12", "date").is_ok());
        assert!(table_editor_values::parse_update_value("12:30:45", "time").is_ok());
        assert!(table_editor_values::parse_update_value("2026-09-12T12:30:45Z", "timestamptz").is_ok());
        assert!(table_editor_values::parse_update_value("not-a-time", "time").is_err());
        assert!(table_editor_values::parse_update_value("{\"ok\":true}", "jsonb").is_ok());
        assert!(table_editor_values::parse_update_value("not-json", "jsonb").is_err());
        assert!(table_editor_values::parse_update_value("deadbeef", "bytea").is_err());
    }

    #[test]
    fn filter_operators_follow_column_type_and_keep_null_operators_universally() {
        assert!(DbProApp::filter_operator_supported(
            "numeric(20,4)",
            &UiTableFilterOperator::GreaterThan
        ));
        assert!(!DbProApp::filter_operator_supported(
            "numeric(20,4)",
            &UiTableFilterOperator::Contains
        ));
        assert!(DbProApp::filter_operator_supported(
            "uuid",
            &UiTableFilterOperator::Equals
        ));
        assert!(!DbProApp::filter_operator_supported(
            "uuid",
            &UiTableFilterOperator::StartsWith
        ));
        assert!(DbProApp::filter_operator_supported(
            "boolean",
            &UiTableFilterOperator::IsNotNull
        ));
    }

    #[test]
    fn filter_draft_supports_multiple_and_editable_same_column_filters() {
        let mut app = DbProApp {
            table_state: TableState {
                table_info: Some(UiTableInfo {
                    schema: "public".to_owned(),
                    name: "orders".to_owned(),
                    row_count: None,
                    columns: vec![crate::UiTableColumn {
                        name: "amount".to_owned(),
                        data_type: "numeric(12,2)".to_owned(),
                        nullable: false,
                        default: None,
                        is_primary_key: false,
                        ..Default::default()
                    }],
                    primary_key: None,
                    indexes: Vec::new(),
                    foreign_keys: Vec::new(),
                    check_constraints: Vec::new(),
                    dependencies: Vec::new(),
                }),
                table_data_filter_column: "amount".to_owned(),
                table_data_filter_operator: UiTableFilterOperator::GreaterThan,
                table_data_filter_value: "10.00".to_owned(),
                ..Default::default()
            },
            ..Default::default()
        };
        app.commit_table_filter_draft();
        app.table_state.table_data_filter_operator = UiTableFilterOperator::LessThan;
        app.table_state.table_data_filter_value = "20.00".to_owned();
        app.commit_table_filter_draft();

        assert_eq!(app.table_state.table_data_filters.len(), 2);
        app.table_state.table_data_filter_editing = Some(0);
        app.table_state.table_data_filter_value = "11.00".to_owned();
        app.commit_table_filter_draft();
        assert_eq!(app.table_state.table_data_filters[0].value, "11.00");
        assert_eq!(app.table_state.table_data_filters.len(), 2);
    }
}
