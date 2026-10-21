//! Routine workbench surface and typed intents.

use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::query::{placeholder_style_for_driver, render_placeholder, PlaceholderStyle};
use crate::UiRoutineParameter;
use db_pro_core::domain::query::QueryParam;
use egui::RichText;
use lucide_icons::Icon;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum RoutineWorkbenchAction {
    PreviewDdl,
    ApplyDdl,
    RequestDrop,
    ConfirmDrop,
    CancelDrop,
    /// Placeholder-only invoke SQL and its typed bind arguments (#252). `Err`
    /// is a typed validation failure naming the offending parameter — nothing
    /// is dispatched, and no literal argument value ever enters the SQL text.
    Execute {
        sql: String,
        arguments: Result<Vec<QueryParam>, String>,
    },
    OpenInQuery(String),
}

/// The workbench's generated invocation: SQL that carries only provider
/// placeholders, and the typed values bound to them
/// (`docs/goals/goal-phase-b-routines.md` §9.3/§12.1).
#[derive(Debug, PartialEq)]
pub(super) struct RoutineInvocation {
    pub(super) sql: String,
    pub(super) arguments: Result<Vec<QueryParam>, String>,
    /// True when an argument's type is not directly bindable (array / enum /
    /// composite / unknown) and was bound as text with an explicit `::type`
    /// cast — the §9.3 stated limitation.
    pub(super) bound_with_cast: bool,
}

pub(super) struct RoutineWorkbenchContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) function: &'a UiFunctionSummary,
    pub(super) source_draft: &'a mut String,
    pub(super) ddl_preview: Option<&'a str>,
    pub(super) drop_confirm: &'a mut bool,
    pub(super) parameter_values: &'a mut Vec<String>,
    pub(super) parameter_nulls: &'a mut Vec<bool>,
    /// SQL dialect for highlighted previews (#336).
    pub(super) dialect: crate::editor::SqlDialect,
    /// Active driver — decides placeholder spelling (`$n` / `?` / `@pn`) via the
    /// same mechanism the query editor uses (#252).
    pub(super) driver: &'a str,
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
            draw_ddl_preview(context.theme, context.dialect, preview, ui);
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
    crate::components::Textarea::new(context.source_draft, "routine source...", context.theme)
            .min_rows(8)
            .show(ui);
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

fn draw_ddl_preview(theme: DbProTheme, dialect: crate::editor::SqlDialect, preview: &str, ui: &mut egui::Ui) {
    ui.add_space(SPACE_SM);
    ui.label(RichText::new("DDL preview").small().color(theme.text_secondary));
    CodeBlock::new(preview, theme).language("sql").sql_dialect(dialect).show(ui);
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
        let invoke = build_routine_invoke_sql(context.function, context.parameter_values, context.parameter_nulls, context.driver);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("Generated SQL")
                .small()
                .color(context.theme.text_secondary),
        );
        CodeBlock::new(&invoke.sql, context.theme).language("sql").sql_dialect(context.dialect).show(ui);
        if invoke.bound_with_cast {
            ui.label(
                RichText::new("Types that cannot be bound directly are passed as text with an explicit cast.")
                    .small()
                    .color(context.theme.text_muted),
            );
        }
        ui.add_space(SPACE_SM);
        action = draw_invoke_actions(context.theme, is_procedure, invoke, ui);
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
    invoke: RoutineInvocation,
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
            action = Some(RoutineWorkbenchAction::Execute {
                sql: invoke.sql.clone(),
                arguments: invoke.arguments,
            });
        }
        if Button::new(theme)
            .icon(Icon::FileCode2)
            .text("Open SQL in editor")
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            action = Some(RoutineWorkbenchAction::OpenInQuery(invoke.sql));
        }
    });
    action
}

fn draw_parameter_inputs(context: &mut RoutineWorkbenchContext<'_>, ui: &mut egui::Ui, inputs: &[&UiRoutineParameter]) {
    for (index, parameter) in inputs.iter().enumerate() {
        ui.horizontal(|ui| {
            let label = argument_label(parameter, index);
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
            if Checkbox::new(&mut is_null, "NULL", context.theme).show(ui).changed() {
                if let Some(slot) = context.parameter_nulls.get_mut(index) {
                    *slot = is_null;
                }
            }
            ui.add_enabled_ui(!is_null, |ui| {
                if let Some(value) = context.parameter_values.get_mut(index) {
                    input(
                        ui,
                        value,
                        if parameter.has_default {
                            parameter.default_expr.as_str()
                        } else {
                            "value"
                        },
                        ui.available_width(),
                        context.theme,
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

/// Generate the invoke statement for the workbench (#252).
///
/// Invariant (`goal-phase-b-routines.md` §12.1): the returned SQL contains
/// **only** provider placeholders (`$1..$n` on PostgreSQL) plus the literal
/// keyword `DEFAULT` — never a literal argument value. Argument values leave
/// this surface as typed `QueryParam`s bound to those placeholders through the
/// same `RunQuery` command path the query editor uses. An argument whose text
/// does not match its declared type is a typed validation failure
/// (`arguments == Err`) naming the parameter; it is never spliced, dropped, or
/// silently coerced into `NULL`.
pub(super) fn build_routine_invoke_sql(
    function: &UiFunctionSummary,
    values: &[String],
    nulls: &[bool],
    driver: &str,
) -> RoutineInvocation {
    let inputs = input_parameters(function);
    let style = placeholder_style_for_driver(driver);
    let mut rendered: Vec<String> = Vec::with_capacity(inputs.len());
    let mut arguments: Vec<QueryParam> = Vec::with_capacity(inputs.len());
    let mut invalid: Option<String> = None;
    let mut bound_with_cast = false;
    for (index, parameter) in inputs.iter().enumerate() {
        let label = argument_label(parameter, index);
        let is_null = nulls.get(index).copied().unwrap_or(false);
        let raw = values.get(index).map(String::as_str).unwrap_or("");
        if !is_null && raw.is_empty() && parameter.has_default {
            rendered.push("DEFAULT".to_owned());
            continue;
        }
        // Types whose sqlx bind OID differs from the declared type (and the
        // text-bound §9.3 families) ride behind an explicit `::type` cast so
        // function resolution matches the routine signature exactly.
        let family = type_family(&parameter.data_type);
        let cast = if requires_explicit_cast(&parameter.data_type) {
            match sanitize_cast_type(&parameter.data_type) {
                Ok(token) => Some(token),
                Err(reason) => {
                    invalid.get_or_insert(format!("parameter {label}: {reason}"));
                    None
                }
            }
        } else {
            None
        };
        let value = if is_null {
            Ok(QueryParam::Null)
        } else {
            bind_argument(&parameter.data_type, raw).map_err(|reason| format!("parameter {label}: {reason}"))
        };
        rendered.push(placeholder_token(style, arguments.len() + 1, cast.as_deref()));
        match value {
            Ok(value) => {
                bound_with_cast |= cast.is_some() && family == TypeFamily::Other;
                arguments.push(value);
            }
            Err(reason) => {
                invalid.get_or_insert(reason);
            }
        }
    }
    let qualified = result_grid_export::qualified_sql_name(&function.schema, &function.name);
    let arg_list = rendered.join(", ");
    let sql = if function.routine_type.eq_ignore_ascii_case("PROCEDURE") {
        format!("CALL {qualified}({arg_list});")
    } else {
        format!("SELECT * FROM {qualified}({arg_list});")
    };
    RoutineInvocation {
        sql,
        arguments: match invalid {
            Some(reason) => Err(reason),
            None => Ok(arguments),
        },
        bound_with_cast,
    }
}

fn argument_label(parameter: &UiRoutineParameter, index: usize) -> String {
    if parameter.name.is_empty() {
        format!("arg{}", index + 1)
    } else {
        parameter.name.clone()
    }
}

/// Placeholder spelling for bound argument `index` (1-based), with the §9.3
/// explicit cast when the type is not directly bindable. The cast is emitted
/// only for the PostgreSQL `$n` style — routines are the PostgreSQL surface
/// (SQLite reports every routine operation `Unsupported`, goal §9.2).
fn placeholder_token(style: PlaceholderStyle, index: usize, cast: Option<&str>) -> String {
    let mut token = render_placeholder(style, index);
    if let (Some(cast), PlaceholderStyle::NumberedDollar) = (cast, style) {
        token.push_str("::");
        token.push_str(cast);
    }
    token
}

/// §9.3 type families from a `format_type` string. Anything that is not a
/// directly bindable family (arrays, enums, composites, domains over them,
/// unknown) classifies as `Other` and is text-bound with an explicit cast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeFamily {
    Bool,
    Integer,
    Float,
    Decimal,
    Text,
    Uuid,
    Timestamp,
    Time,
    Interval,
    Inet,
    Json,
    Bytes,
    Other,
}

fn type_family(data_type: &str) -> TypeFamily {
    let normalized = data_type.trim().to_ascii_lowercase();
    // Arrays (`integer[]`, `_text` legacy display) are not directly bindable.
    if normalized.contains('[') || normalized.starts_with('_') {
        return TypeFamily::Other;
    }
    let base = normalized.split('(').next().unwrap_or("").trim();
    match base {
        "bool" | "boolean" => TypeFamily::Bool,
        "int2" | "smallint" | "serial2" | "integer" | "int" | "int4" | "serial4" | "oid" | "bigint" | "int8"
        | "serial8" | "bigserial" => TypeFamily::Integer,
        "real" | "float4" | "float" | "float8" | "double precision" => TypeFamily::Float,
        "numeric" | "decimal" => TypeFamily::Decimal,
        "text" | "varchar" | "character varying" | "character" | "char" | "bpchar" | "name" | "citext" => {
            TypeFamily::Text
        }
        "uuid" => TypeFamily::Uuid,
        "date" => TypeFamily::Timestamp,
        _ if base.starts_with("timestamp") => TypeFamily::Timestamp,
        _ if base.starts_with("time") => TypeFamily::Time,
        "interval" => TypeFamily::Interval,
        "inet" | "cidr" => TypeFamily::Inet,
        "json" | "jsonb" => TypeFamily::Json,
        "bytea" => TypeFamily::Bytes,
        _ => TypeFamily::Other,
    }
}

/// Whether the placeholder needs an explicit `::type` cast for resolution.
///
/// sqlx prepares with the *bind-side* OID of each value, and PostgreSQL does
/// not apply narrowing casts when resolving a routine signature: an `integer`
/// parameter bound through `QueryParam::Int64` arrives as `bigint`, which made
/// live calls fail with "function … does not exist" (verified on #252's PG 18
/// fixture). Casting the placeholder to the declared type pins resolution to
/// the signature; §12.1 stays intact because a cast carries a type name, never
/// an argument value. Types whose bind OID already equals the declaration need
/// no cast. Text-bound families (§9.3 arrays/enums/composites) always cast —
/// that cast *is* the binding mechanism there.
fn requires_explicit_cast(data_type: &str) -> bool {
    let normalized = data_type.trim().to_ascii_lowercase();
    let base = normalized.split('(').next().unwrap_or("").trim();
    let binds_exact = match type_family(data_type) {
        TypeFamily::Bool => matches!(base, "bool" | "boolean"),
        TypeFamily::Integer => matches!(base, "int8" | "bigint"),
        TypeFamily::Float => matches!(base, "float8" | "float" | "double precision"),
        TypeFamily::Decimal => true,
        TypeFamily::Text => matches!(base, "text" | "varchar" | "character varying"),
        TypeFamily::Uuid | TypeFamily::Interval | TypeFamily::Bytes => true,
        // Whether a TIME value carries an offset (and thus binds as `timetz`
        // instead of `time`) depends on the entered text, so both always cast.
        TypeFamily::Time => false,
        TypeFamily::Timestamp => matches!(base, "date" | "timestamp with time zone"),
        TypeFamily::Inet => base == "inet",
        TypeFamily::Json => base == "jsonb",
        TypeFamily::Other => false,
    };
    !binds_exact
}

/// Validate a catalog type name for use in a generated `::type` cast.
///
/// The token comes from server introspection, which a malicious database can
/// shape — so anything beyond a plain (possibly schema-qualified, possibly
/// parameterized, possibly array) type name is rejected instead of
/// interpolated. Commas only occur inside typmod parentheses (`numeric(10,2)`);
/// a hostile identifier containing them would arrive quoted from `format_type`,
/// and the quote character is rejected here. This is a guard, not a quoting
/// mechanism.
fn sanitize_cast_type(data_type: &str) -> Result<String, String> {
    let token = data_type.trim();
    if token.is_empty()
        || !token.chars().all(|ch| {
            ch.is_ascii_alphanumeric() || matches!(ch, '_' | ' ' | '.' | '(' | ')' | '[' | ']' | ',')
        })
    {
        return Err(format!("data type \"{data_type}\" cannot be safely cast in generated SQL"));
    }
    Ok(token.to_owned())
}

/// Map one form value to its typed `QueryParam` per the §9.3 family table.
/// Validation is local and cheap where the app already owns the parser
/// (bool/int/float/decimal/uuid/timestamp/time/json/bytea); families whose
/// authoritative parser lives in the provider adapter (interval, inet, and
/// the `Other` cast-bound text) bind their string through and surface a
/// typed `QueryFailed` at bind time — never a SQL-text coercion.
// cc-scan:allow COMPLEXITY — classifier ladder — one arm per §9.3 type family
fn bind_argument(data_type: &str, raw: &str) -> Result<QueryParam, String> {
    let trimmed = raw.trim();
    let mismatch = || -> String {
        if trimmed.is_empty() {
            format!("value for type {data_type} is empty — check NULL to pass a typed NULL")
        } else {
            format!("\"{raw}\" does not match the declared type {data_type}")
        }
    };
    match type_family(data_type) {
        TypeFamily::Bool => parse_bool(trimmed).map(QueryParam::Bool).ok_or_else(mismatch),
        TypeFamily::Integer => trimmed
            .parse::<i64>()
            .map(QueryParam::Int64)
            .map_err(|_| mismatch()),
        TypeFamily::Float => trimmed.parse::<f64>().map(QueryParam::Float64).map_err(|_| mismatch()),
        TypeFamily::Decimal => trimmed
            .parse::<bigdecimal::BigDecimal>()
            .map(|_| QueryParam::Decimal(trimmed.to_owned()))
            .map_err(|_| mismatch()),
        TypeFamily::Text => Ok(QueryParam::Text(raw.to_owned())),
        TypeFamily::Uuid => uuid::Uuid::parse_str(trimmed)
            .map(|_| QueryParam::Uuid(trimmed.to_owned()))
            .map_err(|_| mismatch()),
        TypeFamily::Timestamp => {
            if is_valid_timestamp(trimmed) {
                Ok(QueryParam::DateTime(trimmed.to_owned()))
            } else {
                Err(mismatch())
            }
        }
        TypeFamily::Time => {
            if is_valid_time(trimmed) {
                Ok(QueryParam::Time(trimmed.to_owned()))
            } else {
                Err(mismatch())
            }
        }
        TypeFamily::Interval => {
            if trimmed.is_empty() {
                Err(mismatch())
            } else {
                Ok(QueryParam::Interval(trimmed.to_owned()))
            }
        }
        TypeFamily::Inet => {
            if trimmed.is_empty() {
                Err(mismatch())
            } else {
                Ok(QueryParam::Inet(trimmed.to_owned()))
            }
        }
        TypeFamily::Json => serde_json::from_str::<serde_json::Value>(trimmed)
            .map(QueryParam::Json)
            .map_err(|_| mismatch()),
        TypeFamily::Bytes => parse_hex_bytes(trimmed).map(QueryParam::Bytes).ok_or_else(mismatch),
        // Not directly bindable: bound as text and cast in the generated SQL.
        // The text is the user's exact input — a quoted value stays a quoted
        // value, an empty field is the empty string, never a silent NULL.
        TypeFamily::Other => Ok(QueryParam::Text(raw.to_owned())),
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "t" | "1" | "yes" | "on" => Some(true),
        "false" | "f" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn is_valid_timestamp(value: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(value).is_ok()
        || chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f").is_ok()
        || chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f").is_ok()
        || chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
}

fn is_valid_time(value: &str) -> bool {
    chrono::NaiveTime::parse_from_str(value, "%H:%M:%S%.f").is_ok()
        || chrono::NaiveTime::parse_from_str(value, "%H:%M").is_ok()
        || chrono::NaiveTime::parse_from_str(value, "%H:%M:%S%.f%:z").is_ok()
}

/// `bytea` hex input, with or without the PostgreSQL `\x` prefix.
fn parse_hex_bytes(value: &str) -> Option<Vec<u8>> {
    let hex = value
        .strip_prefix("\\x")
        .or_else(|| value.strip_prefix("0x"))
        .unwrap_or(value);
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|pair| u8::from_str_radix(&hex[pair..pair + 2], 16).ok())
        .collect()
}

#[cfg(test)]
#[path = "tests/routine_workbench_surface_view_tests.rs"]
mod tests;

