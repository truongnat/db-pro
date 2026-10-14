//! Saved database tasks activity (#206).
use super::*;
use db_pro_core::domain::saved_task::{SavedTask, SavedTaskPayload, SavedTaskRunTrigger, SavedTaskStore};
use uuid::Uuid;

pub(crate) const SAVED_TASKS_STORAGE_KEY: &str = "dbpro.native.saved-tasks-v1";

impl DbProApp {
    pub(crate) fn load_saved_tasks_from_storage(&mut self, storage: &dyn eframe::Storage) {
        if let Some(raw) = storage.get_string(SAVED_TASKS_STORAGE_KEY) {
            if let Ok(store) = serde_json::from_str::<SavedTaskStore>(&raw) {
                self.saved_tasks.store = store;
            }
        }
    }

    pub(crate) fn persist_saved_tasks(&self, storage: &mut dyn eframe::Storage) {
        if let Ok(raw) = serde_json::to_string(&self.saved_tasks.store) {
            storage.set_string(SAVED_TASKS_STORAGE_KEY, raw);
        }
    }

    pub(super) fn draw_tasks_activity(&mut self, ui: &mut egui::Ui) {
        let action = saved_tasks_surface_view::SavedTasksSurfaceContext {
            theme: self.theme,
            saved_tasks: &mut self.saved_tasks,
        }
        .draw(ui);
        if let Some(action) = action {
            self.apply_saved_tasks_surface_action(action);
        }
    }

    fn apply_saved_tasks_surface_action(&mut self, action: saved_tasks_surface_view::SavedTasksSurfaceAction) {
        use saved_tasks_surface_view::SavedTasksSurfaceAction;

        match action {
            SavedTasksSurfaceAction::NewSqlTask => self.begin_new_sql_task(),
            SavedTasksSurfaceAction::NewBackupTask => self.begin_new_backup_task(),
            SavedTasksSurfaceAction::SaveDraft => self.commit_task_draft(),
            SavedTasksSurfaceAction::CancelDraft => self.saved_tasks.draft = None,
            SavedTasksSurfaceAction::RunTask(task_id) => {
                self.run_saved_task(task_id, SavedTaskRunTrigger::Manual);
            }
            SavedTasksSurfaceAction::DisableSchedule(task_id) => {
                if let Err(error) = self.saved_tasks.disable_schedule(task_id) {
                    self.feedback.runtime_message = error;
                }
            }
            SavedTasksSurfaceAction::EnableSchedule(task_id) => self.enable_task_schedule(task_id, 60),
            SavedTasksSurfaceAction::DeleteTask(task_id) => {
                self.saved_tasks.request_delete(task_id);
                self.feedback.runtime_message = "Confirm before deleting the task".into();
            }
            SavedTasksSurfaceAction::ConfirmDelete(task_id) => {
                if self.saved_tasks.pending_delete_task_id == Some(task_id) {
                    self.saved_tasks.confirm_delete();
                }
            }
            SavedTasksSurfaceAction::CancelDelete => {
                self.saved_tasks.pending_delete_task_id = None;
            }
        }
    }

    fn begin_new_sql_task(&mut self) {
        let connection_id = self
            .connection
            .lifecycle
            .active_connection_id()
            .map(str::to_owned)
            .unwrap_or_else(|| self.connection.catalog.get(0).map(|c| c.id.clone()).unwrap_or_default());
        self.saved_tasks.begin_sql_draft(
            connection_id,
            self.query.session.active_text().to_owned(),
            chrono::Utc::now(),
        );
    }

    fn begin_new_backup_task(&mut self) {
        let connection_id = self
            .connection
            .lifecycle
            .active_connection_id()
            .map(str::to_owned)
            .unwrap_or_else(|| self.connection.catalog.get(0).map(|c| c.id.clone()).unwrap_or_default());
        self.saved_tasks.begin_backup_draft(connection_id, chrono::Utc::now());
    }

    fn commit_task_draft(&mut self) {
        match self.saved_tasks.commit_draft(chrono::Utc::now()) {
            Ok(()) => {
                self.feedback.runtime_message = "Saved task".to_owned();
            }
            Err(err) => {
                self.feedback.runtime_message = err;
            }
        }
    }

    pub(crate) fn enable_task_schedule(&mut self, task_id: Uuid, interval_secs: u64) {
        match self
            .saved_tasks
            .enable_schedule(task_id, interval_secs, chrono::Utc::now())
        {
            Ok(()) => {
                self.feedback.runtime_message = format!("Schedule enabled every {interval_secs}s (app must stay open)");
            }
            Err(error) => self.feedback.runtime_message = error,
        }
    }

    pub(crate) fn tick_saved_task_scheduler(&mut self) {
        for (task_id, trigger) in self.saved_tasks.take_due_tasks(chrono::Utc::now()) {
            self.run_saved_task(task_id, trigger);
        }
    }

    pub(crate) fn run_saved_task(&mut self, task_id: Uuid, trigger: SavedTaskRunTrigger) {
        let task = match self.saved_tasks.prepare_run(
            task_id,
            trigger,
            self.preferences.settings.general.confirm_destructive_queries,
        ) {
            saved_task_state::SavedTaskRunPreparation::Ready(task) => *task,
            saved_task_state::SavedTaskRunPreparation::NeedsConfirmation => {
                self.feedback.runtime_message =
                    "Destructive task requires confirmation — check Confirm in the Tasks pane".to_owned();
                return;
            }
            saved_task_state::SavedTaskRunPreparation::Blocked(message) => {
                self.feedback.runtime_message = message;
                return;
            }
            saved_task_state::SavedTaskRunPreparation::NotFound => {
                self.feedback.runtime_message = "Saved task not found".to_owned();
                return;
            }
        };

        let started = chrono::Utc::now();
        match self.dispatch_saved_task_payload(&task) {
            Ok(queued) => {
                self.saved_tasks.note_pending(
                    queued.request_id.0,
                    saved_task_state::PendingTaskRun {
                        task_id,
                        trigger,
                        started_at: started,
                        export_path: queued.export_path,
                        export_format: queued.export_format,
                    },
                );
                self.feedback.runtime_message = "Task is running; waiting for the runtime to finish".to_owned();
            }
            Err(message) => {
                let finished = chrono::Utc::now();
                let (status, message) =
                    self.saved_tasks
                        .record_run(task_id, trigger, started, finished, Err(message));
                if status == db_pro_core::domain::saved_task::SavedTaskRunStatus::Failed
                    && trigger != SavedTaskRunTrigger::Manual
                {
                    self.feedback.runtime_message = format!("Scheduled task failed: {message}");
                } else {
                    self.feedback.runtime_message = message;
                }
            }
        }
    }

    pub(super) fn finish_saved_task(&mut self, request_id: u64, result: Result<String, String>) {
        let Some(pending) = self.saved_tasks.take_pending(request_id) else {
            return;
        };
        let finished = chrono::Utc::now();
        let (status, message) =
            self.saved_tasks
                .record_run(pending.task_id, pending.trigger, pending.started_at, finished, result);
        self.feedback.runtime_message = if status == db_pro_core::domain::saved_task::SavedTaskRunStatus::Failed
            && pending.trigger != SavedTaskRunTrigger::Manual
        {
            format!("Scheduled task failed: {message}")
        } else {
            message
        };
    }

    pub(super) fn complete_saved_task_query(&mut self, request_id: crate::RequestId, result: &crate::UiQueryResult) {
        let Some(pending) = self.saved_tasks.pending_runs.get(&request_id.0) else {
            return;
        };
        let outcome = match pending.export_path.clone() {
            Some(path) => write_task_export(&path, pending.export_format.as_deref().unwrap_or("csv"), result),
            None => Ok(format!("Query finished · {} rows", result.row_count)),
        };
        self.finish_saved_task(request_id.0, outcome);
    }

    fn dispatch_saved_task_payload(&mut self, task: &SavedTask) -> Result<TaskQueued, String> {
        match &task.payload {
            SavedTaskPayload::Sql { sql } => self.dispatch_sql_task(task, sql),
            SavedTaskPayload::Backup {
                output_path,
                custom_format,
            } => self.dispatch_backup_task(task, output_path, *custom_format),
            SavedTaskPayload::Export { table, format } => self.dispatch_export_task(task, table.as_deref(), format),
            SavedTaskPayload::Maintenance { operation, target } => {
                self.dispatch_maintenance_task(task, operation, target.as_deref())
            }
        }
    }

    fn dispatch_sql_task(&mut self, task: &SavedTask, sql: &str) -> Result<TaskQueued, String> {
        if sql.trim().is_empty() {
            return Err("SQL payload is empty".to_owned());
        }
        self.connection
            .lifecycle
            .set_active_connection_id(Some(task.connection_id.clone()));
        self.set_active_query_text(sql);
        self.workspace.active_tab = WorkspaceTab::Query;
        // Task-level confirmation already satisfied destructive policy (#206).
        let version = self.query.session.active_buffer_version();
        let execution_range = (0, sql.len());
        self.send_query_run(
            task.connection_id.clone(),
            sql.to_owned(),
            execution_range,
            version,
            false,
        )
        .map(|request_id| TaskQueued {
            request_id,
            export_path: None,
            export_format: None,
        })
        .ok_or_else(|| self.feedback.runtime_message.clone())
    }

    fn dispatch_backup_task(
        &mut self,
        task: &SavedTask,
        output_path: &str,
        custom_format: bool,
    ) -> Result<TaskQueued, String> {
        if output_path.trim().is_empty() {
            return Err("backup output path is required".to_owned());
        }
        let request_id = self.next_request_id();
        if !self.dispatch_command(UiCommand::Backup {
            request_id,
            connection_id: task.connection_id.clone(),
            output_path: output_path.to_owned(),
            custom_format,
        }) {
            return Err("Runtime worker unavailable".to_owned());
        }
        Ok(TaskQueued {
            request_id,
            export_path: None,
            export_format: None,
        })
    }

    fn dispatch_export_task(&mut self, task: &SavedTask, table: Option<&str>, format: &str) -> Result<TaskQueued, String> {
        let Some(table) = table else {
            return Err("export task needs a table or an active result set".to_owned());
        };
        let fmt = saved_task_sql::validate_export_format(format)?;
        let connection = self
            .connection
            .catalog
            .find(&task.connection_id)
            .ok_or_else(|| "saved task connection is not available".to_owned())?;
        let sql = saved_task_sql::build_export_query(&connection.driver, table)?;
        let path = task_export_path(task.id, fmt);
        let version = self.query.session.active_buffer_version();
        let execution_range = (0, sql.len());
        self.send_query_run(task.connection_id.clone(), sql, execution_range, version, false)
            .map(|request_id| TaskQueued {
                request_id,
                export_path: Some(path),
                export_format: Some(fmt.to_owned()),
            })
            .ok_or_else(|| self.feedback.runtime_message.clone())
    }

    fn dispatch_maintenance_task(
        &mut self,
        task: &SavedTask,
        operation: &str,
        target: Option<&str>,
    ) -> Result<TaskQueued, String> {
        let connection = self
            .connection
            .catalog
            .find(&task.connection_id)
            .ok_or_else(|| "saved task connection is not available".to_owned())?;
        let plan = saved_task_sql::plan_maintenance(&connection.driver, operation, target)?;
        let request_id = self.next_request_id();
        if !self.dispatch_command(UiCommand::MonitoringMaintenance {
            request_id,
            connection_id: task.connection_id.clone(),
            schema: plan.schema,
            table: plan.table,
            action: plan.action,
            confirmed: true,
        }) {
            return Err("Runtime worker unavailable".to_owned());
        }
        Ok(TaskQueued {
            request_id,
            export_path: None,
            export_format: None,
        })
    }
}

struct TaskQueued {
    request_id: crate::RequestId,
    export_path: Option<String>,
    export_format: Option<String>,
}

fn task_export_path(task_id: Uuid, format: &str) -> String {
    std::env::temp_dir()
        .join(format!("db-pro-export-{task_id}.{format}"))
        .to_string_lossy()
        .into_owned()
}

fn write_task_export(path: &str, format: &str, result: &crate::UiQueryResult) -> Result<String, String> {
    let body = match format {
        "json" => super::result_grid_export::format_result_json(result),
        "tsv" => super::result_grid_export::format_result_delimited(result, "\t"),
        _ => super::result_grid_export::format_result_delimited(result, ","),
    };
    super::query_view::write_file_atomically(std::path::Path::new(path), body.as_bytes())
        .map_err(|error| format!("export file was not written: {error}"))?;
    Ok(format!("Exported {} rows to {path}", result.rows.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DbProApp, TaskBridge, UiCell, UiColumn, UiCommand, UiConnectionSummary, UiEvent, UiQueryResult, UiSslMode};
    use db_pro_core::domain::monitoring::MaintenanceAction;
    use db_pro_core::domain::saved_task::{
        SavedTask, SavedTaskPayload, SavedTaskRunStatus, SavedTaskRunTrigger, SavedTaskStore,
    };

    fn postgres_app() -> (DbProApp, std::sync::mpsc::Receiver<UiCommand>, std::sync::mpsc::SyncSender<UiEvent>) {
        let (bridge, command_rx, event_tx) = TaskBridge::with_channels();
        let mut app = DbProApp::with_task_bridge(bridge);
        *app.connection.catalog.connections_mut() = vec![UiConnectionSummary {
            id: "conn-1".to_owned(),
            name: "Local".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: UiSslMode::Disable,
            readonly: false,
            tags: Vec::new(),
            group: None,
            favorite: false,
            environment: "Development".to_owned(),
        }];
        *app.connection.lifecycle.active_connection_id_mut() = Some("conn-1".to_owned());
        app.connection.lifecycle.set_connected(true);
        app.preferences.settings.general.confirm_destructive_queries = false;
        (app, command_rx, event_tx)
    }

    fn task(id: uuid::Uuid, payload: SavedTaskPayload) -> SavedTask {
        let now = chrono::Utc::now();
        SavedTask {
            id,
            name: "task".into(),
            description: String::new(),
            connection_id: "conn-1".into(),
            payload,
            created_at: now,
            updated_at: now,
            last_run: None,
            schedule: None,
        }
    }

    #[test]
    fn export_writes_a_file_and_maintenance_targets_the_table_when_runtime_finishes() {
        let (mut app, command_rx, event_tx) = postgres_app();
        let export_id = uuid::Uuid::new_v4();
        let maintenance_id = uuid::Uuid::new_v4();
        let mut store = SavedTaskStore::new();
        store
            .upsert(task(
                export_id,
                SavedTaskPayload::Export {
                    table: Some("public.orders".into()),
                    format: "csv".into(),
                },
            ))
            .unwrap();
        store
            .upsert(task(
                maintenance_id,
                SavedTaskPayload::Maintenance {
                    operation: "vacuum".into(),
                    target: Some("public.orders".into()),
                },
            ))
            .unwrap();
        app.saved_tasks.store = store;
        let path = task_export_path(export_id, "csv");
        let _ = std::fs::remove_file(&path);

        app.run_saved_task(export_id, SavedTaskRunTrigger::Manual);
        assert!(app.saved_tasks.store.runs.is_empty());
        assert!(app.saved_tasks.store.tasks.iter().all(|task| task.last_run.is_none()));
        assert!(!std::path::Path::new(&path).exists());
        assert!(!app.feedback.runtime_message.contains("Exported"), "{}", app.feedback.runtime_message);

        let UiCommand::RunQuery { request_id, sql, .. } = command_rx.try_recv().expect("export query") else {
            panic!("export did not read the table");
        };
        assert!(sql.contains("orders"), "{sql}");
        assert!(!sql.to_ascii_lowercase().contains("vacuum"), "{sql}");

        event_tx
            .send(UiEvent::QueryCompleted {
                request_id,
                result: UiQueryResult {
                    columns: vec![UiColumn {
                        name: "name".into(),
                        data_type: "text".into(),
                        nullable: true,
                    }],
                    rows: vec![vec![UiCell::Text("Alice".into())]],
                    row_count: 1,
                    duration_ms: 1,
                },
            })
            .unwrap();
        app.apply_runtime_events();

        let body = std::fs::read_to_string(&path).expect("export file");
        assert!(body.contains("name"), "{body}");
        assert!(body.contains("Alice"), "{body}");
        let export_run = app
            .saved_tasks
            .store
            .tasks
            .iter()
            .find(|task| task.id == export_id)
            .and_then(|task| task.last_run.clone())
            .expect("export run");
        assert_eq!(export_run.status, SavedTaskRunStatus::Success);
        assert!(export_run.message.contains(&path), "{}", export_run.message);
        let _ = std::fs::remove_file(&path);

        app.run_saved_task(maintenance_id, SavedTaskRunTrigger::Manual);
        assert!(app
            .saved_tasks
            .store
            .tasks
            .iter()
            .find(|task| task.id == maintenance_id)
            .unwrap()
            .last_run
            .is_none());
        let UiCommand::MonitoringMaintenance {
            request_id,
            schema,
            table,
            action,
            confirmed,
            ..
        } = command_rx.try_recv().expect("maintenance command")
        else {
            panic!("maintenance was not sent to the monitor");
        };
        assert_eq!(schema.as_deref(), Some("public"));
        assert_eq!(table.as_deref(), Some("orders"));
        assert_eq!(action, MaintenanceAction::Vacuum);
        assert!(confirmed);
        assert!(!matches!(command_rx.try_recv(), Ok(UiCommand::RunQuery { .. })));

        event_tx
            .send(UiEvent::MonitoringActionCompleted {
                request_id,
                action: "maintenance".into(),
                backend_id: 0,
                succeeded: true,
            })
            .unwrap();
        app.apply_runtime_events();

        let maintenance_run = app
            .saved_tasks
            .store
            .tasks
            .iter()
            .find(|task| task.id == maintenance_id)
            .and_then(|task| task.last_run.clone())
            .expect("maintenance run");
        assert_eq!(maintenance_run.status, SavedTaskRunStatus::Success);
        assert!(maintenance_run.message.contains("Maintenance finished"), "{}", maintenance_run.message);
        assert!(!maintenance_run.message.contains("1000"), "{}", maintenance_run.message);
        assert!(!maintenance_run.message.to_ascii_lowercase().contains("select"), "{}", maintenance_run.message);
    }
}
