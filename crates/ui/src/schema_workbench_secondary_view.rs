//! Schema Workbench dependency and documentation surfaces.
use super::schema_workbench::SchemaWorkbenchState;
use super::*;
use crate::components::{Button, ButtonSize, ButtonVariant};
use db_pro_core::domain::object_mutation::ObjectDependencyEdge;
use egui::RichText;

pub(super) struct SchemaWorkbenchSecondaryContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) workbench: &'a mut SchemaWorkbenchState,
    pub(super) edges: &'a [ObjectDependencyEdge],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SchemaWorkbenchSecondaryAction {
    GenerateMarkdown,
    GenerateHtml,
    OpenDocsAsQuery(String),
}

pub(super) fn draw_dependency_navigator(context: &mut SchemaWorkbenchSecondaryContext<'_>, ui: &mut egui::Ui) {
    ui.label(RichText::new("Object dependencies").strong());
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Filter");
        ui.text_edit_singleline(&mut context.workbench.dependency_filter);
    });
    ui.add_space(6.0);
    let filter = context.workbench.dependency_filter.to_ascii_lowercase();
    if context.edges.is_empty() {
        ui.label(RichText::new("No dependency edges in the loaded schema summary.").color(context.theme.text_muted));
        return;
    }
    egui::Grid::new("dep_grid").num_columns(3).striped(true).show(ui, |ui| {
        ui.label(RichText::new("From").strong());
        ui.label(RichText::new("Relation").strong());
        ui.label(RichText::new("To").strong());
        ui.end_row();
        for edge in context.edges {
            let line = format!(
                "{:?}.{}.{} {} {:?}.{}.{}",
                edge.from_kind,
                edge.from_schema.as_deref().unwrap_or("-"),
                edge.from_name,
                edge.relation,
                edge.to_kind,
                edge.to_schema.as_deref().unwrap_or("-"),
                edge.to_name
            );
            if !filter.is_empty() && !line.to_ascii_lowercase().contains(&filter) {
                continue;
            }
            ui.label(format!(
                "{}.{}",
                edge.from_schema.as_deref().unwrap_or("-"),
                edge.from_name
            ));
            ui.label(&edge.relation);
            ui.label(format!("{}.{}", edge.to_schema.as_deref().unwrap_or("-"), edge.to_name));
            ui.end_row();
        }
    });
}

pub(super) fn draw_docs_export(
    context: &mut SchemaWorkbenchSecondaryContext<'_>,
    ui: &mut egui::Ui,
) -> Option<SchemaWorkbenchSecondaryAction> {
    let mut action = None;
    ui.label(RichText::new("Schema documentation export").strong());
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if Button::new(context.theme)
            .text("Generate Markdown")
            .size(ButtonSize::Sm)
            .variant(ButtonVariant::Secondary)
            .show(ui)
            .clicked()
        {
            action = Some(SchemaWorkbenchSecondaryAction::GenerateMarkdown);
        }
        if Button::new(context.theme)
            .text("Generate HTML")
            .size(ButtonSize::Sm)
            .variant(ButtonVariant::Secondary)
            .show(ui)
            .clicked()
        {
            action = Some(SchemaWorkbenchSecondaryAction::GenerateHtml);
        }
        if Button::new(context.theme)
            .text("Open as query")
            .size(ButtonSize::Sm)
            .variant(ButtonVariant::Ghost)
            .show(ui)
            .clicked()
            && !context.workbench.docs_markdown.is_empty()
        {
            action = Some(SchemaWorkbenchSecondaryAction::OpenDocsAsQuery(
                context.workbench.docs_markdown.clone(),
            ));
        }
    });
    ui.add_space(6.0);
    ui.add(
        egui::TextEdit::multiline(&mut context.workbench.docs_markdown)
            .desired_rows(18)
            .desired_width(f32::INFINITY)
            .code_editor(),
    );
    action
}
