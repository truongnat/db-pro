use chrono::{DateTime, Utc};
use db_pro_core::domain::saved_task::{SavedTask, SavedTaskPayload, SavedTaskRunTrigger, SavedTaskStore, TaskSchedule};
use uuid::Uuid;

/// UI state for saved-task editing, scheduling, and confirmation flows.
#[derive(Debug)]
pub(crate) struct SavedTaskState {
    pub(super) store: SavedTaskStore,
    pub(super) draft: Option<SavedTask>,
    pub(super) dirty: bool,
    pub(super) confirm_destructive: bool,
    pub(super) pending_destructive_task_id: Option<Uuid>,
}

impl Default for SavedTaskState {
    fn default() -> Self {
        Self {
            store: SavedTaskStore::new(),
            draft: None,
            dirty: false,
            confirm_destructive: false,
            pending_destructive_task_id: None,
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

    pub(crate) fn delete_task(&mut self, task_id: Uuid) -> bool {
        let removed = self.store.remove(&task_id);
        if removed {
            self.dirty = true;
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::SavedTaskState;
    use chrono::Utc;

    #[test]
    fn default_state_is_empty_and_requires_no_confirmation() {
        let state = SavedTaskState::default();

        assert!(state.store.tasks.is_empty());
        assert!(state.draft.is_none());
        assert!(!state.dirty);
        assert!(!state.confirm_destructive);
        assert!(state.pending_destructive_task_id.is_none());
    }

    #[test]
    fn draft_lifecycle_is_owned_by_saved_task_state() {
        let now = Utc::now();
        let mut state = SavedTaskState::default();

        state.begin_sql_draft("connection-1".to_owned(), "select 1".to_owned(), now);
        assert!(state.draft.is_some());
        state.commit_draft(now).expect("valid draft should save");

        assert!(state.draft.is_none());
        assert!(state.dirty);
        assert_eq!(state.store.tasks.len(), 1);
    }

    #[test]
    fn destructive_tasks_cannot_be_scheduled_by_state_policy() {
        let now = Utc::now();
        let mut state = SavedTaskState::default();
        state.begin_sql_draft("connection-1".to_owned(), "drop table users".to_owned(), now);
        state.commit_draft(now).expect("valid draft should save");
        let task_id = state.store.tasks[0].id;

        let error = state
            .enable_schedule(task_id, 60, now)
            .expect_err("destructive task must be blocked");
        assert!(error.contains("Destructive tasks"));
    }
}
