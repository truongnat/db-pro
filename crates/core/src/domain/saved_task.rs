//! Saved database tasks and run history (#206).
//!
//! Payloads reference connection IDs only — never embed secrets.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SavedTaskKind {
    Sql,
    Export,
    Backup,
    Maintenance,
}

impl SavedTaskKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sql => "SQL",
            Self::Export => "Export",
            Self::Backup => "Backup",
            Self::Maintenance => "Maintenance",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SavedTaskPayload {
    Sql {
        sql: String,
    },
    Export {
        /// Qualified table name or empty for active query export.
        table: Option<String>,
        format: String,
    },
    Backup {
        output_path: String,
        custom_format: bool,
    },
    Maintenance {
        /// Provider-aware maintenance verb, e.g. `vacuum`, `analyze`.
        operation: String,
        target: Option<String>,
    },
}

impl SavedTaskPayload {
    pub fn kind(&self) -> SavedTaskKind {
        match self {
            Self::Sql { .. } => SavedTaskKind::Sql,
            Self::Export { .. } => SavedTaskKind::Export,
            Self::Backup { .. } => SavedTaskKind::Backup,
            Self::Maintenance { .. } => SavedTaskKind::Maintenance,
        }
    }

    /// True when running this payload may mutate data or schema.
    pub fn is_destructive(&self) -> bool {
        match self {
            Self::Sql { sql } => {
                let lower = sql.to_ascii_lowercase();
                [
                    "insert", "update", "delete", "drop", "truncate", "alter", "create", "grant", "revoke",
                ]
                .iter()
                .any(|kw| {
                    lower
                        .split(|c: char| c.is_whitespace() || c == ';' || c == '(')
                        .any(|tok| tok == *kw)
                })
            }
            Self::Export { .. } => false,
            Self::Backup { .. } => false,
            Self::Maintenance { operation, .. } => {
                let op = operation.to_ascii_lowercase();
                op.contains("vacuum") || op.contains("reindex") || op.contains("cluster")
            }
        }
    }

    /// Reject payloads that embed obvious secrets.
    pub fn validate_no_embedded_secrets(&self) -> Result<(), String> {
        let blob = serde_json::to_string(self).unwrap_or_default().to_ascii_lowercase();
        for needle in ["password=", "password\":", "secret=", "api_key", "private_key"] {
            if blob.contains(needle) {
                return Err("saved task payloads must not embed secrets".to_owned());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SavedTaskRunStatus {
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedTaskRun {
    pub id: Uuid,
    pub task_id: Uuid,
    pub status: SavedTaskRunStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedTask {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    /// Connection id reference — secrets stay in the secret store.
    pub connection_id: String,
    pub payload: SavedTaskPayload,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_run: Option<SavedTaskRun>,
}

impl SavedTask {
    pub fn kind(&self) -> SavedTaskKind {
        self.payload.kind()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SavedTaskStore {
    pub version: u32,
    pub tasks: Vec<SavedTask>,
    pub runs: Vec<SavedTaskRun>,
}

impl SavedTaskStore {
    pub const CURRENT_VERSION: u32 = 1;
    pub const MAX_RUNS: usize = 200;

    pub fn new() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            tasks: Vec::new(),
            runs: Vec::new(),
        }
    }

    pub fn upsert(&mut self, task: SavedTask) -> Result<(), String> {
        task.payload.validate_no_embedded_secrets()?;
        if task.name.trim().is_empty() {
            return Err("task name is required".to_owned());
        }
        if task.connection_id.trim().is_empty() {
            return Err("connection_id is required".to_owned());
        }
        if let Some(existing) = self.tasks.iter_mut().find(|t| t.id == task.id) {
            *existing = task;
        } else {
            self.tasks.push(task);
        }
        Ok(())
    }

    pub fn remove(&mut self, id: &Uuid) -> bool {
        let before = self.tasks.len();
        self.tasks.retain(|t| &t.id != id);
        self.runs.retain(|r| &r.task_id != id);
        before != self.tasks.len()
    }

    pub fn record_run(&mut self, run: SavedTaskRun) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == run.task_id) {
            task.last_run = Some(run.clone());
            task.updated_at = run.finished_at;
        }
        self.runs.insert(0, run);
        if self.runs.len() > Self::MAX_RUNS {
            self.runs.truncate(Self::MAX_RUNS);
        }
    }

    pub fn runs_for(&self, task_id: &Uuid) -> Vec<&SavedTaskRun> {
        self.runs.iter().filter(|r| &r.task_id == task_id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_embedded_password_in_sql_payload() {
        let payload = SavedTaskPayload::Sql {
            sql: "SELECT 1 -- password=secret".into(),
        };
        assert!(payload.validate_no_embedded_secrets().is_err());
    }

    #[test]
    fn destructive_sql_detected() {
        let payload = SavedTaskPayload::Sql {
            sql: "DELETE FROM users WHERE id = 1".into(),
        };
        assert!(payload.is_destructive());
        let select = SavedTaskPayload::Sql {
            sql: "SELECT * FROM users".into(),
        };
        assert!(!select.is_destructive());
    }

    #[test]
    fn store_upsert_and_run_history() {
        let mut store = SavedTaskStore::new();
        let id = Uuid::new_v4();
        let task = SavedTask {
            id,
            name: "Nightly export".into(),
            description: "CSV".into(),
            connection_id: "conn-1".into(),
            payload: SavedTaskPayload::Export {
                table: Some("public.orders".into()),
                format: "csv".into(),
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_run: None,
        };
        store.upsert(task).unwrap();
        let run = SavedTaskRun {
            id: Uuid::new_v4(),
            task_id: id,
            status: SavedTaskRunStatus::Success,
            started_at: chrono::Utc::now(),
            finished_at: chrono::Utc::now(),
            duration_ms: 12,
            message: "ok".into(),
        };
        store.record_run(run);
        assert_eq!(
            store.tasks[0].last_run.as_ref().unwrap().status,
            SavedTaskRunStatus::Success
        );
        assert_eq!(store.runs_for(&id).len(), 1);
    }
}
