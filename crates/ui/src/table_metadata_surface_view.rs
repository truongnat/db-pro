//! Table metadata presentation for constraints and dependencies.

use super::*;
use crate::components::badge::{Badge, BadgeVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::table::{Table, TableColumn};
use crate::{UiDependencyDirection, UiDependencyKind, UiTableInfo};
use egui::RichText;
use lucide_icons::Icon;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TableMetadataAction {
    OpenTable(String),
}

pub(super) struct TableMetadataContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) info: &'a UiTableInfo,
    pub(super) search: &'a mut String,
    pub(super) constraint_filter: &'a mut String,
    pub(super) dependency_filter: &'a mut String,
}

impl TableMetadataContext<'_> {
    pub(super) fn draw_constraints(&mut self, ui: &mut egui::Ui) {
        let rows = self.constraint_rows();
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            self.draw_constraint_toolbar(ui);
            let filter = self.search.trim().to_lowercase();
            let rows = rows
                .into_iter()
                .filter(|row| filter.is_empty() || row.matches(&filter))
                .collect::<Vec<_>>();
            if rows.is_empty() {
                empty_state(
                    ui,
                    Icon::ShieldCheck,
                    "No constraints found",
                    "No table constraints match the selected category or search filter.",
                    self.theme,
                );
                return;
            }
            let columns = [
                TableColumn::fixed("Type", 140.0),
                TableColumn::new("Constraint Name").width(220.0),
                TableColumn::new("Target Columns / Expression").width(320.0),
                TableColumn::new("Details"),
            ];
            egui::ScrollArea::horizontal()
                .id_salt("constraints-table-scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    Table::new(&columns, self.theme).row_height(34.0).show(
                        ui,
                        rows.len(),
                        |_| false,
                        |_| {},
                        |_| {},
                        |_| {},
                        |ui, row_idx, col_idx| self.draw_constraint_cell(ui, &rows[row_idx], col_idx),
                    );
                });
        });
    }

    pub(super) fn draw_dependencies(&mut self, ui: &mut egui::Ui) -> Vec<TableMetadataAction> {
        let mut actions = Vec::new();
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            self.draw_dependency_toolbar(ui);
            let filter = self.search.trim().to_lowercase();
            let dependencies = self
                .info
                .dependencies
                .iter()
                .filter(|dependency| {
                    dependency_matches_direction(dependency, self.dependency_filter)
                        && (filter.is_empty() || dependency_matches_filter(dependency, &filter))
                })
                .collect::<Vec<_>>();
            if dependencies.is_empty() {
                empty_state(
                    ui,
                    Icon::GitBranch,
                    "No dependencies found",
                    "No incoming or outgoing dependency relations match the filter.",
                    self.theme,
                );
                return;
            }
            actions.extend(self.draw_dependency_table(ui, dependencies));
        });
        actions
    }

    fn draw_dependency_table(
        &self,
        ui: &mut egui::Ui,
        dependencies: Vec<&crate::UiTableDependency>,
    ) -> Vec<TableMetadataAction> {
        let mut actions = Vec::new();
        let columns = [
            TableColumn::fixed("Direction", 150.0),
            TableColumn::fixed("Kind", 110.0),
            TableColumn::new("Object Name").width(220.0),
            TableColumn::new("Relationship Details"),
        ];
        egui::ScrollArea::horizontal()
            .id_salt("dependencies-table-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                Table::new(&columns, self.theme).row_height(34.0).show(
                    ui,
                    dependencies.len(),
                    |_| false,
                    |_| {},
                    |_| {},
                    |_| {},
                    |ui, row_idx, col_idx| {
                        draw_dependency_cell(ui, dependencies[row_idx], col_idx, self.info, self.theme, &mut actions);
                    },
                );
            });
        actions
    }

    fn draw_constraint_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            section_label(ui, "CONSTRAINTS", self.theme);
            ui.add_space(8.0);
            self.draw_search_input(ui, "Filter constraints…", 200.0);
            ui.add_space(12.0);
            for (value, label) in [
                ("all", "All"),
                ("pk", "Primary Key"),
                ("unique", "Unique"),
                ("check", "Check"),
                ("not_null", "Not Null"),
            ] {
                if ui.selectable_label(*self.constraint_filter == value, label).clicked() {
                    *self.constraint_filter = value.to_owned();
                }
            }
        });
        ui.add_space(8.0);
    }

    fn draw_dependency_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            section_label(ui, "DEPENDENCIES", self.theme);
            ui.add_space(8.0);
            self.draw_search_input(ui, "Filter dependencies…", 200.0);
        });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for (value, label) in [
                ("all", "All"),
                ("depends_on", "Depends On (Outgoing)"),
                ("depended_by", "Depended By (Incoming)"),
            ] {
                if ui.selectable_label(*self.dependency_filter == value, label).clicked() {
                    *self.dependency_filter = value.to_owned();
                }
            }
            ui.add_space(12.0);
            ui.label(
                RichText::new(format!("Total: {} dependencies", self.info.dependencies.len()))
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );
        });
        ui.add_space(8.0);
    }

    fn constraint_rows(&self) -> Vec<ConstraintRow> {
        let mut rows = Vec::new();
        let filter = self.constraint_filter.as_str();
        if filter == "all" || filter == "pk" {
            rows.extend(self.primary_key_rows());
        }
        if filter == "all" || filter == "unique" {
            rows.extend(self.unique_rows());
        }
        if filter == "all" || filter == "check" {
            rows.extend(self.check_rows());
        }
        if filter == "all" || filter == "not_null" {
            rows.extend(self.not_null_rows());
        }
        rows
    }

    fn primary_key_rows(&self) -> Vec<ConstraintRow> {
        self.info
            .primary_key
            .as_ref()
            .map(|primary_key| {
                vec![ConstraintRow {
                    name: format!("pk_{}", self.info.name),
                    kind: "PRIMARY KEY",
                    variant: BadgeVariant::Warning,
                    icon: Icon::KeyRound,
                    color: self.theme.warning,
                    expression: primary_key.join(", "),
                    details: "Primary key column uniqueness and NOT NULL enforcement".to_owned(),
                }]
            })
            .unwrap_or_default()
    }

    fn unique_rows(&self) -> Vec<ConstraintRow> {
        self.info
            .indexes
            .iter()
            .filter(|index| index.unique && !index.primary)
            .map(|index| ConstraintRow {
                name: index.name.clone(),
                kind: "UNIQUE",
                variant: BadgeVariant::Default,
                icon: Icon::BadgeCheck,
                color: self.theme.accent,
                expression: index.columns.join(", "),
                details: "Unique constraint / unique index".to_owned(),
            })
            .collect()
    }

    fn check_rows(&self) -> Vec<ConstraintRow> {
        self.info
            .check_constraints
            .iter()
            .map(|check| ConstraintRow {
                name: check.name.clone(),
                kind: "CHECK",
                variant: BadgeVariant::Success,
                icon: Icon::ShieldCheck,
                color: self.theme.success,
                expression: check.definition.clone(),
                details: "SQL Boolean condition validation".to_owned(),
            })
            .collect()
    }

    fn not_null_rows(&self) -> Vec<ConstraintRow> {
        self.info
            .columns
            .iter()
            .filter(|column| !column.nullable && !column.is_primary_key)
            .map(|column| ConstraintRow {
                name: format!("nn_{}_{}", self.info.name, column.name),
                kind: "NOT NULL",
                variant: BadgeVariant::Outline,
                icon: Icon::ShieldAlert,
                color: self.theme.text_secondary,
                expression: column.name.clone(),
                details: format!("Column {} cannot contain NULL", column.name),
            })
            .collect()
    }

    fn draw_search_input(&mut self, ui: &mut egui::Ui, hint: &str, width: f32) {
        input(ui, self.search, hint, width, self.theme);
        if !self.search.is_empty()
            && Button::new(self.theme)
                .icon(Icon::X)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .tooltip("Clear filter")
                .show(ui)
                .clicked()
        {
            self.search.clear();
        }
    }

    fn draw_constraint_cell(&self, ui: &mut egui::Ui, row: &ConstraintRow, col_idx: usize) {
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
    }
}

struct ConstraintRow {
    name: String,
    kind: &'static str,
    variant: BadgeVariant,
    icon: Icon,
    color: egui::Color32,
    expression: String,
    details: String,
}

impl ConstraintRow {
    fn matches(&self, filter: &str) -> bool {
        self.name.to_lowercase().contains(filter)
            || self.expression.to_lowercase().contains(filter)
            || self.details.to_lowercase().contains(filter)
    }
}

fn dependency_matches_direction(dependency: &crate::UiTableDependency, filter: &str) -> bool {
    match filter {
        "depends_on" => dependency.direction == UiDependencyDirection::DependsOn,
        "depended_by" => dependency.direction == UiDependencyDirection::DependedBy,
        _ => true,
    }
}

fn dependency_matches_filter(dependency: &crate::UiTableDependency, filter: &str) -> bool {
    dependency.name.to_lowercase().contains(filter)
        || dependency.schema.to_lowercase().contains(filter)
        || dependency.details.to_lowercase().contains(filter)
}

fn draw_dependency_cell(
    ui: &mut egui::Ui,
    dependency: &crate::UiTableDependency,
    col_idx: usize,
    info: &UiTableInfo,
    theme: DbProTheme,
    actions: &mut Vec<TableMetadataAction>,
) {
    match col_idx {
        0 => {
            ui.horizontal(|ui| match dependency.direction {
                UiDependencyDirection::DependsOn => {
                    ui.label(icon_text(Icon::ArrowUpRight, "", theme.warning));
                    Badge::new("DEPENDS ON", theme)
                        .variant(BadgeVariant::Warning)
                        .compact(true)
                        .show(ui);
                }
                UiDependencyDirection::DependedBy => {
                    ui.label(icon_text(Icon::ArrowDownLeft, "", theme.accent));
                    Badge::new("DEPENDED BY", theme)
                        .variant(BadgeVariant::Default)
                        .compact(true)
                        .show(ui);
                }
            });
        }
        1 => {
            let (label, variant) = match dependency.kind {
                UiDependencyKind::Table => ("TABLE", BadgeVariant::Default),
                UiDependencyKind::View => ("VIEW", BadgeVariant::Secondary),
                UiDependencyKind::ForeignKey => ("FK", BadgeVariant::Outline),
                UiDependencyKind::Trigger => ("TRIGGER", BadgeVariant::Warning),
                UiDependencyKind::Function => ("FUNCTION", BadgeVariant::Success),
                UiDependencyKind::Sequence => ("SEQUENCE", BadgeVariant::Secondary),
            };
            Badge::new(label, theme).variant(variant).compact(true).show(ui);
        }
        2 => {
            let qualified = if dependency.schema.is_empty() {
                dependency.name.clone()
            } else {
                format!("{}.{}", dependency.schema, dependency.name)
            };
            ui.horizontal(|ui| {
                ui.label(RichText::new(&qualified).strong().color(theme.text_primary));
                if dependency.kind == UiDependencyKind::Table
                    && dependency.name != info.name
                    && Button::new(theme)
                        .icon(Icon::ExternalLink)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Open table workspace")
                        .show(ui)
                        .clicked()
                {
                    actions.push(TableMetadataAction::OpenTable(dependency.name.clone()));
                }
            });
        }
        3 => {
            ui.label(
                RichText::new(&dependency.details)
                    .font(font_caption())
                    .color(theme.text_secondary),
            );
        }
        _ => {}
    }
}
