//! State owned by the event-trigger administration surface.

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
