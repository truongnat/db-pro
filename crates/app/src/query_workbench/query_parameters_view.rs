//! Query bind-parameter panel rendering and in-memory value editing.
use super::*;
use egui::RichText;

pub(super) struct QueryParametersContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) session: &'a mut QuerySessionState,
    pub(super) supports_parameters: bool,
    /// Active connection driver. Discovery follows it (`?` is not a placeholder on PostgreSQL or SQL Server).
    pub(super) driver: &'a str,
}

pub(super) fn draw_parameters_panel(context: &mut QueryParametersContext<'_>, ui: &mut egui::Ui) {
    let params = crate::query::discover_sql_parameters_for_driver(context.session.active_text(), context.driver);
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
    let mut value_changed = false;
    let mut secret_changed = false;
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
        let response = crate::components::input::Input::new(
            &mut value,
            format!("Value for {}", parameter.name),
            theme,
        )
        .width(180.0)
        .password(is_secret)
        .show(ui);
        value_changed = response.changed();
        secret_changed = Checkbox::new(&mut is_secret, "secret", theme).show(ui).changed();
    });
    if let Some(document) = session.documents.get_mut(document_index) {
        // An untouched field must stay absent. A present key is filled, so writing
        // "" here would bind an empty string instead of failing closed.
        if value_changed {
            document.parameter_values.insert(parameter.name.clone(), value);
        }
        if secret_changed {
            if is_secret {
                document.parameter_secrets.insert(parameter.name.clone());
            } else {
                document.parameter_secrets.remove(&parameter.name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::QueryDocument;

    #[test]
    fn opening_the_panel_does_not_store_an_empty_parameter_value() {
        let mut session = QuerySessionState::default();
        session.add_document(QueryDocument::new("doc-1", "Query 1", "SELECT :id"));
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let _ = crate::test_frame::frame(&ctx, Default::default(), |ui| {
            let mut context = QueryParametersContext {
                theme: DbProTheme::default(),
                session: &mut session,
                supports_parameters: true,
                driver: "PostgreSQL",
            };
            draw_parameters_panel(&mut context, ui);
        });
        let document = session.active_document().expect("document");
        assert!(
            document.parameter_values.is_empty(),
            "opening the panel must not insert an empty value, got {:?}",
            document.parameter_values
        );
        assert!(document.parameter_secrets.is_empty());
    }
}
