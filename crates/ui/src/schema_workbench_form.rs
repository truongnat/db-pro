//! Schema workbench object form and SQL preview panes.
use super::schema_workbench::{ConstraintKindUi, SchemaWorkbenchMode};
use super::*;
use crate::components::{Button, ButtonSize, ButtonVariant};
use db_pro_core::domain::object_mutation::ObjectAction;

impl DbProApp {
    pub(super) fn draw_workbench_form(&mut self, ui: &mut egui::Ui) {
        let mode = self.schema_workbench.mode;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(220.0);
                ui.label(RichText::new("Schema").small().color(self.theme.text_secondary));
                crate::components::input::Input::new(&mut self.schema_workbench.schema, "main", self.theme)
                    .width(220.0)
                    .show(ui);
            });
            ui.add_space(SPACE_MD);
            ui.vertical(|ui| {
                ui.set_width(280.0);
                ui.label(RichText::new("Name").small().color(self.theme.text_secondary));
                crate::components::input::Input::new(&mut self.schema_workbench.name, "object_name", self.theme)
                    .width(280.0)
                    .show(ui);
            });
        });
        ui.add_space(SPACE_SM);

        match mode {
            SchemaWorkbenchMode::Table => {
                ui.label(
                    RichText::new("Columns CSV (name:type[:pk|:nn])")
                        .small()
                        .color(self.theme.text_secondary),
                );
                crate::components::input::Input::new(
                    &mut self.schema_workbench.columns_csv,
                    "id:INTEGER:pk,name:TEXT",
                    self.theme,
                )
                .width(520.0)
                .show(ui);
            }
            SchemaWorkbenchMode::Column => {
                ui.horizontal(|ui| {
                    ui.label("Table");
                    ui.text_edit_singleline(&mut self.schema_workbench.parent_table);
                    ui.label("Type");
                    ui.text_edit_singleline(&mut self.schema_workbench.data_type);
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.schema_workbench.nullable, "Nullable");
                    ui.checkbox(&mut self.schema_workbench.is_pk, "PK (create table only)");
                    ui.label("Default");
                    ui.text_edit_singleline(&mut self.schema_workbench.default_expr);
                });
                ui.horizontal(|ui| {
                    ui.label("Rename to");
                    ui.text_edit_singleline(&mut self.schema_workbench.new_name);
                });
            }
            SchemaWorkbenchMode::View => {
                ui.checkbox(&mut self.schema_workbench.materialized, "Materialized");
                ui.label("SELECT body");
                ui.add(
                    egui::TextEdit::multiline(&mut self.schema_workbench.select_sql)
                        .desired_rows(4)
                        .desired_width(f32::INFINITY),
                );
            }
            SchemaWorkbenchMode::Index => {
                ui.horizontal(|ui| {
                    ui.label("Table");
                    ui.text_edit_singleline(&mut self.schema_workbench.parent_table);
                    ui.checkbox(&mut self.schema_workbench.unique, "Unique");
                });
                ui.label("Columns CSV");
                ui.text_edit_singleline(&mut self.schema_workbench.columns_csv);
            }
            SchemaWorkbenchMode::Constraint => {
                ui.horizontal(|ui| {
                    ui.label("Table");
                    ui.text_edit_singleline(&mut self.schema_workbench.parent_table);
                    egui::ComboBox::from_id_salt("constraint_kind")
                        .selected_text(match self.schema_workbench.constraint_kind {
                            ConstraintKindUi::PrimaryKey => "Primary key",
                            ConstraintKindUi::Unique => "Unique",
                            ConstraintKindUi::Check => "Check",
                            ConstraintKindUi::ForeignKey => "Foreign key",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.schema_workbench.constraint_kind,
                                ConstraintKindUi::PrimaryKey,
                                "Primary key",
                            );
                            ui.selectable_value(
                                &mut self.schema_workbench.constraint_kind,
                                ConstraintKindUi::Unique,
                                "Unique",
                            );
                            ui.selectable_value(
                                &mut self.schema_workbench.constraint_kind,
                                ConstraintKindUi::Check,
                                "Check",
                            );
                            ui.selectable_value(
                                &mut self.schema_workbench.constraint_kind,
                                ConstraintKindUi::ForeignKey,
                                "Foreign key",
                            );
                        });
                });
                ui.label("Columns CSV");
                ui.text_edit_singleline(&mut self.schema_workbench.columns_csv);
                if self.schema_workbench.constraint_kind == ConstraintKindUi::Check {
                    ui.label("Expression");
                    ui.text_edit_singleline(&mut self.schema_workbench.expression);
                }
                if self.schema_workbench.constraint_kind == ConstraintKindUi::ForeignKey {
                    ui.horizontal(|ui| {
                        ui.label("Ref schema");
                        ui.text_edit_singleline(&mut self.schema_workbench.ref_schema);
                        ui.label("Ref table");
                        ui.text_edit_singleline(&mut self.schema_workbench.ref_table);
                    });
                    ui.label("Ref columns CSV");
                    ui.text_edit_singleline(&mut self.schema_workbench.ref_columns_csv);
                    ui.label("ON DELETE");
                    ui.text_edit_singleline(&mut self.schema_workbench.on_delete);
                }
            }
            SchemaWorkbenchMode::Trigger => {
                ui.horizontal(|ui| {
                    ui.label("Table");
                    ui.text_edit_singleline(&mut self.schema_workbench.parent_table);
                    ui.label("Timing");
                    ui.text_edit_singleline(&mut self.schema_workbench.timing);
                    ui.label("Event");
                    ui.text_edit_singleline(&mut self.schema_workbench.event);
                });
                ui.label("Body");
                ui.add(
                    egui::TextEdit::multiline(&mut self.schema_workbench.body)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY),
                );
            }
            SchemaWorkbenchMode::Sequence => {
                ui.horizontal(|ui| {
                    ui.label("Start");
                    ui.text_edit_singleline(&mut self.schema_workbench.start);
                    ui.label("Increment");
                    ui.text_edit_singleline(&mut self.schema_workbench.increment);
                    ui.checkbox(&mut self.schema_workbench.cycle, "Cycle");
                });
            }
            SchemaWorkbenchMode::Type => {
                ui.label("Enum values CSV");
                ui.text_edit_singleline(&mut self.schema_workbench.enum_values_csv);
            }
            SchemaWorkbenchMode::SchemaDb => {
                ui.label("Schema name uses Name field; Database create/drop uses Name as DB name.");
                ui.checkbox(&mut self.schema_workbench.cascade, "CASCADE on drop schema");
            }
            SchemaWorkbenchMode::Extension => {
                ui.label("Extension schema (optional)");
                ui.text_edit_singleline(&mut self.schema_workbench.extension_schema);
                ui.checkbox(&mut self.schema_workbench.cascade, "CASCADE on drop");
            }
            SchemaWorkbenchMode::Comment => {
                ui.horizontal(|ui| {
                    ui.label("Parent (column comments)");
                    ui.text_edit_singleline(&mut self.schema_workbench.parent_table);
                });
                ui.label("Comment text (empty clears)");
                ui.text_edit_singleline(&mut self.schema_workbench.comment_text);
            }
            SchemaWorkbenchMode::Partition => {
                ui.horizontal(|ui| {
                    ui.label("Parent table");
                    ui.text_edit_singleline(&mut self.schema_workbench.parent_table);
                });
                ui.label("FOR VALUES …");
                ui.text_edit_singleline(&mut self.schema_workbench.partition_bound);
            }
            SchemaWorkbenchMode::Dependencies | SchemaWorkbenchMode::Docs => {}
        }

        ui.add_space(SPACE_MD);
        ui.horizontal_wrapped(|ui| {
            if secondary_button(ui, "Plan create", self.theme).clicked() {
                self.plan_workbench_action(ObjectAction::Create);
            }
            if secondary_button(ui, "Plan drop", self.theme).clicked() {
                self.plan_workbench_action(ObjectAction::Drop);
            }
            if mode == SchemaWorkbenchMode::Column && secondary_button(ui, "Plan rename", self.theme).clicked() {
                self.plan_workbench_action(ObjectAction::Rename);
            }
            if mode == SchemaWorkbenchMode::View
                && self.schema_workbench.materialized
                && secondary_button(ui, "Plan refresh", self.theme).clicked()
            {
                self.plan_workbench_action(ObjectAction::Refresh);
            }
            if mode == SchemaWorkbenchMode::Comment && secondary_button(ui, "Plan comment", self.theme).clicked() {
                self.plan_workbench_action(ObjectAction::Comment);
            }
            if mode == SchemaWorkbenchMode::SchemaDb {
                if secondary_button(ui, "Plan create database", self.theme).clicked() {
                    self.plan_database_action(ObjectAction::Create);
                }
                if secondary_button(ui, "Plan drop database", self.theme).clicked() {
                    self.plan_database_action(ObjectAction::Drop);
                }
            }
        });
    }

    pub(super) fn draw_workbench_preview(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 8.0).max(0.0));
            section_label(ui, "PREVIEW", self.theme);
            ui.add_space(SPACE_SM);
            if let Some(err) = &self.schema_workbench.preview_error {
                ui.colored_label(self.theme.danger, err);
                ui.add_space(SPACE_XS);
            }
            if !self.schema_workbench.preview_safety.is_empty() {
                ui.label(
                    RichText::new(format!(
                        "Safety: {} · fingerprint {}",
                        self.schema_workbench.preview_safety, self.schema_workbench.preview_fingerprint
                    ))
                    .small()
                    .color(self.theme.text_muted),
                );
                ui.add_space(SPACE_XS);
            }
            ui.label(RichText::new("SQL").small().strong().color(self.theme.text_secondary));
            editor_frame(self.theme).show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.schema_workbench.preview_sql)
                        .desired_rows(8)
                        .desired_width(f32::INFINITY)
                        .code_editor(),
                );
            });
            ui.add_space(SPACE_SM);
            ui.horizontal(|ui| {
                let can_apply = !self.schema_workbench.preview_sql.trim().is_empty()
                    && self.schema_workbench.preview_error.is_none()
                    && self.can_mutate_active_connection();
                if Button::new(self.theme)
                    .text("Apply DDL…")
                    .variant(ButtonVariant::Default)
                    .size(ButtonSize::Sm)
                    .enabled(can_apply)
                    .show(ui)
                    .clicked()
                {
                    self.schema_workbench.apply_confirmation = true;
                }
                let can_open_editor = !self.schema_workbench.preview_sql.trim().is_empty();
                if Button::new(self.theme)
                    .text("Open in SQL editor")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .enabled(can_open_editor)
                    .show(ui)
                    .clicked()
                    && can_open_editor
                {
                    let sql = self.schema_workbench.preview_sql.clone();
                    self.new_query_document();
                    if let Some(doc) = self.query_documents.last_mut() {
                        doc.set_text(sql);
                    }
                    self.active_tab = WorkspaceTab::Query;
                    self.activity = Activity::Queries;
                }
            });
        });

        if self.schema_workbench.apply_confirmation {
            egui::Window::new("Confirm DDL apply")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label("Apply the previewed DDL to the active connection?");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if Button::new(self.theme)
                            .text("Cancel")
                            .size(ButtonSize::Sm)
                            .variant(ButtonVariant::Ghost)
                            .show(ui)
                            .clicked()
                        {
                            self.schema_workbench.apply_confirmation = false;
                        }
                        if Button::new(self.theme)
                            .text("Apply")
                            .size(ButtonSize::Sm)
                            .variant(ButtonVariant::Destructive)
                            .show(ui)
                            .clicked()
                        {
                            self.schema_workbench.apply_confirmation = false;
                            self.apply_workbench_ddl();
                        }
                    });
                });
        }
    }
}
