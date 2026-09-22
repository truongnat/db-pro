//! Schema workbench object form and SQL preview panes.
use super::schema_workbench::{
    ConstraintKindUi, SchemaWorkbenchMode, SchemaWorkbenchState, TableDesignerColumn,
};
use super::*;
use crate::components::dialog::Dialog;
use crate::components::{Button, ButtonSize, ButtonVariant};
use db_pro_core::domain::object_mutation::ObjectAction;
use lucide_icons::Icon;

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
            ui.label(RichText::new("Table / Object Name").small().color(context.theme.text_secondary));
            crate::components::input::Input::new(&mut context.workbench.name, "table_name", context.theme)
                .width(280.0)
                .show(ui);
        });
    });
    ui.add_space(SPACE_SM);

    match mode {
        SchemaWorkbenchMode::Table => {
            draw_table_designer(context, ui);
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
            .variant(ButtonVariant::Default)
            .size(ButtonSize::Sm)
            .icon(Icon::Plus)
            .show(ui)
            .clicked()
        {
            action = Some(SchemaWorkbenchFormAction::PlanObject(ObjectAction::Create));
        }
        if Button::new(context.theme)
            .text("Plan drop")
            .variant(ButtonVariant::Destructive)
            .size(ButtonSize::Sm)
            .icon(Icon::Trash2)
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

fn draw_table_designer(context: &mut SchemaWorkbenchFormContext<'_>, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Columns Designer")
                .strong()
                .color(context.theme.text_primary),
        );
        ui.add_space(SPACE_SM);
        if Button::new(context.theme)
            .text("Add Column")
            .icon(Icon::Plus)
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            context.workbench.add_table_column();
        }
        if Button::new(context.theme)
            .text("+ UUID PK")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            context.workbench.table_columns.push(TableDesignerColumn {
                name: "uuid".into(),
                data_type: "UUID".into(),
                nullable: false,
                is_pk: true,
                auto_increment: false,
                default_expr: "gen_random_uuid()".into(),
                comment: "Unique identifier".into(),
            });
            context.workbench.sync_table_columns_to_csv();
        }
        if Button::new(context.theme)
            .text("+ Timestamps")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            context.workbench.table_columns.push(TableDesignerColumn {
                name: "updated_at".into(),
                data_type: "TIMESTAMPTZ".into(),
                nullable: false,
                is_pk: false,
                auto_increment: false,
                default_expr: "CURRENT_TIMESTAMP".into(),
                comment: "Last updated timestamp".into(),
            });
            context.workbench.sync_table_columns_to_csv();
        }
        if Button::new(context.theme)
            .text("Clear")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            context.workbench.table_columns.clear();
            context.workbench.sync_table_columns_to_csv();
        }
    });

    ui.add_space(SPACE_SM);

    let mut remove_idx = None;
    let mut move_up_idx = None;
    let mut move_down_idx = None;

    let available_width = ui.available_width();
    let col_count = context.workbench.table_columns.len();

    egui::Frame::none()
        .fill(context.theme.surface_panel)
        .stroke(egui::Stroke::new(1.0, context.theme.border_subtle))
        .rounding(4.0)
        .inner_margin(egui::Margin::same(6.0))
        .show(ui, |ui| {
            ui.set_min_width(available_width);

            // Table Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("#").small().strong().color(context.theme.text_muted));
                ui.add_space(12.0);
                ui.label(RichText::new("Column Name").small().strong().color(context.theme.text_muted));
                ui.add_space(110.0);
                ui.label(RichText::new("Data Type").small().strong().color(context.theme.text_muted));
                ui.add_space(70.0);
                ui.label(RichText::new("PK").small().strong().color(context.theme.text_muted));
                ui.add_space(8.0);
                ui.label(RichText::new("Nullable").small().strong().color(context.theme.text_muted));
                ui.add_space(8.0);
                ui.label(RichText::new("AutoInc").small().strong().color(context.theme.text_muted));
                ui.add_space(14.0);
                ui.label(RichText::new("Default Expression").small().strong().color(context.theme.text_muted));
            });
            ui.separator();

            let types = [
                "INTEGER",
                "BIGINT",
                "VARCHAR(255)",
                "TEXT",
                "BOOLEAN",
                "TIMESTAMPTZ",
                "TIMESTAMP",
                "NUMERIC(12,2)",
                "FLOAT8",
                "JSONB",
                "UUID",
                "BYTEA",
                "DATE",
            ];

            for idx in 0..col_count {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{}", idx + 1)).small().color(context.theme.text_muted));

                    let col = &mut context.workbench.table_columns[idx];

                    // Name
                    ui.add(
                        egui::TextEdit::singleline(&mut col.name)
                            .hint_text("column_name")
                            .desired_width(150.0),
                    );

                    // Type combobox
                    egui::ComboBox::from_id_salt(format!("col_type_{idx}"))
                        .selected_text(&col.data_type)
                        .width(120.0)
                        .show_ui(ui, |ui| {
                            for t in &types {
                                ui.selectable_value(&mut col.data_type, (*t).to_string(), *t);
                            }
                        });

                    // PK
                    if ui.checkbox(&mut col.is_pk, "").changed() && col.is_pk {
                        col.nullable = false;
                    }

                    // Nullable
                    let can_be_nullable = !col.is_pk;
                    ui.add_enabled(can_be_nullable, egui::Checkbox::without_text(&mut col.nullable));

                    // Auto inc
                    ui.checkbox(&mut col.auto_increment, "");

                    // Default
                    ui.add(
                        egui::TextEdit::singleline(&mut col.default_expr)
                            .hint_text("default / expr")
                            .desired_width(130.0),
                    );

                    // Reorder & delete buttons
                    if idx > 0 && ui.small_button("▲").clicked() {
                        move_up_idx = Some(idx);
                    }
                    if idx + 1 < col_count && ui.small_button("▼").clicked() {
                        move_down_idx = Some(idx);
                    }
                    if ui.small_button("✕").clicked() {
                        remove_idx = Some(idx);
                    }
                });
                ui.add_space(2.0);
            }
        });

    if let Some(idx) = remove_idx {
        context.workbench.remove_table_column(idx);
    }
    if let Some(idx) = move_up_idx {
        context.workbench.move_table_column_up(idx);
    }
    if let Some(idx) = move_down_idx {
        context.workbench.move_table_column_down(idx);
    }

    context.workbench.sync_table_columns_to_csv();
}

pub(super) fn draw_workbench_preview(
    context: &mut SchemaWorkbenchFormContext<'_>,
    ui: &mut egui::Ui,
) -> Option<SchemaWorkbenchFormAction> {
    let mut action = None;
    card_frame(context.theme).show(ui, |ui| {
        ui.set_min_width(ui.available_width() - 8.0);
        section_label(ui, "MUTATION PREVIEW (SQL & SAFETY)", context.theme);
        ui.add_space(SPACE_SM);

        if let Some(err) = &context.workbench.preview_error {
            ui.label(RichText::new(format!("Planning error: {err}")).color(context.theme.danger));
            ui.add_space(SPACE_SM);
        }

        if !context.workbench.preview_safety.is_empty() {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Safety classification:")
                        .small()
                        .color(context.theme.text_secondary),
                );
                badge(
                    ui,
                    &context.workbench.preview_safety,
                    context.theme.accent_soft,
                    context.theme.accent,
                );
            });
            ui.add_space(SPACE_SM);
        }

        let sql = &context.workbench.preview_sql;
        if sql.is_empty() {
            ui.label(
                RichText::new("Click 'Plan create' or other plan actions above to generate SQL preview")
                    .small()
                    .color(context.theme.text_muted),
            );
        } else {
            ui.add(
                egui::TextEdit::multiline(&mut context.workbench.preview_sql.clone())
                    .desired_rows(6)
                    .desired_width(f32::INFINITY)
                    .interactive(false),
            );
            ui.add_space(SPACE_SM);
            ui.horizontal(|ui| {
                if Button::new(context.theme)
                    .text("Open in Query Editor")
                    .icon(Icon::ExternalLink)
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    action = Some(SchemaWorkbenchFormAction::OpenSql(context.workbench.preview_sql.clone()));
                }

                if context.can_mutate {
                    if Button::new(context.theme)
                        .text("Apply DDL to Database")
                        .icon(Icon::Check)
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        context.workbench.apply_confirmation = true;
                    }
                } else {
                    ui.label(
                        RichText::new("Read-only connection: cannot apply DDL")
                            .small()
                            .color(context.theme.text_muted),
                    );
                }
            });
        }
    });

    if context.workbench.apply_confirmation {
        let mut open = true;
        Dialog::new(&mut open, "Confirm Schema Mutation", context.theme)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Are you sure you want to execute this DDL statement against the database?")
                        .color(context.theme.text_primary),
                );
                ui.add_space(SPACE_SM);
                ui.label(
                    RichText::new(&context.workbench.preview_sql)
                        .monospace()
                        .small()
                        .color(context.theme.accent),
                );
                ui.add_space(SPACE_MD);
                ui.horizontal(|ui| {
                    if Button::new(context.theme)
                        .text("Execute DDL")
                        .variant(ButtonVariant::Destructive)
                        .show(ui)
                        .clicked()
                    {
                        context.workbench.apply_confirmation = false;
                        action = Some(SchemaWorkbenchFormAction::ApplyDdl);
                    }
                    if Button::new(context.theme)
                        .text("Cancel")
                        .variant(ButtonVariant::Secondary)
                        .show(ui)
                        .clicked()
                    {
                        context.workbench.apply_confirmation = false;
                    }
                });
            });
        if !open {
            context.workbench.apply_confirmation = false;
        }
    }

    action
}
