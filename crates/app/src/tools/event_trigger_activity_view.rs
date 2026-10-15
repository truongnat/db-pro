use super::command_dispatch::RuntimeCommandDispatcher;
use super::event_trigger_state::EventTriggerState;
use super::*;

#[path = "event_trigger_surface_view.rs"]
mod event_trigger_surface_view;

pub(super) struct EventTriggerActivityContext<'a, 'bridge> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut EventTriggerState,
    pub(super) connection_id: Option<&'a str>,
    pub(super) driver: &'a str,
    pub(super) command_dispatcher: &'a mut RuntimeCommandDispatcher<'bridge>,
    pub(super) feedback: &'a mut FeedbackState,
}

impl EventTriggerActivityContext<'_, '_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) {
        let actions = event_trigger_surface_view::EventTriggerSurfaceContext {
            theme: self.theme,
            state: self.state,
        }
        .draw(ui);
        self.apply_actions(actions);
    }

    fn apply_actions(&mut self, actions: Vec<event_trigger_surface_view::EventTriggerSurfaceAction>) {
        for action in actions {
            match action {
                event_trigger_surface_view::EventTriggerSurfaceAction::Refresh => self.request_event_triggers(),
                event_trigger_surface_view::EventTriggerSurfaceAction::PreviewDrop(name) => {
                    match db_pro_core::domain::event_trigger::preview_drop_event_trigger(&name) {
                        Ok(sql) => {
                            self.state.event_trigger_error = None;
                            self.state.event_trigger_ddl_preview = Some(sql);
                        }
                        Err(err) => self.state.event_trigger_error = Some(err),
                    }
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::Alter { name, mode } => {
                    match db_pro_core::domain::event_trigger::preview_alter_event_trigger(&name, &mode) {
                        Ok(sql) => {
                            self.state.event_trigger_error = None;
                            self.state.event_trigger_apply_confirm =
                                Some(event_trigger_state::EventTriggerApply::Alter { name, mode, sql });
                        }
                        Err(err) => self.state.event_trigger_error = Some(err),
                    }
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::RequestDrop(name) => {
                    self.state.event_trigger_drop_confirm = Some(name);
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::PreviewCreate {
                    name,
                    event,
                    function_ref,
                    tags_csv,
                } => {
                    match db_pro_core::domain::event_trigger::preview_create_event_trigger(
                        &name,
                        &event,
                        &function_ref,
                        &tags_csv,
                    ) {
                        Ok(sql) => {
                            self.state.event_trigger_error = None;
                            self.state.event_trigger_ddl_preview = Some(sql);
                        }
                        Err(err) => self.state.event_trigger_error = Some(err),
                    }
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::Create => {
                    match db_pro_core::domain::event_trigger::preview_create_event_trigger(
                        &self.state.event_trigger_create_name,
                        &self.state.event_trigger_create_event,
                        &self.state.event_trigger_create_function,
                        &self.state.event_trigger_create_tags,
                    ) {
                        Ok(sql) => {
                            self.state.event_trigger_error = None;
                            self.state.event_trigger_apply_confirm =
                                Some(event_trigger_state::EventTriggerApply::Create {
                                    name: self.state.event_trigger_create_name.clone(),
                                    event: self.state.event_trigger_create_event.clone(),
                                    function_ref: self.state.event_trigger_create_function.clone(),
                                    tags_csv: self.state.event_trigger_create_tags.clone(),
                                    sql,
                                });
                        }
                        Err(err) => self.state.event_trigger_error = Some(err),
                    }
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::ConfirmApply => {
                    self.confirm_event_trigger_apply();
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::CancelApply => {
                    self.state.event_trigger_apply_confirm = None;
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::ClosePreview => {
                    self.state.event_trigger_ddl_preview = None;
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::ConfirmDrop(name) => {
                    self.drop_event_trigger_confirmed(&name);
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::CancelDrop => {
                    self.state.event_trigger_drop_confirm = None;
                }
            }
        }
    }

    fn request_event_triggers(&mut self) {
        let Some(connection_id) = self.connection_id else {
            self.state.event_trigger_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.driver.to_ascii_lowercase().contains("postgres") {
            self.state.event_trigger_error = Some("Event triggers are PostgreSQL-only".into());
            return;
        }
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(list_event_triggers_command(request_id, connection_id.to_owned()));
    }
    /// Dispatch the create/alter the confirm dialog displayed — the payload
    /// comes from `event_trigger_apply_confirm`, never re-read from the form.
    fn confirm_event_trigger_apply(&mut self) {
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let Some(pending) = self.state.event_trigger_apply_confirm.take() else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        let command = match pending {
            event_trigger_state::EventTriggerApply::Create {
                name,
                event,
                function_ref,
                tags_csv,
                ..
            } => UiCommand::CreateEventTrigger {
                request_id,
                connection_id: connection_id.to_owned(),
                name,
                event,
                function_ref,
                tags_csv,
                confirmed: true,
            },
            event_trigger_state::EventTriggerApply::Alter { name, mode, .. } => {
                UiCommand::AlterEventTrigger {
                    request_id,
                    connection_id: connection_id.to_owned(),
                    name,
                    mode,
                    confirmed: true,
                }
            }
        };
        if self.dispatch(command) {
            self.state.event_trigger_ddl_preview = None;
        }
    }

    fn drop_event_trigger_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection_id else {
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(drop_event_trigger_command(
            request_id,
            connection_id.to_owned(),
            name.to_owned(),
        ));
    }

    fn dispatch(&mut self, command: UiCommand) -> bool {
        self.command_dispatcher.dispatch(command, self.feedback)
    }
}

pub(super) fn list_event_triggers_command(request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::ListEventTriggers {
        request_id,
        connection_id,
    }
}


fn drop_event_trigger_command(request_id: RequestId, connection_id: String, name: String) -> UiCommand {
    UiCommand::DropEventTrigger {
        request_id,
        connection_id,
        name,
        confirmed: true,
    }
}

