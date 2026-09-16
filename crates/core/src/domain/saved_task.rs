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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SavedTaskRunTrigger {
    #[default]
    Manual,
    Scheduled,
    Retry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MissedRunPolicy {
    /// Skip all missed intervals; schedule from "now".
    #[default]
    SkipMissed,
    /// Run once for the backlog, then advance to the next future slot.
    RunOnce,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskSchedule {
    pub enabled: bool,
    /// Simple interval schedule (seconds). Cron-like labels may be stored in `expression`.
    pub interval_secs: u64,
    /// Optional human/cron-like expression for display (e.g. `every 5m`).
    #[serde(default)]
    pub expression: String,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub missed_run_policy: MissedRunPolicy,
    /// Max automatic retries after a failed scheduled run (0 = no retry).
    #[serde(default)]
    pub max_retries: u32,
    #[serde(default)]
    pub retry_count: u32,
    /// Explicit opt-in required before a destructive task can be scheduled.
    #[serde(default)]
    pub allow_destructive: bool,
}

impl TaskSchedule {
    pub fn every_secs(interval_secs: u64) -> Self {
        let interval_secs = interval_secs.max(1);
        Self {
            enabled: true,
            interval_secs,
            expression: format!("every {interval_secs}s"),
            next_run_at: Some(chrono::Utc::now() + chrono::Duration::seconds(interval_secs as i64)),
            last_scheduled_at: None,
            missed_run_policy: MissedRunPolicy::SkipMissed,
            max_retries: 0,
            retry_count: 0,
            allow_destructive: false,
        }
    }
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
    #[serde(default)]
    pub trigger: SavedTaskRunTrigger,
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
    #[serde(default)]
    pub schedule: Option<TaskSchedule>,
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
        if let Some(schedule) = &task.schedule {
            if schedule.enabled && task.payload.is_destructive() && !schedule.allow_destructive {
                return Err("destructive tasks cannot be scheduled without explicit allow_destructive".to_owned());
            }
            if schedule.enabled && schedule.interval_secs == 0 {
                return Err("schedule interval_secs must be >= 1".to_owned());
            }
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

    pub fn set_schedule_enabled(&mut self, id: &Uuid, enabled: bool) -> Result<(), String> {
        let Some(task) = self.tasks.iter_mut().find(|t| &t.id == id) else {
            return Err("task not found".to_owned());
        };
        let Some(schedule) = task.schedule.as_mut() else {
            return Err("task has no schedule".to_owned());
        };
        if enabled && task.payload.is_destructive() && !schedule.allow_destructive {
            return Err("destructive schedule blocked".to_owned());
        }
        schedule.enabled = enabled;
        if enabled && schedule.next_run_at.is_none() {
            schedule.next_run_at = Some(chrono::Utc::now() + chrono::Duration::seconds(schedule.interval_secs as i64));
        }
        task.updated_at = chrono::Utc::now();
        Ok(())
    }

    /// In-app scheduler tick. Returns task ids that should run now.
    ///
    /// Semantics: only while the app process is alive — there is no background daemon.
    /// Missed runs while the app was closed follow [`MissedRunPolicy`].
    pub fn due_scheduled_task_ids(&mut self, now: chrono::DateTime<chrono::Utc>) -> Vec<Uuid> {
        let mut due = Vec::new();
        for task in &mut self.tasks {
            let Some(schedule) = task.schedule.as_mut() else {
                continue;
            };
            if !schedule.enabled {
                continue;
            }
            if task.payload.is_destructive() && !schedule.allow_destructive {
                schedule.enabled = false;
                continue;
            }
            let Some(next) = schedule.next_run_at else {
                schedule.next_run_at = Some(now + chrono::Duration::seconds(schedule.interval_secs as i64));
                continue;
            };
            if next > now {
                continue;
            }
            // Deduplicate: do not fire twice for the same scheduled slot.
            if schedule.last_scheduled_at == Some(next) {
                schedule.next_run_at = Some(next + chrono::Duration::seconds(schedule.interval_secs as i64));
                continue;
            }
            match schedule.missed_run_policy {
                MissedRunPolicy::SkipMissed => {
                    // Advance from now so closed-app gaps are not backfilled.
                    schedule.last_scheduled_at = Some(next);
                    schedule.next_run_at = Some(now + chrono::Duration::seconds(schedule.interval_secs as i64));
                    due.push(task.id);
                }
                MissedRunPolicy::RunOnce => {
                    schedule.last_scheduled_at = Some(next);
                    schedule.next_run_at = Some(now + chrono::Duration::seconds(schedule.interval_secs as i64));
                    due.push(task.id);
                }
            }
        }
        due
    }

    pub fn record_run(&mut self, run: SavedTaskRun) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == run.task_id) {
            if let Some(schedule) = task.schedule.as_mut() {
                if run.trigger == SavedTaskRunTrigger::Scheduled || run.trigger == SavedTaskRunTrigger::Retry {
                    if run.status == SavedTaskRunStatus::Failed && schedule.retry_count < schedule.max_retries {
                        schedule.retry_count += 1;
                        // Retry soon while the app is still active.
                        schedule.next_run_at = Some(run.finished_at + chrono::Duration::seconds(5));
                    } else if run.status == SavedTaskRunStatus::Success {
                        schedule.retry_count = 0;
                    }
                }
            }
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
            schedule: None,
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
            trigger: SavedTaskRunTrigger::Manual,
        };
        store.record_run(run);
        assert_eq!(
            store.tasks[0].last_run.as_ref().unwrap().status,
            SavedTaskRunStatus::Success
        );
        assert_eq!(store.runs_for(&id).len(), 1);
    }

    #[test]
    fn destructive_schedule_requires_explicit_allow() {
        let mut store = SavedTaskStore::new();
        let id = Uuid::new_v4();
        let err = store
            .upsert(SavedTask {
                id,
                name: "Wipe".into(),
                description: String::new(),
                connection_id: "conn-1".into(),
                payload: SavedTaskPayload::Sql {
                    sql: "DELETE FROM t".into(),
                },
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                last_run: None,
                schedule: Some(TaskSchedule::every_secs(60)),
            })
            .unwrap_err();
        assert!(err.contains("allow_destructive"));
    }

    #[test]
    fn due_schedule_fires_once_and_advances() {
        let mut store = SavedTaskStore::new();
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let mut schedule = TaskSchedule::every_secs(30);
        schedule.next_run_at = Some(now - chrono::Duration::seconds(1));
        store
            .upsert(SavedTask {
                id,
                name: "Ping".into(),
                description: String::new(),
                connection_id: "conn-1".into(),
                payload: SavedTaskPayload::Sql { sql: "SELECT 1".into() },
                created_at: now,
                updated_at: now,
                last_run: None,
                schedule: Some(schedule),
            })
            .unwrap();
        let due = store.due_scheduled_task_ids(now);
        assert_eq!(due, vec![id]);
        let due_again = store.due_scheduled_task_ids(now);
        assert!(due_again.is_empty());
        let next = store.tasks[0].schedule.as_ref().unwrap().next_run_at.unwrap();
        assert!(next > now);
    }
}
