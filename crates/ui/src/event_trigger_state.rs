//! State owned by the event-trigger administration surface.

use super::{RequestId, UiCommand};

pub(super) struct EventTriggerState {
    pub(super) event_trigger_inventory: Option<db_pro_core::domain::event_trigger::EventTriggerInventory>,
    pub(super) event_trigger_error: Option<String>,
    pub(super) event_trigger_create_name: String,
    pub(super) event_trigger_create_event: String,
    pub(super) event_trigger_create_function: String,
    pub(super) event_trigger_create_tags: String,
    pub(super) event_trigger_ddl_preview: Option<String>,
    pub(super) event_trigger_drop_confirm: Option<String>,
}

impl Default for EventTriggerState {
    fn default() -> Self {
        Self {
            event_trigger_inventory: None,
            event_trigger_error: None,
            event_trigger_create_name: String::new(),
            event_trigger_create_event: "ddl_command_end".to_owned(),
            event_trigger_create_function: String::new(),
            event_trigger_create_tags: String::new(),
            event_trigger_ddl_preview: None,
            event_trigger_drop_confirm: None,
        }
    }
}

impl EventTriggerState {
    pub(super) fn list_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::ListEventTriggers {
            request_id,
            connection_id,
        }
    }

    pub(super) fn create_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::CreateEventTrigger {
            request_id,
            connection_id,
            name: self.event_trigger_create_name.clone(),
            event: self.event_trigger_create_event.clone(),
            function_ref: self.event_trigger_create_function.clone(),
            tags_csv: self.event_trigger_create_tags.clone(),
            confirmed: true,
        }
    }

    pub(super) fn drop_command(&self, request_id: RequestId, connection_id: String, name: String) -> UiCommand {
        UiCommand::DropEventTrigger {
            request_id,
            connection_id,
            name,
            confirmed: true,
        }
    }

    pub(super) fn alter_command(
        &self,
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
}
