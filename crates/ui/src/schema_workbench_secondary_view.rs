//! Schema Workbench dependency and documentation surfaces.
use super::schema_workbench::SchemaWorkbenchState;
use super::*;
use crate::components::{Button, ButtonSize, ButtonVariant};
use db_pro_core::domain::object_mutation::ObjectDependencyEdge;
use egui::RichText;
use lucide_icons::Icon;

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
    OpenSqlAsQuery(String),
    ClearHistory,
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

pub(super) fn draw_execution_history(
    context: &mut SchemaWorkbenchSecondaryContext<'_>,
    ui: &mut egui::Ui,
) -> Option<SchemaWorkbenchSecondaryAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label(RichText::new("DDL Execution History").font(font_subheading()).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if Button::new(context.theme)
                .icon(Icon::Trash2)
                .text("Clear history")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Ghost)
                .show(ui)
                .clicked()
            {
                action = Some(SchemaWorkbenchSecondaryAction::ClearHistory);
            }
        });
    });
    ui.add_space(SPACE_SM);

    if context.workbench.history.is_empty() {
        card_frame(context.theme).show(ui, |ui| {
            empty_state(
                ui,
                Icon::History,
                "No DDL execution history yet",
                "Mutations executed through the Schema Workbench will appear here with statements and safety rating.",
                context.theme,
            );
        });
        return action;
    }

    for entry in &context.workbench.history {
        egui::Frame::none()
            .fill(context.theme.surface_panel)
            .stroke(egui::Stroke::new(1.0, context.theme.border_subtle))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&entry.timestamp).small().color(context.theme.text_muted));
                    badge(
                        ui,
                        &entry.object_kind,
                        context.theme.surface_app,
                        context.theme.text_secondary,
                    );
                    ui.label(RichText::new(&entry.object_name).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if entry.success {
                            badge(
                                ui,
                                "SUCCESS",
                                context.theme.success_soft(),
                                context.theme.success,
                            );
                        } else {
                            badge(
                                ui,
                                "FAILED",
                                context.theme.danger_soft(),
                                context.theme.danger,
                            );
                        }
                        badge(
                            ui,
                            &entry.safety,
                            context.theme.accent_soft,
                            context.theme.accent,
                        );
                    });
                });
                ui.add_space(4.0);
                CodeBlock::new(&entry.sql, context.theme).language("sql").show(ui);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if Button::new(context.theme)
                        .icon(Icon::Code2)
                        .text("Open in SQL editor")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Secondary)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(SchemaWorkbenchSecondaryAction::OpenSqlAsQuery(entry.sql.clone()));
                    }
                    if Button::new(context.theme)
                        .icon(Icon::Copy)
                        .text("Copy SQL")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Ghost)
                        .show(ui)
                        .clicked()
                    {
                        ui.output_mut(|o| o.copied_text = entry.sql.clone());
                    }
                });
            });
        ui.add_space(SPACE_SM);
    }
    action
}
