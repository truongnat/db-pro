use super::*;
use super::{event_trigger_state::EventTriggerState, RequestId, UiCommand};

#[path = "event_trigger_surface_view.rs"]
mod event_trigger_surface_view;

impl DbProApp {
    pub(super) fn draw_event_trigger_activity(&mut self, ui: &mut egui::Ui) {
        let actions = event_trigger_surface_view::EventTriggerSurfaceContext {
            theme: self.theme,
            state: &mut self.management.event_trigger,
        }
        .draw(ui);
        self.apply_event_trigger_surface_actions(actions);
    }

    fn apply_event_trigger_surface_actions(
        &mut self,
        actions: Vec<event_trigger_surface_view::EventTriggerSurfaceAction>,
    ) {
        for action in actions {
            match action {
                event_trigger_surface_view::EventTriggerSurfaceAction::Refresh => self.request_event_triggers(),
                event_trigger_surface_view::EventTriggerSurfaceAction::PreviewDrop(name) => {
                    self.management.event_trigger.event_trigger_ddl_preview =
                        db_pro_core::domain::event_trigger::preview_drop_event_trigger(&name).ok();
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::Alter { name, mode } => {
                    self.alter_event_trigger_confirmed(&name, &mode);
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::RequestDrop(name) => {
                    self.management.event_trigger.event_trigger_drop_confirm = Some(name);
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::PreviewCreate {
                    name,
                    event,
                    function_ref,
                    tags_csv,
                } => {
                    self.management.event_trigger.event_trigger_ddl_preview =
                        db_pro_core::domain::event_trigger::preview_create_event_trigger(
                            &name,
                            &event,
                            &function_ref,
                            &tags_csv,
                        )
                        .ok();
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::Create => {
                    self.create_event_trigger_confirmed();
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::ClosePreview => {
                    self.management.event_trigger.event_trigger_ddl_preview = None;
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::ConfirmDrop(name) => {
                    self.drop_event_trigger_confirmed(&name);
                }
                event_trigger_surface_view::EventTriggerSurfaceAction::CancelDrop => {
                    self.management.event_trigger.event_trigger_drop_confirm = None;
                }
            }
        }
    }

    fn request_event_triggers(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.management.event_trigger.event_trigger_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.active_driver().to_ascii_lowercase().contains("postgres") {
            self.management.event_trigger.event_trigger_error = Some("Event triggers are PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(list_event_triggers_command(request_id, connection_id));
    }

    fn create_event_trigger_confirmed(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(create_event_trigger_command(
            &self.management.event_trigger,
            request_id,
            connection_id,
        ));
    }

    fn drop_event_trigger_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            drop_event_trigger_command(request_id, connection_id, name.to_owned()),
        );
    }

    fn alter_event_trigger_confirmed(&mut self, name: &str, mode: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(alter_event_trigger_command(
            request_id,
            connection_id,
            name.to_owned(),
            mode.to_owned(),
        ));
    }
}

pub(super) fn list_event_triggers_command(request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::ListEventTriggers {
        request_id,
        connection_id,
    }
}

fn create_event_trigger_command(
    state: &EventTriggerState,
    request_id: RequestId,
    connection_id: String,
) -> UiCommand {
    UiCommand::CreateEventTrigger {
        request_id,
        connection_id,
        name: state.event_trigger_create_name.clone(),
        event: state.event_trigger_create_event.clone(),
        function_ref: state.event_trigger_create_function.clone(),
        tags_csv: state.event_trigger_create_tags.clone(),
        confirmed: true,
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

fn alter_event_trigger_command(
    request_id: RequestId,
    connection_id: String,
    name: String,
    mode: String,
) -> UiCommand {
    UiCommand::AlterEventTrigger {
        request_id,
        connection_id,
        name,
        mode,
        confirmed: true,
    }
}
