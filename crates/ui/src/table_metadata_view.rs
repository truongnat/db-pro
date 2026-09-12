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
        let Some(info) = self.table_info.clone() else {
            self.draw_table_structure_placeholder(ui);
            return;
        };

        self.draw_table_structure_metrics(ui, &info);
        ui.add_space(8.0);
        self.draw_table_structure_columns_table(ui, &info);
    }

    /// Top metric chips summarising columns, keys, and row count.
    fn draw_table_structure_metrics(&self, ui: &mut egui::Ui, info: &UiTableInfo) {
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                section_label(ui, "METRICS", self.theme);
                Badge::new(&format!("{} columns", info.columns.len()), self.theme)
                    .variant(BadgeVariant::Default)
                    .show(ui);
                if let Some(pk) = &info.primary_key {
                    Badge::new(&format!("PK: {}", pk.join(", ")), self.theme)
                        .variant(BadgeVariant::Warning)
                        .show(ui);
                }
                Badge::new(&format!("{} indexes", info.indexes.len()), self.theme)
                    .variant(BadgeVariant::Secondary)
                    .show(ui);
                Badge::new(&format!("{} foreign keys", info.foreign_keys.len()), self.theme)
                    .variant(BadgeVariant::Secondary)
                    .show(ui);
                if !info.check_constraints.is_empty() {
                    Badge::new(
                        &format!("{} check constraints", info.check_constraints.len()),
                        self.theme,
                    )
                    .variant(BadgeVariant::Outline)
                    .show(ui);
                }
                if !info.dependencies.is_empty() {
                    Badge::new(&format!("{} dependencies", info.dependencies.len()), self.theme)
                        .variant(BadgeVariant::Outline)
                        .show(ui);
                }
                if let Some(row_count) = info.row_count {
                    Badge::new(&format!("{row_count} rows"), self.theme)
                        .variant(BadgeVariant::Secondary)
                        .show(ui);
                }
            });
        });
    }

    /// Columns table with search filter, type badges, nullability, PK/FK flags, and default expressions.
    fn draw_table_structure_columns_table(&mut self, ui: &mut egui::Ui, info: &UiTableInfo) {
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                section_label(ui, "COLUMNS", self.theme);
                ui.add_space(8.0);
                input(
                    ui,
                    &mut self.table_structure_search,
                    "Search columns or types…",
                    220.0,
                    self.theme,
                );
                if !self.table_structure_search.is_empty()
                    && compact_icon_button(ui, Icon::X, self.theme)
                        .on_hover_text("Clear filter")
                        .clicked()
                {
                    self.table_structure_search.clear();
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

            let filter_lower = self.table_structure_search.trim().to_lowercase();
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
                TableColumn::new("Column Name").width(220.0),
                TableColumn::new("Data Type").width(180.0),
                TableColumn::fixed("Nullable", 110.0),
                TableColumn::fixed("Key", 90.0),
                TableColumn::new("Default Expression"),
            ];

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
                                ui.label(
                                    RichText::new(&column.name)
                                        .font(font_ui_label())
                                        .strong()
                                        .color(self.theme.text_primary),
                                );
                            });
                        }
                        1 => {
                            ui.label(
                                RichText::new(&column.data_type)
                                    .monospace()
                                    .color(self.theme.text_secondary),
                            );
                        }
                        2 => {
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
                        3 => {
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
                            } else {
                                ui.label(RichText::new("—").font(font_caption()).color(self.theme.text_muted));
                            }
                        }
                        4 => {
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
    }

    /// Draw the Indexes tab: full index metadata table with search filter and unique badges.
    pub(super) fn draw_table_indexes_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table_info.clone() else {
            ui.label(RichText::new("Table structure is still loading…").color(self.theme.text_muted));
            return;
        };

        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                section_label(ui, "INDEXES", self.theme);
                ui.add_space(8.0);
                input(
                    ui,
                    &mut self.table_metadata_search,
                    "Filter indexes…",
                    220.0,
                    self.theme,
                );
                if !self.table_metadata_search.is_empty()
                    && compact_icon_button(ui, Icon::X, self.theme)
                        .on_hover_text("Clear filter")
                        .clicked()
                {
                    self.table_metadata_search.clear();
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

            let filter_lower = self.table_metadata_search.trim().to_lowercase();
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
                TableColumn::fixed("Type", 120.0),
                TableColumn::new("Status"),
            ];

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
                                ui.label(RichText::new(&index.name).strong().color(self.theme.text_primary));
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
                            if index.unique {
                                Badge::new("UNIQUE", self.theme)
                                    .variant(BadgeVariant::Default)
                                    .compact(true)
                                    .show(ui);
                            } else {
                                Badge::new("BTREE INDEX", self.theme)
                                    .variant(BadgeVariant::Secondary)
                                    .compact(true)
                                    .show(ui);
                            }
                        }
                        3 => {
                            ui.label(
                                RichText::new(if index.unique {
                                    "Enforces uniqueness"
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
    }

    /// Draw the Foreign Keys tab: relations table with target jump and copy actions.
    pub(super) fn draw_table_relations_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table_info.clone() else {
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
                    &mut self.table_metadata_search,
                    "Filter foreign keys…",
                    220.0,
                    self.theme,
                );
                if !self.table_metadata_search.is_empty()
                    && compact_icon_button(ui, Icon::X, self.theme)
                        .on_hover_text("Clear filter")
                        .clicked()
                {
                    self.table_metadata_search.clear();
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

            let filter_lower = self.table_metadata_search.trim().to_lowercase();
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
                        4 if Button::new(self.theme)
                            .icon(Icon::ExternalLink)
                            .text("Open Table")
                            .size(ButtonSize::Sm)
                            .variant(ButtonVariant::Ghost)
                            .show(ui)
                            .clicked() =>
                        {
                            switch_table = Some(relation.to_table.clone());
                        }
                        _ => {}
                    }
                },
            );

            if let Some(target) = switch_table {
                self.selected_table = Some(target);
                self.request_table_info();
                self.request_table_data();
            }
        });
    }

    /// Draw the Constraints tab: categorized constraints (PK, FK, Unique, Check, NOT NULL).
    pub(super) fn draw_table_constraints_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table_info.clone() else {
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
                    &mut self.table_metadata_search,
                    "Filter constraints…",
                    200.0,
                    self.theme,
                );
                if !self.table_metadata_search.is_empty()
                    && compact_icon_button(ui, Icon::X, self.theme)
                        .on_hover_text("Clear filter")
                        .clicked()
                {
                    self.table_metadata_search.clear();
                }

                ui.add_space(12.0);

                // Category segment selector
                for (val, label) in [
                    ("all", "All"),
                    ("pk", "Primary Key"),
                    ("fk", "Foreign Key"),
                    ("unique", "Unique"),
                    ("check", "Check"),
                    ("not_null", "Not Null"),
                ] {
                    let active = self.table_constraint_filter == val;
                    if ui.selectable_label(active, label).clicked() {
                        self.table_constraint_filter = val.to_owned();
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
                if self.table_constraint_filter == "all" || self.table_constraint_filter == "pk" {
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

            // 2. Foreign Keys
            if self.table_constraint_filter == "all" || self.table_constraint_filter == "fk" {
                for fk in &info.foreign_keys {
                    list.push(ConstraintRow {
                        name: fk.name.clone(),
                        kind: "FOREIGN KEY",
                        variant: BadgeVariant::Default,
                        icon: Icon::ArrowRightLeft,
                        color: self.theme.accent,
                        expression: format!(
                            "({}) → {}.{}({})",
                            fk.from_columns.join(", "),
                            fk.to_schema,
                            fk.to_table,
                            fk.to_columns.join(", ")
                        ),
                        details: format!("References {}.{}", fk.to_schema, fk.to_table),
                    });
                }
            }

            // 3. Unique constraints from indexes
            if self.table_constraint_filter == "all" || self.table_constraint_filter == "unique" {
                for idx in &info.indexes {
                    if idx.unique {
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
            if self.table_constraint_filter == "all" || self.table_constraint_filter == "check" {
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
            if self.table_constraint_filter == "all" || self.table_constraint_filter == "not_null" {
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

            let filter_lower = self.table_metadata_search.trim().to_lowercase();
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
    }

    /// Draw the Dependencies tab: real dependency graph entries (Incoming & Outgoing).
    pub(super) fn draw_table_dependencies_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table_info.clone() else {
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
                    &mut self.table_metadata_search,
                    "Filter dependencies…",
                    200.0,
                    self.theme,
                );
                if !self.table_metadata_search.is_empty()
                    && compact_icon_button(ui, Icon::X, self.theme)
                        .on_hover_text("Clear filter")
                        .clicked()
                {
                    self.table_metadata_search.clear();
                }
            });
            ui.add_space(6.0);

            ui.horizontal_wrapped(|ui| {
                for (val, label) in [
                    ("all", "All"),
                    ("depends_on", "Depends On (Outgoing)"),
                    ("depended_by", "Depended By (Incoming)"),
                ] {
                    let active = self.table_dependency_filter == val;
                    if ui.selectable_label(active, label).clicked() {
                        self.table_dependency_filter = val.to_owned();
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

            let filter_lower = self.table_metadata_search.trim().to_lowercase();
            let matching_deps: Vec<_> = info
                .dependencies
                .iter()
                .filter(|dep| {
                    let matches_direction = match self.table_dependency_filter.as_str() {
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
                                    && compact_icon_button(ui, Icon::ExternalLink, self.theme)
                                        .on_hover_text("Open table workspace")
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

            if let Some(target) = jump_target {
                self.selected_table = Some(target);
                self.request_table_info();
                self.request_table_data();
            }
        });
    }
}
