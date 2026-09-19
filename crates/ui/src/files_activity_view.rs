//! Files / workspace activity sidebar: tree, search, migrations, tasks, graph.
use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::segmented_control;
use egui::{vec2, Align, Layout, RichText};
use lucide_icons::Icon;

impl DbProApp {
    pub(super) fn draw_files_activity(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            section_label(ui, "WORKSPACE", self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if Button::new(self.theme)
                    .icon(Icon::FolderOpen)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Add / open folder")
                    .show(ui)
                    .clicked()
                {
                    self.request_open_workspace_folder();
                }
                if !self.workspace.files.ide_workspace.roots.is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::RefreshCw)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Refresh tree")
                        .show(ui)
                        .clicked()
                {
                    self.refresh_workspace_folder();
                }
            });
        });
        ui.add_space(6.0);

        if self.workspace.files.ide_workspace.roots.is_empty() {
            ui.label(
                RichText::new("Open a folder to browse SQL, migrations, and project files.")
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.add_space(8.0);
            if Button::new(self.theme)
                .icon(Icon::FolderOpen)
                .text("Open Folder")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.request_open_workspace_folder();
            }
            if !self.workspace.files.ide_workspace.recent_roots.is_empty() {
                ui.add_space(12.0);
                section_label(ui, "RECENT", self.theme);
                ui.add_space(6.0);
                let recent = self.workspace.files.ide_workspace.recent_roots.clone();
                for path in recent.into_iter().take(8) {
                    let label = path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.display().to_string());
                    if sidebar_item(ui, Icon::Folder, &label, false, self.theme)
                        .on_hover_text(path.display().to_string())
                        .clicked()
                    {
                        self.open_workspace_folder(path);
                    }
                }
            }
            return;
        }

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
            for (index, root) in self.workspace.files.ide_workspace.roots.clone().into_iter().enumerate() {
                let label = root
                    .path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| root.path.display().to_string());
                let selected = self.workspace.files.ide_workspace.active_root == index;
                if ui.selectable_label(selected, label).clicked() {
                    self.workspace.files.ide_workspace.active_root = index;
                }
            }
            if Button::new(self.theme)
                .text("+ Root")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.request_open_workspace_folder();
            }
            if Button::new(self.theme)
                .text("Remove")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.workspace.files.ide_workspace.remove_active_root();
            }
        });
        ui.add_space(4.0);
        if let Some(path) = self.workspace.files.ide_workspace.primary_path() {
            ui.label(
                RichText::new(path.display().to_string())
                    .small()
                    .monospace()
                    .color(self.theme.text_secondary),
            );
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let trusted = self.workspace.files.ide_workspace.is_trusted();
            if ui.selectable_label(trusted, "Trusted").clicked() {
                self.workspace.files.ide_workspace.set_trusted(true);
            }
            if ui.selectable_label(!trusted, "Untrusted").clicked() {
                self.workspace.files.ide_workspace.set_trusted(false);
            }
            if Button::new(self.theme)
                .icon(Icon::X)
                .text("Close")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.close_workspace_folder();
            }
        });
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for (index, env) in self
                .workspace
                .files
                .ide_workspace
                .environments
                .clone()
                .into_iter()
                .enumerate()
            {
                let selected = self.workspace.files.ide_workspace.active_environment == index;
                if ui.selectable_label(selected, &env.name).clicked() {
                    self.workspace.files.ide_workspace.set_active_environment(index);
                    self.feedback.runtime_message = format!("Environment → {}", env.name);
                }
            }
        });
        if let Some(drift) = self.workspace.files.ide_workspace.schema_drift_message.clone() {
            ui.label(RichText::new(drift).small().color(self.theme.warning));
        }
        if let Some(error) = self.workspace.files.ide_workspace.last_error.clone() {
            ui.label(RichText::new(error).small().color(self.theme.danger));
        }

        ui.add_space(6.0);
        let tab_labels = ["Tree", "Search", "Migrations", "Tasks", "Graph", "Git"];
        let selected_tab = match self.workspace.files_panel_tab {
            FilesPanelTab::Tree => 0,
            FilesPanelTab::Search => 1,
            FilesPanelTab::Migrations => 2,
            FilesPanelTab::Tasks => 3,
            FilesPanelTab::Graph => 4,
            FilesPanelTab::Git => 5,
        };
        if let Some(next) = segmented_control(ui, &tab_labels, selected_tab, false, self.theme) {
            self.workspace.files_panel_tab = match next {
                1 => FilesPanelTab::Search,
                2 => FilesPanelTab::Migrations,
                3 => FilesPanelTab::Tasks,
                4 => FilesPanelTab::Graph,
                5 => FilesPanelTab::Git,
                _ => FilesPanelTab::Tree,
            };
            if self.workspace.files_panel_tab == FilesPanelTab::Git {
                self.refresh_git_status();
            }
        }
        ui.add_space(6.0);

        match self.workspace.files_panel_tab {
            FilesPanelTab::Tree => self.draw_files_tree_tab(ui),
            FilesPanelTab::Search => self.draw_files_search_tab(ui),
            FilesPanelTab::Migrations => self.draw_files_migrations_tab(ui),
            FilesPanelTab::Tasks => self.draw_files_tasks_tab(ui),
            FilesPanelTab::Graph => self.draw_files_graph_tab(ui),
            FilesPanelTab::Git => self.draw_files_git_tab(ui),
        }

        ui.add_space(10.0);
        self.draw_files_agent_context_strip(ui);
    }

    /// Quiet agent-context strip: icon actions instead of a wrapped button soup.
    fn draw_files_agent_context_strip(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            section_label(ui, "AGENT CONTEXT", self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if Button::new(self.theme)
                    .icon(Icon::Trash2)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Clear context")
                    .show(ui)
                    .clicked()
                {
                    self.clear_workspace_context_items();
                }
                if Button::new(self.theme)
                    .icon(Icon::GitCompare)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Check schema drift")
                    .show(ui)
                    .clicked()
                {
                    self.refresh_schema_drift_watch();
                }
                if Button::new(self.theme)
                    .icon(Icon::Camera)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Export schema snapshot")
                    .show(ui)
                    .clicked()
                {
                    self.export_live_schema_snapshot();
                }
                if Button::new(self.theme)
                    .icon(Icon::Columns2)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Toggle split editor")
                    .show(ui)
                    .clicked()
                {
                    self.toggle_split_editor();
                }
            });
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("+ File")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .tooltip("Add active file to agent context")
                .show(ui)
                .clicked()
            {
                if let Some(doc) = self
                    .query_session_state
                    .documents
                    .get(self.query_session_state.active_document_index)
                {
                    if let Some(path) = doc.file_path.clone() {
                        self.add_workspace_context_item(path);
                    } else {
                        self.add_workspace_context_item(format!("query:{}", doc.title));
                    }
                }
            }
            if Button::new(self.theme)
                .text("+ Selection")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .tooltip("Add current SQL selection")
                .show(ui)
                .clicked()
            {
                let selected = self.query_session_state.selected_text.clone();
                if !selected.trim().is_empty() {
                    self.add_workspace_context_item(format!(
                        "selection:{}",
                        selected.chars().take(80).collect::<String>()
                    ));
                }
            }
            if Button::new(self.theme)
                .text("+ Table")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .tooltip("Add selected table")
                .show(ui)
                .clicked()
            {
                if let Some(table) = self.schema_explorer.selected_table.clone() {
                    self.add_workspace_context_item(format!("table:{table}"));
                }
            }
        });
        if self.workspace.files.workspace_context_items.is_empty() {
            ui.label(
                RichText::new("No context chips yet — add a file, selection, or table.")
                    .small()
                    .color(self.theme.text_muted),
            );
        } else {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                for item in self.workspace.files.workspace_context_items.clone() {
                    let short = if item.len() > 28 {
                        format!("{}…", &item.chars().take(27).collect::<String>())
                    } else {
                        item.clone()
                    };
                    if tag_chip(ui, &short, true, self.theme) {
                        self.workspace
                            .files
                            .workspace_context_items
                            .retain(|existing| existing != &item);
                    }
                }
            });
        }
    }

    fn draw_files_tree_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("New SQL")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
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
            if Button::new(self.theme)
                .text("New folder")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                if let Err(error) = self.workspace.files.ide_workspace.create_folder("", "new-folder") {
                    self.feedback.runtime_message = error;
                }
            }
        });
        ui.add_space(6.0);
        // Breadcrumb for active file-backed document (#266).
        if let Some(path) = self
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .and_then(|doc| doc.file_path.clone())
        {
            ui.label(RichText::new(path).small().monospace().color(self.theme.text_muted));
            ui.add_space(4.0);
        }
        section_label(
            ui,
            format!("FILES · {}", self.workspace.files.ide_workspace.index().len()),
            self.theme,
        );
        ui.add_space(6.0);
        let tree = self.workspace.files.ide_workspace.tree().to_vec();
        for node in &tree {
            self.draw_workspace_tree_node(ui, node, 0);
        }
    }

    fn draw_files_search_tab(&mut self, ui: &mut egui::Ui) {
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
                self.run_workspace_search();
            }
            if Button::new(self.theme)
                .text("Preview")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.preview_workspace_replace();
            }
            if Button::new(self.theme)
                .text("Replace all")
                .variant(ButtonVariant::Destructive)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.apply_workspace_replace();
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
                self.apply_workspace_refactor();
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

    fn draw_files_migrations_tab(&mut self, ui: &mut egui::Ui) {
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

    fn draw_files_tasks_tab(&mut self, ui: &mut egui::Ui) {
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
            self.run_workspace_task();
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

    fn draw_files_graph_tab(&mut self, ui: &mut egui::Ui) {
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

    fn draw_files_git_tab(&mut self, ui: &mut egui::Ui) {
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
                    self.refresh_git_status();
                }
            });
        });
        ui.add_space(4.0);
        self.check_external_file_changes();
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
            self.commit_git_staged();
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
                    self.stage_git_path(&entry.path);
                }
                if Button::new(self.theme)
                    .icon(Icon::Minus)
                    .text("Unstage")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.unstage_git_path(&entry.path);
                }
                if Button::new(self.theme)
                    .icon(Icon::GitCompare)
                    .text("Diff")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.diff_git_path(&entry.path);
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

    fn draw_workspace_tree_node(&mut self, ui: &mut egui::Ui, node: &ide_workspace::WorkspaceFileNode, depth: usize) {
        let indent = depth as f32 * 12.0;
        ui.horizontal(|ui| {
            ui.add_space(indent);
            if node.is_dir {
                let expanded = self
                    .workspace
                    .files
                    .ide_workspace
                    .expanded
                    .contains(&node.relative_path);
                let chevron = if expanded {
                    Icon::ChevronDown
                } else {
                    Icon::ChevronRight
                };
                let response = sidebar_item(ui, chevron, &node.name, false, self.theme);
                let mut create_sql = false;
                let mut delete_node = false;
                context_action_menu(ui, &response, self.theme, |ui, close_menu| {
                    if ctx_menu_item(
                        ui,
                        Some(Icon::FileCode2),
                        "New SQL here",
                        None,
                        self.theme.text_primary,
                        self.theme,
                    )
                    .clicked()
                    {
                        create_sql = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(
                        ui,
                        Some(Icon::Trash2),
                        "Delete folder",
                        None,
                        self.theme.danger,
                        self.theme,
                    )
                    .clicked()
                    {
                        delete_node = true;
                        *close_menu = true;
                    }
                });
                if response.clicked() {
                    if expanded {
                        self.workspace.files.ide_workspace.expanded.remove(&node.relative_path);
                    } else {
                        self.workspace
                            .files
                            .ide_workspace
                            .expanded
                            .insert(node.relative_path.clone());
                    }
                }
                if create_sql {
                    let _ = self.workspace.files.ide_workspace.create_file(
                        &node.relative_path,
                        "query.sql",
                        "-- new query\nSELECT 1;\n",
                    );
                }
                if delete_node {
                    let _ = self.workspace.files.ide_workspace.delete_path(&node.relative_path);
                }
            } else {
                let icon = if node.name.ends_with(".sql") {
                    Icon::FileCode2
                } else {
                    Icon::FileText
                };
                let selected = self
                    .query_session_state
                    .documents
                    .get(self.query_session_state.active_document_index)
                    .and_then(|doc| doc.file_path.as_ref())
                    .is_some_and(|path| path == &node.absolute_path.to_string_lossy());
                let response =
                    sidebar_item(ui, icon, &node.name, selected, self.theme).on_hover_text(&node.relative_path);
                let mut open_file = false;
                let mut add_context = false;
                let mut delete_node = false;
                let mut find_refs = false;
                context_action_menu(ui, &response, self.theme, |ui, close_menu| {
                    if ctx_menu_item(
                        ui,
                        Some(Icon::FileCode2),
                        "Open",
                        None,
                        self.theme.text_primary,
                        self.theme,
                    )
                    .clicked()
                    {
                        open_file = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(
                        ui,
                        Some(Icon::Plus),
                        "Add to Agent context",
                        None,
                        self.theme.text_primary,
                        self.theme,
                    )
                    .clicked()
                    {
                        add_context = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(
                        ui,
                        Some(Icon::Search),
                        "Find references",
                        None,
                        self.theme.text_primary,
                        self.theme,
                    )
                    .clicked()
                    {
                        find_refs = true;
                        *close_menu = true;
                    }
                    if ctx_menu_item(ui, Some(Icon::Trash2), "Delete", None, self.theme.danger, self.theme).clicked() {
                        delete_node = true;
                        *close_menu = true;
                    }
                });
                if response.clicked() && node.name.ends_with(".sql") {
                    open_file = true;
                }
                if open_file && node.name.ends_with(".sql") {
                    self.open_workspace_sql_file(node.relative_path.clone());
                }
                if add_context {
                    self.add_workspace_context_item(node.absolute_path.to_string_lossy().into_owned());
                }
                if find_refs {
                    let stem = node.name.trim_end_matches(".sql").to_owned();
                    self.workspace.files.workspace_search_query = stem;
                    self.workspace.files_panel_tab = FilesPanelTab::Search;
                    self.run_workspace_search();
                }
                if delete_node {
                    let _ = self.workspace.files.ide_workspace.delete_path(&node.relative_path);
                }
            }
        });
        if node.is_dir
            && self
                .workspace
                .files
                .ide_workspace
                .expanded
                .contains(&node.relative_path)
        {
            for child in &node.children {
                self.draw_workspace_tree_node(ui, child, depth + 1);
            }
        }
    }
}
