use super::*;
use crate::components::badge::{Badge, BadgeVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::table::{Table, TableColumn};
use crate::{UiDependencyDirection, UiDependencyKind};
use egui::{Align, Color32, Layout, RichText};
use lucide_icons::Icon;

impl DbProApp {
    /// Draw the Foreign Keys tab: relations table with target jump and copy actions.
    pub(super) fn draw_table_relations_view(&mut self, ui: &mut egui::Ui) {
        let Some(info) = self.table.state.table_info.clone() else {
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
                    &mut self.table.state.table_metadata_search,
                    "Filter foreign keys…",
                    220.0,
                    self.theme,
                );
                if !self.table.state.table_metadata_search.is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Clear filter")
                        .show(ui)
                        .clicked()
                {
                    self.table.state.table_metadata_search.clear();
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

            let filter_lower = self.table.state.table_metadata_search.trim().to_lowercase();
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
        let Some(info) = self.table.state.table_info.clone() else {
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
                    &mut self.table.state.table_metadata_search,
                    "Filter constraints…",
                    200.0,
                    self.theme,
                );
                if !self.table.state.table_metadata_search.is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Clear filter")
                        .show(ui)
                        .clicked()
                {
                    self.table.state.table_metadata_search.clear();
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
                    let active = self.table.state.table_constraint_filter == val;
                    if ui.selectable_label(active, label).clicked() {
                        self.table.state.table_constraint_filter = val.to_owned();
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
                if self.table.state.table_constraint_filter == "all" || self.table.state.table_constraint_filter == "pk"
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
            if self.table.state.table_constraint_filter == "all" || self.table.state.table_constraint_filter == "unique"
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
            if self.table.state.table_constraint_filter == "all" || self.table.state.table_constraint_filter == "check"
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
            if self.table.state.table_constraint_filter == "all"
                || self.table.state.table_constraint_filter == "not_null"
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

            let filter_lower = self.table.state.table_metadata_search.trim().to_lowercase();
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
        let Some(info) = self.table.state.table_info.clone() else {
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
                    &mut self.table.state.table_metadata_search,
                    "Filter dependencies…",
                    200.0,
                    self.theme,
                );
                if !self.table.state.table_metadata_search.is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Clear filter")
                        .show(ui)
                        .clicked()
                {
                    self.table.state.table_metadata_search.clear();
                }
            });
            ui.add_space(6.0);

            ui.horizontal_wrapped(|ui| {
                for (val, label) in [
                    ("all", "All"),
                    ("depends_on", "Depends On (Outgoing)"),
                    ("depended_by", "Depended By (Incoming)"),
                ] {
                    let active = self.table.state.table_dependency_filter == val;
                    if ui.selectable_label(active, label).clicked() {
                        self.table.state.table_dependency_filter = val.to_owned();
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

            let filter_lower = self.table.state.table_metadata_search.trim().to_lowercase();
            let matching_deps: Vec<_> = info
                .dependencies
                .iter()
                .filter(|dep| {
                    let matches_direction = match self.table.state.table_dependency_filter.as_str() {
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
