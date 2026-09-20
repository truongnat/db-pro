//! Schema object workspace — views/triggers/routines (#192 routine workbench).
use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use db_pro_core::application::ObjectMutationService;
use db_pro_core::domain::object_mutation::{
    MutationOptions, ObjectAction, ObjectDefinition, ObjectMutationRequest, ObjectRef, RoutineDefinition,
};
use db_pro_core::ports::dialect::SqlDialect;
use egui::RichText;
use lucide_icons::Icon;

struct SchemaObjectDetails {
    icon: Icon,
    kind: String,
    name: String,
    schema: String,
    definition: String,
    metadata: Option<String>,
    query: String,
}

struct QuoteDialect;

impl SqlDialect for QuoteDialect {
    fn placeholder(&self, index: usize) -> String {
        format!("${index}")
    }

    fn quote_identifier(&self, ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }
}

impl DbProApp {
    pub(super) fn draw_schema_object_workspace(&mut self, ui: &mut egui::Ui) {
        let Some(selection) = self.schema.explorer.selected_schema_object.clone() else {
            self.activate_welcome_tab();
            return;
        };
        let is_view = matches!(selection, SchemaObjectSelection::View(_));
        let is_function = matches!(selection, SchemaObjectSelection::Function { .. });
        let Some(details) = self.resolve_schema_object(&selection) else {
            return;
        };

        toolbar_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width().max(0.0));
            ui.horizontal_wrapped(|ui| {
                self.draw_schema_object_breadcrumb(
                    ui,
                    details.icon,
                    &details.kind,
                    &details.schema,
                    &details.name,
                    details.metadata.as_deref(),
                );
                if is_view {
                    self.draw_schema_object_view_tabs(ui);
                }
                if Button::new(self.theme)
                    .icon(Icon::FileCode2)
                    .text("Open in Query")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.set_active_query_text(details.query.clone());
                    self.workspace.active_tab = WorkspaceTab::Query;
                }
            });
        });
        ui.add_space(SPACE_MD);
        if is_function {
            self.draw_routine_workbench(ui, &selection);
        } else if is_view && self.schema.explorer.schema_object_view == SchemaObjectView::Data {
            if self.table.data_query.result.is_none()
                && self.table.data_query.request.is_none()
                && self.table.data_query.error.is_none()
            {
                self.request_table_data();
            }
            self.draw_table_data(ui, &details.name);
        } else {
            self.draw_schema_definition(ui, &details.kind, &details.definition);
        }
    }

    pub(crate) fn sync_routine_workbench_from(&mut self, function: &UiFunctionSummary) {
        self.management.routine.routine_source_draft = function.definition.clone();
        let inputs: Vec<_> = function
            .parameters
            .iter()
            .filter(|p| {
                let mode = p.mode.to_ascii_uppercase();
                mode == "IN" || mode == "INOUT" || mode == "VARIADIC" || mode.is_empty()
            })
            .collect();
        self.management.routine.routine_param_values = inputs
            .iter()
            .map(|p| {
                if p.has_default {
                    p.default_expr.clone()
                } else {
                    String::new()
                }
            })
            .collect();
        self.management.routine.routine_param_nulls = vec![false; inputs.len()];
    }

    fn draw_routine_workbench(&mut self, ui: &mut egui::Ui, selection: &SchemaObjectSelection) {
        let SchemaObjectSelection::Function {
            name,
            identity_arguments,
        } = selection
        else {
            return;
        };
        let Some(function) = self
            .schema
            .explorer
            .schema
            .functions
            .iter()
            .find(|f| &f.name == name && &f.identity_arguments == identity_arguments)
            .cloned()
        else {
            return;
        };

        // Source editor + apply/drop
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, format!("{} SOURCE", function.routine_type), self.theme);
            ui.add_space(SPACE_SM);
            ui.add(
                egui::TextEdit::multiline(&mut self.management.routine.routine_source_draft)
                    .code_editor()
                    .desired_width(ui.available_width())
                    .desired_rows(12),
            );
            ui.add_space(SPACE_SM);
            ui.horizontal_wrapped(|ui| {
                if Button::new(self.theme)
                    .icon(Icon::Eye)
                    .text("Preview DDL")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.preview_routine_mutation(&function, ObjectAction::Alter);
                }
                if Button::new(self.theme)
                    .icon(Icon::Check)
                    .text("Apply CREATE OR REPLACE")
                    .variant(ButtonVariant::Default)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.preview_routine_mutation(&function, ObjectAction::Alter);
                    if let Some(sql) = self.management.routine.routine_ddl_preview.clone() {
                        self.set_active_query_text(sql);
                        self.workspace.active_tab = WorkspaceTab::Query;
                        self.dispatch_query();
                    }
                }
                if Button::new(self.theme)
                    .text("Drop…")
                    .variant(ButtonVariant::Destructive)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.management.routine.routine_drop_confirm = true;
                }
            });
            if let Some(preview) = &self.management.routine.routine_ddl_preview {
                ui.add_space(SPACE_SM);
                ui.label(RichText::new("DDL preview").small().color(self.theme.text_secondary));
                CodeBlock::new(preview, self.theme).language("sql").show(ui);
            }
            if self.management.routine.routine_drop_confirm {
                ui.add_space(SPACE_SM);
                ui.colored_label(
                    self.theme.warning,
                    format!(
                        "Drop {} {}.{}({})?",
                        function.routine_type, function.schema, function.name, function.identity_arguments
                    ),
                );
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .text("Confirm drop")
                        .variant(ButtonVariant::Destructive)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.preview_routine_mutation(&function, ObjectAction::Drop);
                        if let Some(sql) = self.management.routine.routine_ddl_preview.clone() {
                            self.set_active_query_text(sql);
                            self.workspace.active_tab = WorkspaceTab::Query;
                            self.dispatch_query();
                        }
                        self.management.routine.routine_drop_confirm = false;
                    }
                    if Button::new(self.theme)
                        .icon(Icon::X)
                        .text("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.management.routine.routine_drop_confirm = false;
                    }
                });
            }
        });

        ui.add_space(SPACE_MD);

        // Execute / Call form
        card_frame(self.theme).show(ui, |ui| {
            let is_proc = function.routine_type.eq_ignore_ascii_case("PROCEDURE");
            section_label(
                ui,
                if is_proc { "CALL PROCEDURE" } else { "EXECUTE FUNCTION" },
                self.theme,
            );
            ui.add_space(SPACE_SM);
            let inputs: Vec<_> = function
                .parameters
                .iter()
                .filter(|p| {
                    let mode = p.mode.to_ascii_uppercase();
                    mode == "IN" || mode == "INOUT" || mode == "VARIADIC" || mode.is_empty()
                })
                .cloned()
                .collect();
            if self.management.routine.routine_param_values.len() != inputs.len() {
                self.sync_routine_workbench_from(&function);
            }
            if inputs.is_empty() {
                ui.label(
                    RichText::new("No input parameters.")
                        .small()
                        .color(self.theme.text_muted),
                );
            } else {
                for (idx, param) in inputs.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let label = if param.name.is_empty() {
                            format!("arg{}", idx + 1)
                        } else {
                            param.name.clone()
                        };
                        ui.label(
                            RichText::new(format!("{label} · {} · {}", param.data_type, param.mode))
                                .color(self.theme.text_secondary),
                        );
                        if param.has_default {
                            badge(ui, "default", self.theme.surface_active, self.theme.text_muted);
                        }
                    });
                    ui.horizontal(|ui| {
                        let is_null = self
                            .management
                            .routine
                            .routine_param_nulls
                            .get(idx)
                            .copied()
                            .unwrap_or(false);
                        let mut null_flag = is_null;
                        if ui.checkbox(&mut null_flag, "NULL").changed() {
                            if let Some(slot) = self.management.routine.routine_param_nulls.get_mut(idx) {
                                *slot = null_flag;
                            }
                        }
                        ui.add_enabled_ui(!null_flag, |ui| {
                            if let Some(value) = self.management.routine.routine_param_values.get_mut(idx) {
                                ui.add(
                                    egui::TextEdit::singleline(value)
                                        .desired_width(ui.available_width())
                                        .hint_text(if param.has_default {
                                            param.default_expr.as_str()
                                        } else {
                                            "value"
                                        }),
                                );
                            }
                        });
                    });
                    ui.add_space(4.0);
                }
            }
            ui.add_space(SPACE_SM);
            let invoke_sql = build_routine_invoke_sql(
                &function,
                &self.management.routine.routine_param_values,
                &self.management.routine.routine_param_nulls,
            );
            ui.label(RichText::new("Generated SQL").small().color(self.theme.text_secondary));
            CodeBlock::new(&invoke_sql, self.theme).language("sql").show(ui);
            ui.add_space(SPACE_SM);
            ui.horizontal(|ui| {
                if Button::new(self.theme)
                    .icon(Icon::Play)
                    .text(if is_proc { "Call" } else { "Execute" })
                    .variant(ButtonVariant::Default)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.set_active_query_text(invoke_sql.clone());
                    self.workspace.active_tab = WorkspaceTab::Query;
                    self.dispatch_query();
                }
                if Button::new(self.theme)
                    .icon(Icon::FileCode2)
                    .text("Open SQL in editor")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.set_active_query_text(invoke_sql.clone());
                    self.workspace.active_tab = WorkspaceTab::Query;
                }
            });
            if is_proc {
                ui.label(
                    RichText::new("Procedures use CALL and go through the destructive-query confirmation gate.")
                        .small()
                        .color(self.theme.text_muted),
                );
            }
        });
    }

    fn preview_routine_mutation(&mut self, function: &UiFunctionSummary, action: ObjectAction) {
        let definition = RoutineDefinition {
            schema: function.schema.clone(),
            name: function.name.clone(),
            routine_type: function.routine_type.clone(),
            identity_arguments: function.identity_arguments.clone(),
            definition_sql: self.management.routine.routine_source_draft.clone(),
            replace: true,
        };
        let request = ObjectMutationRequest {
            driver: self.active_driver().to_owned(),
            action,
            target: Some(ObjectRef {
                kind: db_pro_core::domain::object_mutation::ObjectKind::Routine,
                schema: Some(function.schema.clone()),
                name: function.name.clone(),
                parent: None,
            }),
            definition: ObjectDefinition::Routine(definition),
            options: MutationOptions {
                cascade: false,
                if_exists: true,
                if_not_exists: false,
                dry_run: action == ObjectAction::GenerateDdl,
            },
        };
        match ObjectMutationService::plan(&request, &QuoteDialect) {
            Ok(plan) => {
                self.management.routine.routine_ddl_preview = Some(plan.statements.join(";\n"));
                self.feedback.runtime_message = format!(
                    "Routine DDL preview · {} statement(s) · {}",
                    plan.statements.len(),
                    plan.safety
                );
            }
            Err(error) => {
                self.management.routine.routine_ddl_preview = None;
                self.feedback.runtime_message = format!("Routine plan failed: {error}");
            }
        }
    }

    fn resolve_schema_object(&self, selection: &SchemaObjectSelection) -> Option<SchemaObjectDetails> {
        match selection {
            SchemaObjectSelection::View(name) => {
                let view = self
                    .schema
                    .explorer
                    .schema
                    .views
                    .iter()
                    .find(|view| &view.name == name)?
                    .clone();
                Some(SchemaObjectDetails {
                    icon: Icon::Eye,
                    kind: "VIEW".to_owned(),
                    name: view.name.clone(),
                    schema: view.schema.clone(),
                    definition: view.definition,
                    metadata: None,
                    query: format!("SELECT *\nFROM \"{}\".\"{}\"\nLIMIT 100;", view.schema, view.name),
                })
            }
            SchemaObjectSelection::Trigger(name) => {
                let trigger = self
                    .schema
                    .explorer
                    .schema
                    .triggers
                    .iter()
                    .find(|trigger| &trigger.name == name)?
                    .clone();
                Some(SchemaObjectDetails {
                    icon: Icon::Zap,
                    kind: "TRIGGER".to_owned(),
                    name: trigger.name,
                    schema: trigger.schema,
                    definition: trigger.definition.clone(),
                    metadata: Some(format!(
                        "{} · {} · {}",
                        trigger.table_name, trigger.timing, trigger.event
                    )),
                    query: trigger.definition,
                })
            }
            SchemaObjectSelection::Function {
                name,
                identity_arguments,
            } => {
                let function = self
                    .schema
                    .explorer
                    .schema
                    .functions
                    .iter()
                    .find(|function| &function.name == name && &function.identity_arguments == identity_arguments)?
                    .clone();
                let display_name = if function.identity_arguments.is_empty() {
                    function.name.clone()
                } else {
                    format!("{}({})", function.name, function.identity_arguments)
                };
                Some(SchemaObjectDetails {
                    icon: Icon::Code2,
                    kind: function.routine_type,
                    name: display_name,
                    schema: function.schema,
                    definition: function.definition.clone(),
                    metadata: Some(format!(
                        "{} · {} · returns {}{}",
                        function.language,
                        function.volatility,
                        function.data_type,
                        if function.security_definer {
                            " · SECURITY DEFINER"
                        } else {
                            ""
                        }
                    )),
                    query: function.definition,
                })
            }
        }
    }

    fn draw_schema_object_breadcrumb(
        &self,
        ui: &mut egui::Ui,
        icon: Icon,
        kind: &str,
        schema: &str,
        name: &str,
        metadata: Option<&str>,
    ) {
        ui.label(icon_text(icon, kind, self.theme.text_primary));
        ui.label(icon_text(Icon::ChevronRight, "", self.theme.text_muted));
        ui.label(
            RichText::new(format!("{schema}.{name}"))
                .strong()
                .color(self.theme.accent),
        );
        if let Some(metadata) = metadata {
            badge(ui, metadata, self.theme.surface_active, self.theme.text_secondary);
        }
    }

    fn draw_schema_object_view_tabs(&mut self, ui: &mut egui::Ui) {
        for (view, icon, label) in [
            (SchemaObjectView::Definition, Icon::Code2, "Definition"),
            (SchemaObjectView::Data, Icon::Table2, "Data"),
        ] {
            let selected = self.schema.explorer.schema_object_view == view;
            let tab = tab_frame(self.theme, selected).show(ui, |ui| {
                ui.selectable_label(selected, icon_text(icon, label, self.theme.text_primary))
            });
            if tab.inner.clicked() {
                self.schema.explorer.schema_object_view = view;
                if view == SchemaObjectView::Data {
                    self.table.data_query.result = None;
                    self.table.data_query.total_rows = None;
                    self.table.data_query.error = None;
                    self.table.data_query.offset = 0;
                    self.request_table_data();
                }
            }
        }
    }

    fn draw_schema_definition(&self, ui: &mut egui::Ui, kind: &str, definition: &str) {
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            section_label(ui, format!("{kind} DEFINITION"), self.theme);
            ui.add_space(SPACE_SM);
            CodeBlock::new(definition, self.theme).language("sql").show(ui);
        });
    }
}

/// Build SELECT/CALL SQL for a routine from typed form values (#192).
pub(crate) fn build_routine_invoke_sql(function: &UiFunctionSummary, values: &[String], nulls: &[bool]) -> String {
    let inputs: Vec<_> = function
        .parameters
        .iter()
        .filter(|p| {
            let mode = p.mode.to_ascii_uppercase();
            mode == "IN" || mode == "INOUT" || mode == "VARIADIC" || mode.is_empty()
        })
        .collect();
    let mut args = Vec::new();
    for (idx, param) in inputs.iter().enumerate() {
        if nulls.get(idx).copied().unwrap_or(false) {
            args.push("NULL".to_owned());
            continue;
        }
        let raw = values.get(idx).map(String::as_str).unwrap_or("");
        if raw.is_empty() && param.has_default {
            args.push("DEFAULT".to_owned());
            continue;
        }
        args.push(quote_sql_literal_or_raw(raw, &param.data_type));
    }
    let qualified = format!(
        "\"{}\".\"{}\"",
        function.schema.replace('"', "\"\""),
        function.name.replace('"', "\"\"")
    );
    let arg_list = args.join(", ");
    if function.routine_type.eq_ignore_ascii_case("PROCEDURE") {
        format!("CALL {qualified}({arg_list});")
    } else {
        format!("SELECT * FROM {qualified}({arg_list});")
    }
}

fn quote_sql_literal_or_raw(value: &str, data_type: &str) -> String {
    let ty = data_type.to_ascii_lowercase();
    if ty.contains("int")
        || ty.contains("numeric")
        || ty.contains("decimal")
        || ty.contains("float")
        || ty.contains("double")
        || ty.contains("real")
        || ty.contains("bool")
        || ty == "money"
    {
        if value.trim().is_empty() {
            "NULL".to_owned()
        } else {
            value.trim().to_owned()
        }
    } else if value.eq_ignore_ascii_case("null") {
        "NULL".to_owned()
    } else {
        format!("'{}'", value.replace('\'', "''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UiRoutineParameter;

    fn sample_fn(routine_type: &str, params: Vec<UiRoutineParameter>) -> UiFunctionSummary {
        UiFunctionSummary {
            schema: "public".into(),
            name: "calc".into(),
            routine_type: routine_type.into(),
            data_type: "integer".into(),
            definition: "CREATE FUNCTION ...".into(),
            identity_arguments: "x integer".into(),
            language: "sql".into(),
            volatility: "VOLATILE".into(),
            security_definer: false,
            parameters: params,
        }
    }

    #[test]
    fn function_execute_uses_select_and_null_handling() {
        let function = sample_fn(
            "FUNCTION",
            vec![
                UiRoutineParameter {
                    name: "x".into(),
                    data_type: "integer".into(),
                    mode: "IN".into(),
                    has_default: false,
                    default_expr: String::new(),
                },
                UiRoutineParameter {
                    name: "label".into(),
                    data_type: "text".into(),
                    mode: "IN".into(),
                    has_default: true,
                    default_expr: "'hi'".into(),
                },
            ],
        );
        let sql = build_routine_invoke_sql(&function, &["42".into(), String::new()], &[false, false]);
        assert_eq!(sql, "SELECT * FROM \"public\".\"calc\"(42, DEFAULT);");
        let sql_null = build_routine_invoke_sql(&function, &["42".into(), "x".into()], &[false, true]);
        assert_eq!(sql_null, "SELECT * FROM \"public\".\"calc\"(42, NULL);");
    }

    #[test]
    fn procedure_execute_uses_call() {
        let function = sample_fn("PROCEDURE", vec![]);
        let sql = build_routine_invoke_sql(&function, &[], &[]);
        assert_eq!(sql, "CALL \"public\".\"calc\"();");
    }
}
