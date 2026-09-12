use super::*;
use crate::components::alert::{Alert, AlertVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use egui::{FontFamily, FontId, Frame, Margin, Rounding, Stroke};
use lucide_icons::Icon;

/// Paging state for the table data editor toolbar.
struct TableDataPaging {
    page_range: String,
    total_rows: u64,
    has_next: bool,
    has_previous: bool,
}

impl DbProApp {
    pub(crate) fn draw_table_data(&mut self, ui: &mut egui::Ui, table_name: &str) {
        let Some(result) = self.table_data_result.clone() else {
            self.draw_table_data_placeholder(ui, table_name);
            return;
        };

        let can_mutate = self.can_mutate_active_connection();
        let total_rows = self.table_data_total_rows.unwrap_or(result.row_count);
        let paging = TableDataPaging {
            page_range: if total_rows > 0 {
                let start = self.table_data_offset + 1;
                let end = (self.table_data_offset + result.row_count).min(total_rows);
                format!("{start}–{end} of {total_rows}")
            } else {
                "0 rows".to_owned()
            },
            total_rows,
            has_next: self.table_data_offset.saturating_add(self.table_data_limit) < total_rows,
            has_previous: self.table_data_offset > 0,
        };

        self.draw_table_data_unified_toolbar(ui, table_name, &result, can_mutate, &paging);
        ui.add_space(4.0);

        let data_width = ui.max_rect().width();
        grid_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((data_width - 24.0).max(0.0));
            self.draw_result_grid(ui, &result);
        });
        self.draw_discard_changes_confirmation(ui);
    }

    /// Loading / failed placeholder shown while table data is not available.
    fn draw_table_data_placeholder(&mut self, ui: &mut egui::Ui, table_name: &str) {
        grid_frame(self.theme).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(28.0);
                let failed = self.table_data_error.as_deref();
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
                        self.table_data_error = None;
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
                    if self.staged_changes.is_empty() {
                        self.request_table_data();
                    } else {
                        self.runtime_message = "Apply or discard staged changes before refreshing".to_owned();
                    }
                }

                if can_mutate {
                    ui.separator();
                    if compact_button_with_icon(ui, Icon::Plus, "Add Row", self.theme)
                        .on_hover_text("Insert new row")
                        .clicked()
                    {
                        self.open_insert_row();
                    }

                    if !self.staged_changes.is_empty() {
                        ui.separator();
                        let counts = self.staged_changes.counts();
                        crate::components::badge::Badge::new(
                            &format!(
                                "{} pending · +{} ~{} -{}",
                                counts.total(),
                                counts.inserts,
                                counts.updates,
                                counts.deletes
                            ),
                            self.theme,
                        )
                        .variant(crate::components::badge::BadgeVariant::Warning)
                        .compact(true)
                        .show(ui);
                        if compact_button_with_icon(ui, Icon::Check, "Apply", self.theme)
                            .on_hover_text("Apply all staged changes (Cmd/Ctrl+S)")
                            .clicked()
                        {
                            self.apply_staged_changes();
                        }
                        if compact_button_with_icon(ui, Icon::Undo2, "Discard", self.theme)
                            .on_hover_text("Discard all staged changes (Cmd/Ctrl+Z)")
                            .clicked()
                        {
                            if self.staged_changes.counts().total() > 1 {
                                self.discard_changes_confirmation = true;
                            } else {
                                self.discard_staged_changes();
                            }
                        }
                    }
                } else if self.connected {
                    ui.separator();
                    ui.label(
                        RichText::new("Read-only")
                            .font(font_caption())
                            .color(self.theme.warning),
                    );
                }

                if self.selected_rows.len() > 1 {
                    crate::components::badge::Badge::new(
                        &format!("{} rows selected", self.selected_rows.len()),
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
                            let col_label = if self.table_data_filter_column.is_empty() {
                                "All columns".to_owned()
                            } else {
                                self.table_data_filter_column.clone()
                            };
                            let col_color = if self.table_data_filter_column.is_empty() {
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
                                            &mut self.table_data_filter_column,
                                            String::new(),
                                            "All columns (Instant)",
                                        )
                                        .clicked()
                                    {
                                        self.table_data_filter_operator = UiTableFilterOperator::default();
                                        self.grid_filter = self.table_data_filter_value.clone();
                                    }
                                    for col in &column_names {
                                        ui.selectable_value(
                                            &mut self.table_data_filter_column,
                                            col.clone(),
                                            col.as_str(),
                                        );
                                    }
                                });

                            ui.add(egui::Separator::default().vertical());

                            let operator_label = match self.table_data_filter_operator {
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
                                    for (operator, label) in [
                                        (UiTableFilterOperator::Equals, "equals"),
                                        (UiTableFilterOperator::NotEquals, "not equals"),
                                        (UiTableFilterOperator::Contains, "contains"),
                                        (UiTableFilterOperator::StartsWith, "starts with"),
                                        (UiTableFilterOperator::EndsWith, "ends with"),
                                        (UiTableFilterOperator::GreaterThan, ">"),
                                        (UiTableFilterOperator::GreaterThanOrEqual, ">="),
                                        (UiTableFilterOperator::LessThan, "<"),
                                        (UiTableFilterOperator::LessThanOrEqual, "<="),
                                        (UiTableFilterOperator::IsNull, "IS NULL"),
                                        (UiTableFilterOperator::IsNotNull, "IS NOT NULL"),
                                    ] {
                                        if ui
                                            .selectable_value(&mut self.table_data_filter_operator, operator, label)
                                            .clicked()
                                        {
                                            operator_changed = true;
                                        }
                                    }
                                });

                            // Search / Filter Input
                            let is_all_cols = self.table_data_filter_column.is_empty();
                            let is_null_operator = matches!(
                                self.table_data_filter_operator,
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
                                &mut self.grid_filter
                            } else {
                                &mut self.table_data_filter_value
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
                                    self.table_data_filter_value.clear();
                                }
                                self.commit_table_filter_draft();
                            }

                            // Trailing clear button
                            let has_text = if is_all_cols {
                                !self.grid_filter.is_empty()
                            } else {
                                !self.table_data_filter_value.is_empty()
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
                                    self.grid_filter.clear();
                                } else {
                                    let filter_column = self.table_data_filter_column.clone();
                                    self.table_data_filters.retain(|filter| filter.column != filter_column);
                                    self.table_data_filter_value.clear();
                                    self.table_data_filter_operator = UiTableFilterOperator::default();
                                    self.table_data_offset = 0;
                                    self.request_table_data();
                                }
                            }
                        });
                    });

                    if !self.table_data_filters.is_empty() {
                        let mut remove_filter = None;
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("Filters:").small().color(self.theme.text_muted));
                            for (index, filter) in self.table_data_filters.iter().enumerate() {
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
                                    .small_button(format!("{} {}{}  ×", filter.column, operator, value))
                                    .on_hover_text("Remove filter")
                                    .clicked()
                                {
                                    remove_filter = Some(index);
                                }
                            }
                            if ui.small_button("Clear all").clicked() {
                                remove_filter = Some(usize::MAX);
                            }
                        });
                        if let Some(index) = remove_filter {
                            if index == usize::MAX {
                                self.clear_table_filters();
                            } else {
                                self.remove_table_filter(index);
                            }
                        }
                    }

                    // Compact Sort Selector
                    let sort_active = self.table_data_sort_column.is_some() || self.grid_sort_column.is_some();
                    let sort_label = if let Some(ref col) = self.table_data_sort_column {
                        format!("Sort: {col} {}", if self.table_data_sort_desc { "↓" } else { "↑" })
                    } else if let Some(idx) = self.grid_sort_column {
                        if let Some(col) = column_names.get(idx) {
                            format!("Sort: {col} {}", if self.grid_sort_desc { "↓" } else { "↑" })
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
                                self.table_data_sort_column = None;
                                self.grid_sort_column = None;
                                self.reload_table_data_from_start();
                            }
                            for col in &column_names {
                                let is_sel = self.table_data_sort_column.as_deref() == Some(col.as_str());
                                if ui.selectable_label(is_sel, col.as_str()).clicked() {
                                    if is_sel {
                                        self.table_data_sort_desc = !self.table_data_sort_desc;
                                    } else {
                                        self.table_data_sort_column = Some(col.clone());
                                        self.table_data_sort_desc = false;
                                    }
                                    self.reload_table_data_from_start();
                                }
                            }
                        });
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if paging.total_rows > 0
                        && compact_button_with_icon(ui, Icon::ChevronsLeft, "First", self.theme)
                            .on_hover_text("First page")
                            .clicked()
                        && self.table_data_offset > 0
                        && self.staged_changes.is_empty()
                    {
                        self.table_data_offset = 0;
                        self.request_table_data();
                    }

                    if compact_icon_button_enabled(ui, Icon::ChevronRight, paging.has_next, self.theme)
                        .on_hover_text("Next page")
                        .clicked()
                        && self.staged_changes.is_empty()
                    {
                        self.table_data_offset = self.table_data_offset.saturating_add(self.table_data_limit);
                        self.request_table_data();
                    }

                    ui.label(
                        RichText::new(&paging.page_range)
                            .font(font_caption())
                            .color(self.theme.text_secondary),
                    );

                    if compact_icon_button_enabled(ui, Icon::ChevronLeft, paging.has_previous, self.theme)
                        .on_hover_text("Previous page")
                        .clicked()
                        && self.staged_changes.is_empty()
                    {
                        self.table_data_offset = self.table_data_offset.saturating_sub(self.table_data_limit);
                        self.request_table_data();
                    }

                    if paging.total_rows > 0
                        && compact_button_with_icon(ui, Icon::ChevronsRight, "Last", self.theme)
                            .on_hover_text("Last page")
                            .clicked()
                        && paging.has_next
                        && self.staged_changes.is_empty()
                    {
                        let last_page = paging.total_rows.saturating_sub(1) / self.table_data_limit;
                        self.table_data_offset = last_page.saturating_mul(self.table_data_limit);
                        self.request_table_data();
                    }

                    let prev_limit = self.table_data_limit;
                    let limit_label = format!("{} / page", self.table_data_limit);
                    egui::ComboBox::from_id_salt(("table-data-limit-select", table_name))
                        .selected_text(RichText::new(&limit_label).size(11.0).color(self.theme.text_secondary))
                        .width(90.0)
                        .show_ui(ui, |ui| {
                            for limit_opt in [50, 100, 250, 500, 1000] {
                                ui.selectable_value(
                                    &mut self.table_data_limit,
                                    limit_opt,
                                    format!("{limit_opt} / page"),
                                );
                            }
                        });
                    if self.table_data_limit != prev_limit {
                        self.reset_table_data_page();
                    }
                });
            });
        });
    }

    pub(crate) fn open_duplicate_row(&mut self, result: &UiQueryResult, row_index: usize) {
        if !self.can_mutate_active_connection() {
            self.runtime_message = "Connect with write access to insert rows".to_owned();
            return;
        }
        let Some(info) = self.table_info.clone() else {
            self.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        let Some(row) = result.rows.get(row_index) else {
            return;
        };
        self.insert_row_values = info
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                if col.is_primary_key {
                    String::new()
                } else if let Some(cell) = row.get(i) {
                    match cell {
                        UiCell::Null => String::new(),
                        _ => crate::cell_text(cell),
                    }
                } else {
                    String::new()
                }
            })
            .collect();
        self.insert_row_error.clear();
        self.insert_row_open = true;
    }

    fn open_insert_row(&mut self) {
        if !self.can_mutate_active_connection() {
            self.runtime_message = "Connect with write access to insert rows".to_owned();
            return;
        }
        let Some(info) = self.table_info.clone() else {
            self.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        self.insert_row_values = vec![String::new(); info.columns.len()];
        self.insert_row_error.clear();
        self.insert_row_open = true;
    }

    pub(crate) fn generate_sample_value(column_name: &str, data_type: &str) -> String {
        let lower_type = data_type.to_ascii_lowercase();
        let lower_name = column_name.to_ascii_lowercase();
        let rand_num = (rand::random::<u32>() % 9000) + 1000;

        if lower_type.contains("uuid") || lower_type.contains("guid") {
            uuid::Uuid::new_v4().to_string()
        } else if lower_type.contains("timestamptz")
            || lower_type.contains("timestamp")
            || lower_type.contains("datetime")
        {
            chrono::Utc::now().to_rfc3339()
        } else if lower_type.contains("date") {
            chrono::Utc::now().format("%Y-%m-%d").to_string()
        } else if lower_type.contains("time") {
            chrono::Utc::now().format("%H:%M:%S").to_string()
        } else if lower_type.contains("bool") {
            if rand::random::<bool>() {
                "true".to_owned()
            } else {
                "false".to_owned()
            }
        } else if lower_type.contains("int")
            || lower_type.contains("serial")
            || lower_type.contains("bigint")
            || lower_type.contains("smallint")
        {
            rand_num.to_string()
        } else if lower_type.contains("float")
            || lower_type.contains("double")
            || lower_type.contains("real")
            || lower_type.contains("numeric")
            || lower_type.contains("decimal")
        {
            format!("{}.{:02}", rand_num / 10, rand_num % 100)
        } else if lower_type.contains("json") {
            r#"{"status": "active", "version": 1}"#.to_owned()
        } else if lower_name.contains("email") || lower_name.contains("mail") {
            format!("user_{rand_num}@example.com")
        } else if lower_name.contains("username") || lower_name.contains("user_name") {
            format!("user_{rand_num}")
        } else if lower_name.contains("first_name") || lower_name.contains("firstname") {
            "Alex".to_owned()
        } else if lower_name.contains("last_name") || lower_name.contains("lastname") {
            "Morgan".to_owned()
        } else if lower_name.contains("full_name") || lower_name.contains("fullname") || lower_name == "name" {
            format!("Alex Morgan {}", rand_num % 100)
        } else if lower_name.contains("phone") || lower_name.contains("tel") || lower_name.contains("mobile") {
            format!("+1-555-{:04}", rand_num)
        } else if lower_name.contains("url") || lower_name.contains("website") || lower_name.contains("link") {
            format!("https://example.com/items/{rand_num}")
        } else if lower_name.contains("avatar")
            || lower_name.contains("image")
            || lower_name.contains("icon")
            || lower_name.contains("photo")
        {
            format!("https://picsum.photos/seed/{rand_num}/200")
        } else if lower_name.contains("slug") || lower_name.contains("code") {
            format!("item-{rand_num}")
        } else if lower_name.contains("title") || lower_name.contains("headline") || lower_name.contains("subject") {
            format!("Sample Title {rand_num}")
        } else if lower_name.contains("desc")
            || lower_name.contains("content")
            || lower_name.contains("note")
            || lower_name.contains("bio")
            || lower_name.contains("comment")
            || lower_name.contains("body")
        {
            format!("Sample description for {column_name}")
        } else if lower_name.contains("status") || lower_name.contains("state") {
            "active".to_owned()
        } else if lower_name.contains("role") {
            "user".to_owned()
        } else if lower_name.contains("address") || lower_name.contains("street") {
            format!("{rand_num} Market Street")
        } else if lower_name.contains("city") {
            "San Francisco".to_owned()
        } else if lower_name.contains("country") {
            "US".to_owned()
        } else if lower_name.contains("ip") {
            format!("192.168.1.{}", rand_num % 254 + 1)
        } else {
            format!("{column_name}_{rand_num}")
        }
    }

    pub(crate) fn parse_insert_value(raw: &str, data_type: &str) -> Result<Option<UiCell>, String> {
        let normalized_type = data_type.to_ascii_lowercase();
        let value = raw.trim();
        if value.is_empty() && !Self::is_text_type(&normalized_type) {
            return Ok(None);
        }
        if value.eq_ignore_ascii_case("null") {
            return Ok(Some(UiCell::Null));
        }
        if Self::is_text_type(&normalized_type) {
            return Ok(Some(UiCell::Text(raw.to_owned())));
        }
        if Self::is_binary_type(&normalized_type) {
            return Err("binary values require a binary editor; use NULL or a query parameter".to_owned());
        }
        if normalized_type.contains("uuid") || normalized_type.contains("guid") {
            return uuid::Uuid::parse_str(value)
                .map(|u| Some(UiCell::Text(u.to_string())))
                .map_err(|_| format!("{value} is not a valid UUID"));
        }
        if normalized_type.contains("bool") {
            return match value.to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" => Ok(Some(UiCell::Boolean(true))),
                "false" | "0" | "no" => Ok(Some(UiCell::Boolean(false))),
                _ => Err("expected true or false".to_owned()),
            };
        }
        if normalized_type.contains("int") || normalized_type.contains("serial") {
            return value
                .parse::<i64>()
                .map(|number| Some(UiCell::Number(number.to_string())))
                .map_err(|_| format!("{value} is not a valid integer"));
        }
        if normalized_type.contains("real") || normalized_type.contains("float") || normalized_type.contains("double") {
            return value
                .parse::<f64>()
                .map(|number| Some(UiCell::Number(number.to_string())))
                .map_err(|_| format!("{value} is not a valid floating-point number"));
        }
        if Self::is_decimal_type(&normalized_type) {
            return Self::parse_decimal_value(value, data_type).map(Some);
        }
        if normalized_type.contains("json") {
            return serde_json::from_str::<serde_json::Value>(value)
                .map(|_| Some(UiCell::Json(value.to_owned())))
                .map_err(|_| "expected valid JSON".to_owned());
        }
        if normalized_type.contains("timestamptz")
            || normalized_type.contains("timestamp with time zone")
            || (normalized_type.contains("timestamp") && normalized_type.contains("timezone"))
        {
            return Self::validate_timestamp_with_timezone(value).map(|_| Some(UiCell::Text(value.to_owned())));
        }
        if normalized_type.contains("timestamp") {
            return Self::validate_timestamp(value).map(|_| Some(UiCell::Text(value.to_owned())));
        }
        if normalized_type == "date" || normalized_type.starts_with("date(") {
            return chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map(|_| Some(UiCell::Text(value.to_owned())))
                .map_err(|_| format!("{value} is not a valid date (expected YYYY-MM-DD)"));
        }
        if normalized_type.starts_with("time") {
            return Self::validate_time(value).map(|_| Some(UiCell::Text(value.to_owned())));
        }
        Ok(Some(UiCell::Text(value.to_owned())))
    }

    pub(crate) fn parse_update_value(raw: &str, data_type: &str) -> Result<UiCell, String> {
        let normalized_type = data_type.to_ascii_lowercase();
        if raw.trim().eq_ignore_ascii_case("null") {
            return Ok(UiCell::Null);
        }
        if raw.is_empty() {
            if Self::is_text_type(&normalized_type) {
                return Ok(UiCell::Text(String::new()));
            }
            return Err(format!("{} cannot be empty; use NULL to clear it", data_type));
        }
        if Self::is_text_type(&normalized_type) {
            // Do not trim text: empty, whitespace-only, and NULL are distinct values.
            return Ok(UiCell::Text(raw.to_owned()));
        }
        Self::parse_insert_value(raw, data_type).map(|value| value.unwrap_or(UiCell::Null))
    }

    fn is_text_type(normalized_type: &str) -> bool {
        normalized_type == "text"
            || normalized_type.starts_with("varchar")
            || normalized_type.starts_with("character varying")
            || normalized_type.starts_with("char")
            || normalized_type.starts_with("bpchar")
            || normalized_type == "citext"
    }

    fn is_binary_type(normalized_type: &str) -> bool {
        normalized_type.contains("bytea")
            || normalized_type.contains("blob")
            || normalized_type.contains("binary")
            || normalized_type.contains("varbinary")
    }

    fn validate_timestamp(value: &str) -> Result<(), String> {
        [
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%dT%H:%M:%S",
        ]
        .iter()
        .any(|format| chrono::NaiveDateTime::parse_from_str(value, format).is_ok())
        .then_some(())
        .ok_or_else(|| format!("{value} is not a valid timestamp"))
    }

    fn validate_timestamp_with_timezone(value: &str) -> Result<(), String> {
        chrono::DateTime::parse_from_rfc3339(value)
            .or_else(|_| chrono::DateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f %:z"))
            .map(|_| ())
            .map_err(|_| format!("{value} is not a valid timestamptz (include a timezone offset)"))
    }

    fn validate_time(value: &str) -> Result<(), String> {
        if chrono::NaiveTime::parse_from_str(value, "%H:%M:%S%.f").is_ok()
            || chrono::NaiveTime::parse_from_str(value, "%H:%M:%S").is_ok()
        {
            return Ok(());
        }
        chrono::DateTime::parse_from_str(&format!("1970-01-01 {value}"), "%Y-%m-%d %H:%M:%S%.f %:z")
            .map(|_| ())
            .map_err(|_| format!("{value} is not a valid time"))
    }

    fn is_decimal_type(normalized_type: &str) -> bool {
        normalized_type
            .split('(')
            .next()
            .is_some_and(|name| matches!(name.trim(), "numeric" | "decimal"))
    }

    fn decimal_constraints(data_type: &str) -> Option<(u64, i64)> {
        let normalized_type = data_type.to_ascii_lowercase();
        if !Self::is_decimal_type(&normalized_type) {
            return None;
        }
        let arguments = normalized_type.split_once('(')?.1.split_once(')')?.0;
        let mut parts = arguments.split(',').map(str::trim);
        let precision = parts.next()?.parse::<u64>().ok()?;
        let scale = parts.next().and_then(|part| part.parse::<i64>().ok()).unwrap_or(0);
        Some((precision, scale))
    }

    fn parse_decimal_value(value: &str, data_type: &str) -> Result<UiCell, String> {
        let decimal = value
            .parse::<BigDecimal>()
            .map_err(|_| format!("{value} is not a valid exact decimal"))?;
        let actual_scale = decimal.fractional_digit_count();
        if let Some((precision, declared_scale)) = Self::decimal_constraints(data_type) {
            if actual_scale > declared_scale {
                return Err(format!("{value} has more than {declared_scale} fractional digits"));
            }
            let effective_precision = if actual_scale < 0 {
                decimal.digits().saturating_add((-actual_scale) as u64)
            } else {
                decimal.digits()
            };
            if effective_precision > precision {
                return Err(format!("{value} exceeds NUMERIC precision {precision}"));
            }
        }
        Ok(UiCell::Number(value.to_owned()))
    }

    fn submit_insert_row(&mut self) {
        let Some(table) = self.selected_table.clone() else {
            self.insert_row_error = "Select a table before inserting a row".to_owned();
            return;
        };
        let Some(info) = self.table_info.clone() else {
            self.insert_row_error = "Table structure is still loading".to_owned();
            return;
        };
        let mut columns = Vec::new();
        let mut values = Vec::new();
        for (column, raw) in info.columns.iter().zip(&self.insert_row_values) {
            if raw.trim().is_empty() && !column.nullable && column.default.is_none() {
                self.insert_row_error = format!("{} is required", column.name);
                return;
            }
            match Self::parse_insert_value(raw, &column.data_type) {
                Ok(Some(value)) => {
                    columns.push(column.name.clone());
                    values.push(value);
                }
                Ok(None) => {}
                Err(error) => {
                    self.insert_row_error = format!("{}: {error}", column.name);
                    return;
                }
            }
            if matches!(values.last(), Some(UiCell::Null)) && !column.nullable {
                self.insert_row_error = format!("{} is NOT NULL; enter a value instead", column.name);
                return;
            }
        }
        if columns.is_empty() {
            self.insert_row_error = "Enter at least one value; leave defaulted columns empty".to_owned();
            return;
        }
        self.staged_changes.push(StagedChange::Insert { columns, values });
        self.insert_row_open = false;
        self.insert_row_error.clear();
        self.runtime_message = format!("Row staged for {}", table);
    }

    pub(super) fn draw_insert_row_dialog(&mut self, ctx: &egui::Context) {
        let Some(info) = self.table_info.clone() else {
            self.insert_row_open = false;
            return;
        };
        if self.insert_row_values.len() != info.columns.len() {
            self.insert_row_values = vec![String::new(); info.columns.len()];
        }
        let mut open = self.insert_row_open;
        let mut submit = false;
        let mut cancel = false;
        let title = format!("Insert Row · {}", info.name);
        let description = format!("Schema: {} · {} columns", info.schema, info.columns.len());

        egui::Area::new(egui::Id::new("insert_row_modal_area"))
            .order(egui::Order::Foreground)
            .fixed_pos(egui::Pos2::ZERO)
            .show(ctx, |ui| {
                Dialog::new(&mut open, &title, self.theme)
                    .description(&description)
                    .width(620.0)
                    .show(ui, |ui| {
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
                                    for val in &mut self.insert_row_values {
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
                                            && self.insert_row_values[index].trim().is_empty()
                                        {
                                            self.insert_row_values[index] =
                                                Self::generate_sample_value(&column.name, &column.data_type);
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
                                        if self.insert_row_values[index].trim().is_empty() {
                                            self.insert_row_values[index] =
                                                Self::generate_sample_value(&column.name, &column.data_type);
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
                                                    RichText::new("*required")
                                                        .font(font_caption())
                                                        .color(self.theme.danger),
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
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if !self.insert_row_values[index].is_empty() {
                                                    let clear_btn = Button::new(self.theme)
                                                        .icon(Icon::X)
                                                        .size(ButtonSize::IconSm)
                                                        .variant(ButtonVariant::Ghost)
                                                        .show(ui);
                                                    if clear_btn.on_hover_text("Clear this field").clicked() {
                                                        self.insert_row_values[index].clear();
                                                    }
                                                }

                                                if column.nullable && self.insert_row_values[index] != "NULL" {
                                                    let null_btn = Button::new(self.theme)
                                                        .text("NULL")
                                                        .size(ButtonSize::Sm)
                                                        .variant(ButtonVariant::Ghost)
                                                        .show(ui);
                                                    if null_btn.on_hover_text("Set value to literal NULL").clicked() {
                                                        self.insert_row_values[index] = "NULL".to_owned();
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
                                                    self.insert_row_values[index] =
                                                        Self::generate_sample_value(&column.name, &column.data_type);
                                                }
                                            });
                                        });

                                        ui.add_space(4.0);

                                        let is_null_val =
                                            self.insert_row_values[index].trim().eq_ignore_ascii_case("null");
                                        let placeholder = if column.default.is_some() {
                                            "Leave empty for DEFAULT, or enter value / click Gen..."
                                        } else if column.nullable {
                                            "Enter value, click NULL, or click Gen..."
                                        } else {
                                            "Enter value or click Gen..."
                                        };

                                        let val_ref = &mut self.insert_row_values[index];
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
                                    });
                                    ui.add_space(6.0);
                                }
                            });

                        if !self.insert_row_error.is_empty() {
                            ui.add_space(8.0);
                            Alert::new("Cannot Stage Insert", &self.insert_row_error, self.theme)
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
            });

        if submit {
            self.submit_insert_row();
        }
        if cancel || !open {
            self.insert_row_open = false;
            self.insert_row_error.clear();
        }
    }

    pub(crate) fn submit_ddl(&mut self) {
        if !self.can_mutate_active_connection() {
            self.runtime_message = "Connect with write access to execute DDL".to_owned();
            return;
        }
        if self.ddl_execution_request.is_some() {
            return;
        }
        let Some(sql) = self.table_ddl.clone() else {
            self.runtime_message = "Load the table DDL before executing it".to_owned();
            return;
        };
        if sql.trim().is_empty() {
            self.runtime_message = "DDL cannot be empty".to_owned();
            return;
        }
        let Some(connection) = self.active_connection().cloned() else {
            self.runtime_message = "Connect to a database before executing DDL".to_owned();
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(UiCommand::ExecuteDdl {
            request_id,
            connection_id: connection.id,
            sql,
        });
        self.ddl_execution_request = Some(request_id);
        self.ddl_execute_confirmation = false;
        self.runtime_message = "Executing DDL…".to_owned();
    }

    /// Loading / failed placeholder shown while the DDL is not available.
    pub(super) fn draw_table_ddl_placeholder(&mut self, ui: &mut egui::Ui, table_name: &str) {
        grid_frame(self.theme).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(28.0);
                let failed = self.table_ddl_error.as_deref();
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
                        "Editable preview · execution is confirmation-gated"
                    } else {
                        "Read-only preview"
                    })
                    .small()
                    .color(self.theme.text_muted),
                );
                if writable
                    && self.ddl_execution_request.is_none()
                    && primary_button_with_icon(ui, Icon::Play, "Apply DDL", self.theme).clicked()
                {
                    request_execution = true;
                }
            });
            ui.add_space(8.0);
            if let Some(error) = self.table_ddl_error.as_deref() {
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
                    self.table_ddl_error = None;
                }
            });
        });
        request_execution
    }

    /// Confirmation gate shown before the DDL is executed against the database.
    pub(super) fn draw_ddl_confirmation_card(&mut self, ui: &mut egui::Ui, impact: &str) {
        let Some(ddl) = self.table_ddl.as_deref() else {
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
                    self.ddl_execute_confirmation = false;
                }
                _ => {}
            }
        }
    }
    pub(crate) fn request_table_info(&mut self) {
        let (Some(connection_id), Some(table)) = (self.active_connection_id.clone(), self.selected_table.clone())
        else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.table_info_request = Some(request_id);
        self.dispatch_command(UiCommand::LoadTableInfo {
            request_id,
            connection_id,
            schema: self.active_schema().to_owned(),
            table,
        });
        self.runtime_message = "Loading table structure…".to_owned();
    }

    pub(crate) fn can_mutate_active_connection(&self) -> bool {
        self.connected && self.active_connection().is_some_and(|connection| !connection.readonly)
    }

    pub(crate) fn row_identity(
        result: &UiQueryResult,
        info: &UiTableInfo,
        row_index: usize,
    ) -> Result<(Vec<String>, Vec<UiCell>), String> {
        let Some(primary_key) = info.primary_key.as_ref() else {
            return Err("This table has no primary key for safe row editing".to_owned());
        };
        let mut pk_values = Vec::with_capacity(primary_key.len());
        for pk_column in primary_key {
            let Some(pk_index) = result.columns.iter().position(|item| item.name == *pk_column) else {
                return Err(format!(
                    "The primary-key column {pk_column} is not present in this result"
                ));
            };
            let Some(pk_cell) = result.rows.get(row_index).and_then(|row| row.get(pk_index)) else {
                return Err("The selected row is no longer available".to_owned());
            };
            if matches!(pk_cell, UiCell::Null) {
                return Err(format!("A NULL primary key ({pk_column}) cannot identify a row"));
            }
            pk_values.push(pk_cell.clone());
        }
        Ok((primary_key.clone(), pk_values))
    }

    pub(crate) fn begin_data_cell_edit(&mut self, row_index: usize, column_index: usize, cell: &UiCell) {
        if !self.can_mutate_active_connection() {
            self.runtime_message = "Connect with write access to edit rows".to_owned();
            return;
        }
        if self.staged_row_deleted(row_index) {
            self.runtime_message = "Discard the staged delete before editing this row".to_owned();
            return;
        }
        self.selected_cell = Some((row_index, column_index));
        self.selected_row = Some(row_index);
        self.selected_rows.clear();
        self.selected_rows.insert(row_index);
        self.selection_anchor_row = Some(row_index);
        self.selection_anchor_cell = Some((row_index, column_index));
        self.data_editing_cell = Some((row_index, column_index));
        self.data_edit_error = None;
        self.data_edit_value = match cell {
            UiCell::Null => "NULL".to_owned(),
            _ => crate::cell_text(cell),
        };
        self.copy_status.clear();
    }

    pub(crate) fn submit_data_cell_edit(
        &mut self,
        result: &UiQueryResult,
        row_index: usize,
        column_index: usize,
    ) -> bool {
        let Some(info) = self.table_info.clone() else {
            self.runtime_message = "Table structure is still loading".to_owned();
            return false;
        };
        let Some(column) = result.columns.get(column_index).map(|column| column.name.clone()) else {
            self.data_editing_cell = None;
            return false;
        };
        let Some(column_info) = info.columns.iter().find(|item| item.name == column) else {
            self.runtime_message = "The selected column is not present in the table metadata".to_owned();
            self.data_editing_cell = None;
            return false;
        };
        if Self::is_binary_type(&column_info.data_type.to_ascii_lowercase()) {
            let error = "Binary values cannot be edited with the normal text editor".to_owned();
            self.data_edit_error = Some(error.clone());
            self.runtime_message = format!("{}: {error}", column_info.name);
            return false;
        }
        let value = match Self::parse_update_value(&self.data_edit_value, &column_info.data_type) {
            Ok(value) => value,
            Err(error) => {
                self.runtime_message = format!("{}: {error}", column_info.name);
                self.data_edit_error = Some(error);
                return false;
            }
        };
        if matches!(value, UiCell::Null) && !column_info.nullable {
            let error = format!("{} is NOT NULL; enter a value instead", column_info.name);
            self.runtime_message = error.clone();
            self.data_edit_error = Some(error);
            return false;
        }
        let (pk_columns, pk_values) = match Self::row_identity(result, &info, row_index) {
            Ok(identity) => identity,
            Err(error) => {
                self.runtime_message = error;
                self.data_edit_error = Some(self.runtime_message.clone());
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
            self.data_editing_cell = None;
            self.runtime_message = "The selected cell is no longer available".to_owned();
            self.data_edit_error = Some(self.runtime_message.clone());
            return false;
        };
        self.staged_changes.stage_update(StagedChange::Update {
            row_index,
            column_index,
            column,
            data_type: column_info.data_type.clone(),
            original,
            value,
            pk_columns,
            pk_values,
        });
        self.data_editing_cell = None;
        self.data_edit_value.clear();
        self.data_edit_error = None;
        let counts = self.staged_changes.counts();
        self.runtime_message = format!(
            "{} staged change(s): +{} ~{} -{}",
            counts.total(),
            counts.inserts,
            counts.updates,
            counts.deletes
        );
        true
    }

    pub(crate) fn request_delete_selected_data_rows(&mut self, result: &UiQueryResult) {
        if !self.can_mutate_active_connection() {
            self.runtime_message = "Connect with write access to delete rows".to_owned();
            return;
        }
        let row_indexes: Vec<usize> = if self.selected_rows.is_empty() {
            self.selected_row.into_iter().collect()
        } else {
            self.selected_rows.iter().copied().collect()
        };
        if row_indexes.is_empty() {
            self.runtime_message = "Select a row before deleting".to_owned();
            return;
        }
        let Some(info) = self.table_info.clone() else {
            self.runtime_message = "Table structure is still loading".to_owned();
            return;
        };
        self.data_editing_cell = None;
        self.data_edit_value.clear();
        self.data_edit_error = None;
        self.data_delete_confirmation = false;
        for row_index in row_indexes {
            if self.staged_row_deleted(row_index) {
                continue;
            }
            let (pk_columns, pk_values) = match Self::row_identity(result, &info, row_index) {
                Ok(identity) => identity,
                Err(error) => {
                    self.runtime_message = error;
                    return;
                }
            };
            self.staged_changes.stage_delete(StagedChange::Delete {
                row_index,
                pk_columns,
                pk_values,
            });
        }
        self.runtime_message = format!(
            "{} row(s) marked for deletion · {} staged change(s)",
            self.staged_changes
                .iter()
                .filter(|change| matches!(change, StagedChange::Delete { .. }))
                .count(),
            self.staged_changes.counts().total()
        );
    }

    pub(crate) fn staged_cell_value(&self, row_index: usize, column_index: usize) -> Option<UiCell> {
        self.staged_changes.iter().rev().find_map(|change| match change {
            StagedChange::Update {
                row_index: changed_row,
                column_index: changed_column,
                value,
                ..
            } if *changed_row == row_index && *changed_column == column_index => Some(value.clone()),
            _ => None,
        })
    }

    pub(crate) fn staged_row_deleted(&self, row_index: usize) -> bool {
        self.staged_changes.iter().any(
            |change| matches!(change, StagedChange::Delete { row_index: changed_row, .. } if *changed_row == row_index),
        )
    }

    pub(crate) fn revert_staged_cell(&mut self, row_index: usize, column_index: usize) {
        if self.staged_changes.revert_cell(row_index, column_index) {
            self.runtime_message = "Cell change reverted".to_owned();
        }
    }

    pub(crate) fn revert_staged_row(&mut self, row_index: usize) {
        if self.staged_changes.revert_row(row_index) {
            self.runtime_message = "Row changes reverted".to_owned();
        }
    }

    pub(crate) fn discard_staged_changes(&mut self) {
        if self.staged_apply_request.is_some() {
            self.runtime_message = "Wait for the current database write before discarding".to_owned();
            return;
        }
        self.staged_changes.clear();
        self.data_editing_cell = None;
        self.data_edit_error = None;
        self.data_delete_confirmation = false;
        self.discard_changes_confirmation = false;
        self.data_edit_value.clear();
        self.table_data_result = None;
        self.table_data_error = None;
        self.runtime_message = "Staged changes discarded".to_owned();
        self.request_table_data();
    }

    fn draw_discard_changes_confirmation(&mut self, ui: &mut egui::Ui) {
        if !self.discard_changes_confirmation {
            return;
        }
        let counts = self.staged_changes.counts();
        let description = format!(
            "Discard {} pending changes (+{} inserts, {} updates, {} deletes) and restore the server state?",
            counts.total(),
            counts.inserts,
            counts.updates,
            counts.deletes
        );
        let mut open = true;
        let mut confirm = false;
        let mut cancel = false;
        let theme = self.theme;
        Dialog::new(&mut open, "Discard pending changes?", theme)
            .description(&description)
            .id_salt("discard-table-changes")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if Button::new(theme)
                        .text("Discard changes")
                        .variant(ButtonVariant::Destructive)
                        .show(ui)
                        .clicked()
                    {
                        confirm = true;
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
        if confirm {
            self.discard_staged_changes();
        } else if cancel || !open {
            self.discard_changes_confirmation = false;
        }
    }

    pub(crate) fn apply_staged_changes(&mut self) {
        if self.staged_apply_request.is_some() {
            return;
        }
        if self.staged_changes.is_empty() {
            return;
        }
        let Some(connection) = self.active_connection().cloned() else {
            self.runtime_message = "Connect to a database before applying changes".to_owned();
            return;
        };
        let Some(table) = self.selected_table.clone() else {
            self.runtime_message = "Select a table before applying changes".to_owned();
            return;
        };
        let mut changes = Vec::new();
        let mut updates = std::collections::BTreeMap::<
            usize,
            (Vec<String>, Vec<String>, Vec<UiCell>, Vec<String>, Vec<UiCell>),
        >::new();
        for change in self.staged_changes.iter() {
            match change {
                StagedChange::Update {
                    row_index,
                    column,
                    data_type,
                    value,
                    pk_columns,
                    pk_values,
                    ..
                } => {
                    let entry = updates.entry(*row_index).or_insert_with(|| {
                        (
                            Vec::new(),
                            Vec::new(),
                            Vec::new(),
                            pk_columns.clone(),
                            pk_values.clone(),
                        )
                    });
                    entry.0.push(column.clone());
                    entry.1.push(data_type.clone());
                    entry.2.push(value.clone());
                }
                StagedChange::Delete {
                    pk_columns, pk_values, ..
                } => changes.push(UiTableMutation::Delete {
                    pk_columns: pk_columns.clone(),
                    pk_values: pk_values.clone(),
                }),
                StagedChange::Insert { columns, values } => changes.push(UiTableMutation::Insert {
                    columns: columns.clone(),
                    values: values.clone(),
                }),
            }
        }
        changes.extend(
            updates.into_values().map(
                |(columns, data_types, values, pk_columns, pk_values)| UiTableMutation::Update {
                    columns,
                    data_types,
                    values,
                    pk_columns,
                    pk_values,
                },
            ),
        );
        let request_id = self.task_bridge.next_request_id();
        let command = UiCommand::ApplyTableChanges {
            request_id,
            connection_id: connection.id,
            schema: self.active_schema().to_owned(),
            table,
            changes,
        };
        if self.task_bridge.send(command).is_ok() {
            self.staged_apply_request = Some(request_id);
            self.table_mutation_request = Some(request_id);
            let counts = self.staged_changes.counts();
            self.runtime_message = format!(
                "Applying {} changes in one transaction (+{} ~{} -{})…",
                counts.total(),
                counts.inserts,
                counts.updates,
                counts.deletes
            );
        } else {
            self.runtime_message = "Could not send staged change to runtime".to_owned();
        }
    }

    pub(crate) fn staged_apply_completed(&mut self) {
        self.staged_apply_request = None;
        self.table_mutation_request = None;
        self.staged_changes.clear();
        self.runtime_message = "All staged changes applied".to_owned();
        self.show_toast_success("All staged changes applied successfully");
        self.table_data_result = None;
        self.table_data_total_rows = None;
        self.table_data_error = None;
        self.request_table_data();
    }

    pub(crate) fn staged_apply_failed(&mut self, message: &str) {
        self.staged_apply_request = None;
        self.table_mutation_request = None;
        let formatted = format!("Staged change failed · {message}");
        self.runtime_message = formatted.clone();
        self.show_toast_error(formatted);
    }

    pub(crate) fn request_table_ddl(&mut self) {
        let (Some(connection_id), Some(table)) = (self.active_connection_id.clone(), self.selected_table.clone())
        else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.table_ddl_request = Some(request_id);
        self.dispatch_command(UiCommand::LoadTableDdl {
            request_id,
            connection_id,
            schema: self.active_schema().to_owned(),
            table,
        });
        self.runtime_message = "Loading table DDL…".to_owned();
    }

    pub(crate) fn request_table_data(&mut self) {
        let Some(connection_id) = self.active_connection_id.clone() else {
            return;
        };
        let table = self
            .selected_table
            .clone()
            .or_else(|| match self.selected_schema_object.as_ref() {
                Some(SchemaObjectSelection::View(name)) => Some(name.clone()),
                _ => None,
            });
        let Some(table) = table else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.table_data_request = Some(request_id);
        let sorts = self
            .table_data_sort_column
            .clone()
            .map(|column| UiTableDataSort {
                column,
                descending: self.table_data_sort_desc,
            })
            .into_iter()
            .collect();
        self.dispatch_command(UiCommand::LoadTableData {
            request_id,
            connection_id,
            schema: self.active_schema().to_owned(),
            table,
            limit: self.table_data_limit,
            offset: self.table_data_offset,
            filters: self.table_data_filters.clone(),
            sorts,
        });
        self.runtime_message = "Loading table data…".to_owned();
    }

    pub(crate) fn commit_table_filter_draft(&mut self) {
        if !self.staged_changes.is_empty() {
            self.runtime_message = "Apply or discard staged changes before changing filters".to_owned();
            return;
        }
        let column = self.table_data_filter_column.trim();
        if column.is_empty() {
            return;
        }
        let is_null_operator = matches!(
            self.table_data_filter_operator,
            UiTableFilterOperator::IsNull | UiTableFilterOperator::IsNotNull
        );
        if !is_null_operator && self.table_data_filter_value.is_empty() {
            self.runtime_message = "Enter a filter value first".to_owned();
            return;
        }
        let data_type = self
            .table_info
            .as_ref()
            .and_then(|info| info.columns.iter().find(|item| item.name == column))
            .map(|item| item.data_type.clone())
            .unwrap_or_else(|| "text".to_owned());
        if !is_null_operator {
            if let Err(error) = Self::parse_update_value(&self.table_data_filter_value, &data_type) {
                self.runtime_message = format!("Invalid filter for {column}: {error}");
                return;
            }
        }
        let filter = UiTableDataFilter {
            column: column.to_owned(),
            data_type,
            operator: self.table_data_filter_operator.clone(),
            value: if is_null_operator {
                String::new()
            } else {
                self.table_data_filter_value.clone()
            },
        };
        if let Some(existing) = self
            .table_data_filters
            .iter_mut()
            .find(|item| item.column == filter.column)
        {
            *existing = filter;
        } else {
            self.table_data_filters.push(filter);
        }
        self.table_data_offset = 0;
        self.request_table_data();
    }

    pub(crate) fn remove_table_filter(&mut self, index: usize) {
        if !self.staged_changes.is_empty() {
            self.runtime_message = "Apply or discard staged changes before changing filters".to_owned();
            return;
        }
        if index < self.table_data_filters.len() {
            self.table_data_filters.remove(index);
            self.table_data_offset = 0;
            self.request_table_data();
        }
    }

    pub(crate) fn clear_table_filters(&mut self) {
        if !self.staged_changes.is_empty() {
            self.runtime_message = "Apply or discard staged changes before changing filters".to_owned();
            return;
        }
        self.table_data_filters.clear();
        self.table_data_offset = 0;
        self.request_table_data();
    }

    pub(crate) fn reset_table_data_page(&mut self) {
        self.table_data_offset = 0;
        self.request_table_data();
    }

    pub(crate) fn reload_table_data_from_start(&mut self) {
        if !self.staged_changes.is_empty() {
            self.runtime_message = "Apply or discard staged changes before reloading".to_owned();
            return;
        }
        self.table_data_offset = 0;
        self.table_data_result = None;
        self.table_data_total_rows = None;
        self.table_data_error = None;
        self.request_table_data();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sample_value_types() {
        let uuid_val = DbProApp::generate_sample_value("id", "uuid");
        assert!(uuid::Uuid::parse_str(&uuid_val).is_ok());

        let time_val = DbProApp::generate_sample_value("created_at", "timestamptz");
        assert!(chrono::DateTime::parse_from_rfc3339(&time_val).is_ok());

        let date_val = DbProApp::generate_sample_value("birth_date", "date");
        assert_eq!(date_val.len(), 10);

        let email_val = DbProApp::generate_sample_value("user_email", "varchar");
        assert!(email_val.contains('@'));

        let bool_val = DbProApp::generate_sample_value("is_active", "boolean");
        assert!(bool_val == "true" || bool_val == "false");
    }

    #[test]
    fn test_parse_insert_value_uuid() {
        let valid_uuid = "a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11";
        let parsed = DbProApp::parse_insert_value(valid_uuid, "uuid");
        assert_eq!(parsed, Ok(Some(UiCell::Text(valid_uuid.to_owned()))));

        let invalid_uuid = "not-a-uuid";
        assert!(DbProApp::parse_insert_value(invalid_uuid, "uuid").is_err());
    }

    #[test]
    fn update_parser_preserves_null_empty_and_whitespace_text() {
        assert_eq!(DbProApp::parse_update_value("NULL", "text"), Ok(UiCell::Null));
        assert_eq!(
            DbProApp::parse_update_value("", "text"),
            Ok(UiCell::Text(String::new()))
        );
        assert_eq!(
            DbProApp::parse_update_value("   ", "text"),
            Ok(UiCell::Text("   ".to_owned()))
        );
        assert!(DbProApp::parse_update_value("", "integer").is_err());
    }

    #[test]
    fn update_parser_validates_temporal_json_and_binary_values() {
        assert!(DbProApp::parse_update_value("2026-09-12", "date").is_ok());
        assert!(DbProApp::parse_update_value("12:30:45", "time").is_ok());
        assert!(DbProApp::parse_update_value("2026-09-12T12:30:45Z", "timestamptz").is_ok());
        assert!(DbProApp::parse_update_value("not-a-time", "time").is_err());
        assert!(DbProApp::parse_update_value("{\"ok\":true}", "jsonb").is_ok());
        assert!(DbProApp::parse_update_value("not-json", "jsonb").is_err());
        assert!(DbProApp::parse_update_value("deadbeef", "bytea").is_err());
    }
}
