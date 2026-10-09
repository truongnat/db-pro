// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
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
            draw_trigger_designer(context, ui);
        }
        SchemaWorkbenchMode::Sequence => {
            draw_sequence_designer(context, ui);
        }
        SchemaWorkbenchMode::Type => {
            draw_type_designer(context, ui);
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
        SchemaWorkbenchMode::Dependencies | SchemaWorkbenchMode::Docs | SchemaWorkbenchMode::History => {}
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

fn draw_trigger_designer(context: &mut SchemaWorkbenchFormContext<'_>, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Trigger Designer")
                .strong()
                .color(context.theme.text_primary),
        );
        ui.add_space(SPACE_MD);
        if Button::new(context.theme)
            .text("+ Audit Trail Preset")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            context.workbench.timing = "AFTER".into();
            context.workbench.event = "INSERT,UPDATE,DELETE".into();
            context.workbench.body = "FOR EACH ROW EXECUTE FUNCTION audit_log_changes()".into();
        }
        if Button::new(context.theme)
            .text("+ Auto Updated-At Preset")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            context.workbench.timing = "BEFORE".into();
            context.workbench.event = "UPDATE".into();
            context.workbench.body = "FOR EACH ROW EXECUTE FUNCTION set_updated_at_timestamp()".into();
        }
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
            ui.label(RichText::new("Timing").small().color(context.theme.text_secondary));
            let timings = ["BEFORE", "AFTER", "INSTEAD OF"];
            let timing_labels: Vec<String> = timings.iter().map(|t| (*t).to_owned()).collect();
            let mut timing_selected = timings.iter().position(|t| *t == context.workbench.timing).unwrap_or(0);
            let timing_previous = timing_selected;
            crate::components::Select::new("trigger_timing_cb", &mut timing_selected, &timing_labels)
                .theme(context.theme)
                .width(140.0)
                .size(crate::components::SelectSize::Sm)
                .variant(crate::components::SelectVariant::Ghost)
                .show(ui);
            if timing_selected != timing_previous {
                context.workbench.timing = timings[timing_selected].to_owned();
            }
        });
        ui.add_space(SPACE_MD);
        ui.vertical(|ui| {
            ui.set_width(220.0);
            ui.label(RichText::new("Events CSV (e.g. INSERT, UPDATE)").small().color(context.theme.text_secondary));
            crate::components::input::Input::new(&mut context.workbench.event, "INSERT, UPDATE", context.theme)
                .width(220.0)
                .show(ui);
        });
    });
    ui.add_space(SPACE_SM);
    ui.vertical(|ui| {
        // cc-scan:allow LINE_TOO_LONG — literal must not wrap
        ui.label(RichText::new("Trigger Body / Action (e.g. FOR EACH ROW EXECUTE FUNCTION ...):").small().color(context.theme.text_secondary));
        crate::components::Textarea::new(
                &mut context.workbench.body,
                "FOR EACH ROW EXECUTE FUNCTION ...",
                context.theme,
            )
            .min_rows(4)
            .show(ui);
    });
}

fn draw_sequence_designer(context: &mut SchemaWorkbenchFormContext<'_>, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Sequence Designer")
                .strong()
                .color(context.theme.text_primary),
        );
        ui.add_space(SPACE_MD);
        if Button::new(context.theme)
            .text("+ Reset to 1..N Defaults")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            context.workbench.start = "1".into();
            context.workbench.increment = "1".into();
            context.workbench.cycle = false;
        }
    });
    ui.add_space(SPACE_SM);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(160.0);
            ui.label(RichText::new("Start Value").small().color(context.theme.text_secondary));
            crate::components::input::Input::new(&mut context.workbench.start, "1", context.theme)
                .width(160.0)
                .show(ui);
        });
        ui.add_space(SPACE_MD);
        ui.vertical(|ui| {
            ui.set_width(160.0);
            ui.label(RichText::new("Increment By").small().color(context.theme.text_secondary));
            crate::components::input::Input::new(&mut context.workbench.increment, "1", context.theme)
                .width(160.0)
                .show(ui);
        });
        ui.add_space(SPACE_MD);
        ui.vertical(|ui| {
            ui.label(RichText::new("Cycle").small().color(context.theme.text_secondary));
            ui.checkbox(&mut context.workbench.cycle, "Enable CYCLE");
        });
    });
}

fn draw_type_designer(context: &mut SchemaWorkbenchFormContext<'_>, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Custom Enum Type Designer")
                .strong()
                .color(context.theme.text_primary),
        );
        ui.add_space(SPACE_MD);
        if Button::new(context.theme)
            .text("+ Status Enum Preset")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            context.workbench.enum_values_csv = "'draft', 'pending', 'active', 'archived'".into();
        }
        if Button::new(context.theme)
            .text("+ Priority Enum Preset")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            context.workbench.enum_values_csv = "'low', 'normal', 'high', 'urgent'".into();
        }
    });
    ui.add_space(SPACE_SM);
    ui.vertical(|ui| {
        // cc-scan:allow LINE_TOO_LONG — literal must not wrap
        ui.label(RichText::new("Enum Values CSV (comma separated values):").small().color(context.theme.text_secondary));
        // cc-scan:allow LINE_TOO_LONG — literal must not wrap
        crate::components::input::Input::new(&mut context.workbench.enum_values_csv, "'val1', 'val2', 'val3'", context.theme)
            .width(520.0)
            .show(ui);
    });
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
            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
            context.workbench.select_sql = "SELECT date_trunc('day', created_at) AS date,\n       count(*) AS total_records,\n       sum(amount) AS total_amount\nFROM transactions\nGROUP BY 1\nORDER BY 1 DESC;".to_owned();
        }
        if Button::new(context.theme)
            .text("+ Multi-Table JOIN")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
            context.workbench.select_sql = "SELECT o.id AS order_id,\n       u.name AS customer_name,\n       o.total_price,\n       o.status\nFROM orders o\nJOIN users u ON u.id = o.user_id\nWHERE o.status != 'cancelled';".to_owned();
        }
    });
    ui.add_space(SPACE_SM);

    ui.vertical(|ui| {
        ui.label(RichText::new("Query Definition (SELECT Statement):").small().color(context.theme.text_secondary));
        crate::components::Textarea::new(&mut context.workbench.select_sql, "SELECT ...", context.theme)
            .min_rows(5)
            .show(ui);
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
            let method_labels: Vec<String> = methods.iter().map(|m| (*m).to_owned()).collect();
            let mut method_selected = methods.iter().position(|m| *m == context.workbench.index_method).unwrap_or(0);
            let method_previous = method_selected;
            crate::components::Select::new("index_method_cb", &mut method_selected, &method_labels)
                .theme(context.theme)
                .width(140.0)
                .size(crate::components::SelectSize::Sm)
                .variant(crate::components::SelectVariant::Ghost)
                .show(ui);
            if method_selected != method_previous {
                context.workbench.index_method = methods[method_selected].to_owned();
            }
        });
        ui.add_space(SPACE_MD);
        ui.vertical(|ui| {
            ui.set_width(260.0);
            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
            ui.label(RichText::new("Columns CSV (e.g. email, created_at DESC)").small().color(context.theme.text_secondary));
            crate::components::input::Input::new(&mut context.workbench.columns_csv, "col1, col2", context.theme)
                .width(260.0)
                .show(ui);
        });
    });
    ui.add_space(SPACE_SM);
    ui.vertical(|ui| {
        // cc-scan:allow LINE_TOO_LONG — literal must not wrap
        ui.label(RichText::new("WHERE Predicate / Partial Index (Optional):").small().color(context.theme.text_secondary));
        // cc-scan:allow LINE_TOO_LONG — literal must not wrap
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
        const CONSTRAINT_KINDS: [(ConstraintKindUi, &str); 4] = [
            (ConstraintKindUi::PrimaryKey, "Primary key (PK)"),
            (ConstraintKindUi::Unique, "Unique constraint (UQ)"),
            (ConstraintKindUi::Check, "Check constraint (CK)"),
            (ConstraintKindUi::ForeignKey, "Foreign key (FK)"),
        ];
        let kind_labels: Vec<String> = CONSTRAINT_KINDS.iter().map(|(_, label)| (*label).to_owned()).collect();
        let mut kind_selected = CONSTRAINT_KINDS
            .iter()
            .position(|(kind, _)| *kind == context.workbench.constraint_kind)
            .unwrap_or(0);
        let kind_previous = kind_selected;
        crate::components::Select::new("constraint_kind_cb", &mut kind_selected, &kind_labels)
            .theme(context.theme)
            .width(180.0)
            .size(crate::components::SelectSize::Sm)
            .variant(crate::components::SelectVariant::Ghost)
            .show(ui);
        if kind_selected != kind_previous {
            context.workbench.constraint_kind = CONSTRAINT_KINDS[kind_selected].0;
        }
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
            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
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
                let action_labels: Vec<String> = actions.iter().map(|a| (*a).to_owned()).collect();
                let mut action_selected = actions.iter().position(|a| *a == context.workbench.on_delete).unwrap_or(0);
                let action_previous = action_selected;
                crate::components::Select::new("fk_on_delete_cb", &mut action_selected, &action_labels)
                    .theme(context.theme)
                    .width(180.0)
                    .size(crate::components::SelectSize::Sm)
                    .variant(crate::components::SelectVariant::Ghost)
                    .show(ui);
                if action_selected != action_previous {
                    context.workbench.on_delete = actions[action_selected].to_owned();
                }
            });
            ui.add_space(SPACE_MD);
            ui.vertical(|ui| {
                ui.set_width(180.0);
                ui.label(RichText::new("ON UPDATE").small().color(context.theme.text_secondary));
                let action_labels: Vec<String> = actions.iter().map(|a| (*a).to_owned()).collect();
                let mut action_selected = actions.iter().position(|a| *a == context.workbench.on_update).unwrap_or(0);
                let action_previous = action_selected;
                crate::components::Select::new("fk_on_update_cb", &mut action_selected, &action_labels)
                    .theme(context.theme)
                    .width(180.0)
                    .size(crate::components::SelectSize::Sm)
                    .variant(crate::components::SelectVariant::Ghost)
                    .show(ui);
                if action_selected != action_previous {
                    context.workbench.on_update = actions[action_selected].to_owned();
                }
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

    egui::Frame::NONE
        .fill(context.theme.surface_panel)
        .corner_radius(egui::CornerRadius::same(6.0 as u8))
        .inner_margin(egui::Margin::same(8.0 as i8))
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
                    crate::components::Input::new(&mut col.name, "column_name", context.theme)
                        .width(150.0)
                        .show(ui);

                    // Type select — same widget as every other picker
                    let type_salt = format!("col_type_{idx}");
                    let type_labels: Vec<String> = types.iter().map(|t| (*t).to_owned()).collect();
                    let mut type_selected = types.iter().position(|t| *t == col.data_type).unwrap_or(0);
                    let type_previous = type_selected;
                    crate::components::Select::new(&type_salt, &mut type_selected, &type_labels)
                        .theme(context.theme)
                        .width(120.0)
                        .size(crate::components::SelectSize::Sm)
                        .variant(crate::components::SelectVariant::Ghost)
                        .show(ui);
                    if type_selected != type_previous {
                        col.data_type = types[type_selected].to_owned();
                    }

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
                    crate::components::Input::new(&mut col.default_expr, "default / expr", context.theme)
                        .width(130.0)
                        .show(ui);

                    // Reorder & delete buttons — icon buttons, not glyph labels
                    if idx > 0
                        && compact_icon_button(ui, Icon::ChevronUp, context.theme)
                            .on_hover_text("Move column up")
                            .clicked()
                    {
                        move_up_idx = Some(idx);
                    }
                    if idx + 1 < col_count
                        && compact_icon_button(ui, Icon::ChevronDown, context.theme)
                            .on_hover_text("Move column down")
                            .clicked()
                    {
                        move_down_idx = Some(idx);
                    }
                    if compact_icon_button(ui, Icon::X, context.theme)
                        .on_hover_text("Remove column")
                        .clicked()
                    {
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
    egui::Frame::NONE
        .fill(context.theme.surface_panel)
        .corner_radius(egui::CornerRadius::same(6.0 as u8))
        .inner_margin(egui::Margin::same(10.0 as i8))
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
                            .wrap(),
                        );
                    });
            }
        });

    if context.workbench.apply_confirmation {
        let mut confirm_action = None;
        let mut open = true;
        Dialog::new(&mut open, "Confirm Apply DDL Mutation", context.theme)
            .show(ui, |ui| {
                ui.colored_label(
                    context.theme.warning,
                    "Applying this DDL will directly modify the active database schema.",
                );
                ui.add_space(SPACE_SM);
                CodeBlock::new(&sql, context.theme).language("sql").show(ui);
                ui.add_space(SPACE_MD);
                ui.horizontal(|ui| {
                    if Button::new(context.theme)
                        .text("Apply DDL")
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .icon(Icon::Check)
                        .show(ui)
                        .clicked()
                    {
                        confirm_action = Some(SchemaWorkbenchFormAction::ApplyDdl);
                        context.workbench.apply_confirmation = false;
                    }
                    if Button::new(context.theme)
                        .text("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .icon(Icon::X)
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
        if confirm_action.is_some() {
            action = confirm_action;
        }
    }

    action
}
