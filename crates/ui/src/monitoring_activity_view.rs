use super::command_dispatch::RuntimeCommandDispatcher;
use super::monitoring_state::MonitoringState;
use super::*;

pub(super) enum MonitoringActivityEffect {
    OpenQuery(String),
}

pub(super) struct MonitoringActivityContext<'a, 'bridge> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut MonitoringState,
    pub(super) connected: bool,
    pub(super) driver: &'a str,
    pub(super) connection_name: &'a str,
    pub(super) connection_id: Option<&'a str>,
    pub(super) command_dispatcher: &'a mut RuntimeCommandDispatcher<'bridge>,
    pub(super) feedback: &'a mut FeedbackState,
}

impl MonitoringActivityContext<'_, '_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Vec<MonitoringActivityEffect> {
        section_label(ui, "MONITOR", self.theme);
        ui.add_space(SPACE_SM);

        let actions = monitoring_surface_view::MonitoringSurfaceContext {
            theme: self.theme,
            connected: self.connected,
            driver: self.driver,
            connection_name: self.connection_name,
            state: self.state,
        }
        .draw(ui);
        let mut effects = Vec::new();
        for action in actions {
            match action {
                monitoring_surface_view::MonitoringSurfaceAction::Refresh => {
                    self.request_monitoring_snapshot();
                }
                monitoring_surface_view::MonitoringSurfaceAction::Sessions(action) => {
                    self.apply_monitoring_sessions_action(action, &mut effects);
                }
                monitoring_surface_view::MonitoringSurfaceAction::Workload(action) => {
                    self.apply_monitoring_workload_action(action, &mut effects);
                }
            }
        }

        self.request_monitoring_poll_if_due();

        let confirmation_actions = monitoring_confirmation_view::MonitoringConfirmationContext {
            theme: self.theme,
            terminate_backend_id: self.state.monitoring_terminate_confirm,
            cancel_backend_id: self.state.monitoring_cancel_confirm,
            maintenance_action: self.state.monitoring_maintenance_confirm,
            reset_statistics: self.state.monitoring_reset_stats_confirm,
        }
        .draw(ui.ctx());
        self.apply_monitoring_confirmation_actions(confirmation_actions);
        effects
    }

    fn request_monitoring_poll_if_due(&mut self) {
        if !self.connected || !self.state.monitoring_poll {
            return;
        }
        let due = self
            .state
            .monitoring_last_poll
            .map(|time| time.elapsed() >= std::time::Duration::from_secs(5))
            .unwrap_or(true);
        if due {
            self.request_monitoring_snapshot();
        }
    }

    fn apply_monitoring_confirmation_actions(
        &mut self,
        actions: Vec<monitoring_confirmation_view::MonitoringConfirmationAction>,
    ) {
        for action in actions {
            match action {
                monitoring_confirmation_view::MonitoringConfirmationAction::ConfirmTerminate(backend_id) => {
                    if self.terminate_monitoring_backend(backend_id) {
                        self.state.monitoring_terminate_confirm = None;
                    }
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::CancelTerminate => {
                    self.state.monitoring_terminate_confirm = None;
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::ConfirmCancelBackend(backend_id) => {
                    self.cancel_monitoring_backend(backend_id);
                    self.state.monitoring_cancel_confirm = None;
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::DismissCancelBackend => {
                    self.state.monitoring_cancel_confirm = None;
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::ConfirmMaintenance(action) => {
                    if self.run_monitoring_maintenance(action) {
                        self.state.monitoring_maintenance_confirm = None;
                    }
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::CancelMaintenance => {
                    self.state.monitoring_maintenance_confirm = None;
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::ConfirmResetStatistics => {
                    if self.reset_monitoring_statistics() {
                        self.state.monitoring_reset_stats_confirm = false;
                    }
                }
                monitoring_confirmation_view::MonitoringConfirmationAction::CancelResetStatistics => {
                    self.state.monitoring_reset_stats_confirm = false;
                }
            }
        }
    }

    fn terminate_monitoring_backend(&mut self, backend_id: i64) -> bool {
        let Some(connection_id) = self.connection_id else {
            return false;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(terminate_backend_command(
            request_id,
            connection_id.to_owned(),
            backend_id,
        ))
    }

    fn run_monitoring_maintenance(&mut self, action: db_pro_core::domain::monitoring::MaintenanceAction) -> bool {
        let Some(connection_id) = self.connection_id else {
            return false;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(maintenance_command(
            self.state,
            request_id,
            connection_id.to_owned(),
            action,
        ))
    }

    fn reset_monitoring_statistics(&mut self) -> bool {
        let Some(connection_id) = self.connection_id else {
            return false;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(reset_statements_command(request_id, connection_id.to_owned()))
    }

    fn apply_monitoring_sessions_action(
        &mut self,
        action: monitoring_sessions_view::MonitoringSessionsAction,
        effects: &mut Vec<MonitoringActivityEffect>,
    ) {
        match action {
            monitoring_sessions_view::MonitoringSessionsAction::OpenSql(query) => {
                effects.push(MonitoringActivityEffect::OpenQuery(query));
            }
            monitoring_sessions_view::MonitoringSessionsAction::Cancel(backend_id) => {
                self.state.monitoring_cancel_confirm = Some(backend_id);
            }
            monitoring_sessions_view::MonitoringSessionsAction::RequestTerminate(backend_id) => {
                self.state.monitoring_terminate_confirm = Some(backend_id);
            }
        }
    }

    fn apply_monitoring_workload_action(
        &mut self,
        action: monitoring_workload_view::MonitoringWorkloadAction,
        effects: &mut Vec<MonitoringActivityEffect>,
    ) {
        match action {
            monitoring_workload_view::MonitoringWorkloadAction::SortChanged(sort) => {
                self.state.monitoring_stat_sort = sort;
                self.request_monitoring_workload();
            }
            monitoring_workload_view::MonitoringWorkloadAction::ResetStatistics => {
                self.state.monitoring_reset_stats_confirm = true;
            }
            monitoring_workload_view::MonitoringWorkloadAction::OpenSql(query) => {
                effects.push(MonitoringActivityEffect::OpenQuery(query));
            }
        }
    }

    fn cancel_monitoring_backend(&mut self, backend_id: i64) {
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(cancel_backend_command(
            request_id,
            connection_id.to_owned(),
            backend_id,
        ));
    }

    fn request_monitoring_workload(&mut self) {
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(workload_command(
            self.state,
            request_id,
            connection_id.to_owned(),
        ));
    }

    fn request_monitoring_snapshot(&mut self) {
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        if self.dispatch(snapshot_command(
            request_id,
            connection_id.to_owned(),
            self.state.monitoring_stat_sort,
        )) {
            self.state.monitoring_last_poll = Some(std::time::Instant::now());
        }
    }

    fn dispatch(&mut self, command: UiCommand) -> bool {
        self.command_dispatcher.dispatch(command, self.feedback)
    }
}

pub(super) fn snapshot_command(
    request_id: RequestId,
    connection_id: String,
    sort: db_pro_core::domain::monitoring::StatStatementSort,
) -> UiCommand {
    UiCommand::MonitoringSnapshot {
        request_id,
        connection_id,
        sort,
    }
}

fn workload_command(state: &MonitoringState, request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::MonitoringStatStatements {
        request_id,
        connection_id,
        sort: state.monitoring_stat_sort,
        limit: 100,
    }
}

fn cancel_backend_command(request_id: RequestId, connection_id: String, backend_id: i64) -> UiCommand {
    UiCommand::MonitoringCancelBackend {
        request_id,
        connection_id,
        backend_id,
    }
}

fn terminate_backend_command(request_id: RequestId, connection_id: String, backend_id: i64) -> UiCommand {
    UiCommand::MonitoringTerminateBackend {
        request_id,
        connection_id,
        backend_id,
    }
}

fn maintenance_target(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn maintenance_command(
    state: &MonitoringState,
    request_id: RequestId,
    connection_id: String,
    action: db_pro_core::domain::monitoring::MaintenanceAction,
) -> UiCommand {
    UiCommand::MonitoringMaintenance {
        request_id,
        connection_id,
        schema: maintenance_target(&state.monitoring_maintenance_schema),
        table: maintenance_target(&state.monitoring_maintenance_table),
        action,
        confirmed: true,
    }
}

fn reset_statements_command(request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::MonitoringResetStatStatements {
        request_id,
        connection_id,
        confirmed: true,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        maintenance_command, workload_command, MonitoringActivityContext, MonitoringState, RequestId, UiCommand,
    };

    #[test]
    fn workload_command_uses_state_sort_and_bounded_limit() {
        let state = MonitoringState {
            monitoring_stat_sort: db_pro_core::domain::monitoring::StatStatementSort::Calls,
            ..MonitoringState::default()
        };

        assert!(matches!(
            workload_command(&state, RequestId(7), "source".to_owned()),
            UiCommand::MonitoringStatStatements {
                request_id: RequestId(7),
                connection_id,
                sort: db_pro_core::domain::monitoring::StatStatementSort::Calls,
                limit: 100,
            } if connection_id == "source"
        ));
    }

    #[test]
    fn maintenance_command_is_explicitly_confirmed() {
        let action = db_pro_core::domain::monitoring::MaintenanceAction::Vacuum;

        assert!(matches!(
            maintenance_command(&MonitoringState::default(), RequestId(8), "source".to_owned(), action),
            UiCommand::MonitoringMaintenance {
                request_id: RequestId(8),
                connection_id,
                action: db_pro_core::domain::monitoring::MaintenanceAction::Vacuum,
                confirmed: true,
                ..
            } if connection_id == "source"
        ));
    }

    #[test]
    fn failed_maintenance_dispatch_preserves_confirmation() {
        let (mut bridge, command_rx, _event_tx) = crate::TaskBridge::with_channels();
        drop(command_rx);
        let mut dispatcher = crate::app::command_dispatch::RuntimeCommandDispatcher::new(&mut bridge);
        let mut state = MonitoringState {
            monitoring_maintenance_confirm: Some(db_pro_core::domain::monitoring::MaintenanceAction::Vacuum),
            ..MonitoringState::default()
        };
        let mut feedback = crate::app::FeedbackState::default();

        MonitoringActivityContext {
            theme: crate::DbProTheme::default(),
            state: &mut state,
            connected: true,
            driver: "PostgreSQL",
            connection_name: "source",
            connection_id: Some("conn-1"),
            command_dispatcher: &mut dispatcher,
            feedback: &mut feedback,
        }
        .apply_monitoring_confirmation_actions(vec![
            crate::app::monitoring_confirmation_view::MonitoringConfirmationAction::ConfirmMaintenance(
                db_pro_core::domain::monitoring::MaintenanceAction::Vacuum,
            ),
        ]);

        assert!(state.monitoring_maintenance_confirm.is_some());
    }

    #[test]
    fn snapshot_sort_vacuum_target_and_hub_name_drivers_without_sessions() {
        let (mut bridge, command_rx, _event_tx) = crate::TaskBridge::with_channels();
        let mut dispatcher = crate::app::command_dispatch::RuntimeCommandDispatcher::new(&mut bridge);
        let mut state = MonitoringState {
            monitoring_stat_sort: db_pro_core::domain::monitoring::StatStatementSort::Calls,
            monitoring_maintenance_schema: "public".to_owned(),
            monitoring_maintenance_table: "orders".to_owned(),
            monitoring_maintenance_confirm: Some(db_pro_core::domain::monitoring::MaintenanceAction::Vacuum),
            ..MonitoringState::default()
        };
        let mut feedback = crate::app::FeedbackState::default();
        let mut activity = MonitoringActivityContext {
            theme: crate::DbProTheme::default(),
            state: &mut state,
            connected: true,
            driver: "PostgreSQL",
            connection_name: "source",
            connection_id: Some("conn-1"),
            command_dispatcher: &mut dispatcher,
            feedback: &mut feedback,
        };

        activity.request_monitoring_snapshot();
        match command_rx.try_recv() {
            Ok(UiCommand::MonitoringSnapshot { sort, connection_id, .. }) => {
                assert_eq!(connection_id, "conn-1");
                assert_eq!(sort, db_pro_core::domain::monitoring::StatStatementSort::Calls);
            }
            Ok(_) => panic!("snapshot did not use the selected sort"),
            Err(error) => panic!("snapshot sent no command: {error}"),
        }

        activity.apply_monitoring_confirmation_actions(vec![
            crate::app::monitoring_confirmation_view::MonitoringConfirmationAction::ConfirmMaintenance(
                db_pro_core::domain::monitoring::MaintenanceAction::Vacuum,
            ),
        ]);
        match command_rx.try_recv() {
            Ok(UiCommand::MonitoringMaintenance {
                schema,
                table,
                action,
                confirmed,
                ..
            }) => {
                assert_eq!(schema.as_deref(), Some("public"));
                assert_eq!(table.as_deref(), Some("orders"));
                assert_eq!(action, db_pro_core::domain::monitoring::MaintenanceAction::Vacuum);
                assert!(confirmed);
            }
            Ok(_) => panic!("vacuum did not target the chosen table"),
            Err(error) => panic!("vacuum sent no command: {error}"),
        }

        assert_eq!(
            super::super::tools_hub_surface_view::monitor_card_description("MySQL"),
            "MySQL has no sessions"
        );
        assert_eq!(
            super::super::tools_hub_surface_view::monitor_card_description("SQL Server"),
            "SQL Server has no sessions"
        );
        let hub = include_str!("tools_hub_surface_view.rs");
        assert!(hub.contains("monitor_card_description(self.driver)"));
        let locks = include_str!("monitoring_snapshot_view.rs");
        let error_at = locks.find("lock_error").expect("lock panel reads lock_error");
        let empty_at = locks.find("blocking.is_empty()").expect("empty locks still hide");
        assert!(error_at < empty_at);
    }

    #[test]
    fn cancel_delete_replace_and_shell_wait_for_confirm() {
        let (mut bridge, command_rx, _event_tx) = crate::TaskBridge::with_channels();
        let mut dispatcher = crate::app::command_dispatch::RuntimeCommandDispatcher::new(&mut bridge);
        let mut state = MonitoringState::default();
        let mut feedback = crate::app::FeedbackState::default();
        {
            let mut activity = MonitoringActivityContext {
                theme: crate::DbProTheme::default(),
                state: &mut state,
                connected: true,
                driver: "PostgreSQL",
                connection_name: "source",
                connection_id: Some("conn-1"),
                command_dispatcher: &mut dispatcher,
                feedback: &mut feedback,
            };
            activity.apply_monitoring_sessions_action(
                super::super::monitoring_sessions_view::MonitoringSessionsAction::Cancel(42),
                &mut Vec::new(),
            );
            assert_eq!(activity.state.monitoring_cancel_confirm, Some(42));
            assert!(matches!(
                command_rx.try_recv(),
                Err(std::sync::mpsc::TryRecvError::Empty)
            ));
            activity.apply_monitoring_confirmation_actions(vec![
                crate::app::monitoring_confirmation_view::MonitoringConfirmationAction::ConfirmCancelBackend(42),
            ]);
        }
        match command_rx.try_recv() {
            Ok(UiCommand::MonitoringCancelBackend { backend_id, .. }) => assert_eq!(backend_id, 42),
            Ok(_) => panic!("confirm did not cancel the backend"),
            Err(error) => panic!("cancel confirm sent no command: {error}"),
        }

        let task_id = uuid::Uuid::new_v4();
        let now = chrono::Utc::now();
        let mut tasks = super::super::saved_task_state::SavedTaskState::default();
        tasks
            .store
            .upsert(db_pro_core::domain::saved_task::SavedTask {
                id: task_id,
                name: "nightly".into(),
                description: String::new(),
                connection_id: "conn-1".into(),
                payload: db_pro_core::domain::saved_task::SavedTaskPayload::Sql {
                    sql: "SELECT 1".into(),
                },
                created_at: now,
                updated_at: now,
                last_run: None,
                schedule: None,
            })
            .unwrap();
        tasks.request_delete(task_id);
        assert_eq!(tasks.store.tasks.len(), 1);
        assert_eq!(tasks.pending_delete_task_id, Some(task_id));
        assert!(tasks.confirm_delete());
        assert!(tasks.store.tasks.is_empty());

        let dir = std::env::temp_dir().join(format!(
            "dbpro-confirm-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        std::fs::create_dir_all(dir.join("migrations")).unwrap();
        let sql_path = dir.join("migrations/001_init.sql");
        std::fs::write(&sql_path, "CREATE TABLE users (id INT);").unwrap();
        let mut files = super::super::workspace_files_state::WorkspaceFilesState::default();
        files.ide_workspace.open_root(dir.clone()).unwrap();
        files.ide_workspace.set_trusted(true);
        files.workspace_search_query = "users".into();
        files.workspace_replace_query = "accounts".into();
        files.request_replace_all(&mut feedback);
        assert!(files.pending_replace_all);
        assert!(std::fs::read_to_string(&sql_path).unwrap().contains("users"));
        files.confirm_replace_all(&mut feedback);
        let replaced = std::fs::read_to_string(&sql_path).unwrap();
        assert!(replaced.contains("accounts"), "{replaced}");
        assert!(!files.pending_replace_all);

        files.workspace_task_command = "echo shell-confirmed".into();
        files.request_shell(&mut feedback);
        assert!(files.pending_shell);
        assert!(files.ide_workspace.last_task.is_none());
        files.confirm_shell(&mut feedback);
        let result = files.ide_workspace.last_task.as_ref().expect("confirmed shell ran");
        assert_eq!(result.command, "echo shell-confirmed");
        assert!(result.exit_code.is_some());
        assert!(feedback.runtime_message.contains("Task exit"));
        let _ = std::fs::remove_dir_all(dir);

        let tasks_view = include_str!("tasks_view.rs");
        assert!(tasks_view.contains("self.saved_tasks.request_delete(task_id)"));
        let files_tab = include_str!("files_activity_tabs.rs");
        assert!(files_tab.contains("request_replace_all"));
        assert!(files_tab.contains("request_shell"));
        assert!(files_tab.contains("confirm_replace_all"));
        assert!(files_tab.contains("confirm_shell"));
    }
}
