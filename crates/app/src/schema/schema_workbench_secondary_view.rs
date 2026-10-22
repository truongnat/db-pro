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
    /// SQL dialect used for preview/history highlighting (#336).
    pub(super) dialect: crate::editor::SqlDialect,
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
        crate::components::input::SearchInput::new(
            &mut context.workbench.dependency_filter,
            "from, relation, to…",
            context.theme,
        )
        .width(220.0)
        .show(ui);
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
    crate::components::Textarea::new(&mut context.workbench.docs_markdown, "Markdown notes...", context.theme)
        .min_rows(4)
        .show(ui);
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
            if context.workbench.history_clear_confirm {
                if Button::new(context.theme)
                    .text("Keep history")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Ghost)
                    .show(ui)
                    .clicked()
                {
                    context.workbench.history_clear_confirm = false;
                }
                if Button::new(context.theme)
                    .icon(Icon::Trash2)
                    .text("Confirm clear")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Destructive)
                    .tooltip("Delete the DDL execution history")
                    .show(ui)
                    .clicked()
                {
                    context.workbench.history_clear_confirm = false;
                    action = Some(SchemaWorkbenchSecondaryAction::ClearHistory);
                }
                ui.label(RichText::new("Clear all history?").small().color(context.theme.warning));
            } else if Button::new(context.theme)
                .icon(Icon::Trash2)
                .text("Clear history")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Ghost)
                .show(ui)
                .clicked()
            {
                context.workbench.history_clear_confirm = true;
            }
        });
    });
    ui.add_space(SPACE_SM);

    if context.workbench.history.is_empty() {
        Card::new(context.theme).frame().show(ui, |ui| {
            EmptyState::new(
                Icon::History,
                "No DDL execution history yet",
                "Mutations executed through the Schema Workbench will appear here with statements and safety rating.",
                context.theme,
            )
            .show(ui);
        });
        return action;
    }

    for entry in &context.workbench.history {
        Card::new(context.theme).frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&entry.timestamp).small().color(context.theme.text_muted));
                Badge::new(&entry.object_kind, context.theme)
                    .variant(BadgeVariant::Secondary)
                    .compact(true)
                    .show(ui);
                ui.label(RichText::new(&entry.object_name).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if entry.success {
                        Badge::new("SUCCESS", context.theme)
                            .tint(context.theme.success)
                            .compact(true)
                            .show(ui);
                    } else {
                        Badge::new("FAILED", context.theme)
                            .tint(context.theme.danger)
                            .compact(true)
                            .show(ui);
                    }
                    Badge::new(&entry.safety, context.theme)
                        .tint(context.theme.accent)
                        .compact(true)
                        .show(ui);
                });
            });
            ui.add_space(4.0);
            CodeBlock::new(&entry.sql, context.theme)
                .language("sql")
                .sql_dialect(context.dialect)
                .show(ui);
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
                    ui.output_mut(|o| o.commands.push(egui::OutputCommand::CopyText(entry.sql.clone())));
                }
            });
        });
        ui.add_space(SPACE_SM);
    }
    action
}
