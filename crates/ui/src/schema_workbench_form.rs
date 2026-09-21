//! Schema workbench object form and SQL preview panes.
use super::schema_workbench::{ConstraintKindUi, SchemaWorkbenchMode, SchemaWorkbenchState};
use super::*;
use crate::components::dialog::Dialog;
use crate::components::{Button, ButtonSize, ButtonVariant};
use db_pro_core::domain::object_mutation::ObjectAction;

pub(super) struct SchemaWorkbenchFormContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) workbench: &'a mut SchemaWorkbenchState,
    pub(super) can_mutate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SchemaWorkbenchFormAction {
    PlanObject(ObjectAction),
    PlanDatabase(ObjectAction),
    ApplyDdl,
    OpenSql(String),
}

pub(super) fn draw_workbench_form(
    context: &mut SchemaWorkbenchFormContext<'_>,
    ui: &mut egui::Ui,
) -> Option<SchemaWorkbenchFormAction> {
    let mode = context.workbench.mode;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(220.0);
            ui.label(RichText::new("Schema").small().color(context.theme.text_secondary));
            crate::components::input::Input::new(&mut context.workbench.schema, "main", context.theme)
                .width(220.0)
                .show(ui);
        });
        ui.add_space(SPACE_MD);
        ui.vertical(|ui| {
            ui.set_width(280.0);
            ui.label(RichText::new("Name").small().color(context.theme.text_secondary));
            crate::components::input::Input::new(&mut context.workbench.name, "object_name", context.theme)
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
                    .color(context.theme.text_secondary),
            );
            crate::components::input::Input::new(
                &mut context.workbench.columns_csv,
                "id:INTEGER:pk,name:TEXT",
                context.theme,
            )
            .width(520.0)
            .show(ui);
        }
        SchemaWorkbenchMode::Column => {
            ui.horizontal(|ui| {
                ui.label("Table");
                ui.text_edit_singleline(&mut context.workbench.parent_table);
                ui.label("Type");
                ui.text_edit_singleline(&mut context.workbench.data_type);
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut context.workbench.nullable, "Nullable");
                ui.checkbox(&mut context.workbench.is_pk, "PK (create table only)");
                ui.label("Default");
                ui.text_edit_singleline(&mut context.workbench.default_expr);
            });
            ui.horizontal(|ui| {
                ui.label("Rename to");
                ui.text_edit_singleline(&mut context.workbench.new_name);
            });
        }
        SchemaWorkbenchMode::View => {
            ui.checkbox(&mut context.workbench.materialized, "Materialized");
            ui.label("SELECT body");
            ui.add(
                egui::TextEdit::multiline(&mut context.workbench.select_sql)
                    .desired_rows(4)
                    .desired_width(f32::INFINITY),
            );
        }
        SchemaWorkbenchMode::Index => {
            ui.horizontal(|ui| {
                ui.label("Table");
                ui.text_edit_singleline(&mut context.workbench.parent_table);
                ui.checkbox(&mut context.workbench.unique, "Unique");
            });
            ui.label("Columns CSV");
            ui.text_edit_singleline(&mut context.workbench.columns_csv);
        }
        SchemaWorkbenchMode::Constraint => {
            ui.horizontal(|ui| {
                ui.label("Table");
                ui.text_edit_singleline(&mut context.workbench.parent_table);
                egui::ComboBox::from_id_salt("constraint_kind")
                    .selected_text(match context.workbench.constraint_kind {
                        ConstraintKindUi::PrimaryKey => "Primary key",
                        ConstraintKindUi::Unique => "Unique",
                        ConstraintKindUi::Check => "Check",
                        ConstraintKindUi::ForeignKey => "Foreign key",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut context.workbench.constraint_kind,
                            ConstraintKindUi::PrimaryKey,
                            "Primary key",
                        );
                        ui.selectable_value(
                            &mut context.workbench.constraint_kind,
                            ConstraintKindUi::Unique,
                            "Unique",
                        );
                        ui.selectable_value(&mut context.workbench.constraint_kind, ConstraintKindUi::Check, "Check");
                        ui.selectable_value(
                            &mut context.workbench.constraint_kind,
                            ConstraintKindUi::ForeignKey,
                            "Foreign key",
                        );
                    });
            });
            ui.label("Columns CSV");
            ui.text_edit_singleline(&mut context.workbench.columns_csv);
            if context.workbench.constraint_kind == ConstraintKindUi::Check {
                ui.label("Expression");
                ui.text_edit_singleline(&mut context.workbench.expression);
            }
            if context.workbench.constraint_kind == ConstraintKindUi::ForeignKey {
                ui.horizontal(|ui| {
                    ui.label("Ref schema");
                    ui.text_edit_singleline(&mut context.workbench.ref_schema);
                    ui.label("Ref table");
                    ui.text_edit_singleline(&mut context.workbench.ref_table);
                });
                ui.label("Ref columns CSV");
                ui.text_edit_singleline(&mut context.workbench.ref_columns_csv);
                ui.label("ON DELETE");
                ui.text_edit_singleline(&mut context.workbench.on_delete);
            }
        }
        SchemaWorkbenchMode::Trigger => {
            ui.horizontal(|ui| {
                ui.label("Table");
                ui.text_edit_singleline(&mut context.workbench.parent_table);
                ui.label("Timing");
                ui.text_edit_singleline(&mut context.workbench.timing);
                ui.label("Event");
                ui.text_edit_singleline(&mut context.workbench.event);
            });
            ui.label("Body");
            ui.add(
                egui::TextEdit::multiline(&mut context.workbench.body)
                    .desired_rows(3)
                    .desired_width(f32::INFINITY),
            );
        }
        SchemaWorkbenchMode::Sequence => {
            ui.horizontal(|ui| {
                ui.label("Start");
                ui.text_edit_singleline(&mut context.workbench.start);
                ui.label("Increment");
                ui.text_edit_singleline(&mut context.workbench.increment);
                ui.checkbox(&mut context.workbench.cycle, "Cycle");
            });
        }
        SchemaWorkbenchMode::Type => {
            ui.label("Enum values CSV");
            ui.text_edit_singleline(&mut context.workbench.enum_values_csv);
        }
        SchemaWorkbenchMode::SchemaDb => {
            ui.label("Schema name uses Name field; Database create/drop uses Name as DB name.");
            ui.checkbox(&mut context.workbench.cascade, "CASCADE on drop schema");
        }
        SchemaWorkbenchMode::Extension => {
            ui.label("Extension schema (optional)");
            ui.text_edit_singleline(&mut context.workbench.extension_schema);
            ui.checkbox(&mut context.workbench.cascade, "CASCADE on drop");
        }
        SchemaWorkbenchMode::Comment => {
            ui.horizontal(|ui| {
                ui.label("Parent (column comments)");
                ui.text_edit_singleline(&mut context.workbench.parent_table);
            });
            ui.label("Comment text (empty clears)");
            ui.text_edit_singleline(&mut context.workbench.comment_text);
        }
        SchemaWorkbenchMode::Partition => {
            ui.horizontal(|ui| {
                ui.label("Parent table");
                ui.text_edit_singleline(&mut context.workbench.parent_table);
            });
            ui.label("FOR VALUES …");
            ui.text_edit_singleline(&mut context.workbench.partition_bound);
        }
        SchemaWorkbenchMode::Dependencies | SchemaWorkbenchMode::Docs => {}
    }

    ui.add_space(SPACE_MD);
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        if Button::new(context.theme)
            .text("Plan create")
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            action = Some(SchemaWorkbenchFormAction::PlanObject(ObjectAction::Create));
        }
        if Button::new(context.theme)
            .text("Plan drop")
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            action = Some(SchemaWorkbenchFormAction::PlanObject(ObjectAction::Drop));
        }
        if mode == SchemaWorkbenchMode::Column
            && Button::new(context.theme)
                .text("Plan rename")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
        {
            action = Some(SchemaWorkbenchFormAction::PlanObject(ObjectAction::Rename));
        }
        if mode == SchemaWorkbenchMode::View
            && context.workbench.materialized
            && Button::new(context.theme)
                .text("Plan refresh")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
        {
            action = Some(SchemaWorkbenchFormAction::PlanObject(ObjectAction::Refresh));
        }
        if mode == SchemaWorkbenchMode::Comment
            && Button::new(context.theme)
                .text("Plan comment")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
        {
            action = Some(SchemaWorkbenchFormAction::PlanObject(ObjectAction::Comment));
        }
        if mode == SchemaWorkbenchMode::SchemaDb {
            if Button::new(context.theme)
                .text("Plan create database")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                action = Some(SchemaWorkbenchFormAction::PlanDatabase(ObjectAction::Create));
            }
            if Button::new(context.theme)
                .text("Plan drop database")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                action = Some(SchemaWorkbenchFormAction::PlanDatabase(ObjectAction::Drop));
            }
        }
    });
    action
}

pub(super) fn draw_workbench_preview(
    context: &mut SchemaWorkbenchFormContext<'_>,
    ui: &mut egui::Ui,
) -> Option<SchemaWorkbenchFormAction> {
    let mut action = None;
    card_frame(context.theme).show(ui, |ui| {
        ui.set_min_width((ui.available_width() - 8.0).max(0.0));
        section_label(ui, "PREVIEW", context.theme);
        ui.add_space(SPACE_SM);
        if let Some(err) = &context.workbench.preview_error {
            ui.colored_label(context.theme.danger, err);
            ui.add_space(SPACE_XS);
        }
        if !context.workbench.preview_safety.is_empty() {
            ui.label(
                RichText::new(format!(
                    "Safety: {} · fingerprint {}",
                    context.workbench.preview_safety, context.workbench.preview_fingerprint
                ))
                .small()
                .color(context.theme.text_muted),
            );
            ui.add_space(SPACE_XS);
        }
        ui.label(
            RichText::new("SQL")
                .small()
                .strong()
                .color(context.theme.text_secondary),
        );
        editor_frame(context.theme).show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut context.workbench.preview_sql)
                    .desired_rows(8)
                    .desired_width(f32::INFINITY)
                    .code_editor(),
            );
        });
        ui.add_space(SPACE_SM);
        ui.horizontal(|ui| {
            let can_apply = !context.workbench.preview_sql.trim().is_empty()
                && context.workbench.preview_error.is_none()
                && context.can_mutate;
            if Button::new(context.theme)
                .text("Apply DDL…")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .enabled(can_apply)
                .show(ui)
                .clicked()
            {
                context.workbench.apply_confirmation = true;
            }
            let can_open_editor = !context.workbench.preview_sql.trim().is_empty();
            if Button::new(context.theme)
                .text("Open in SQL editor")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .enabled(can_open_editor)
                .show(ui)
                .clicked()
                && can_open_editor
            {
                let sql = context.workbench.preview_sql.clone();
                action = Some(SchemaWorkbenchFormAction::OpenSql(sql));
            }
        });
    });

    if context.workbench.apply_confirmation {
        let mut open = true;
        Dialog::new(&mut open, "Confirm DDL apply", context.theme)
            .width(460.0)
            .id_salt("schema_workbench_apply_dialog")
            .show_framed_ctx(ui.ctx(), |frame| {
                frame.body(|ui| {
                    ui.label("Apply the previewed DDL to the active connection?");
                });
                frame.footer(|ui| {
                    if Button::new(context.theme)
                        .text("Cancel")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Ghost)
                        .show(ui)
                        .clicked()
                    {
                        context.workbench.apply_confirmation = false;
                    }
                    if Button::new(context.theme)
                        .text("Apply")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Destructive)
                        .show(ui)
                        .clicked()
                    {
                        context.workbench.apply_confirmation = false;
                        action = Some(SchemaWorkbenchFormAction::ApplyDdl);
                    }
                });
            });
        if !open {
            context.workbench.apply_confirmation = false;
        }
    }
    action
}
