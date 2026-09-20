use super::files_agent_context_view::{ActiveQueryContext, FilesAgentContextAction, FilesAgentContextView};
use super::files_tree_view::{FilesTreeAction, FilesTreeContext};
use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use egui::{Align, Layout, RichText};
use lucide_icons::Icon;

impl DbProApp {
    /// Quiet agent-context strip: icon actions instead of a wrapped button soup.
    pub(super) fn draw_files_agent_context_strip(&mut self, ui: &mut egui::Ui) {
        let active_document = self.query.session.active_document().map(|document| ActiveQueryContext {
            path: document.file_path.clone(),
            title: document.title.clone(),
        });
        let actions = FilesAgentContextView {
            theme: self.theme,
            context_items: &self.workspace.files.workspace_context_items,
            active_document: active_document.as_ref(),
            selected_text: &self.query.session.selected_text,
            selected_table: self.schema.explorer.selected_table.as_deref(),
        }
        .draw(ui);
        for action in actions {
            self.apply_files_agent_context_action(action);
        }
    }

    fn apply_files_agent_context_action(&mut self, action: FilesAgentContextAction) {
        match action {
            FilesAgentContextAction::Clear => self.workspace.files.clear_context_items(),
            FilesAgentContextAction::RefreshSchemaDrift => {
                let names = self
                    .schema
                    .explorer
                    .schema
                    .table_details
                    .iter()
                    .map(|table| format!("{}.{}", table.schema, table.name))
                    .collect::<Vec<_>>();
                self.workspace
                    .files
                    .refresh_schema_drift_watch(&names, &mut self.feedback);
            }
            FilesAgentContextAction::ExportSchemaSnapshot => self
                .workspace
                .files
                .export_live_schema_snapshot(&self.schema.explorer.schema, &mut self.feedback),
            FilesAgentContextAction::ToggleSplitEditor => self.toggle_split_editor(),
            FilesAgentContextAction::AddItem(item) => self.workspace.files.add_context_item(item),
            FilesAgentContextAction::RemoveItem(item) => {
                self.workspace
                    .files
                    .workspace_context_items
                    .retain(|existing| existing != &item);
            }
        }
    }

    pub(super) fn draw_files_tree_tab(&mut self, ui: &mut egui::Ui) {
        let active_file_path = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .and_then(|doc| doc.file_path.clone())
            .map(|path| path.to_owned());
        let tree = self.workspace.files.ide_workspace.tree().to_vec();
        let actions = FilesTreeContext {
            theme: self.theme,
            file_count: self.workspace.files.ide_workspace.index().len(),
            active_file_path: active_file_path.as_deref(),
            tree: &tree,
            expanded: &self.workspace.files.ide_workspace.expanded,
        }
        .draw(ui);
        for action in actions {
            self.apply_files_tree_action(action);
        }
    }

    fn apply_files_tree_action(&mut self, action: FilesTreeAction) {
        match action {
            FilesTreeAction::NewSql => {
                match self
                    .workspace
                    .files
                    .ide_workspace
                    .create_file("", "untitled.sql", "-- new query\nSELECT 1;\n")
                {
                    Ok(path) => {
                        let relative = path
                            .file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "untitled.sql".to_owned());
                        self.open_workspace_sql_file(relative);
                    }
                    Err(error) => self.feedback.runtime_message = error,
                }
            }
            FilesTreeAction::NewFolder => {
                if let Err(error) = self.workspace.files.ide_workspace.create_folder("", "new-folder") {
                    self.feedback.runtime_message = error;
                }
            }
            FilesTreeAction::ToggleDirectory(path) => {
                if self.workspace.files.ide_workspace.expanded.contains(&path) {
                    self.workspace.files.ide_workspace.expanded.remove(&path);
                } else {
                    self.workspace.files.ide_workspace.expanded.insert(path);
                }
            }
            FilesTreeAction::CreateSql(path) => {
                let _ = self
                    .workspace
                    .files
                    .ide_workspace
                    .create_file(&path, "query.sql", "-- new query\nSELECT 1;\n");
            }
            FilesTreeAction::Delete(path) => {
                let _ = self.workspace.files.ide_workspace.delete_path(&path);
            }
            FilesTreeAction::OpenFile(path) => self.open_workspace_sql_file(path),
            FilesTreeAction::AddContext(path) => self.workspace.files.add_context_item(path),
            FilesTreeAction::FindReferences(stem) => {
                self.workspace.files.workspace_search_query = stem;
                self.workspace.files_panel_tab = FilesPanelTab::Search;
                self.workspace.files.run_search(&mut self.feedback);
            }
        }
    }

    pub(super) fn draw_files_search_tab(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::TextEdit::singleline(&mut self.workspace.files.workspace_search_query)
                .hint_text("Find in files…")
                .desired_width(ui.available_width()),
        );
        ui.add_space(4.0);
        ui.add(
            egui::TextEdit::singleline(&mut self.workspace.files.workspace_replace_query)
                .hint_text("Replace with…")
                .desired_width(ui.available_width()),
        );
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("Find")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.workspace.files.run_search(&mut self.feedback);
            }
            if Button::new(self.theme)
                .text("Preview")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.workspace.files.preview_replace(&mut self.feedback);
            }
            if Button::new(self.theme)
                .text("Replace all")
                .variant(ButtonVariant::Destructive)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.workspace.files.apply_replace(&mut self.feedback);
            }
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.workspace.files.workspace_refactor_from)
                    .hint_text("Rename from")
                    .desired_width(90.0),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.workspace.files.workspace_refactor_to)
                    .hint_text("to")
                    .desired_width(90.0),
            );
            if Button::new(self.theme)
                .text("Refactor")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.workspace.files.apply_refactor(&mut self.feedback);
            }
        });
        if !self.workspace.files.workspace_replace_previews.is_empty() {
            ui.add_space(6.0);
            section_label(ui, "REPLACE PREVIEW", self.theme);
            for preview in self
                .workspace
                .files
                .workspace_replace_previews
                .clone()
                .into_iter()
                .take(30)
            {
                ui.label(
                    RichText::new(format!(
                        "{}::{} · {} hits",
                        preview.root_id, preview.relative_path, preview.replacements
                    ))
                    .small()
                    .color(self.theme.text_secondary),
                );
            }
        }
        if !self.workspace.files.workspace_search_hits.is_empty() {
            ui.add_space(6.0);
            section_label(ui, "SEARCH RESULTS", self.theme);
            ui.add_space(4.0);
            let hits = self.workspace.files.workspace_search_hits.clone();
            for hit in hits.into_iter().take(40) {
                let label = format!("{}:{}", hit.relative_path, hit.line);
                if sidebar_item(ui, Icon::Search, &label, false, self.theme)
                    .on_hover_text(&hit.preview)
                    .clicked()
                    && hit.relative_path.ends_with(".sql")
                {
                    self.open_workspace_sql_file(format!("{}::{}", hit.root_id, hit.relative_path));
                }
            }
        }
    }

    pub(super) fn draw_files_migrations_tab(&mut self, ui: &mut egui::Ui) {
        let migrations = self.workspace.files.ide_workspace.detect_migrations();
        if migrations.is_empty() {
            ui.label(
                RichText::new("No migration SQL detected under migrations/ paths.")
                    .small()
                    .color(self.theme.text_muted),
            );
            return;
        }
        for entry in migrations {
            let label = format!("{} · {:?}", entry.version, entry.status);
            if sidebar_item(ui, Icon::FileCode2, &label, false, self.theme)
                .on_hover_text(&entry.relative_path)
                .clicked()
            {
                self.open_workspace_sql_file(entry.relative_path);
            }
        }
    }

    pub(super) fn draw_files_tasks_tab(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::TextEdit::singleline(&mut self.workspace.files.workspace_task_command)
                .hint_text("shell command in workspace root…")
                .desired_width(ui.available_width()),
        );
        ui.add_space(4.0);
        if Button::new(self.theme)
            .text("Run")
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            self.workspace.files.run_task(&mut self.feedback);
        }
        if let Some(result) = self.workspace.files.ide_workspace.last_task.clone() {
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!(
                    "$ {} · exit {:?} · {}ms",
                    result.command, result.exit_code, result.duration_ms
                ))
                .small()
                .monospace()
                .color(self.theme.text_secondary),
            );
            if !result.stdout.is_empty() {
                ui.label(
                    RichText::new(result.stdout.chars().take(800).collect::<String>())
                        .small()
                        .monospace()
                        .color(self.theme.text_muted),
                );
            }
            if !result.stderr.is_empty() {
                ui.label(
                    RichText::new(result.stderr.chars().take(400).collect::<String>())
                        .small()
                        .monospace()
                        .color(self.theme.danger),
                );
            }
        }
        ui.add_space(8.0);
        section_label(ui, "BENCHMARK (local timing)", self.theme);
        if Button::new(self.theme)
            .text("Run sample suite")
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            let cases = vec![
                ide_workspace::BenchmarkCase {
                    name: "select-1".to_owned(),
                    sql: "SELECT 1".to_owned(),
                },
                ide_workspace::BenchmarkCase {
                    name: "select-now".to_owned(),
                    sql: "SELECT CURRENT_TIMESTAMP".to_owned(),
                },
            ];
            let results = ide_workspace::measure_local_benchmark(&cases, 5);
            self.feedback.runtime_message = results
                .into_iter()
                .map(|result| format!("{}={}ms", result.name, result.avg_ms))
                .collect::<Vec<_>>()
                .join(", ");
        }
    }

    pub(super) fn draw_files_graph_tab(&mut self, ui: &mut egui::Ui) {
        let edges = self.workspace.files.ide_workspace.dependency_edges();
        if edges.is_empty() {
            ui.label(
                RichText::new("No FROM/JOIN object references found yet.")
                    .small()
                    .color(self.theme.text_muted),
            );
            return;
        }
        for edge in edges.into_iter().take(60) {
            ui.label(
                RichText::new(format!("{} → {}", edge.from_file, edge.object_name))
                    .small()
                    .monospace()
                    .color(self.theme.text_secondary),
            );
        }
    }

    pub(super) fn draw_files_git_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            section_label(ui, "GIT", self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if Button::new(self.theme)
                    .icon(Icon::RefreshCw)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Refresh git status")
                    .show(ui)
                    .clicked()
                {
                    self.workspace.files.refresh_git_status(&mut self.feedback);
                }
            });
        });
        ui.add_space(4.0);
        let documents: Vec<(String, String, bool)> = self
            .query
            .session
            .documents
            .iter()
            .filter_map(|doc| {
                let path = doc.file_path.clone()?;
                Some((path, doc.text().to_owned(), doc.dirty))
            })
            .collect();
        self.workspace.files.check_external_file_changes(&documents);
        if let Some(path) = self.workspace.files.workspace_external_change.clone() {
            ui.colored_label(
                self.theme.warning,
                format!("Disk changed for {path} — unsaved editor buffer was kept."),
            );
            ui.horizontal(|ui| {
                if Button::new(self.theme)
                    .text("Reload from disk")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.reload_workspace_file_from_disk(&path);
                }
                if Button::new(self.theme)
                    .icon(Icon::X)
                    .text("Dismiss")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.workspace.files.workspace_external_change = None;
                }
            });
            ui.add_space(6.0);
        }
        if let Some(error) = self.workspace.files.git_last_error.clone() {
            ui.colored_label(self.theme.danger, error);
        }
        let Some(status) = self.workspace.files.git_status.clone() else {
            ui.label(
                RichText::new("Refresh to probe Git for the active workspace root.")
                    .small()
                    .color(self.theme.text_muted),
            );
            return;
        };
        if !status.available {
            ui.label(RichText::new(&status.message).small().color(self.theme.text_muted));
            return;
        }
        ui.label(
            RichText::new(format!(
                "branch {} · {}",
                status.branch.as_deref().unwrap_or("?"),
                status.message
            ))
            .small()
            .color(self.theme.text_secondary),
        );
        ui.add_space(6.0);
        ui.add(
            egui::TextEdit::singleline(&mut self.workspace.files.git_commit_message)
                .hint_text("commit message (explicit only — never auto)")
                .desired_width(ui.available_width()),
        );
        if Button::new(self.theme)
            .text("Commit staged")
            .variant(ButtonVariant::Destructive)
            .size(ButtonSize::Sm)
            .show(ui)
            .clicked()
        {
            self.workspace.files.commit_git_staged(&mut self.feedback);
        }
        ui.add_space(8.0);
        if status.entries.is_empty() {
            ui.label(
                RichText::new("Working tree clean.")
                    .small()
                    .color(self.theme.text_muted),
            );
        }
        for entry in status.entries.iter().take(80) {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("[{}] {}", entry.code.trim(), entry.path))
                        .small()
                        .monospace()
                        .color(self.theme.text_primary),
                );
            });
            ui.horizontal(|ui| {
                if Button::new(self.theme)
                    .icon(Icon::Plus)
                    .text("Stage")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.workspace.files.stage_git_path(&entry.path, &mut self.feedback);
                }
                if Button::new(self.theme)
                    .icon(Icon::Minus)
                    .text("Unstage")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.workspace.files.unstage_git_path(&entry.path, &mut self.feedback);
                }
                if Button::new(self.theme)
                    .icon(Icon::GitCompare)
                    .text("Diff")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.workspace.files.diff_git_path(&entry.path);
                }
                if Button::new(self.theme)
                    .icon(Icon::FileCode2)
                    .text("Open")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.open_workspace_sql_file(entry.path.clone());
                }
            });
            ui.add_space(4.0);
        }
        if let Some(diff) = self.workspace.files.git_diff.clone() {
            ui.add_space(8.0);
            section_label(ui, format!("DIFF · {} vs {}", diff.path, diff.against), self.theme);
            ui.add_space(4.0);
            egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                ui.label(
                    RichText::new(diff.text.chars().take(8_000).collect::<String>())
                        .small()
                        .monospace()
                        .color(self.theme.text_muted),
                );
            });
        }
    }
}
