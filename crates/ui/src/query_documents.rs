//! Query document tab lifecycle (new / close / duplicate / history open).
use super::*;

/// Explicit dependencies for query-document lifecycle transitions.
///
/// The lifecycle owns query-tab state and coordinates only the stateful side
/// effects that are part of closing/opening a document. App-level commands
/// such as executing a query remain at the composition root.
pub(crate) struct QueryDocumentContext<'a> {
    pub(super) query_session: &'a mut QuerySessionState,
    pub(super) query_editor: &'a mut QueryEditorState,
    pub(super) workspace: &'a mut WorkspaceFeatureState,
    pub(super) agent: &'a mut AgentState,
    pub(super) query_output: &'a mut QueryOutputState,
    pub(super) schema_explorer: &'a mut SchemaExplorerState,
    pub(super) task_bridge: &'a mut TaskBridge,
    pub(super) feedback: &'a mut FeedbackState,
    active_connection_id: Option<String>,
    active_schema: String,
}

impl QueryDocumentContext<'_> {
    pub(crate) fn new_document(&mut self, scratch: bool) {
        let (document_id, index) = self.next_document_identity();
        let prefix = if scratch { "Scratch" } else { "Query" };
        let mut document = QueryDocument::new(document_id, format!("{prefix} {index}"), String::new());
        document.connection_id = self.active_connection_id.clone();
        document.schema = Some(self.active_schema.clone());
        self.query_session.add_document(document);
        self.activate_query_surface();
        if scratch {
            self.feedback.set_runtime_message("Opened scratch SQL tab");
        }
    }

    pub(crate) fn open_history_entry(&mut self, entry: &UiQueryHistoryEntry) {
        let (document_id, document_number) = self.next_document_identity();
        let mut document = QueryDocument::new(document_id, format!("History {document_number}"), entry.sql.clone());
        document.connection_id = entry.connection_id.clone();
        document.schema = entry.schema.clone();
        self.query_session.add_document(document);
        self.activate_query_surface();
    }

    pub(crate) fn rename_document_inline(&mut self, index: usize) {
        let Some(document) = self.query_session.documents.get_mut(index) else {
            return;
        };
        if document.title.starts_with("Scratch ") {
            let number = document
                .title
                .trim_start_matches("Scratch ")
                .parse::<u32>()
                .unwrap_or(1);
            document.title = format!("Scratch {}", number + 1);
        } else if let Some(rest) = document.title.strip_prefix("Query ") {
            let number = rest.parse::<u32>().unwrap_or(1);
            document.title = format!("Query {}", number + 1);
        } else {
            document.title = format!("{} (renamed)", document.title);
        }
        self.feedback
            .set_runtime_message(format!("Renamed tab to {}", document.title));
    }

    pub(crate) fn close_document(&mut self, index: usize) {
        if index >= self.query_session.documents.len() {
            return;
        }

        self.cancel_prediction(index);
        let closed = self.query_session.documents[index].clone();
        self.cancel_agent_run(&closed.id);
        self.query_session.remove_document(index);
        self.query_output.tabs_by_document.remove(&closed.id);

        if self.query_session.documents.is_empty() {
            self.query_session.active_document_index = 0;
            self.reset_cursor();
            if self.workspace.active_tab == WorkspaceTab::Query {
                self.activate_fallback_surface();
            }
            self.feedback.set_runtime_message(format!("Closed {}", closed.title));
            return;
        }

        self.sync_active_cursor_and_selection();
        let active_title = self
            .query_session
            .active_document()
            .map(|document| document.title.as_str())
            .unwrap_or(closed.title.as_str());
        self.feedback.set_runtime_message(format!("Closed {active_title}"));
    }

    pub(crate) fn request_close_document(&mut self, index: usize) {
        if self
            .query_session
            .documents
            .get(index)
            .is_some_and(QueryDocument::is_dirty)
        {
            self.query_session.pending_dirty_close = Some(index);
        } else {
            self.close_document(index);
        }
    }

    pub(crate) fn duplicate_document(&mut self, index: usize) {
        let Some(source) = self.query_session.documents.get(index).cloned() else {
            return;
        };
        let (document_id, _) = self.next_document_identity();
        let mut document = QueryDocument::new(document_id, format!("{} (Copy)", source.title), source.text());
        document.connection_id = source.connection_id.or_else(|| self.active_connection_id.clone());
        document.schema = source.schema.or_else(|| Some(self.active_schema.clone()));
        self.query_session.add_document(document);
        self.query_editor.query_focus_editor_on_open = true;
        self.workspace.active_tab = WorkspaceTab::Query;
        self.feedback
            .set_runtime_message(format!("Duplicated {}", source.title));
    }

    pub(crate) fn close_other_documents(&mut self, keep_index: usize) {
        if keep_index >= self.query_session.documents.len() {
            return;
        }
        for index in 0..self.query_session.documents.len() {
            if index != keep_index {
                self.cancel_prediction(index);
            }
        }
        self.query_session.keep_document(keep_index);
        self.feedback.set_runtime_message("Closed other queries");
    }

    pub(crate) fn close_documents_to_right(&mut self, index: usize) {
        if index >= self.query_session.documents.len() {
            return;
        }
        for query_index in index + 1..self.query_session.documents.len() {
            self.cancel_prediction(query_index);
        }
        self.query_session.close_documents_to_right(index);
        self.feedback.set_runtime_message("Closed queries to the right");
    }

    pub(crate) fn close_all_tabs(&mut self) {
        for index in 0..self.query_session.documents.len() {
            self.cancel_prediction(index);
        }
        self.workspace.welcome_open = true;
        self.query_session
            .replace_with_document(QueryDocument::new("query-1", "Query 1", String::new()));
        self.schema_explorer.selected_table = None;
        self.schema_explorer.selected_schema_object = None;
        self.workspace.active_tab = WorkspaceTab::Welcome;
        self.feedback.set_runtime_message("Closed all tabs");
    }

    pub(crate) fn close_welcome_tab(&mut self) {
        self.workspace.welcome_open = false;
        if self.workspace.active_tab == WorkspaceTab::Welcome {
            self.activate_fallback_surface();
        }
        self.feedback.set_runtime_message("Closed Welcome");
    }

    pub(crate) fn activate_welcome_tab(&mut self) {
        self.workspace.welcome_open = true;
        self.workspace.active_tab = WorkspaceTab::Welcome;
    }

    fn next_document_identity(&self) -> (String, usize) {
        let mut number = self.query_session.documents.len().saturating_add(1);
        loop {
            let id = format!("query-{number}");
            if !self.query_session.documents.iter().any(|document| document.id == id) {
                return (id, number);
            }
            number = number.saturating_add(1);
        }
    }

    fn activate_query_surface(&mut self) {
        self.query_editor.query_focus_editor_on_open = true;
        self.reset_cursor();
        self.workspace.activity = Activity::Queries;
        self.workspace.sidebar_open = true;
        self.workspace.active_tab = WorkspaceTab::Query;
    }

    pub(crate) fn reset_cursor(&mut self) {
        self.query_editor.query_cursor_line = 1;
        self.query_editor.query_cursor_column = 1;
    }

    fn sync_active_cursor_and_selection(&mut self) {
        let Some(document) = self.query_session.active_document() else {
            self.reset_cursor();
            return;
        };
        self.query_editor.query_cursor_line = document.cursor.line + 1;
        self.query_editor.query_cursor_column = document.cursor.col + 1;
        if !document.selection.is_empty() {
            let (start, end) = document.selection.normalized();
            self.query_session.selected_text = document.buffer.slice(start, end).to_owned();
        } else {
            self.query_session.selected_text.clear();
        }
    }

    fn cancel_prediction(&mut self, index: usize) {
        let request_id = self.query_session.invalidate_prediction(index);
        if let Some(request_id) = request_id {
            let _ = self
                .task_bridge
                .send_best_effort(UiCommand::CancelSqlPrediction { request_id });
        }
    }

    fn cancel_agent_run(&mut self, document_id: &str) {
        let Some(run_id) = self
            .agent
            .sessions
            .get(document_id)
            .and_then(|session| session.active_run_id)
        else {
            self.agent.sessions.remove(document_id);
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        let _ = self
            .task_bridge
            .send_best_effort(UiCommand::CancelAgentRun { request_id, run_id });
        self.agent.sessions.remove(document_id);
    }

    fn activate_fallback_surface(&mut self) {
        if !self.query_session.documents.is_empty() {
            self.workspace.active_tab = WorkspaceTab::Query;
        } else if self.schema_explorer.selected_table.is_some() {
            self.workspace.active_tab = WorkspaceTab::Table;
        } else if self.schema_explorer.selected_schema_object.is_some() {
            self.workspace.active_tab = WorkspaceTab::SchemaObject;
        } else {
            self.activate_welcome_tab();
        }
    }
}

impl DbProApp {
    fn query_document_context(&mut self) -> QueryDocumentContext<'_> {
        let active_connection_id = self.connection.lifecycle.active_connection_id().map(str::to_owned);
        let active_schema = self.active_schema().to_owned();
        QueryDocumentContext {
            query_session: &mut self.query_session_state,
            query_editor: &mut self.query_editor,
            workspace: &mut self.workspace,
            agent: &mut self.agent,
            query_output: &mut self.query_output_state,
            schema_explorer: &mut self.schema_explorer,
            task_bridge: &mut self.task_bridge,
            feedback: &mut self.feedback,
            active_connection_id,
            active_schema,
        }
    }

    pub(crate) fn new_query_document(&mut self) {
        self.query_document_context().new_document(false);
    }

    /// Disposable scratch tab for throwaway SQL (#211).
    pub(crate) fn new_scratch_query_document(&mut self) {
        self.query_document_context().new_document(true);
    }

    /// Cycle a simple numbered rename for the open query tab (#211).
    pub(crate) fn rename_query_document_inline(&mut self, index: usize) {
        self.query_document_context().rename_document_inline(index);
    }

    pub(crate) fn open_history_entry(&mut self, entry: &UiQueryHistoryEntry, run: bool) {
        {
            self.query_document_context().open_history_entry(entry);
        }
        if run {
            self.dispatch_query();
        }
    }

    pub(crate) fn close_query_document(&mut self, index: usize) {
        self.query_document_context().close_document(index);
    }

    pub(crate) fn request_close_query_document(&mut self, index: usize) {
        self.query_document_context().request_close_document(index);
    }

    pub(crate) fn duplicate_query_document(&mut self, index: usize) {
        self.query_document_context().duplicate_document(index);
    }

    pub(crate) fn close_other_query_documents(&mut self, keep_index: usize) {
        self.query_document_context().close_other_documents(keep_index);
    }

    pub(crate) fn close_query_documents_to_right(&mut self, index: usize) {
        self.query_document_context().close_documents_to_right(index);
    }

    pub(crate) fn close_all_tabs(&mut self) {
        self.query_document_context().close_all_tabs();
    }

    pub(crate) fn close_welcome_tab(&mut self) {
        self.query_document_context().close_welcome_tab();
    }

    pub(crate) fn activate_welcome_tab(&mut self) {
        self.query_document_context().activate_welcome_tab();
    }

    pub(super) fn reset_query_cursor(&mut self) {
        self.query_document_context().reset_cursor();
    }

    pub(super) fn execute_pending_navigation(&mut self, action: PendingNavigationAction) {
        match action {
            PendingNavigationAction::OpenTable(table) => self.open_table(table),
            PendingNavigationAction::ChangeSchema(schema) => self.activate_schema(&schema),
            PendingNavigationAction::ChangeConnection(connection_id) => {
                let connection = self.connection.catalog.find(&connection_id).cloned();
                if let Some(connection) = connection {
                    self.connect_to_connection(&connection);
                }
            }
            PendingNavigationAction::CloseWorkspace(tab) => self.request_close_workspace_tab(tab),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_document_context_binds_explicit_connection_and_schema() {
        let mut query_session = QuerySessionState::default();
        let mut query_editor = QueryEditorState::default();
        let mut workspace = WorkspaceFeatureState::default();
        let mut agent = AgentState::default();
        let mut query_output = QueryOutputState::default();
        let mut schema_explorer = SchemaExplorerState::default();
        let mut task_bridge = TaskBridge::default();
        let mut feedback = FeedbackState::default();

        QueryDocumentContext {
            query_session: &mut query_session,
            query_editor: &mut query_editor,
            workspace: &mut workspace,
            agent: &mut agent,
            query_output: &mut query_output,
            schema_explorer: &mut schema_explorer,
            task_bridge: &mut task_bridge,
            feedback: &mut feedback,
            active_connection_id: Some("connection-1".to_owned()),
            active_schema: "analytics".to_owned(),
        }
        .new_document(false);

        let document = query_session.active_document().expect("new document");
        assert_eq!(document.connection_id.as_deref(), Some("connection-1"));
        assert_eq!(document.schema.as_deref(), Some("analytics"));
        assert_eq!(workspace.active_tab, WorkspaceTab::Query);
        assert!(query_editor.query_focus_editor_on_open);
    }

    #[test]
    fn closing_document_context_removes_document_owned_output_state() {
        let mut query_session = QuerySessionState::default();
        query_session.add_document(QueryDocument::new("query-1", "Query 1", "select 1"));
        query_session.add_document(QueryDocument::new("query-2", "Query 2", "select 2"));
        query_session.select_document(0);
        let mut query_editor = QueryEditorState::default();
        let mut workspace = WorkspaceFeatureState::default();
        let mut agent = AgentState::default();
        let mut query_output = QueryOutputState::default();
        query_output
            .tabs_by_document
            .insert("query-1".to_owned(), OutputTab::Messages);
        let mut schema_explorer = SchemaExplorerState::default();
        let mut task_bridge = TaskBridge::default();
        let mut feedback = FeedbackState::default();

        QueryDocumentContext {
            query_session: &mut query_session,
            query_editor: &mut query_editor,
            workspace: &mut workspace,
            agent: &mut agent,
            query_output: &mut query_output,
            schema_explorer: &mut schema_explorer,
            task_bridge: &mut task_bridge,
            feedback: &mut feedback,
            active_connection_id: None,
            active_schema: "public".to_owned(),
        }
        .close_document(0);

        assert_eq!(query_session.documents.len(), 1);
        assert_eq!(
            query_session.active_document().map(|doc| doc.id.as_str()),
            Some("query-2")
        );
        assert!(!query_output.tabs_by_document.contains_key("query-1"));
        assert_eq!(workspace.active_tab, WorkspaceTab::Welcome);
    }
}
