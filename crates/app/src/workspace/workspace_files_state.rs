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
    /// A `git` subprocess running on a worker thread; new operations are
    /// refused while it is in flight so the draw path never blocks (#391).
    pub(super) git_task: Option<std::sync::mpsc::Receiver<git_workspace::GitTaskOutcome>>,
    pub(super) git_last_error: Option<String>,
    /// Directory scan running on a worker thread; `true` replaces all roots on
    /// completion (open/refresh), `false` appends them (restore/add) (#393).
    pub(super) scan_task: Option<(bool, std::sync::mpsc::Receiver<Vec<ide_workspace_scan::ScannedRoot>>)>,
    /// File reads in flight on worker threads; `(path, receiver)` so a second
    /// open of the same path is deduped while pending (#377).
    pub(super) file_read_tasks:
        Vec<(std::path::PathBuf, std::sync::mpsc::Receiver<ide_workspace_scan::FileReadOutcome>)>,
    /// Last time open documents were stat'ed for external changes (#405).
    pub(super) external_check_at: Option<std::time::Instant>,
    pub(super) workspace_file_mtimes: std::collections::HashMap<String, u64>,
    /// Dirty buffers whose disk file diverged, queued per path (#414). More
    /// than one file can change under a single scan (git checkout, codegen);
    /// a lone `Option` dropped every warning but the last.
    pub(super) workspace_external_changes: Vec<String>,
    pub(super) workspace_search_query: String,
    pub(super) workspace_replace_query: String,
    pub(super) workspace_search_hits: Vec<ide_workspace::SearchHit>,
    pub(super) workspace_replace_previews: Vec<ide_workspace::ReplacePreview>,
    pub(super) workspace_task_command: String,
    pub(super) workspace_refactor_from: String,
    pub(super) workspace_refactor_to: String,
    pub(super) workspace_context_items: Vec<String>,
    pub(super) panel_tab: FilesPanelTab,
    pub(super) pending_replace_all: bool,
    pub(super) pending_shell: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct WorkspaceSearchDraft {
    pub(super) search_query: String,
    pub(super) replace_query: String,
    pub(super) refactor_from: String,
    pub(super) refactor_to: String,
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

    pub(super) fn search_draft(&self) -> WorkspaceSearchDraft {
        WorkspaceSearchDraft {
            search_query: self.workspace_search_query.clone(),
            replace_query: self.workspace_replace_query.clone(),
            refactor_from: self.workspace_refactor_from.clone(),
            refactor_to: self.workspace_refactor_to.clone(),
        }
    }

    pub(super) fn apply_search_draft(&mut self, draft: WorkspaceSearchDraft) {
        self.workspace_search_query = draft.search_query;
        self.workspace_replace_query = draft.replace_query;
        self.workspace_refactor_from = draft.refactor_from;
        self.workspace_refactor_to = draft.refactor_to;
    }

    pub(super) fn toggle_directory(&mut self, path: String) {
        if !self.ide_workspace.expanded.insert(path.clone()) {
            self.ide_workspace.expanded.remove(&path);
        }
    }

    /// Resolve one pending external-change warning; other queued paths stay
    /// pending so a multi-file change keeps every warning (#414).
    pub(super) fn dismiss_external_change(&mut self, path: &str) {
        self.workspace_external_changes.retain(|queued| queued != path);
    }

    pub(super) fn close(&mut self, shell: &mut WorkspaceShellState, feedback: &mut FeedbackState) {
        self.ide_workspace.close();
        self.workspace_search_hits.clear();
        self.workspace_replace_previews.clear();
        self.workspace_external_changes.clear();
        self.workspace_context_items.clear();
        shell.split_editor_secondary = None;
        feedback.set_runtime_message("Workspace closed");
    }

    pub(super) fn refresh(&mut self, feedback: &mut FeedbackState) {
        let roots: Vec<(String, std::path::PathBuf)> = self
            .ide_workspace
            .roots
            .iter()
            .map(|root| (root.id.clone(), root.path.clone()))
            .collect();
        if roots.is_empty() {
            feedback.set_runtime_message("Open a workspace folder first");
            return;
        }
        if self.begin_scan(roots, true) {
            feedback.set_runtime_message("Scanning workspace…");
        } else {
            feedback.set_runtime_message("Workspace scan already running…");
        }
    }

    /// Start a background scan for `roots`; `replace` swaps the whole tree on
    /// completion. Refuses while a scan is already running so read_dir storms
    /// never queue up on a click (#393).
    fn begin_scan(&mut self, roots: Vec<(String, std::path::PathBuf)>, replace: bool) -> bool {
        if self.scan_task.is_some() {
            return false;
        }
        self.scan_task = Some((replace, ide_workspace_scan::spawn_scan_task(roots)));
        true
    }

    /// Open `paths` as workspace roots via a background scan (#393). `replace`
    /// swaps the whole tree on completion; otherwise the paths append.
    /// Returns Err on a bad path / busy scan so the caller reports it.
    pub(super) fn request_open_folders(
        &mut self,
        paths: Vec<std::path::PathBuf>,
        replace: bool,
    ) -> Result<(), String> {
        if paths.is_empty() {
            return Ok(());
        }
        let existing = self.ide_workspace.roots.len();
        let mut roots = Vec::with_capacity(paths.len());
        for (index, path) in paths.into_iter().enumerate() {
            if !path.is_dir() {
                return Err(format!("not a folder: {}", path.display()));
            }
            let id = if replace {
                format!("root-{}", index + 1)
            } else {
                format!("root-{}", existing + index + 1)
            };
            roots.push((id, path));
        }
        if self.begin_scan(roots, replace) {
            Ok(())
        } else {
            Err("Workspace scan already running…".to_owned())
        }
    }

    /// Queue a file read on a worker thread (#377); the document is applied in
    /// `drain_file_reads` once the content arrives.
    pub(super) fn request_file_read(&mut self, path: std::path::PathBuf, feedback: &mut FeedbackState) {
        self.file_read_tasks
            .push((path.clone(), ide_workspace_scan::spawn_file_read_task(path)));
        feedback.set_runtime_message("Opening file…");
    }

    /// True while a scan/file-read worker still owns results to apply — the
    /// frame loop keeps repainting until they drain.
    pub(super) fn background_tasks_in_flight(&self) -> bool {
        self.git_task.is_some() || self.scan_task.is_some() || !self.file_read_tasks.is_empty()
    }

    /// Drain a finished background scan into the workspace tree (#393).
    /// Returns `(indexed files, first error, scanned root count)`.
    pub(super) fn poll_scan_task(&mut self) -> Option<(usize, Option<String>, usize)> {
        // Take the receiver out first: borrowing `self.scan_task` across the
        // `ide_workspace` mutation below would be a conflicting borrow.
        let (replace, receiver) = self.scan_task.take()?;
        let scanned = match receiver.try_recv() {
            Ok(scanned) => scanned,
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.scan_task = Some((replace, receiver));
                return None;
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                return Some((0, Some("workspace scan ended without a result".to_owned()), 0));
            }
        };
        let root_count = scanned.len();
        let (indexed, error) = self.ide_workspace.apply_scanned_roots(scanned, replace);
        Some((indexed, error, root_count))
    }

    /// Drain finished file reads (#377); the caller applies documents.
    pub(super) fn drain_file_reads(&mut self) -> Vec<ide_workspace_scan::FileReadOutcome> {
        let mut done = Vec::new();
        self.file_read_tasks
            .retain_mut(|(_, receiver)| match receiver.try_recv() {
                Ok(outcome) => {
                    done.push(outcome);
                    false
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => true,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => false,
            });
        done
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

    pub(super) fn request_replace_all(&mut self, feedback: &mut FeedbackState) {
        self.pending_replace_all = true;
        feedback.set_runtime_message("Confirm before replacing all");
    }

    pub(super) fn confirm_replace_all(&mut self, feedback: &mut FeedbackState) {
        if !self.pending_replace_all {
            return;
        }
        self.pending_replace_all = false;
        self.apply_replace(feedback);
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

    pub(super) fn request_shell(&mut self, feedback: &mut FeedbackState) {
        self.pending_shell = true;
        feedback.set_runtime_message("Confirm before running the shell command");
    }

    pub(super) fn confirm_shell(&mut self, feedback: &mut FeedbackState) {
        if !self.pending_shell {
            return;
        }
        self.pending_shell = false;
        self.run_task(feedback);
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

    /// Drain a finished background git operation into UI state (#391).
    pub(super) fn poll_git_task(&mut self, feedback: &mut FeedbackState) {
        let Some(receiver) = self.git_task.as_ref() else {
            return;
        };
        let outcome = match receiver.try_recv() {
            Ok(outcome) => outcome,
            Err(std::sync::mpsc::TryRecvError::Empty) => return,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.git_task = None;
                self.git_last_error = Some("git task ended without a result".into());
                return;
            }
        };
        self.git_task = None;
        match outcome {
            git_workspace::GitTaskOutcome::Status(Ok(status)) => {
                self.git_last_error = if status.available { None } else { Some(status.message.clone()) };
                self.git_status = Some(status);
                feedback.set_runtime_message("Git status refreshed");
            }
            git_workspace::GitTaskOutcome::Status(Err(error)) => self.git_last_error = Some(error),
            git_workspace::GitTaskOutcome::Mutation { result, status } => {
                match result {
                    Ok(message) => {
                        self.git_commit_message.clear();
                        self.git_last_error = None;
                        feedback.set_runtime_message(message.lines().next().unwrap_or("Done").to_owned());
                    }
                    Err(error) => self.git_last_error = Some(error),
                }
                self.git_status = Some(status);
            }
            git_workspace::GitTaskOutcome::Diff(Ok(diff)) => {
                self.git_diff = Some(diff);
                self.git_last_error = None;
            }
            git_workspace::GitTaskOutcome::Diff(Err(error)) => self.git_last_error = Some(error),
        }
    }

    /// Spawn a git operation on a worker thread; refuses while another git
    /// task is in flight so subprocess calls never run on the UI thread (#391).
    fn dispatch_git_task(&mut self, operation: git_workspace::GitOperation, feedback: &mut FeedbackState) -> bool {
        if self.git_task.is_some() {
            feedback.set_runtime_message("Git is already running…");
            return false;
        }
        let Some(root) = self.ide_workspace.primary_path().map(std::path::PathBuf::from) else {
            self.git_last_error = Some("Open a workspace folder first".into());
            return false;
        };
        self.git_task = Some(git_workspace::spawn_git_task(root, operation));
        feedback.set_runtime_message("Git running…");
        true
    }

    pub(super) fn git_task_in_flight(&self) -> bool {
        self.git_task.is_some()
    }

    pub(super) fn refresh_git_status(&mut self, feedback: &mut FeedbackState) {
        if self.ide_workspace.primary_path().is_none() {
            self.git_status = None;
            self.git_last_error = Some("Open a workspace folder first".into());
            return;
        }
        self.dispatch_git_task(git_workspace::GitOperation::RefreshStatus, feedback);
    }

    pub(super) fn stage_git_path(&mut self, relative: &str, feedback: &mut FeedbackState) {
        self.dispatch_git_task(
            git_workspace::GitOperation::Stage {
                relative: relative.to_owned(),
            },
            feedback,
        );
    }

    pub(super) fn unstage_git_path(&mut self, relative: &str, feedback: &mut FeedbackState) {
        self.dispatch_git_task(
            git_workspace::GitOperation::Unstage {
                relative: relative.to_owned(),
            },
            feedback,
        );
    }

    pub(super) fn diff_git_path(&mut self, relative: &str, feedback: &mut FeedbackState) {
        self.dispatch_git_task(
            git_workspace::GitOperation::Diff {
                relative: relative.to_owned(),
            },
            feedback,
        );
    }

    pub(super) fn commit_git_staged(&mut self, feedback: &mut FeedbackState) {
        let message = self.git_commit_message.trim().to_owned();
        if message.is_empty() {
            self.git_last_error = Some("commit message is required".into());
            return;
        }
        self.dispatch_git_task(git_workspace::GitOperation::Commit { message }, feedback);
    }

    /// Poll open documents for disk changes. Throttled to 500ms and works on
    /// borrowed buffers — cloning every doc's text per frame churned GC on
    /// large SQL dumps (#405). Content is only re-read when the mtime moved.
    pub(super) fn check_external_file_changes(&mut self, documents: &[(&str, &str, bool)]) {
        if self
            .external_check_at
            .is_some_and(|checked| checked.elapsed() < std::time::Duration::from_millis(500))
        {
            return;
        }
        self.external_check_at = Some(std::time::Instant::now());
        for (path, buffer, dirty) in documents {
            let path_buf = std::path::PathBuf::from(path);
            let Some(mtime) = git_workspace::disk_mtime_secs(&path_buf) else {
                continue;
            };
            let known = self.workspace_file_mtimes.get(*path).copied();
            if known == Some(mtime) {
                continue;
            }
            if *dirty && git_workspace::disk_diverged_from_buffer(&path_buf, buffer) {
                if !self.workspace_external_changes.iter().any(|queued| queued == path) {
                    self.workspace_external_changes.push((*path).to_owned());
                }
            } else if !dirty {
                self.workspace_file_mtimes.insert((*path).to_owned(), mtime);
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/workspace_files_state_tests.rs"]
mod tests;

