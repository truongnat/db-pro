//! Query bind-parameter panel rendering and in-memory value editing.
use super::*;
use egui::RichText;

pub(super) struct QueryParametersContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) session: &'a mut QuerySessionState,
    pub(super) supports_parameters: bool,
}

pub(super) fn draw_parameters_panel(context: &mut QueryParametersContext<'_>, ui: &mut egui::Ui) {
    let params = crate::query::discover_sql_parameters(context.session.active_text());
    if params.is_empty() {
        return;
    }

    let parameter_count = params.len();
    let document_index = context.session.active_document_index;
    ui.add_space(SPACE_XS);
    ui.horizontal(|ui| {
        ui.colored_label(context.theme.accent, format!("Parameters · {parameter_count}"));
        if !context.supports_parameters {
            ui.label(
                RichText::new("provider does not advertise bindings yet")
                    .small()
                    .color(context.theme.warning),
            );
        }
    });

    for parameter in params {
        draw_parameter_row(context.theme, context.session, document_index, &parameter, ui);
    }

    ui.label(
        RichText::new("Values stay in-memory for this document; secret values are never persisted with drafts.")
            .small()
            .color(context.theme.text_muted),
    );
}

fn draw_parameter_row(
    theme: DbProTheme,
    session: &mut QuerySessionState,
    document_index: usize,
    parameter: &crate::query::DiscoveredParameter,
    ui: &mut egui::Ui,
) {
    let mut value = session
        .documents
        .get(document_index)
        .and_then(|document| document.parameter_values.get(&parameter.name).cloned())
        .unwrap_or_default();
    let mut is_secret = session
        .documents
        .get(document_index)
        .is_some_and(|document| document.parameter_secrets.contains(&parameter.name));
    ui.horizontal(|ui| {
        let kind = match parameter.kind {
            crate::query::ParameterKind::Numbered => "numbered",
            crate::query::ParameterKind::Named => "named",
            crate::query::ParameterKind::Positional => "positional",
        };
        ui.label(
            RichText::new(format!("{} ({kind})", parameter.name))
                .small()
                .color(theme.text_secondary),
        );
        let edit = if is_secret {
            egui::TextEdit::singleline(&mut value).password(true)
        } else {
            egui::TextEdit::singleline(&mut value)
        };
        ui.add(edit.desired_width(180.0));
        ui.checkbox(&mut is_secret, "secret");
    });
    if let Some(document) = session.documents.get_mut(document_index) {
        document.parameter_values.insert(parameter.name.clone(), value);
        if is_secret {
            document.parameter_secrets.insert(parameter.name.clone());
        } else {
            document.parameter_secrets.remove(&parameter.name);
        }
    }
}
