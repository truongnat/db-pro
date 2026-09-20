//! Schema workbench object form and SQL preview panes.
use super::schema_workbench::{ConstraintKindUi, SchemaWorkbenchMode};
use super::*;
use crate::components::{Button, ButtonSize, ButtonVariant};
use db_pro_core::domain::object_mutation::ObjectAction;

impl DbProApp {
    pub(super) fn draw_workbench_form(&mut self, ui: &mut egui::Ui) {
        let mode = self.schema.workbench.mode;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(220.0);
                ui.label(RichText::new("Schema").small().color(self.theme.text_secondary));
                crate::components::input::Input::new(&mut self.schema.workbench.schema, "main", self.theme)
                    .width(220.0)
                    .show(ui);
            });
            ui.add_space(SPACE_MD);
            ui.vertical(|ui| {
                ui.set_width(280.0);
                ui.label(RichText::new("Name").small().color(self.theme.text_secondary));
                crate::components::input::Input::new(&mut self.schema.workbench.name, "object_name", self.theme)
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
                    &mut self.schema.workbench.columns_csv,
                    "id:INTEGER:pk,name:TEXT",
                    self.theme,
                )
                .width(520.0)
                .show(ui);
            }
            SchemaWorkbenchMode::Column => {
                ui.horizontal(|ui| {
                    ui.label("Table");
                    ui.text_edit_singleline(&mut self.schema.workbench.parent_table);
                    ui.label("Type");
                    ui.text_edit_singleline(&mut self.schema.workbench.data_type);
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.schema.workbench.nullable, "Nullable");
                    ui.checkbox(&mut self.schema.workbench.is_pk, "PK (create table only)");
                    ui.label("Default");
                    ui.text_edit_singleline(&mut self.schema.workbench.default_expr);
                });
                ui.horizontal(|ui| {
                    ui.label("Rename to");
                    ui.text_edit_singleline(&mut self.schema.workbench.new_name);
                });
            }
            SchemaWorkbenchMode::View => {
                ui.checkbox(&mut self.schema.workbench.materialized, "Materialized");
                ui.label("SELECT body");
                ui.add(
                    egui::TextEdit::multiline(&mut self.schema.workbench.select_sql)
                        .desired_rows(4)
                        .desired_width(f32::INFINITY),
                );
            }
            SchemaWorkbenchMode::Index => {
                ui.horizontal(|ui| {
                    ui.label("Table");
                    ui.text_edit_singleline(&mut self.schema.workbench.parent_table);
                    ui.checkbox(&mut self.schema.workbench.unique, "Unique");
                });
                ui.label("Columns CSV");
                ui.text_edit_singleline(&mut self.schema.workbench.columns_csv);
            }
            SchemaWorkbenchMode::Constraint => {
                ui.horizontal(|ui| {
                    ui.label("Table");
                    ui.text_edit_singleline(&mut self.schema.workbench.parent_table);
                    egui::ComboBox::from_id_salt("constraint_kind")
                        .selected_text(match self.schema.workbench.constraint_kind {
                            ConstraintKindUi::PrimaryKey => "Primary key",
                            ConstraintKindUi::Unique => "Unique",
                            ConstraintKindUi::Check => "Check",
                            ConstraintKindUi::ForeignKey => "Foreign key",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.schema.workbench.constraint_kind,
                                ConstraintKindUi::PrimaryKey,
                                "Primary key",
                            );
                            ui.selectable_value(
                                &mut self.schema.workbench.constraint_kind,
                                ConstraintKindUi::Unique,
                                "Unique",
                            );
                            ui.selectable_value(
                                &mut self.schema.workbench.constraint_kind,
                                ConstraintKindUi::Check,
                                "Check",
                            );
                            ui.selectable_value(
                                &mut self.schema.workbench.constraint_kind,
                                ConstraintKindUi::ForeignKey,
                                "Foreign key",
                            );
                        });
                });
                ui.label("Columns CSV");
                ui.text_edit_singleline(&mut self.schema.workbench.columns_csv);
                if self.schema.workbench.constraint_kind == ConstraintKindUi::Check {
                    ui.label("Expression");
                    ui.text_edit_singleline(&mut self.schema.workbench.expression);
                }
                if self.schema.workbench.constraint_kind == ConstraintKindUi::ForeignKey {
                    ui.horizontal(|ui| {
                        ui.label("Ref schema");
                        ui.text_edit_singleline(&mut self.schema.workbench.ref_schema);
                        ui.label("Ref table");
                        ui.text_edit_singleline(&mut self.schema.workbench.ref_table);
                    });
                    ui.label("Ref columns CSV");
                    ui.text_edit_singleline(&mut self.schema.workbench.ref_columns_csv);
                    ui.label("ON DELETE");
                    ui.text_edit_singleline(&mut self.schema.workbench.on_delete);
                }
            }
            SchemaWorkbenchMode::Trigger => {
                ui.horizontal(|ui| {
                    ui.label("Table");
                    ui.text_edit_singleline(&mut self.schema.workbench.parent_table);
                    ui.label("Timing");
                    ui.text_edit_singleline(&mut self.schema.workbench.timing);
                    ui.label("Event");
                    ui.text_edit_singleline(&mut self.schema.workbench.event);
                });
                ui.label("Body");
                ui.add(
                    egui::TextEdit::multiline(&mut self.schema.workbench.body)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY),
                );
            }
            SchemaWorkbenchMode::Sequence => {
                ui.horizontal(|ui| {
                    ui.label("Start");
                    ui.text_edit_singleline(&mut self.schema.workbench.start);
                    ui.label("Increment");
                    ui.text_edit_singleline(&mut self.schema.workbench.increment);
                    ui.checkbox(&mut self.schema.workbench.cycle, "Cycle");
                });
            }
            SchemaWorkbenchMode::Type => {
                ui.label("Enum values CSV");
                ui.text_edit_singleline(&mut self.schema.workbench.enum_values_csv);
            }
            SchemaWorkbenchMode::SchemaDb => {
                ui.label("Schema name uses Name field; Database create/drop uses Name as DB name.");
                ui.checkbox(&mut self.schema.workbench.cascade, "CASCADE on drop schema");
            }
            SchemaWorkbenchMode::Extension => {
                ui.label("Extension schema (optional)");
                ui.text_edit_singleline(&mut self.schema.workbench.extension_schema);
                ui.checkbox(&mut self.schema.workbench.cascade, "CASCADE on drop");
            }
            SchemaWorkbenchMode::Comment => {
                ui.horizontal(|ui| {
                    ui.label("Parent (column comments)");
                    ui.text_edit_singleline(&mut self.schema.workbench.parent_table);
                });
                ui.label("Comment text (empty clears)");
                ui.text_edit_singleline(&mut self.schema.workbench.comment_text);
            }
            SchemaWorkbenchMode::Partition => {
                ui.horizontal(|ui| {
                    ui.label("Parent table");
                    ui.text_edit_singleline(&mut self.schema.workbench.parent_table);
                });
                ui.label("FOR VALUES …");
                ui.text_edit_singleline(&mut self.schema.workbench.partition_bound);
            }
            SchemaWorkbenchMode::Dependencies | SchemaWorkbenchMode::Docs => {}
        }

        ui.add_space(SPACE_MD);
        ui.horizontal_wrapped(|ui| {
            if Button::new(self.theme)
                .text("Plan create")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.plan_workbench_action(ObjectAction::Create);
            }
            if Button::new(self.theme)
                .text("Plan drop")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.plan_workbench_action(ObjectAction::Drop);
            }
            if mode == SchemaWorkbenchMode::Column
                && Button::new(self.theme)
                    .text("Plan rename")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
            {
                self.plan_workbench_action(ObjectAction::Rename);
            }
            if mode == SchemaWorkbenchMode::View
                && self.schema.workbench.materialized
                && Button::new(self.theme)
                    .text("Plan refresh")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
            {
                self.plan_workbench_action(ObjectAction::Refresh);
            }
            if mode == SchemaWorkbenchMode::Comment
                && Button::new(self.theme)
                    .text("Plan comment")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
            {
                self.plan_workbench_action(ObjectAction::Comment);
            }
            if mode == SchemaWorkbenchMode::SchemaDb {
                if Button::new(self.theme)
                    .text("Plan create database")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.plan_database_action(ObjectAction::Create);
                }
                if Button::new(self.theme)
                    .text("Plan drop database")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
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
            if let Some(err) = &self.schema.workbench.preview_error {
                ui.colored_label(self.theme.danger, err);
                ui.add_space(SPACE_XS);
            }
            if !self.schema.workbench.preview_safety.is_empty() {
                ui.label(
                    RichText::new(format!(
                        "Safety: {} · fingerprint {}",
                        self.schema.workbench.preview_safety, self.schema.workbench.preview_fingerprint
                    ))
                    .small()
                    .color(self.theme.text_muted),
                );
                ui.add_space(SPACE_XS);
            }
            ui.label(RichText::new("SQL").small().strong().color(self.theme.text_secondary));
            editor_frame(self.theme).show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.schema.workbench.preview_sql)
                        .desired_rows(8)
                        .desired_width(f32::INFINITY)
                        .code_editor(),
                );
            });
            ui.add_space(SPACE_SM);
            ui.horizontal(|ui| {
                let can_apply = !self.schema.workbench.preview_sql.trim().is_empty()
                    && self.schema.workbench.preview_error.is_none()
                    && self.can_mutate_active_connection();
                if Button::new(self.theme)
                    .text("Apply DDL…")
                    .variant(ButtonVariant::Default)
                    .size(ButtonSize::Sm)
                    .enabled(can_apply)
                    .show(ui)
                    .clicked()
                {
                    self.schema.workbench.apply_confirmation = true;
                }
                let can_open_editor = !self.schema.workbench.preview_sql.trim().is_empty();
                if Button::new(self.theme)
                    .text("Open in SQL editor")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .enabled(can_open_editor)
                    .show(ui)
                    .clicked()
                    && can_open_editor
                {
                    let sql = self.schema.workbench.preview_sql.clone();
                    self.new_query_document();
                    if let Some(doc) = self.query.session.documents.last_mut() {
                        doc.set_text(sql);
                    }
                    self.workspace.active_tab = WorkspaceTab::Query;
                    self.workspace.activity = Activity::Queries;
                }
            });
        });

        if self.schema.workbench.apply_confirmation {
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
                            self.schema.workbench.apply_confirmation = false;
                        }
                        if Button::new(self.theme)
                            .text("Apply")
                            .size(ButtonSize::Sm)
                            .variant(ButtonVariant::Destructive)
                            .show(ui)
                            .clicked()
                        {
                            self.schema.workbench.apply_confirmation = false;
                            self.apply_workbench_ddl();
                        }
                    });
                });
        }
    }
}
