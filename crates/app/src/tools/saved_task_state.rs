use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use db_pro_core::domain::saved_task::{
    SavedTask, SavedTaskPayload, SavedTaskRun, SavedTaskRunStatus, SavedTaskRunTrigger, SavedTaskStore, TaskSchedule,
};
use uuid::Uuid;

/// Floor for scheduler-driven repaints (#545): a due/imminent slot still gets
/// the old 50 ms cadence so the tick fires promptly.
pub(crate) const SCHEDULER_REPAINT_FLOOR: Duration = Duration::from_millis(50);
/// Ceiling for a single scheduler sleep (#545). `next_run_at` is wall clock
/// while egui repaint delays are monotonic, so an unbounded sleep could strand
/// a slot past a forward clock jump; waking at most every 5 minutes re-reads
/// the deadline. An idle app with one hourly task drops from ~72 000 wakeups
/// per hour (flat 50 ms) to ~12.
pub(crate) const SCHEDULER_REPAINT_CAP: Duration = Duration::from_secs(300);

/// Repaint delay the saved-task scheduler asks for (#545), as a pure decision
/// over the enabled schedules' deadlines so the cadence matrix is testable
/// without wall-clock timing.
///
/// * No enabled schedule → `None`: the scheduler must not force repaints.
/// * Deadline missing/unassigned or within the floor → [`SCHEDULER_REPAINT_FLOOR`]
///   — the next tick assigns the slot and fires due runs immediately.
/// * Otherwise → the real remaining time, capped at [`SCHEDULER_REPAINT_CAP`].
pub(crate) fn scheduler_repaint_delay(deadlines: &[Option<DateTime<Utc>>], now: DateTime<Utc>) -> Option<Duration> {
    let remaining_ms = deadlines
        .iter()
        .map(|deadline| match deadline {
            // Tick fills an unassigned slot on the next frame: treat as due.
            None => 0,
            Some(next) => (*next - now).num_milliseconds(),
        })
        .min()?;
    let clamped = remaining_ms.clamp(
        SCHEDULER_REPAINT_FLOOR.as_millis() as i64,
        SCHEDULER_REPAINT_CAP.as_millis() as i64,
    );
    Some(Duration::from_millis(clamped.max(0) as u64))
}

#[derive(Debug)]
pub(super) struct PendingTaskRun {
    pub(super) task_id: Uuid,
    pub(super) trigger: SavedTaskRunTrigger,
    pub(super) started_at: DateTime<Utc>,
    pub(super) export_path: Option<String>,
    pub(super) export_format: Option<String>,
}

/// UI state for saved-task editing, scheduling, and confirmation flows.
#[derive(Debug)]
pub(crate) struct SavedTaskState {
    pub(super) store: SavedTaskStore,
    pub(super) draft: Option<SavedTask>,
    pub(super) dirty: bool,
    pub(super) confirm_destructive: bool,
    pub(super) pending_destructive_task_id: Option<Uuid>,
    pub(super) pending_delete_task_id: Option<Uuid>,
    pub(super) pending_runs: HashMap<u64, PendingTaskRun>,
}

pub(crate) enum SavedTaskRunPreparation {
    Ready(Box<SavedTask>),
    NeedsConfirmation,
    Blocked(String),
    NotFound,
}

impl Default for SavedTaskState {
    fn default() -> Self {
        Self {
            store: SavedTaskStore::new(),
            draft: None,
            dirty: false,
            confirm_destructive: false,
            pending_destructive_task_id: None,
            pending_delete_task_id: None,
            pending_runs: HashMap::new(),
        }
    }
}

impl SavedTaskState {
    pub(crate) fn begin_sql_draft(&mut self, connection_id: String, sql: String, now: DateTime<Utc>) {
        self.draft = Some(SavedTask {
            id: Uuid::new_v4(),
            name: "SQL task".into(),
            description: String::new(),
            connection_id,
            payload: SavedTaskPayload::Sql { sql },
            created_at: now,
            updated_at: now,
            last_run: None,
            schedule: None,
        });
    }

    pub(crate) fn begin_backup_draft(&mut self, connection_id: String, now: DateTime<Utc>) {
        self.draft = Some(SavedTask {
            id: Uuid::new_v4(),
            name: "Backup".into(),
            description: String::new(),
            connection_id,
            payload: SavedTaskPayload::Backup {
                output_path: String::new(),
                custom_format: false,
            },
            created_at: now,
            updated_at: now,
            last_run: None,
            schedule: None,
        });
    }

    pub(crate) fn commit_draft(&mut self, now: DateTime<Utc>) -> Result<(), String> {
        let Some(mut draft) = self.draft.take() else {
            return Ok(());
        };
        draft.updated_at = now;
        self.store.upsert(draft)?;
        self.dirty = true;
        Ok(())
    }

    pub(crate) fn enable_schedule(
        &mut self,
        task_id: Uuid,
        interval_secs: u64,
        now: DateTime<Utc>,
    ) -> Result<(), String> {
        let Some(task) = self.store.tasks.iter_mut().find(|task| task.id == task_id) else {
            return Err("Saved task not found".to_owned());
        };
        if task.payload.is_destructive() {
            return Err("Destructive tasks cannot be scheduled without an explicit policy allow".to_owned());
        }
        let mut schedule = TaskSchedule::every_secs(interval_secs);
        schedule.enabled = true;
        task.schedule = Some(schedule);
        task.updated_at = now;
        self.dirty = true;
        Ok(())
    }

    pub(crate) fn disable_schedule(&mut self, task_id: Uuid) -> Result<(), String> {
        self.store.set_schedule_enabled(&task_id, false)?;
        self.dirty = true;
        Ok(())
    }

    pub(crate) fn take_due_tasks(&mut self, now: DateTime<Utc>) -> Vec<(Uuid, SavedTaskRunTrigger)> {
        let due = self.store.due_scheduled_task_ids(now);
        if due.is_empty() {
            return Vec::new();
        }
        self.dirty = true;
        due.into_iter()
            .map(|task_id| {
                let trigger = if self
                    .store
                    .tasks
                    .iter()
                    .find(|task| task.id == task_id)
                    .and_then(|task| task.schedule.as_ref())
                    .is_some_and(|schedule| schedule.retry_count > 0)
                {
                    SavedTaskRunTrigger::Retry
                } else {
                    SavedTaskRunTrigger::Scheduled
                };
                (task_id, trigger)
            })
            .collect()
    }

    /// Whether a dispatched saved task is still awaiting its runtime result —
    /// real in-flight work, so the frame cadence stays fast until it lands (#545).
    pub(crate) fn has_inflight_runs(&self) -> bool {
        !self.pending_runs.is_empty()
    }

    /// `next_run_at` of every enabled schedule (#545). `Some(None)` entries are
    /// enabled tasks whose slot the store has not assigned yet; the caller
    /// treats them as due.
    pub(crate) fn enabled_schedule_deadlines(&self) -> Vec<Option<DateTime<Utc>>> {
        self.store
            .tasks
            .iter()
            .filter(|task| task.schedule.as_ref().is_some_and(|schedule| schedule.enabled))
            .map(|task| task.schedule.as_ref().and_then(|schedule| schedule.next_run_at))
            .collect()
    }

    pub(super) fn request_delete(&mut self, task_id: Uuid) {
        self.pending_delete_task_id = Some(task_id);
    }

    pub(super) fn confirm_delete(&mut self) -> bool {
        let Some(task_id) = self.pending_delete_task_id.take() else {
            return false;
        };
        self.delete_task(task_id)
    }

    pub(crate) fn delete_task(&mut self, task_id: Uuid) -> bool {
        let removed = self.store.remove(&task_id);
        if removed {
            self.dirty = true;
        }
        removed
    }

    pub(crate) fn prepare_run(
        &mut self,
        task_id: Uuid,
        trigger: SavedTaskRunTrigger,
        confirm_destructive_queries: bool,
    ) -> SavedTaskRunPreparation {
        let Some(task) = self.store.tasks.iter().find(|task| task.id == task_id).cloned() else {
            return SavedTaskRunPreparation::NotFound;
        };
        if trigger == SavedTaskRunTrigger::Manual
            && task.payload.is_destructive()
            && confirm_destructive_queries
            && !self.confirm_destructive
        {
            self.pending_destructive_task_id = Some(task_id);
            return SavedTaskRunPreparation::NeedsConfirmation;
        }
        if trigger != SavedTaskRunTrigger::Manual
            && task.payload.is_destructive()
            && !task
                .schedule
                .as_ref()
                .is_some_and(|schedule| schedule.allow_destructive)
        {
            return SavedTaskRunPreparation::Blocked("Scheduled destructive task blocked by policy".to_owned());
        }
        self.pending_destructive_task_id = None;
        self.confirm_destructive = false;
        SavedTaskRunPreparation::Ready(Box::new(task))
    }

    pub(crate) fn record_run(
        &mut self,
        task_id: Uuid,
        trigger: SavedTaskRunTrigger,
        started_at: DateTime<Utc>,
        finished_at: DateTime<Utc>,
        result: Result<String, String>,
    ) -> (SavedTaskRunStatus, String) {
        let (status, message) = match result {
            Ok(message) => (SavedTaskRunStatus::Success, message),
            Err(message) => (SavedTaskRunStatus::Failed, message),
        };
        let duration_ms = (finished_at - started_at).num_milliseconds().max(0) as u64;
        self.store.record_run(SavedTaskRun {
            id: Uuid::new_v4(),
            task_id,
            status,
            started_at,
            finished_at,
            duration_ms,
            message: message.clone(),
            trigger,
        });
        self.dirty = true;
        (status, message)
    }

    pub(super) fn note_pending(&mut self, request_id: u64, pending: PendingTaskRun) {
        self.pending_runs.insert(request_id, pending);
    }

    pub(super) fn take_pending(&mut self, request_id: u64) -> Option<PendingTaskRun> {
        self.pending_runs.remove(&request_id)
    }
}

#[cfg(test)]
#[path = "tests/saved_task_state_tests.rs"]
mod tests;

