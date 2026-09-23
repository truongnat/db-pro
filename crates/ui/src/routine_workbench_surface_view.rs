//! Routine workbench surface and typed intents.

use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::UiRoutineParameter;
use egui::RichText;
use lucide_icons::Icon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RoutineWorkbenchAction {
    PreviewDdl,
    ApplyDdl,
    RequestDrop,
    ConfirmDrop,
    CancelDrop,
    Execute(String),
    OpenInQuery(String),
}

pub(super) struct RoutineWorkbenchContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) function: &'a UiFunctionSummary,
    pub(super) source_draft: &'a mut String,
    pub(super) ddl_preview: Option<&'a str>,
    pub(super) drop_confirm: &'a mut bool,
    pub(super) parameter_values: &'a mut Vec<String>,
    pub(super) parameter_nulls: &'a mut Vec<bool>,
}

pub(super) fn draw_workbench(
    context: &mut RoutineWorkbenchContext<'_>,
    ui: &mut egui::Ui,
) -> Option<RoutineWorkbenchAction> {
    let mut action = draw_source_card(context, ui);
    ui.add_space(SPACE_MD);
    if let Some(form_action) = draw_invoke_card(context, ui) {
        action = Some(form_action);
    }
    action
}

fn draw_source_card(context: &mut RoutineWorkbenchContext<'_>, ui: &mut egui::Ui) -> Option<RoutineWorkbenchAction> {
    let mut action = None;
    card_frame(context.theme).show(ui, |ui| {
        draw_source_editor(context, ui);
        action = draw_source_actions(context, ui);
        if let Some(preview) = context.ddl_preview {
            draw_ddl_preview(context.theme, preview, ui);
        }
        if *context.drop_confirm {
            draw_drop_confirmation(context, ui, &mut action);
        }
    });
    action
}

fn draw_source_editor(context: &mut RoutineWorkbenchContext<'_>, ui: &mut egui::Ui) {
    section_label(ui, format!("{} SOURCE", context.function.routine_type), context.theme);
    ui.add_space(SPACE_SM);
    ui.add(
        egui::TextEdit::multiline(context.source_draft)
            .code_editor()
            .desired_width(ui.available_width())
            .desired_rows(12),
    );
    ui.add_space(SPACE_SM);
}

fn draw_source_actions(context: &RoutineWorkbenchContext<'_>, ui: &mut egui::Ui) -> Option<RoutineWorkbenchAction> {
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        if Button::new(context.theme)
            .icon(Icon::Eye)
            .text("Preview DDL")
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            action = Some(RoutineWorkbenchAction::PreviewDdl);
        }
        if Button::new(context.theme)
            .icon(Icon::Check)
            .text("Apply CREATE OR REPLACE")
            .variant(ButtonVariant::Default)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            action = Some(RoutineWorkbenchAction::ApplyDdl);
        }
        if Button::new(context.theme)
            .text("Drop…")
            .variant(ButtonVariant::Destructive)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            action = Some(RoutineWorkbenchAction::RequestDrop);
        }
    });
    action
}

fn draw_ddl_preview(theme: DbProTheme, preview: &str, ui: &mut egui::Ui) {
    ui.add_space(SPACE_SM);
    ui.label(RichText::new("DDL preview").small().color(theme.text_secondary));
    CodeBlock::new(preview, theme).language("sql").show(ui);
}

fn draw_drop_confirmation(
    context: &RoutineWorkbenchContext<'_>,
    ui: &mut egui::Ui,
    action: &mut Option<RoutineWorkbenchAction>,
) {
    ui.add_space(SPACE_SM);
    ui.colored_label(
        context.theme.warning,
        format!(
            "Drop {} {}.{}({})?",
            context.function.routine_type,
            context.function.schema,
            context.function.name,
            context.function.identity_arguments
        ),
    );
    ui.horizontal(|ui| {
        if Button::new(context.theme)
            .text("Confirm drop")
            .variant(ButtonVariant::Destructive)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            *action = Some(RoutineWorkbenchAction::ConfirmDrop);
        }
        if Button::new(context.theme)
            .icon(Icon::X)
            .text("Cancel")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            *action = Some(RoutineWorkbenchAction::CancelDrop);
        }
    });
}

fn draw_invoke_card(context: &mut RoutineWorkbenchContext<'_>, ui: &mut egui::Ui) -> Option<RoutineWorkbenchAction> {
    let inputs = input_parameters(context.function);
    if context.parameter_values.len() != inputs.len() {
        context.parameter_values.resize(inputs.len(), String::new());
        context.parameter_nulls.resize(inputs.len(), false);
    }

    let is_procedure = context.function.routine_type.eq_ignore_ascii_case("PROCEDURE");
    let mut action = None;
    card_frame(context.theme).show(ui, |ui| {
        section_label(
            ui,
            if is_procedure {
                "CALL PROCEDURE"
            } else {
                "EXECUTE FUNCTION"
            },
            context.theme,
        );
        ui.add_space(SPACE_SM);
        if inputs.is_empty() {
            ui.label(
                RichText::new("No input parameters.")
                    .small()
                    .color(context.theme.text_muted),
            );
        } else {
            draw_parameter_inputs(context, ui, &inputs);
        }
        let invoke_sql = build_routine_invoke_sql(context.function, context.parameter_values, context.parameter_nulls);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("Generated SQL")
                .small()
                .color(context.theme.text_secondary),
        );
        CodeBlock::new(&invoke_sql, context.theme).language("sql").show(ui);
        ui.add_space(SPACE_SM);
        action = draw_invoke_actions(context.theme, is_procedure, &invoke_sql, ui);
        if is_procedure {
            ui.label(
                RichText::new("Procedures use CALL and go through the destructive-query confirmation gate.")
                    .small()
                    .color(context.theme.text_muted),
            );
        }
    });
    action
}

fn draw_invoke_actions(
    theme: DbProTheme,
    is_procedure: bool,
    invoke_sql: &str,
    ui: &mut egui::Ui,
) -> Option<RoutineWorkbenchAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        if Button::new(theme)
            .icon(Icon::Play)
            .text(if is_procedure { "Call" } else { "Execute" })
            .variant(ButtonVariant::Default)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            action = Some(RoutineWorkbenchAction::Execute(invoke_sql.to_owned()));
        }
        if Button::new(theme)
            .icon(Icon::FileCode2)
            .text("Open SQL in editor")
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            action = Some(RoutineWorkbenchAction::OpenInQuery(invoke_sql.to_owned()));
        }
    });
    action
}

fn draw_parameter_inputs(context: &mut RoutineWorkbenchContext<'_>, ui: &mut egui::Ui, inputs: &[&UiRoutineParameter]) {
    for (index, parameter) in inputs.iter().enumerate() {
        ui.horizontal(|ui| {
            let label = if parameter.name.is_empty() {
                format!("arg{}", index + 1)
            } else {
                parameter.name.clone()
            };
            ui.label(
                RichText::new(format!("{label} · {} · {}", parameter.data_type, parameter.mode))
                    .color(context.theme.text_secondary),
            );
            if parameter.has_default {
                badge(ui, "default", context.theme.surface_active, context.theme.text_muted);
            }
        });
        ui.horizontal(|ui| {
            let mut is_null = context.parameter_nulls.get(index).copied().unwrap_or(false);
            if ui.checkbox(&mut is_null, "NULL").changed() {
                if let Some(slot) = context.parameter_nulls.get_mut(index) {
                    *slot = is_null;
                }
            }
            ui.add_enabled_ui(!is_null, |ui| {
                if let Some(value) = context.parameter_values.get_mut(index) {
                    ui.add(
                        egui::TextEdit::singleline(value)
                            .desired_width(ui.available_width())
                            .hint_text(if parameter.has_default {
                                parameter.default_expr.as_str()
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

fn input_parameters(function: &UiFunctionSummary) -> Vec<&UiRoutineParameter> {
    function
        .parameters
        .iter()
        .filter(|parameter| {
            let mode = parameter.mode.to_ascii_uppercase();
            matches!(mode.as_str(), "IN" | "INOUT" | "VARIADIC" | "")
        })
        .collect()
}

pub(super) fn build_routine_invoke_sql(function: &UiFunctionSummary, values: &[String], nulls: &[bool]) -> String {
    let inputs = input_parameters(function);
    let args = inputs
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            if nulls.get(index).copied().unwrap_or(false) {
                return "NULL".to_owned();
            }
            let raw = values.get(index).map(String::as_str).unwrap_or("");
            if raw.is_empty() && parameter.has_default {
                return "DEFAULT".to_owned();
            }
            quote_sql_literal_or_raw(raw, &parameter.data_type)
        })
        .collect::<Vec<_>>();
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
    let normalized_type = data_type.to_ascii_lowercase();
    if normalized_type.contains("int")
        || normalized_type.contains("numeric")
        || normalized_type.contains("decimal")
        || normalized_type.contains("float")
        || normalized_type.contains("double")
        || normalized_type.contains("real")
        || normalized_type.contains("bool")
        || normalized_type == "money"
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
                    default_expr: "".into(),
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
