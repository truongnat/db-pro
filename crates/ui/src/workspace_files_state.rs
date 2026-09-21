use super::*;

/// Presentation state for the local SQL workspace and its file/Git activity.
///
/// Database connection state and workspace navigation stay outside this aggregate:
/// this state owns only local files, search/refactor drafts, Git feedback and the
/// file context that can be attached to an agent request.
#[derive(Debug, Default)]
pub(crate) struct WorkspaceFilesState {
    pub(super) ide_workspace: ide_workspace::IdeWorkspaceState,
    pub(super) git_status: Option<git_workspace::GitWorkspaceStatus>,
    pub(super) git_diff: Option<git_workspace::GitDiffResult>,
    pub(super) git_commit_message: String,
    pub(super) git_last_error: Option<String>,
    pub(super) workspace_file_mtimes: std::collections::HashMap<String, u64>,
    pub(super) workspace_external_change: Option<String>,
    pub(super) workspace_search_query: String,
    pub(super) workspace_replace_query: String,
    pub(super) workspace_search_hits: Vec<ide_workspace::SearchHit>,
    pub(super) workspace_replace_previews: Vec<ide_workspace::ReplacePreview>,
    pub(super) workspace_task_command: String,
    pub(super) workspace_refactor_from: String,
    pub(super) workspace_refactor_to: String,
    pub(super) workspace_context_items: Vec<String>,
    pub(super) panel_tab: FilesPanelTab,
}

impl WorkspaceFilesState {
    pub(super) fn panel_tab(&self) -> FilesPanelTab {
        self.panel_tab
    }

    pub(super) fn select_panel_tab(&mut self, tab: FilesPanelTab) {
        self.panel_tab = tab;
    }

    pub(super) fn select_root(&mut self, index: usize) -> bool {
        if index >= self.ide_workspace.roots.len() {
            return false;
        }
        self.ide_workspace.active_root = index;
        true
    }

    pub(super) fn remove_active_root(&mut self) {
        self.ide_workspace.remove_active_root();
    }

    pub(super) fn set_trusted(&mut self, trusted: bool) {
        self.ide_workspace.set_trusted(trusted);
    }

    pub(super) fn select_environment(&mut self, index: usize) -> Option<String> {
        let name = self.ide_workspace.environments.get(index)?.name.clone();
        self.ide_workspace.set_active_environment(index);
        Some(name)
    }

    pub(super) fn set_search_query(&mut self, query: String) {
        self.workspace_search_query = query;
    }

    pub(super) fn toggle_directory(&mut self, path: String) {
        if !self.ide_workspace.expanded.insert(path.clone()) {
            self.ide_workspace.expanded.remove(&path);
        }
    }

    pub(super) fn dismiss_external_change(&mut self) {
        self.workspace_external_change = None;
    }

    pub(super) fn close(&mut self, shell: &mut WorkspaceShellState, feedback: &mut FeedbackState) {
        self.ide_workspace.close();
        self.workspace_search_hits.clear();
        self.workspace_replace_previews.clear();
        self.workspace_context_items.clear();
        shell.split_editor_secondary = None;
        feedback.set_runtime_message("Workspace closed");
    }

    pub(super) fn refresh(&mut self, feedback: &mut FeedbackState) {
        match self.ide_workspace.refresh() {
            Ok(()) => {
                self.ide_workspace.scan_diagnostics();
                feedback.set_runtime_message(format!(
                    "Workspace refreshed · {} files indexed",
                    self.ide_workspace.index().len()
                ));
            }
            Err(error) => feedback.set_runtime_message(format!("Workspace refresh failed: {error}")),
        }
    }

    pub(super) fn run_search(&mut self, feedback: &mut FeedbackState) {
        if self.ide_workspace.roots.is_empty() {
            self.workspace_search_hits.clear();
            feedback.set_runtime_message("Open a workspace folder before searching");
            return;
        }
        self.workspace_search_hits = self.ide_workspace.search(&self.workspace_search_query, 100);
        feedback.set_runtime_message(format!("{} matches", self.workspace_search_hits.len()));
    }

    pub(super) fn preview_replace(&mut self, feedback: &mut FeedbackState) {
        self.workspace_replace_previews = self
            .ide_workspace
            .preview_replace(&self.workspace_search_query, &self.workspace_replace_query);
        feedback.set_runtime_message(format!("{} files would change", self.workspace_replace_previews.len()));
    }

    pub(super) fn apply_replace(&mut self, feedback: &mut FeedbackState) {
        match self
            .ide_workspace
            .apply_replace(&self.workspace_search_query, &self.workspace_replace_query)
        {
            Ok(count) => {
                self.preview_replace(feedback);
                self.run_search(feedback);
                feedback.set_runtime_message(format!("Replaced {count} occurrence(s)"));
            }
            Err(error) => feedback.set_runtime_message(error),
        }
    }

    pub(super) fn add_context_item(&mut self, item: String) {
        if !self.workspace_context_items.iter().any(|existing| existing == &item) {
            self.workspace_context_items.push(item);
        }
    }

    pub(super) fn clear_context_items(&mut self) {
        self.workspace_context_items.clear();
    }

    pub(super) fn export_live_schema_snapshot(&self, schema: &UiSchemaSummary, feedback: &mut FeedbackState) {
        let mut sql = String::from("-- DB Pro schema snapshot\n");
        for table in &schema.table_details {
            sql.push_str(&format!(
                "-- table {}.{} ({} columns)\n",
                table.schema,
                table.name,
                table.columns.len()
            ));
        }
        match self.ide_workspace.export_schema_snapshot(&sql) {
            Ok(path) => feedback.set_runtime_message(format!("Wrote schema snapshot {}", path.display())),
            Err(error) => feedback.set_runtime_message(error),
        }
    }

    pub(super) fn run_task(&mut self, feedback: &mut FeedbackState) {
        let command = self.workspace_task_command.clone();
        match self.ide_workspace.run_task(&command) {
            Ok(result) => {
                feedback.set_runtime_message(format!("Task exit {:?} · {}ms", result.exit_code, result.duration_ms))
            }
            Err(error) => feedback.set_runtime_message(error),
        }
    }

    pub(super) fn apply_refactor(&mut self, feedback: &mut FeedbackState) {
        let from = self.workspace_refactor_from.clone();
        let to = self.workspace_refactor_to.clone();
        match self.ide_workspace.rename_symbol_across_sql(&from, &to) {
            Ok(count) => feedback.set_runtime_message(format!("Refactored {count} occurrence(s)")),
            Err(error) => feedback.set_runtime_message(error),
        }
    }

    pub(super) fn refresh_schema_drift_watch(&mut self, schema_names: &[String], feedback: &mut FeedbackState) {
        let fingerprint = ide_workspace::fingerprint_schema_names(schema_names);
        self.ide_workspace.update_schema_fingerprint(fingerprint);
        if let Some(message) = self.ide_workspace.schema_drift_message.clone() {
            feedback.set_runtime_message(message);
        }
    }

    pub(super) fn refresh_git_status(&mut self, feedback: &mut FeedbackState) {
        let Some(root) = self.ide_workspace.primary_path().map(std::path::PathBuf::from) else {
            self.git_status = None;
            self.git_last_error = Some("Open a workspace folder first".into());
            return;
        };
        let status = git_workspace::probe_git_status(&root);
        self.git_last_error = if status.available {
            None
        } else {
            Some(status.message.clone())
        };
        self.git_status = Some(status);
        feedback.set_runtime_message("Git status refreshed");
    }

    pub(super) fn stage_git_path(&mut self, relative: &str, feedback: &mut FeedbackState) {
        let Some(root) = self.ide_workspace.primary_path() else {
            return;
        };
        match git_workspace::stage_path(root, relative) {
            Ok(()) => {
                self.git_last_error = None;
                self.refresh_git_status(feedback);
            }
            Err(error) => self.git_last_error = Some(error),
        }
    }

    pub(super) fn unstage_git_path(&mut self, relative: &str, feedback: &mut FeedbackState) {
        let Some(root) = self.ide_workspace.primary_path() else {
            return;
        };
        match git_workspace::unstage_path(root, relative) {
            Ok(()) => {
                self.git_last_error = None;
                self.refresh_git_status(feedback);
            }
            Err(error) => self.git_last_error = Some(error),
        }
    }

    pub(super) fn diff_git_path(&mut self, relative: &str) {
        let Some(root) = self.ide_workspace.primary_path() else {
            return;
        };
        match git_workspace::diff_against_head(root, relative) {
            Ok(diff) => {
                self.git_diff = Some(diff);
                self.git_last_error = None;
            }
            Err(error) => self.git_last_error = Some(error),
        }
    }

    pub(super) fn commit_git_staged(&mut self, feedback: &mut FeedbackState) {
        let Some(root) = self.ide_workspace.primary_path() else {
            return;
        };
        match git_workspace::commit_paths(root, &self.git_commit_message) {
            Ok(output) => {
                self.git_commit_message.clear();
                self.git_last_error = None;
                feedback.set_runtime_message(output.lines().next().unwrap_or("Committed"));
                self.refresh_git_status(feedback);
            }
            Err(error) => self.git_last_error = Some(error),
        }
    }

    pub(super) fn check_external_file_changes(&mut self, documents: &[(String, String, bool)]) {
        for (path, buffer, dirty) in documents {
            let path_buf = std::path::PathBuf::from(path);
            let Some(mtime) = git_workspace::disk_mtime_secs(&path_buf) else {
                continue;
            };
            let known = self.workspace_file_mtimes.get(path).copied();
            if known == Some(mtime) {
                continue;
            }
            if *dirty && git_workspace::disk_diverged_from_buffer(&path_buf, buffer) {
                self.workspace_external_change = Some(path.clone());
            } else if !dirty {
                self.workspace_file_mtimes.insert(path.clone(), mtime);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FilesPanelTab, WorkspaceFilesState};

    #[test]
    fn default_workspace_files_state_starts_without_open_roots_or_drafts() {
        let state = WorkspaceFilesState::default();

        assert!(state.ide_workspace.roots.is_empty());
        assert!(state.workspace_search_query.is_empty());
        assert!(state.workspace_replace_previews.is_empty());
        assert!(state.workspace_context_items.is_empty());
        assert!(state.git_status.is_none());
        assert_eq!(state.panel_tab, FilesPanelTab::Tree);
    }

    #[test]
    fn panel_selection_and_invalid_root_selection_stay_inside_workspace_files_state() {
        let mut state = WorkspaceFilesState::default();

        state.select_panel_tab(FilesPanelTab::Search);

        assert_eq!(state.panel_tab(), FilesPanelTab::Search);
        assert!(!state.select_root(0));
    }
}
