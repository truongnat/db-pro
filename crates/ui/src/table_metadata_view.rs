use super::*;
use crate::components::badge::{Badge, BadgeVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::table::{Table, TableColumn};
use crate::{UiDependencyDirection, UiDependencyKind};
use egui::{Align, Color32, Layout, RichText};
use lucide_icons::Icon;

impl DbProApp {
    /// Draw the Structure tab: summary metrics, search bar, and detailed columns table.
    pub(super) fn draw_table_structure_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table_state.table_info.clone() else {
            self.draw_table_structure_placeholder(ui);
            return;
        };

        self.draw_table_structure_metrics(ui, &info);
        ui.add_space(8.0);
        self.draw_table_structure_columns_table(ui, &info);
        self.draw_table_column_detail(ui.ctx(), &info);
    }

    /// Top metric chips summarising columns, keys, and row count.
    fn draw_table_structure_metrics(&self, ui: &mut egui::Ui, info: &UiTableInfo) {
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                section_label(ui, "METRICS", self.theme);
                Badge::new(format!("{} columns", info.columns.len()), self.theme)
                    .variant(BadgeVariant::Default)
                    .show(ui);
                if let Some(pk) = &info.primary_key {
                    Badge::new(format!("PK: {}", pk.join(", ")), self.theme)
                        .variant(BadgeVariant::Warning)
                        .show(ui);
                }
                Badge::new(format!("{} indexes", info.indexes.len()), self.theme)
                    .variant(BadgeVariant::Secondary)
                    .show(ui);
                Badge::new(format!("{} foreign keys", info.foreign_keys.len()), self.theme)
                    .variant(BadgeVariant::Secondary)
                    .show(ui);
                if !info.check_constraints.is_empty() {
                    Badge::new(
                        format!("{} check constraints", info.check_constraints.len()),
                        self.theme,
                    )
                    .variant(BadgeVariant::Outline)
                    .show(ui);
                }
                if !info.dependencies.is_empty() {
                    Badge::new(format!("{} dependencies", info.dependencies.len()), self.theme)
                        .variant(BadgeVariant::Outline)
                        .show(ui);
                }
                if let Some(row_count) = info.row_count {
                    Badge::new(format!("{row_count} rows"), self.theme)
                        .variant(BadgeVariant::Secondary)
                        .show(ui);
                }
            });
        });
    }

    /// Columns table with search filter, type badges, nullability, PK/FK flags, and default expressions.
    fn draw_table_structure_columns_table(&mut self, ui: &mut egui::Ui, info: &UiTableInfo) {
        let mut selected_column = None;
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                section_label(ui, "COLUMNS", self.theme);
                ui.add_space(8.0);
                input(
                    ui,
                    &mut self.table_state.table_structure_search,
                    "Search columns or types…",
                    220.0,
                    self.theme,
                );
                if !self.table_state.table_structure_search.is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Clear filter")
                        .show(ui)
                        .clicked()
                {
                    self.table_state.table_structure_search.clear();
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("Total: {} columns", info.columns.len()))
                            .font(font_caption())
                            .color(self.theme.text_muted),
                    );
                });
            });
            ui.add_space(8.0);

            let filter_lower = self.table_state.table_structure_search.trim().to_lowercase();
            let matching_columns: Vec<_> = info
                .columns
                .iter()
                .filter(|c| {
                    if filter_lower.is_empty() {
                        true
                    } else {
                        c.name.to_lowercase().contains(&filter_lower)
                            || c.data_type.to_lowercase().contains(&filter_lower)
                            || c.default
                                .as_deref()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&filter_lower)
                    }
                })
                .collect();

            if matching_columns.is_empty() {
                empty_state(
                    ui,
                    Icon::Columns3,
                    "No columns match search",
                    "Try entering a different column name or data type.",
                    self.theme,
                );
                return;
            }

            let columns = [
                TableColumn::fixed("#", 46.0),
                TableColumn::new("Column Name").width(220.0),
                TableColumn::new("Data Type").width(180.0),
                TableColumn::fixed("Nullable", 110.0),
                TableColumn::fixed("Key", 90.0),
                TableColumn::new("Default Expression"),
            ];

            egui::ScrollArea::horizontal()
                .id_salt("structure-cols-scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    Table::new(&columns, self.theme).row_height(34.0).show(
                        ui,
                        matching_columns.len(),
                        |_| false,
                        |_| {},
                        |_| {},
                        |_| {},
                        |ui, row_idx, col_idx| {
                            let column = matching_columns[row_idx];
                            let is_fk = info
                                .foreign_keys
                                .iter()
                                .any(|fk| fk.from_columns.contains(&column.name));
                            match col_idx {
                                0 => {
                                    ui.label(
                                        RichText::new(column.ordinal.to_string())
                                            .font(font_caption())
                                            .color(self.theme.text_muted),
                                    );
                                }
                                1 => {
                                    let (icon, color) = if column.is_primary_key {
                                        (Icon::Key, self.theme.warning)
                                    } else if is_fk {
                                        (Icon::ArrowRightLeft, self.theme.accent)
                                    } else {
                                        (Icon::Columns3, self.theme.text_muted)
                                    };
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(char::from(icon).to_string())
                                                .font(egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())))
                                                .color(color),
                                        );
                                        ui.add_space(4.0);
                                        let name_response = ui
                                            .label(
                                                RichText::new(&column.name)
                                                    .font(font_ui_label())
                                                    .strong()
                                                    .color(self.theme.text_primary),
                                            )
                                            .on_hover_text(format!(
                                                "{}{}{}{}",
                                                if column.is_identity { "IDENTITY · " } else { "" },
                                                if column.is_generated { "GENERATED · " } else { "" },
                                                if column.is_unique { "UNIQUE · " } else { "" },
                                                column
                                                    .collation
                                                    .as_deref()
                                                    .map(|value| format!("COLLATION {value}"))
                                                    .unwrap_or_default()
                                            ));
                                        if name_response.clicked() {
                                            selected_column = Some(column.name.clone());
                                        }
                                    });
                                }
                                2 => {
                                    ui.label(
                                        RichText::new(&column.data_type)
                                            .monospace()
                                            .color(self.theme.text_secondary),
                                    )
                                    .on_hover_text("Full database type");
                                }
                                3 => {
                                    if column.nullable {
                                        Badge::new("NULL", self.theme)
                                            .variant(BadgeVariant::Secondary)
                                            .compact(true)
                                            .show(ui);
                                    } else {
                                        Badge::new("NOT NULL", self.theme)
                                            .variant(BadgeVariant::Outline)
                                            .compact(true)
                                            .show(ui);
                                    }
                                }
                                4 => {
                                    if column.is_primary_key {
                                        Badge::new("PK", self.theme)
                                            .variant(BadgeVariant::Warning)
                                            .compact(true)
                                            .show(ui);
                                    } else if is_fk {
                                        Badge::new("FK", self.theme)
                                            .variant(BadgeVariant::Default)
                                            .compact(true)
                                            .show(ui);
                                    } else if column.is_unique {
                                        Badge::new("UNIQUE", self.theme)
                                            .variant(BadgeVariant::Default)
                                            .compact(true)
                                            .show(ui);
                                    } else {
                                        ui.label(RichText::new("—").font(font_caption()).color(self.theme.text_muted));
                                    }
                                }
                                5 => {
                                    ui.label(
                                        RichText::new(column.default.as_deref().unwrap_or("—"))
                                            .font(font_caption())
                                            .color(self.theme.text_secondary),
                                    );
                                }
                                _ => {}
                            }
                        },
                    );
                });
        });
        if selected_column.is_some() {
            self.table_state.table_column_detail = selected_column;
        }
    }

    fn draw_table_column_detail(&mut self, ctx: &egui::Context, info: &UiTableInfo) {
        let Some(column_name) = self.table_state.table_column_detail.clone() else {
            return;
        };
        let Some(column) = info.columns.iter().find(|column| column.name == column_name) else {
            self.table_state.table_column_detail = None;
            return;
        };
        let mut open = true;
        egui::Window::new(format!("Column · {}", column.name))
            .open(&mut open)
            .resizable(false)
            .default_width(360.0)
            .show(ctx, |ui| {
                ui.label(RichText::new(&column.data_type).monospace().strong());
                ui.separator();
                ui.label(format!("Ordinal: {}", column.ordinal));
                ui.label(format!("Nullable: {}", column.nullable));
                ui.label(format!("Default: {}", column.default.as_deref().unwrap_or("—")));
                ui.label(format!("Primary key: {}", column.is_primary_key));
                ui.label(format!("Unique: {}", column.is_unique));
                ui.label(format!("Identity: {}", column.is_identity));
                ui.label(format!("Generated: {}", column.is_generated));
                if let Some(collation) = &column.collation {
                    ui.label(format!("Collation: {collation}"));
                }
            });
        if !open {
            self.table_state.table_column_detail = None;
        }
    }

    /// Draw the Indexes tab: full index metadata table with search filter and unique badges.
    pub(super) fn draw_table_indexes_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table_state.table_info.clone() else {
            ui.label(RichText::new("Table structure is still loading…").color(self.theme.text_muted));
            return;
        };

        let mut selected_index = None;
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                section_label(ui, "INDEXES", self.theme);
                ui.add_space(8.0);
                input(
                    ui,
                    &mut self.table_state.table_metadata_search,
                    "Filter indexes…",
                    220.0,
                    self.theme,
                );
                if !self.table_state.table_metadata_search.is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Clear filter")
                        .show(ui)
                        .clicked()
                {
                    self.table_state.table_metadata_search.clear();
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("Total: {} indexes", info.indexes.len()))
                            .font(font_caption())
                            .color(self.theme.text_muted),
                    );
                });
            });
            ui.add_space(8.0);

            let filter_lower = self.table_state.table_metadata_search.trim().to_lowercase();
            let matching_indexes: Vec<_> = info
                .indexes
                .iter()
                .filter(|idx| {
                    if filter_lower.is_empty() {
                        true
                    } else {
                        idx.name.to_lowercase().contains(&filter_lower)
                            || idx.columns.iter().any(|c| c.to_lowercase().contains(&filter_lower))
                    }
                })
                .collect();

            if matching_indexes.is_empty() {
                empty_state(
                    ui,
                    Icon::List,
                    "No indexes found",
                    "This table has no indexes defined or none match the search.",
                    self.theme,
                );
                return;
            }

            let cols = [
                TableColumn::new("Index Name").width(240.0),
                TableColumn::new("Indexed Columns").width(280.0),
                TableColumn::fixed("Method", 100.0),
                TableColumn::new("INCLUDE").width(180.0),
                TableColumn::new("Predicate").width(220.0),
                TableColumn::new("Status"),
            ];

            egui::ScrollArea::horizontal()
                .id_salt("indexes-table-scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    Table::new(&cols, self.theme).row_height(34.0).show(
                        ui,
                        matching_indexes.len(),
                        |_| false,
                        |_| {},
                        |_| {},
                        |_| {},
                        |ui, row_idx, col_idx| {
                            let index = matching_indexes[row_idx];
                            match col_idx {
                                0 => {
                                    ui.horizontal(|ui| {
                                        ui.label(icon_text(
                                            if index.unique { Icon::BadgeCheck } else { Icon::List },
                                            "",
                                            if index.unique {
                                                self.theme.accent
                                            } else {
                                                self.theme.text_muted
                                            },
                                        ));
                                        let response = ui
                                            .label(RichText::new(&index.name).strong().color(self.theme.text_primary))
                                            .on_hover_text(&index.definition);
                                        if response.clicked() {
                                            selected_index = Some(index.name.clone());
                                        }
                                    });
                                }
                                1 => {
                                    ui.label(
                                        RichText::new(index.columns.join(", "))
                                            .monospace()
                                            .color(self.theme.text_secondary),
                                    );
                                }
                                2 => {
                                    ui.label(
                                        RichText::new(&index.method)
                                            .monospace()
                                            .color(self.theme.text_secondary),
                                    );
                                }
                                3 => {
                                    let include_columns = if index.include_columns.is_empty() {
                                        "—".to_owned()
                                    } else {
                                        index.include_columns.join(", ")
                                    };
                                    ui.label(
                                        RichText::new(include_columns)
                                            .monospace()
                                            .color(self.theme.text_secondary),
                                    );
                                }
                                4 => {
                                    ui.label(
                                        RichText::new(index.predicate.as_deref().unwrap_or("—"))
                                            .monospace()
                                            .color(self.theme.text_secondary),
                                    )
                                    .on_hover_text(&index.definition);
                                }
                                5 => {
                                    ui.label(
                                        RichText::new(if index.unique {
                                            if index.primary {
                                                "PRIMARY KEY"
                                            } else {
                                                "Enforces uniqueness"
                                            }
                                        } else {
                                            "Active index"
                                        })
                                        .font(font_caption())
                                        .color(self.theme.text_muted),
                                    );
                                }
                                _ => {}
                            }
                        },
                    );
                });
        });
        if let Some(index_name) = selected_index {
            self.table_state.table_index_detail = Some(index_name);
        }
        self.draw_table_index_detail(ui.ctx(), &info);
    }

    fn draw_table_index_detail(&mut self, ctx: &egui::Context, info: &UiTableInfo) {
        let Some(index_name) = self.table_state.table_index_detail.clone() else {
            return;
        };
        let Some(index) = info.indexes.iter().find(|index| index.name == index_name) else {
            self.table_state.table_index_detail = None;
            return;
        };
        let mut open = true;
        egui::Window::new(format!("Index · {}", index.name))
            .open(&mut open)
            .resizable(true)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.label(RichText::new(&index.definition).monospace());
                ui.separator();
                ui.label(format!("Method: {}", index.method));
                ui.label(format!("Primary: {} · Unique: {}", index.primary, index.unique));
                ui.label(format!("Columns: {}", index.columns.join(", ")));
                if !index.include_columns.is_empty() {
                    ui.label(format!("INCLUDE: {}", index.include_columns.join(", ")));
                }
                if let Some(predicate) = &index.predicate {
                    ui.label(format!("Predicate: {predicate}"));
                }
            });
        if !open {
            self.table_state.table_index_detail = None;
        }
    }

    /// Draw the Foreign Keys tab: relations table with target jump and copy actions.
    pub(super) fn draw_table_relations_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table_state.table_info.clone() else {
            ui.label(RichText::new("Table structure is still loading…").color(self.theme.text_muted));
            return;
        };

        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                section_label(ui, "FOREIGN KEYS", self.theme);
                ui.add_space(8.0);
                input(
                    ui,
                    &mut self.table_state.table_metadata_search,
                    "Filter foreign keys…",
                    220.0,
                    self.theme,
                );
                if !self.table_state.table_metadata_search.is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Clear filter")
                        .show(ui)
                        .clicked()
                {
                    self.table_state.table_metadata_search.clear();
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("Total: {} foreign keys", info.foreign_keys.len()))
                            .font(font_caption())
                            .color(self.theme.text_muted),
                    );
                });
            });
            ui.add_space(8.0);

            let filter_lower = self.table_state.table_metadata_search.trim().to_lowercase();
            let matching_fks: Vec<_> = info
                .foreign_keys
                .iter()
                .filter(|fk| {
                    if filter_lower.is_empty() {
                        true
                    } else {
                        fk.name.to_lowercase().contains(&filter_lower)
                            || fk.to_table.to_lowercase().contains(&filter_lower)
                            || fk.from_columns.iter().any(|c| c.to_lowercase().contains(&filter_lower))
                    }
                })
                .collect();

            if matching_fks.is_empty() {
                empty_state(
                    ui,
                    Icon::ArrowRightLeft,
                    "No foreign keys found",
                    "This table has no outgoing foreign keys or none match the search.",
                    self.theme,
                );
                return;
            }

            let cols = [
                TableColumn::new("Constraint Name").width(220.0),
                TableColumn::new("Source Columns").width(180.0),
                TableColumn::new("Target Table").width(200.0),
                TableColumn::new("Target Columns").width(180.0),
                TableColumn::new("Action"),
            ];

            let mut switch_table: Option<String> = None;

            egui::ScrollArea::horizontal()
                .id_salt("relations-table-scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    Table::new(&cols, self.theme).row_height(34.0).show(
                        ui,
                        matching_fks.len(),
                        |_| false,
                        |_| {},
                        |_| {},
                        |_| {},
                        |ui, row_idx, col_idx| {
                            let relation = matching_fks[row_idx];
                            match col_idx {
                                0 => {
                                    ui.horizontal(|ui| {
                                        ui.label(icon_text(Icon::ArrowRightLeft, "", self.theme.accent));
                                        ui.label(RichText::new(&relation.name).strong().color(self.theme.text_primary));
                                    });
                                }
                                1 => {
                                    ui.label(
                                        RichText::new(relation.from_columns.join(", "))
                                            .monospace()
                                            .color(self.theme.text_secondary),
                                    );
                                }
                                2 => {
                                    ui.label(
                                        RichText::new(format!("{}.{}", relation.to_schema, relation.to_table))
                                            .strong()
                                            .color(self.theme.text_primary),
                                    );
                                }
                                3 => {
                                    ui.label(
                                        RichText::new(relation.to_columns.join(", "))
                                            .monospace()
                                            .color(self.theme.text_secondary),
                                    );
                                }
                                4 => {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(
                                            RichText::new(format!(
                                                "UPDATE {} · DELETE {}{}",
                                                relation.on_update,
                                                relation.on_delete,
                                                if relation.deferrable {
                                                    if relation.initially_deferred {
                                                        " · DEFERRABLE INITIALLY DEFERRED"
                                                    } else {
                                                        " · DEFERRABLE"
                                                    }
                                                } else {
                                                    ""
                                                }
                                            ))
                                            .font(font_caption())
                                            .color(self.theme.text_muted),
                                        )
                                        .on_hover_text(format!("MATCH {}", relation.match_option));
                                        if Button::new(self.theme)
                                            .icon(Icon::ExternalLink)
                                            .variant(ButtonVariant::Ghost)
                                            .size(ButtonSize::IconSm)
                                            .tooltip("Open referenced table")
                                            .show(ui)
                                            .clicked()
                                        {
                                            switch_table = Some(relation.to_table.clone());
                                        }
                                    });
                                }
                                _ => {}
                            }
                        },
                    );
                });

            if let Some(target) = switch_table {
                self.open_table(target);
            }
        });
    }

    /// Draw the Constraints tab: categorized constraints (PK, FK, Unique, Check, NOT NULL).
    pub(super) fn draw_table_constraints_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table_state.table_info.clone() else {
            ui.label(RichText::new("Table structure is still loading…").color(self.theme.text_muted));
            return;
        };

        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            // Header & Filter Bar
            ui.horizontal(|ui| {
                section_label(ui, "CONSTRAINTS", self.theme);
                ui.add_space(8.0);
                input(
                    ui,
                    &mut self.table_state.table_metadata_search,
                    "Filter constraints…",
                    200.0,
                    self.theme,
                );
                if !self.table_state.table_metadata_search.is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Clear filter")
                        .show(ui)
                        .clicked()
                {
                    self.table_state.table_metadata_search.clear();
                }

                ui.add_space(12.0);

                // Category segment selector
                for (val, label) in [
                    ("all", "All"),
                    ("pk", "Primary Key"),
                    ("unique", "Unique"),
                    ("check", "Check"),
                    ("not_null", "Not Null"),
                ] {
                    let active = self.table_state.table_constraint_filter == val;
                    if ui.selectable_label(active, label).clicked() {
                        self.table_state.table_constraint_filter = val.to_owned();
                    }
                }
            });
            ui.add_space(8.0);

            // Collect all constraints into unified entries
            struct ConstraintRow {
                name: String,
                kind: &'static str,
                variant: BadgeVariant,
                icon: Icon,
                color: Color32,
                expression: String,
                details: String,
            }

            let mut list: Vec<ConstraintRow> = Vec::new();

            // 1. Primary Key
            if let Some(pk) = &info.primary_key {
                if self.table_state.table_constraint_filter == "all" || self.table_state.table_constraint_filter == "pk"
                {
                    list.push(ConstraintRow {
                        name: format!("pk_{}", info.name),
                        kind: "PRIMARY KEY",
                        variant: BadgeVariant::Warning,
                        icon: Icon::KeyRound,
                        color: self.theme.warning,
                        expression: pk.join(", "),
                        details: "Primary key column uniqueness and NOT NULL enforcement".to_owned(),
                    });
                }
            }

            // Unique constraints are represented separately from the primary
            // key index and foreign-key relation metadata.
            if self.table_state.table_constraint_filter == "all" || self.table_state.table_constraint_filter == "unique"
            {
                for idx in &info.indexes {
                    if idx.unique && !idx.primary {
                        list.push(ConstraintRow {
                            name: idx.name.clone(),
                            kind: "UNIQUE",
                            variant: BadgeVariant::Default,
                            icon: Icon::BadgeCheck,
                            color: self.theme.accent,
                            expression: idx.columns.join(", "),
                            details: "Unique constraint / unique index".to_owned(),
                        });
                    }
                }
            }

            // 4. Check Constraints
            if self.table_state.table_constraint_filter == "all" || self.table_state.table_constraint_filter == "check"
            {
                for chk in &info.check_constraints {
                    list.push(ConstraintRow {
                        name: chk.name.clone(),
                        kind: "CHECK",
                        variant: BadgeVariant::Success,
                        icon: Icon::ShieldCheck,
                        color: self.theme.success,
                        expression: chk.definition.clone(),
                        details: "SQL Boolean condition validation".to_owned(),
                    });
                }
            }

            // 5. NOT NULL columns
            if self.table_state.table_constraint_filter == "all"
                || self.table_state.table_constraint_filter == "not_null"
            {
                for col in &info.columns {
                    if !col.nullable && !col.is_primary_key {
                        list.push(ConstraintRow {
                            name: format!("nn_{}_{}", info.name, col.name),
                            kind: "NOT NULL",
                            variant: BadgeVariant::Outline,
                            icon: Icon::ShieldAlert,
                            color: self.theme.text_secondary,
                            expression: col.name.clone(),
                            details: format!("Column {} cannot contain NULL", col.name),
                        });
                    }
                }
            }

            let filter_lower = self.table_state.table_metadata_search.trim().to_lowercase();
            let matching_list: Vec<_> = list
                .into_iter()
                .filter(|row| {
                    if filter_lower.is_empty() {
                        true
                    } else {
                        row.name.to_lowercase().contains(&filter_lower)
                            || row.expression.to_lowercase().contains(&filter_lower)
                            || row.details.to_lowercase().contains(&filter_lower)
                    }
                })
                .collect();

            if matching_list.is_empty() {
                empty_state(
                    ui,
                    Icon::ShieldCheck,
                    "No constraints found",
                    "No table constraints match the selected category or search filter.",
                    self.theme,
                );
                return;
            }

            let cols = [
                TableColumn::fixed("Type", 140.0),
                TableColumn::new("Constraint Name").width(220.0),
                TableColumn::new("Target Columns / Expression").width(320.0),
                TableColumn::new("Details"),
            ];

            egui::ScrollArea::horizontal()
                .id_salt("constraints-table-scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    Table::new(&cols, self.theme).row_height(34.0).show(
                        ui,
                        matching_list.len(),
                        |_| false,
                        |_| {},
                        |_| {},
                        |_| {},
                        |ui, row_idx, col_idx| {
                            let row = &matching_list[row_idx];
                            match col_idx {
                                0 => {
                                    ui.horizontal(|ui| {
                                        ui.label(icon_text(row.icon, "", row.color));
                                        Badge::new(row.kind, self.theme)
                                            .variant(row.variant)
                                            .compact(true)
                                            .show(ui);
                                    });
                                }
                                1 => {
                                    ui.label(RichText::new(&row.name).strong().color(self.theme.text_primary));
                                }
                                2 => {
                                    ui.label(
                                        RichText::new(&row.expression)
                                            .monospace()
                                            .color(self.theme.text_secondary),
                                    );
                                }
                                3 => {
                                    ui.label(
                                        RichText::new(&row.details)
                                            .font(font_caption())
                                            .color(self.theme.text_muted),
                                    );
                                }
                                _ => {}
                            }
                        },
                    );
                });
        });
    }

    /// Draw the Dependencies tab: real dependency graph entries (Incoming & Outgoing).
    pub(super) fn draw_table_dependencies_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table_state.table_info.clone() else {
            ui.label(RichText::new("Table structure is still loading…").color(self.theme.text_muted));
            return;
        };

        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            // Keep the search and direction controls on separate wrapping rows so they remain
            // usable when the workspace is narrower than the full desktop layout.
            ui.horizontal_wrapped(|ui| {
                section_label(ui, "DEPENDENCIES", self.theme);
                ui.add_space(8.0);
                input(
                    ui,
                    &mut self.table_state.table_metadata_search,
                    "Filter dependencies…",
                    200.0,
                    self.theme,
                );
                if !self.table_state.table_metadata_search.is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Clear filter")
                        .show(ui)
                        .clicked()
                {
                    self.table_state.table_metadata_search.clear();
                }
            });
            ui.add_space(6.0);

            ui.horizontal_wrapped(|ui| {
                for (val, label) in [
                    ("all", "All"),
                    ("depends_on", "Depends On (Outgoing)"),
                    ("depended_by", "Depended By (Incoming)"),
                ] {
                    let active = self.table_state.table_dependency_filter == val;
                    if ui.selectable_label(active, label).clicked() {
                        self.table_state.table_dependency_filter = val.to_owned();
                    }
                }
                ui.add_space(12.0);
                ui.label(
                    RichText::new(format!("Total: {} dependencies", info.dependencies.len()))
                        .font(font_caption())
                        .color(self.theme.text_muted),
                );
            });
            ui.add_space(8.0);

            let filter_lower = self.table_state.table_metadata_search.trim().to_lowercase();
            let matching_deps: Vec<_> = info
                .dependencies
                .iter()
                .filter(|dep| {
                    let matches_direction = match self.table_state.table_dependency_filter.as_str() {
                        "depends_on" => dep.direction == UiDependencyDirection::DependsOn,
                        "depended_by" => dep.direction == UiDependencyDirection::DependedBy,
                        _ => true,
                    };
                    if !matches_direction {
                        return false;
                    }
                    if filter_lower.is_empty() {
                        true
                    } else {
                        dep.name.to_lowercase().contains(&filter_lower)
                            || dep.schema.to_lowercase().contains(&filter_lower)
                            || dep.details.to_lowercase().contains(&filter_lower)
                    }
                })
                .collect();

            if matching_deps.is_empty() {
                empty_state(
                    ui,
                    Icon::GitBranch,
                    "No dependencies found",
                    "No incoming or outgoing dependency relations match the filter.",
                    self.theme,
                );
                return;
            }

            let cols = [
                TableColumn::fixed("Direction", 150.0),
                TableColumn::fixed("Kind", 110.0),
                TableColumn::new("Object Name").width(220.0),
                TableColumn::new("Relationship Details"),
            ];

            let mut jump_target: Option<String> = None;

            egui::ScrollArea::horizontal()
                .id_salt("dependencies-table-scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    Table::new(&cols, self.theme).row_height(34.0).show(
                        ui,
                        matching_deps.len(),
                        |_| false,
                        |_| {},
                        |_| {},
                        |_| {},
                        |ui, row_idx, col_idx| {
                            let dep = matching_deps[row_idx];
                            match col_idx {
                                0 => {
                                    ui.horizontal(|ui| match dep.direction {
                                        UiDependencyDirection::DependsOn => {
                                            ui.label(icon_text(Icon::ArrowUpRight, "", self.theme.warning));
                                            Badge::new("DEPENDS ON", self.theme)
                                                .variant(BadgeVariant::Warning)
                                                .compact(true)
                                                .show(ui);
                                        }
                                        UiDependencyDirection::DependedBy => {
                                            ui.label(icon_text(Icon::ArrowDownLeft, "", self.theme.accent));
                                            Badge::new("DEPENDED BY", self.theme)
                                                .variant(BadgeVariant::Default)
                                                .compact(true)
                                                .show(ui);
                                        }
                                    });
                                }
                                1 => {
                                    let (label, variant) = match dep.kind {
                                        UiDependencyKind::Table => ("TABLE", BadgeVariant::Default),
                                        UiDependencyKind::View => ("VIEW", BadgeVariant::Secondary),
                                        UiDependencyKind::ForeignKey => ("FK", BadgeVariant::Outline),
                                        UiDependencyKind::Trigger => ("TRIGGER", BadgeVariant::Warning),
                                        UiDependencyKind::Function => ("FUNCTION", BadgeVariant::Success),
                                        UiDependencyKind::Sequence => ("SEQUENCE", BadgeVariant::Secondary),
                                    };
                                    Badge::new(label, self.theme).variant(variant).compact(true).show(ui);
                                }
                                2 => {
                                    let qualified = if dep.schema.is_empty() {
                                        dep.name.clone()
                                    } else {
                                        format!("{}.{}", dep.schema, dep.name)
                                    };
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&qualified).strong().color(self.theme.text_primary));
                                        if dep.kind == UiDependencyKind::Table
                                            && dep.name != info.name
                                            && Button::new(self.theme)
                                                .icon(Icon::ExternalLink)
                                                .variant(ButtonVariant::Ghost)
                                                .size(ButtonSize::IconSm)
                                                .tooltip("Open table workspace")
                                                .show(ui)
                                                .clicked()
                                        {
                                            jump_target = Some(dep.name.clone());
                                        }
                                    });
                                }
                                3 => {
                                    ui.label(
                                        RichText::new(&dep.details)
                                            .font(font_caption())
                                            .color(self.theme.text_secondary),
                                    );
                                }
                                _ => {}
                            }
                        },
                    );
                });

            if let Some(target) = jump_target {
                self.open_table(target);
            }
        });
    }
}
