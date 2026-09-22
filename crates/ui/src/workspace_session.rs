//! Named workspace sessions and last-session restore (#222).
//!
//! Query text stays owned by QueryDocument persistence — sessions only
//! reference document ids and shell layout. Secrets and result sets are never stored here.

use super::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(crate) const LAST_SESSION_STORAGE_KEY: &str = "dbpro.native.workspace-last-session-v1";
pub(crate) const NAMED_SESSIONS_STORAGE_KEY: &str = "dbpro.native.workspace-sessions-v1";
pub(crate) const SESSION_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct WorkspaceSession {
    pub version: u32,
    pub id: String,
    pub name: String,
    pub updated_at: String,
    pub activity: String,
    pub active_tab: String,
    pub active_connection_id: Option<String>,
    pub selected_schema: Option<String>,
    pub active_document_id: Option<String>,
    /// Ordered open query document ids (references, not content).
    pub open_document_ids: Vec<String>,
    pub pinned_tables: Vec<String>,
    pub sidebar_open: bool,
    pub agent_open: bool,
    pub sidebar_width: f32,
    pub agent_width: f32,
    pub bottom_panel_open: bool,
    pub bottom_panel_height: f32,
    /// Soft warnings recorded at capture/restore time (missing objects etc.).
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct NamedSessionStore {
    pub version: u32,
    pub sessions: Vec<WorkspaceSession>,
}

impl NamedSessionStore {
    pub fn new() -> Self {
        Self {
            version: SESSION_VERSION,
            sessions: Vec::new(),
        }
    }

    pub fn upsert(&mut self, session: WorkspaceSession) {
        if let Some(existing) = self.sessions.iter_mut().find(|s| s.id == session.id) {
            *existing = session;
        } else {
            self.sessions.push(session);
        }
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.sessions.len();
        self.sessions.retain(|s| s.id != id);
        before != self.sessions.len()
    }

    pub fn get(&self, id: &str) -> Option<&WorkspaceSession> {
        self.sessions.iter().find(|s| s.id == id)
    }
}

impl WorkspaceSession {
    pub fn new_named(name: impl Into<String>) -> Self {
        Self {
            version: SESSION_VERSION,
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            activity: "Explorer".into(),
            active_tab: "Query".into(),
            active_connection_id: None,
            selected_schema: None,
            active_document_id: None,
            open_document_ids: Vec::new(),
            pinned_tables: Vec::new(),
            sidebar_open: true,
            agent_open: false,
            sidebar_width: 260.0,
            agent_width: 360.0,
            bottom_panel_open: false,
            bottom_panel_height: 180.0,
            notes: Vec::new(),
        }
    }

    pub fn migrate(mut self) -> Self {
        if self.version == 0 || self.version > SESSION_VERSION {
            self.version = SESSION_VERSION;
        }
        self
    }
}

fn activity_label(activity: Activity) -> &'static str {
    match activity {
        Activity::Explorer => "Explorer",
        Activity::Files => "Files",
        Activity::Queries => "Queries",
        Activity::Data => "Data",
        Activity::History => "History",
        Activity::Transfers => "Transfers",
        Activity::Monitor => "Monitor",
        Activity::Security => "Security",
        Activity::Settings => "Settings",
        Activity::Diagram => "Diagram",
        Activity::Schema => "Schema",
        Activity::Compare => "Compare",
        Activity::Problems => "Problems",
        Activity::Tasks => "Tasks",
    }
}

fn parse_activity(label: &str) -> Activity {
    match label {
        "Files" => Activity::Files,
        "Queries" => Activity::Queries,
        "Data" => Activity::Data,
        "History" => Activity::History,
        "Transfers" => Activity::Transfers,
        "Monitor" => Activity::Monitor,
        "Security" => Activity::Security,
        "Settings" => Activity::Settings,
        "Diagram" => Activity::Diagram,
        "Schema" => Activity::Schema,
        "Compare" => Activity::Compare,
        "Problems" => Activity::Problems,
        "Tasks" => Activity::Tasks,
        _ => Activity::Explorer,
    }
}

fn tab_label(tab: WorkspaceTab) -> &'static str {
    match tab {
        WorkspaceTab::Welcome => "Welcome",
        WorkspaceTab::Query => "Query",
        WorkspaceTab::Table => "Table",
        WorkspaceTab::SchemaObject => "SchemaObject",
        WorkspaceTab::Diagram => "Diagram",
        WorkspaceTab::SchemaWorkbench => "SchemaWorkbench",
        WorkspaceTab::SchemaCompare => "SchemaCompare",
        WorkspaceTab::ComponentGallery => "ComponentGallery",
    }
}

fn parse_tab(label: &str) -> WorkspaceTab {
    match label {
        "Welcome" => WorkspaceTab::Welcome,
        "Table" => WorkspaceTab::Table,
        "SchemaObject" => WorkspaceTab::SchemaObject,
        "Diagram" => WorkspaceTab::Diagram,
        "SchemaWorkbench" => WorkspaceTab::SchemaWorkbench,
        "SchemaCompare" => WorkspaceTab::SchemaCompare,
        "ComponentGallery" => WorkspaceTab::ComponentGallery,
        _ => WorkspaceTab::Query,
    }
}

pub(crate) struct WorkspaceSessionContext<'a> {
    pub(super) workspace: &'a mut WorkspaceFeatureState,
    pub(super) connection: &'a mut ConnectionFeatureState,
    pub(super) schema_explorer: &'a mut SchemaExplorerState,
    pub(super) query_session_state: &'a mut QuerySessionState,
    pub(super) feedback: &'a mut FeedbackState,
    pub(super) preferences: &'a PreferencesState,
}

impl WorkspaceSessionContext<'_> {
    pub(crate) fn capture(&self, name: impl Into<String>) -> WorkspaceSession {
        let mut session = WorkspaceSession::new_named(name);
        session.activity = activity_label(self.workspace.activity).to_owned();
        session.active_tab = tab_label(self.workspace.active_tab).to_owned();
        session.active_connection_id = self.connection.lifecycle.active_connection_id().map(str::to_owned);
        session.selected_schema = self.schema_explorer.selected_schema.clone();
        session.open_document_ids = self
            .query_session_state
            .documents
            .iter()
            .map(|d| d.id.clone())
            .collect();
        session.active_document_id = self
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .map(|d| d.id.clone());
        session.pinned_tables = self.schema_explorer.pinned_tables.clone();
        session.sidebar_open = self.workspace.sidebar_open;
        session.agent_open = self.workspace.agent_open;
        session.sidebar_width = self.workspace.sidebar_width;
        session.agent_width = self.workspace.agent_width;
        session.bottom_panel_open = self.workspace.bottom_panel_open;
        session.bottom_panel_height = self.workspace.bottom_panel_height;
        session
    }

    pub(crate) fn apply(&mut self, session: &WorkspaceSession) {
        let mut notes = Vec::new();
        self.workspace.activity = parse_activity(&session.activity);
        self.workspace.active_tab = parse_tab(&session.active_tab);
        self.workspace.sidebar_open = session.sidebar_open;
        self.workspace.agent_open = session.agent_open;
        self.workspace.set_sidebar_width(session.sidebar_width);
        self.workspace.set_agent_width(session.agent_width);
        self.workspace.bottom_panel_open = session.bottom_panel_open;
        self.workspace.set_bottom_panel_height(session.bottom_panel_height);
        self.schema_explorer.pinned_tables = session.pinned_tables.clone();
        self.schema_explorer.selected_schema = session.selected_schema.clone();

        if let Some(conn_id) = &session.active_connection_id {
            if self.connection.catalog.iter().any(|c| c.id == *conn_id) {
                self.connection
                    .lifecycle
                    .set_active_connection_id(Some(conn_id.clone()));
            } else {
                self.connection.lifecycle.set_active_connection_id(None);
                notes.push(format!(
                    "Connection `{conn_id}` is missing — left disconnected without crashing"
                ));
            }
        } else {
            self.connection.lifecycle.set_active_connection_id(None);
        }

        if !session.open_document_ids.is_empty() {
            let mut reordered = Vec::new();
            for id in &session.open_document_ids {
                if let Some(doc) = self.query_session_state.documents.iter().find(|d| d.id == *id).cloned() {
                    reordered.push(doc);
                } else {
                    notes.push(format!("Query tab `{id}` was not found in persisted documents"));
                }
            }
            for doc in &self.query_session_state.documents {
                if !reordered.iter().any(|d| d.id == doc.id) {
                    reordered.push(doc.clone());
                }
            }
            if !reordered.is_empty() {
                self.query_session_state.documents = reordered;
            }
        }

        if let Some(active_id) = &session.active_document_id {
            if let Some(idx) = self
                .query_session_state
                .documents
                .iter()
                .position(|d| d.id == *active_id)
            {
                self.query_session_state.active_document_index = idx;
            } else {
                notes.push(format!("Active document `{active_id}` missing — kept current tab"));
            }
        }

        self.schema_explorer.pinned_tables.retain(|t| !t.trim().is_empty());
        self.workspace.sessions.last_restore_notes = notes.clone();
        if notes.is_empty() {
            self.feedback.runtime_message = format!("Restored workspace `{}`", session.name);
        } else {
            self.feedback.runtime_message = format!(
                "Restored workspace `{}` with {} recovery note(s)",
                session.name,
                notes.len()
            );
        }
    }

    pub(crate) fn save_named(&mut self) {
        let name = self.workspace.sessions.name_draft.trim();
        let name = if name.is_empty() {
            format!("Workspace {}", chrono::Utc::now().format("%Y-%m-%d %H:%M"))
        } else {
            name.to_owned()
        };
        let session = self.capture(name);
        self.workspace.sessions.name_draft.clear();
        self.workspace.sessions.upsert(session);
        self.feedback.runtime_message = "Named workspace session saved".to_owned();
    }

    pub(crate) fn restore_named(&mut self, id: &str) {
        let Some(session) = self.workspace.sessions.store.get(id).cloned() else {
            self.feedback.runtime_message = "Named session not found".to_owned();
            return;
        };
        self.apply(&session);
    }

    pub(crate) fn duplicate_named(&mut self, id: &str) {
        if self.workspace.sessions.duplicate(id).is_none() {
            self.feedback.runtime_message = "Named session not found".to_owned();
            return;
        }
        self.feedback.runtime_message = "Duplicated workspace session".to_owned();
    }

    pub(crate) fn persist(&self, storage: &mut dyn eframe::Storage) {
        let last = self.capture("Last session");
        if let Ok(raw) = serde_json::to_string(&last) {
            storage.set_string(LAST_SESSION_STORAGE_KEY, raw);
        }
        self.workspace.sessions.persist_named_store(storage);
    }

    pub(crate) fn load_named_sessions(&mut self, storage: &dyn eframe::Storage) {
        self.workspace.sessions.load_named_store(storage);
    }

    pub(crate) fn restore_last(&mut self, storage: &dyn eframe::Storage) {
        if !self.preferences.settings.general.restore_tabs_on_startup {
            return;
        }
        if let Some(raw) = storage.get_string(LAST_SESSION_STORAGE_KEY) {
            if let Ok(session) = serde_json::from_str::<WorkspaceSession>(&raw) {
                self.apply(&session.migrate());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_round_trip_json_preserves_document_refs_only() {
        let mut session = WorkspaceSession::new_named("Focus");
        session.open_document_ids = vec!["doc-1".into(), "doc-2".into()];
        session.active_connection_id = Some("conn-1".into());
        let json = serde_json::to_string(&session).unwrap();
        assert!(!json.to_lowercase().contains("password"));
        assert!(!json.contains("SELECT"));
        let loaded: WorkspaceSession = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.open_document_ids, vec!["doc-1", "doc-2"]);
        assert_eq!(loaded.name, "Focus");
    }

    #[test]
    fn named_store_upsert_and_remove() {
        let mut store = NamedSessionStore::new();
        let session = WorkspaceSession::new_named("A");
        let id = session.id.clone();
        store.upsert(session);
        assert!(store.get(&id).is_some());
        assert!(store.remove(&id));
        assert!(store.get(&id).is_none());
    }
}
