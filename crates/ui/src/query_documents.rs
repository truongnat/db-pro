//! Query document tab lifecycle (new / close / duplicate / history open).
use super::*;

impl DbProApp {
    pub(crate) fn new_query_document(&mut self) {
        let (document_id, index) = self.next_query_document_identity();
        let mut doc = QueryDocument::new(document_id, format!("Query {index}"), String::new());
        doc.connection_id = self.connection_lifecycle.active_connection_id.clone();
        doc.schema = Some(self.active_schema().to_owned());
        self.query_session_state.documents.push(doc);
        self.query_session_state.active_document_index = self.query_session_state.documents.len() - 1;
        self.query_editor.query_focus_editor_on_open = true;
        self.reset_query_cursor();
        self.workspace.activity = Activity::Queries;
        self.workspace.sidebar_open = true;
        self.workspace.active_tab = WorkspaceTab::Query;
    }

    /// Disposable scratch tab for throwaway SQL (#211).
    pub(crate) fn new_scratch_query_document(&mut self) {
        let (document_id, index) = self.next_query_document_identity();
        let mut doc = QueryDocument::new(document_id, format!("Scratch {index}"), String::new());
        doc.connection_id = self.connection_lifecycle.active_connection_id.clone();
        doc.schema = Some(self.active_schema().to_owned());
        self.query_session_state.documents.push(doc);
        self.query_session_state.active_document_index = self.query_session_state.documents.len() - 1;
        self.query_editor.query_focus_editor_on_open = true;
        self.reset_query_cursor();
        self.workspace.activity = Activity::Queries;
        self.workspace.sidebar_open = true;
        self.workspace.active_tab = WorkspaceTab::Query;
        self.feedback.runtime_message = "Opened scratch SQL tab".to_owned();
    }

    /// Cycle a simple numbered rename for the open query tab (#211).
    pub(crate) fn rename_query_document_inline(&mut self, index: usize) {
        let Some(doc) = self.query_session_state.documents.get_mut(index) else {
            return;
        };
        if doc.title.starts_with("Scratch ") {
            let n = doc.title.trim_start_matches("Scratch ").parse::<u32>().unwrap_or(1);
            doc.title = format!("Scratch {}", n + 1);
        } else if let Some(rest) = doc.title.strip_prefix("Query ") {
            let n = rest.parse::<u32>().unwrap_or(1);
            doc.title = format!("Query {}", n + 1);
        } else {
            doc.title = format!("{} (renamed)", doc.title);
        }
        self.feedback.runtime_message = format!("Renamed tab to {}", doc.title);
    }

    pub(crate) fn open_history_entry(&mut self, entry: &UiQueryHistoryEntry, run: bool) {
        let (document_id, document_number) = self.next_query_document_identity();
        let mut document = QueryDocument::new(document_id, format!("History {document_number}"), entry.sql.clone());
        document.connection_id = entry.connection_id.clone();
        document.schema = entry.schema.clone();
        self.query_session_state.documents.push(document);
        self.query_session_state.active_document_index = self.query_session_state.documents.len() - 1;
        self.query_editor.query_focus_editor_on_open = true;
        self.workspace.activity = Activity::Queries;
        self.workspace.active_tab = WorkspaceTab::Query;
        self.reset_query_cursor();
        if run {
            self.dispatch_query();
        }
    }

    pub(crate) fn close_query_document(&mut self, index: usize) {
        if index >= self.query_session_state.documents.len() {
            return;
        }

        self.cancel_prediction_for_document(index);
        let closed_id = self.query_session_state.documents[index].id.clone();
        let closed_title = self.query_session_state.documents[index].title.clone();
        if let Some(run_id) = self
            .agent
            .sessions
            .get(&closed_id)
            .and_then(|session| session.active_run_id)
        {
            let request_id = self.task_bridge.next_request_id();
            self.send_command_best_effort(UiCommand::CancelAgentRun { request_id, run_id });
        }
        self.agent.sessions.remove(&closed_id);
        self.query_session_state.documents.remove(index);
        self.query_output_state.tabs_by_document.remove(&closed_id);

        if self.query_session_state.documents.is_empty() {
            self.query_session_state.active_document_index = 0;
            self.reset_query_cursor();
            if self.workspace.active_tab == WorkspaceTab::Query {
                self.activate_fallback_workspace_tab();
            }
            self.feedback.runtime_message = format!("Closed {closed_title}");
            return;
        }

        if self.query_session_state.active_document_index > index {
            self.query_session_state.active_document_index -= 1;
        } else if self.query_session_state.active_document_index == index {
            self.query_session_state.active_document_index = self
                .query_session_state
                .active_document_index
                .min(self.query_session_state.documents.len() - 1);
        }
        let doc = &self.query_session_state.documents[self.query_session_state.active_document_index];
        self.query_editor.query_cursor_line = doc.cursor.line + 1;
        self.query_editor.query_cursor_column = doc.cursor.col + 1;
        if !doc.selection.is_empty() {
            let (start, end) = doc.selection.normalized();
            self.query_session_state.selected_text = doc.buffer.slice(start, end).to_owned();
        } else {
            self.query_session_state.selected_text.clear();
        }
        self.feedback.runtime_message = format!(
            "Closed {}",
            self.query_session_state.documents[self.query_session_state.active_document_index].title
        );
    }

    pub(super) fn next_query_document_identity(&self) -> (String, usize) {
        let mut number = self.query_session_state.documents.len().saturating_add(1);
        loop {
            let id = format!("query-{number}");
            if !self
                .query_session_state
                .documents
                .iter()
                .any(|document| document.id == id)
            {
                return (id, number);
            }
            number = number.saturating_add(1);
        }
    }

    pub(crate) fn request_close_query_document(&mut self, index: usize) {
        if self
            .query_session_state
            .documents
            .get(index)
            .is_some_and(QueryDocument::is_dirty)
        {
            self.query_session_state.pending_dirty_close = Some(index);
        } else {
            self.close_query_document(index);
        }
    }

    pub(super) fn reset_query_cursor(&mut self) {
        self.query_editor.query_cursor_line = 1;
        self.query_editor.query_cursor_column = 1;
    }

    pub(crate) fn duplicate_query_document(&mut self, index: usize) {
        if index >= self.query_session_state.documents.len() {
            return;
        }
        let src = &self.query_session_state.documents[index];
        let title = format!("{} (Copy)", src.title);
        let content = src.text().to_owned();
        let (document_id, _) = self.next_query_document_identity();
        let mut new_doc = QueryDocument::new(document_id, title, content);
        new_doc.connection_id = src
            .connection_id
            .clone()
            .or_else(|| self.connection_lifecycle.active_connection_id.clone());
        new_doc.schema = src.schema.clone().or_else(|| Some(self.active_schema().to_owned()));
        self.query_session_state.documents.push(new_doc);
        self.query_session_state.active_document_index = self.query_session_state.documents.len() - 1;
        self.query_editor.query_focus_editor_on_open = true;
        self.workspace.active_tab = WorkspaceTab::Query;
        self.feedback.runtime_message = format!("Duplicated {}", self.query_session_state.documents[index].title);
    }

    pub(crate) fn close_other_query_documents(&mut self, keep_index: usize) {
        if keep_index >= self.query_session_state.documents.len() {
            return;
        }
        for index in 0..self.query_session_state.documents.len() {
            if index != keep_index {
                self.cancel_prediction_for_document(index);
            }
        }
        let kept = self.query_session_state.documents[keep_index].clone();
        self.query_session_state.documents = vec![kept];
        self.query_session_state.active_document_index = 0;
        self.feedback.runtime_message = "Closed other queries".to_owned();
    }

    pub(crate) fn close_query_documents_to_right(&mut self, index: usize) {
        if index >= self.query_session_state.documents.len() {
            return;
        }
        for query_index in index + 1..self.query_session_state.documents.len() {
            self.cancel_prediction_for_document(query_index);
        }
        self.query_session_state.documents.truncate(index + 1);
        if self.query_session_state.active_document_index > index {
            self.query_session_state.active_document_index = index;
        }
        self.feedback.runtime_message = "Closed queries to the right".to_owned();
    }

    pub(crate) fn close_all_tabs(&mut self) {
        for index in 0..self.query_session_state.documents.len() {
            self.cancel_prediction_for_document(index);
        }
        self.workspace.welcome_open = true;
        self.query_session_state.documents = vec![QueryDocument::new("query-1", "Query 1", String::new())];
        self.query_session_state.active_document_index = 0;
        self.schema_explorer.selected_table = None;
        self.schema_explorer.selected_schema_object = None;
        self.workspace.active_tab = WorkspaceTab::Welcome;
        self.feedback.runtime_message = "Closed all tabs".to_owned();
    }

    pub(crate) fn close_welcome_tab(&mut self) {
        self.workspace.welcome_open = false;
        if self.workspace.active_tab == WorkspaceTab::Welcome {
            self.activate_fallback_workspace_tab();
        }
        self.feedback.runtime_message = "Closed Welcome".to_owned();
    }

    pub(crate) fn activate_welcome_tab(&mut self) {
        self.workspace.welcome_open = true;
        self.workspace.active_tab = WorkspaceTab::Welcome;
    }

    pub(super) fn activate_fallback_workspace_tab(&mut self) {
        if !self.query_session_state.documents.is_empty() {
            self.workspace.active_tab = WorkspaceTab::Query;
        } else if self.schema_explorer.selected_table.is_some() {
            self.workspace.active_tab = WorkspaceTab::Table;
        } else if self.schema_explorer.selected_schema_object.is_some() {
            self.workspace.active_tab = WorkspaceTab::SchemaObject;
        } else {
            self.activate_welcome_tab();
        }
    }

    pub(crate) fn execute_pending_navigation(&mut self, action: PendingNavigationAction) {
        match action {
            PendingNavigationAction::OpenTable(table) => {
                self.open_table(table);
            }
            PendingNavigationAction::ChangeSchema(schema) => {
                self.activate_schema(&schema);
            }
            PendingNavigationAction::ChangeConnection(connection_id) => {
                if let Some(conn) = self
                    .connection_catalog
                    .connections
                    .iter()
                    .find(|c| c.id == connection_id)
                    .cloned()
                {
                    self.connect_to_connection(&conn);
                }
            }
            PendingNavigationAction::CloseWorkspace(tab) => {
                self.request_close_workspace_tab(tab);
            }
        }
    }
}
