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
            draw_view_designer(context, ui);
        }
        SchemaWorkbenchMode::Index => {
            draw_index_designer(context, ui);
        }
        SchemaWorkbenchMode::Constraint => {
            draw_constraint_designer(context, ui);
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

fn draw_view_designer(context: &mut SchemaWorkbenchFormContext<'_>, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("View Query Designer")
                .strong()
                .color(context.theme.text_primary),
        );
        ui.add_space(SPACE_MD);
        ui.checkbox(&mut context.workbench.materialized, "Materialized View");
        ui.add_space(SPACE_MD);
        if Button::new(context.theme)
            .text("+ Basic SELECT")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            context.workbench.select_sql = "SELECT id, name, created_at\nFROM users\nWHERE active = true;".to_owned();
        }
        if Button::new(context.theme)
            .text("+ Aggregate Summary")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            context.workbench.select_sql = "SELECT date_trunc('day', created_at) AS date,\n       count(*) AS total_records,\n       sum(amount) AS total_amount\nFROM transactions\nGROUP BY 1\nORDER BY 1 DESC;".to_owned();
        }
        if Button::new(context.theme)
            .text("+ Multi-Table JOIN")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            context.workbench.select_sql = "SELECT o.id AS order_id,\n       u.name AS customer_name,\n       o.total_price,\n       o.status\nFROM orders o\nJOIN users u ON u.id = o.user_id\nWHERE o.status != 'cancelled';".to_owned();
        }
    });
    ui.add_space(SPACE_SM);

    ui.vertical(|ui| {
        ui.label(RichText::new("Query Definition (SELECT Statement):").small().color(context.theme.text_secondary));
        ui.add(
            egui::TextEdit::multiline(&mut context.workbench.select_sql)
                .font(egui::TextStyle::Monospace)
                .desired_rows(6)
                .desired_width(f32::INFINITY),
        );
    });
}

fn draw_index_designer(context: &mut SchemaWorkbenchFormContext<'_>, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Index Designer")
                .strong()
                .color(context.theme.text_primary),
        );
        ui.add_space(SPACE_MD);
        ui.checkbox(&mut context.workbench.unique, "Unique Index");
    });
    ui.add_space(SPACE_SM);

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(200.0);
            ui.label(RichText::new("Target Table").small().color(context.theme.text_secondary));
            crate::components::input::Input::new(&mut context.workbench.parent_table, "table_name", context.theme)
                .width(200.0)
                .show(ui);
        });
        ui.add_space(SPACE_MD);
        ui.vertical(|ui| {
            ui.set_width(140.0);
            ui.label(RichText::new("Index Method").small().color(context.theme.text_secondary));
            let methods = ["BTREE", "HASH", "GIN", "GIST", "BRIN"];
            egui::ComboBox::from_id_salt("index_method_cb")
                .selected_text(&context.workbench.index_method)
                .width(140.0)
                .show_ui(ui, |ui| {
                    for m in &methods {
                        ui.selectable_value(&mut context.workbench.index_method, (*m).to_string(), *m);
                    }
                });
        });
        ui.add_space(SPACE_MD);
        ui.vertical(|ui| {
            ui.set_width(260.0);
            ui.label(RichText::new("Columns CSV (e.g. email, created_at DESC)").small().color(context.theme.text_secondary));
            crate::components::input::Input::new(&mut context.workbench.columns_csv, "col1, col2", context.theme)
                .width(260.0)
                .show(ui);
        });
    });
    ui.add_space(SPACE_SM);
    ui.vertical(|ui| {
        ui.label(RichText::new("WHERE Predicate / Partial Index (Optional):").small().color(context.theme.text_secondary));
        crate::components::input::Input::new(&mut context.workbench.index_predicate, "deleted_at IS NULL", context.theme)
            .width(500.0)
            .show(ui);
    });
}

fn draw_constraint_designer(context: &mut SchemaWorkbenchFormContext<'_>, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Constraint Designer")
                .strong()
                .color(context.theme.text_primary),
        );
        ui.add_space(SPACE_MD);
        egui::ComboBox::from_id_salt("constraint_kind_cb")
            .selected_text(match context.workbench.constraint_kind {
                ConstraintKindUi::PrimaryKey => "Primary key (PK)",
                ConstraintKindUi::Unique => "Unique constraint (UQ)",
                ConstraintKindUi::Check => "Check constraint (CK)",
                ConstraintKindUi::ForeignKey => "Foreign key (FK)",
            })
            .width(180.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut context.workbench.constraint_kind,
                    ConstraintKindUi::PrimaryKey,
                    "Primary key (PK)",
                );
                ui.selectable_value(
                    &mut context.workbench.constraint_kind,
                    ConstraintKindUi::Unique,
                    "Unique constraint (UQ)",
                );
                ui.selectable_value(
                    &mut context.workbench.constraint_kind,
                    ConstraintKindUi::Check,
                    "Check constraint (CK)",
                );
                ui.selectable_value(
                    &mut context.workbench.constraint_kind,
                    ConstraintKindUi::ForeignKey,
                    "Foreign key (FK)",
                );
            });
    });
    ui.add_space(SPACE_SM);

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(200.0);
            ui.label(RichText::new("Target Table").small().color(context.theme.text_secondary));
            crate::components::input::Input::new(&mut context.workbench.parent_table, "table_name", context.theme)
                .width(200.0)
                .show(ui);
        });
        ui.add_space(SPACE_MD);
        ui.vertical(|ui| {
            ui.set_width(280.0);
            ui.label(RichText::new("Columns CSV").small().color(context.theme.text_secondary));
            crate::components::input::Input::new(&mut context.workbench.columns_csv, "col_id", context.theme)
                .width(280.0)
                .show(ui);
        });
    });

    if context.workbench.constraint_kind == ConstraintKindUi::Check {
        ui.add_space(SPACE_SM);
        ui.vertical(|ui| {
            ui.label(RichText::new("Check Expression:").small().color(context.theme.text_secondary));
            crate::components::input::Input::new(&mut context.workbench.expression, "price > 0 AND quantity >= 0", context.theme)
                .width(500.0)
                .show(ui);
        });
    }

    if context.workbench.constraint_kind == ConstraintKindUi::ForeignKey {
        ui.add_space(SPACE_SM);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(160.0);
                ui.label(RichText::new("Reference Schema").small().color(context.theme.text_secondary));
                crate::components::input::Input::new(&mut context.workbench.ref_schema, "public", context.theme)
                    .width(160.0)
                    .show(ui);
            });
            ui.add_space(SPACE_MD);
            ui.vertical(|ui| {
                ui.set_width(200.0);
                ui.label(RichText::new("Reference Table").small().color(context.theme.text_secondary));
                crate::components::input::Input::new(&mut context.workbench.ref_table, "users", context.theme)
                    .width(200.0)
                    .show(ui);
            });
            ui.add_space(SPACE_MD);
            ui.vertical(|ui| {
                ui.set_width(200.0);
                ui.label(RichText::new("Reference Columns CSV").small().color(context.theme.text_secondary));
                crate::components::input::Input::new(&mut context.workbench.ref_columns_csv, "id", context.theme)
                    .width(200.0)
                    .show(ui);
            });
        });
        ui.add_space(SPACE_SM);
        ui.horizontal(|ui| {
            let actions = ["NO ACTION", "CASCADE", "SET NULL", "RESTRICT", "SET DEFAULT"];
            ui.vertical(|ui| {
                ui.set_width(180.0);
                ui.label(RichText::new("ON DELETE").small().color(context.theme.text_secondary));
                egui::ComboBox::from_id_salt("fk_on_delete_cb")
                    .selected_text(&context.workbench.on_delete)
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for a in &actions {
                            ui.selectable_value(&mut context.workbench.on_delete, (*a).to_string(), *a);
                        }
                    });
            });
            ui.add_space(SPACE_MD);
            ui.vertical(|ui| {
                ui.set_width(180.0);
                ui.label(RichText::new("ON UPDATE").small().color(context.theme.text_secondary));
                egui::ComboBox::from_id_salt("fk_on_update_cb")
                    .selected_text(&context.workbench.on_update)
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for a in &actions {
                            ui.selectable_value(&mut context.workbench.on_update, (*a).to_string(), *a);
                        }
                    });
            });
        });
    }
}

fn draw_table_designer(context: &mut SchemaWorkbenchFormContext<'_>, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Columns Designer")
                .strong()
                .color(context.theme.text_primary),
        );
        ui.add_space(SPACE_MD);
        if Button::new(context.theme)
            .text("+ Add Column")
            .variant(ButtonVariant::Default)
            .size(ButtonSize::Sm)
            .icon(Icon::Plus)
            .show(ui)
            .clicked()
        {
            context.workbench.add_table_column();
        }
        ui.add_space(SPACE_SM);
        if Button::new(context.theme)
            .text("+ UUID PK")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            context.workbench.table_columns.insert(
                0,
                TableDesignerColumn {
                    name: "id".into(),
                    data_type: "UUID".into(),
                    nullable: false,
                    is_pk: true,
                    auto_increment: false,
                    default_expr: "gen_random_uuid()".into(),
                    comment: "UUID Primary Key".into(),
                },
            );
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
                name: "created_at".into(),
                data_type: "TIMESTAMPTZ".into(),
                nullable: false,
                is_pk: false,
                auto_increment: false,
                default_expr: "CURRENT_TIMESTAMP".into(),
                comment: "Created timestamp".into(),
            });
            context.workbench.table_columns.push(TableDesignerColumn {
                name: "updated_at".into(),
                data_type: "TIMESTAMPTZ".into(),
                nullable: false,
                is_pk: false,
                auto_increment: false,
                default_expr: "CURRENT_TIMESTAMP".into(),
                comment: "Updated timestamp".into(),
            });
            context.workbench.sync_table_columns_to_csv();
        }
    });
    ui.add_space(SPACE_SM);

    let col_count = context.workbench.table_columns.len();
    let mut remove_idx = None;
    let mut move_up_idx = None;
    let mut move_down_idx = None;

    egui::Frame::none()
        .fill(context.theme.surface_panel)
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::same(8.0))
        .show(ui, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(RichText::new("#").small().color(context.theme.text_secondary));
                ui.add_space(4.0);
                ui.label(RichText::new("Column Name").small().strong().color(context.theme.text_secondary));
                ui.add_space(75.0);
                ui.label(RichText::new("Data Type").small().strong().color(context.theme.text_secondary));
                ui.add_space(55.0);
                ui.label(RichText::new("PK").small().strong().color(context.theme.text_secondary));
                ui.add_space(10.0);
                ui.label(RichText::new("Nullable").small().strong().color(context.theme.text_secondary));
                ui.add_space(10.0);
                ui.label(RichText::new("AutoInc").small().strong().color(context.theme.text_secondary));
                ui.add_space(10.0);
                ui.label(RichText::new("Default Expression").small().strong().color(context.theme.text_secondary));
            });
            ui.separator();

            let types = [
                "BIGINT",
                "INTEGER",
                "SMALLINT",
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
    let sql = context.workbench.preview_sql.clone();

    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Live Planned DDL Preview")
                .strong()
                .color(context.theme.text_primary),
        );
        ui.add_space(SPACE_MD);
        let preview_empty = sql.trim().is_empty();
        let apply_enabled = context.can_mutate && !preview_empty;
        ui.add_enabled_ui(apply_enabled, |ui| {
            if Button::new(context.theme)
                .text("Apply DDL to Database")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .icon(Icon::Check)
                .show(ui)
                .clicked()
            {
                context.workbench.apply_confirmation = true;
            }
        });
        ui.add_enabled_ui(!preview_empty, |ui| {
            if Button::new(context.theme)
                .text("Open in SQL Editor")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .icon(Icon::FileCode2)
                .show(ui)
                .clicked()
            {
                action = Some(SchemaWorkbenchFormAction::OpenSql(sql.clone()));
            }
        });
    });

    if let Some(err) = &context.workbench.preview_error {
        ui.add_space(SPACE_XS);
        ui.colored_label(context.theme.danger, format!("Plan error: {err}"));
    }

    ui.add_space(SPACE_SM);
    egui::Frame::none()
        .fill(context.theme.surface_panel)
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            if sql.trim().is_empty() {
                ui.label(
                    RichText::new("-- Click 'Plan create', 'Plan drop' or 'Plan rename' above to generate DDL")
                        .color(context.theme.text_muted)
                        .monospace(),
                );
            } else {
                egui::ScrollArea::vertical()
                    .max_height(220.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(&sql)
                                    .color(context.theme.text_primary)
                                    .monospace(),
                            )
                            .selectable(true)
                            .wrap(),
                        );
                    });
            }
        });

    if context.workbench.apply_confirmation {
        let mut open = true;
        Dialog::new(&mut open, "Confirm DDL Execution", context.theme).show(ui, |ui| {
            ui.label(
                RichText::new("Execute the following planned DDL statements?")
                    .color(context.theme.text_primary),
            );
            ui.add_space(SPACE_SM);
            egui::Frame::none()
                .fill(context.theme.surface_panel)
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ui.label(RichText::new(&sql).monospace().color(context.theme.text_secondary));
                });
            ui.add_space(SPACE_MD);
            ui.horizontal(|ui| {
                if Button::new(context.theme)
                    .text("Execute DDL")
                    .variant(ButtonVariant::Default)
                    .show(ui)
                    .clicked()
                {
                    context.workbench.apply_confirmation = false;
                    action = Some(SchemaWorkbenchFormAction::ApplyDdl);
                }
                if Button::new(context.theme)
                    .text("Cancel")
                    .variant(ButtonVariant::Ghost)
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
