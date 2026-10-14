//! Small workspace-files tabs with explicit presentation inputs.
use super::super::ide_workspace::{DependencyEdge, MigrationEntry};
use super::super::*;
use egui::RichText;
use lucide_icons::Icon;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FilesSecondaryTabAction {
    OpenFile(String),
}

pub(super) fn draw_migrations(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    migrations: &[MigrationEntry],
) -> Vec<FilesSecondaryTabAction> {
    if migrations.is_empty() {
        ui.label(
            RichText::new("No migration SQL detected under migrations/ paths.")
                .small()
                .color(theme.text_muted),
        );
        return Vec::new();
    }
    let mut actions = Vec::new();
    for entry in migrations {
        let label = format!("{} · {:?}", entry.version, entry.status);
        if sidebar_item(ui, Icon::FileCode2, &label, false, theme)
            .on_hover_text(&entry.relative_path)
            .clicked()
        {
            actions.push(FilesSecondaryTabAction::OpenFile(entry.relative_path.clone()));
        }
    }
    actions
}

pub(super) fn draw_graph(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    edges: &[DependencyEdge],
) -> Vec<FilesSecondaryTabAction> {
    if edges.is_empty() {
        ui.label(
            RichText::new("No FROM/JOIN object references found yet.")
                .small()
                .color(theme.text_muted),
        );
        return Vec::new();
    }
    for edge in edges.iter().take(60) {
        ui.label(
            RichText::new(format!("{} → {}", edge.from_file, edge.object_name))
                .small()
                .monospace()
                .color(theme.text_secondary),
        );
    }
    Vec::new()
}
