//! Native UI/runtime protocol facade.

#[path = "runtime_protocol.rs"]
mod protocol;
#[path = "runtime_connection_types.rs"]
mod runtime_connection_types;
#[path = "runtime_query_types.rs"]
mod runtime_query_types;
#[path = "runtime_schema_types.rs"]
mod runtime_schema_types;
#[path = "task_bridge.rs"]
mod task_bridge;

pub use protocol::{UiCommand, UiEvent};
pub use runtime_connection_types::*;
pub use runtime_query_types::*;
pub use runtime_schema_types::*;
pub use task_bridge::TaskBridge;
pub(crate) use task_bridge::MAX_RUNTIME_EVENTS_PER_FRAME;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    /// Regression guard for the S-1 release-hygiene finding: the developer
    /// connection preset is developer-only. `cargo test --release` exercises the
    /// release half of this assertion.
    #[test]
    fn default_draft_identity_follows_build_profile() {
        let draft = UiConnectionDraft::default();

        if cfg!(debug_assertions) {
            assert_eq!(draft.name, "Xe Lạc Hồng (PostgreSQL)");
            assert_eq!(draft.host, "localhost");
            assert_eq!(draft.database, "fullstack_starter");
            assert_eq!(draft.username, "postgres");
            assert_eq!(draft.password, "postgres");
        } else {
            assert!(
                draft.name.is_empty(),
                "release builds must not pre-fill a connection name"
            );
            assert!(
                draft.host.is_empty(),
                "release builds must not pre-fill a connection host"
            );
            assert!(
                draft.database.is_empty(),
                "release builds must not pre-fill a database name"
            );
            assert!(draft.username.is_empty(), "release builds must not pre-fill a username");
            assert!(draft.password.is_empty(), "release builds must not pre-fill a password");
        }
    }

    /// The neutral parts of the draft are the same in every build profile.
    #[test]
    fn default_draft_neutral_fields_are_profile_independent() {
        let draft = UiConnectionDraft::default();

        assert_eq!(draft.port, "5432");
        assert_eq!(draft.driver, UiDriver::Postgres);
        assert_eq!(draft.ssl_mode, UiSslMode::Require);
        assert!(!draft.readonly);
        assert!(!draft.ssh_tunnel_enabled);
        assert_eq!(draft.ssh_port, "22");
        assert!(draft.ssh_host.is_empty());
        assert!(draft.ssh_user.is_empty());
        assert!(draft.ssh_private_key.is_empty());
    }

    #[test]
    fn request_ids_are_monotonic() {
        let mut bridge = TaskBridge::default();
        assert_eq!(bridge.next_request_id(), RequestId(1));
        assert_eq!(bridge.next_request_id(), RequestId(2));
    }

    #[test]
    fn bridge_delivers_typed_commands() {
        let (command_tx, command_rx) = mpsc::channel();
        let (_event_tx, event_rx) = mpsc::channel();
        let bridge = TaskBridge::new(command_tx, event_rx);
        let command = UiCommand::OpenQuery;
        bridge.send(command.clone()).expect("receiver is alive");
        assert_eq!(command_rx.recv().expect("command expected"), command);
    }

    #[test]
    fn bridge_drains_at_most_the_requested_event_batch() {
        let (bridge, _command_rx, event_tx) = TaskBridge::with_channels();
        for request_id in 1..=3 {
            event_tx
                .send(UiEvent::QueryQueued {
                    request_id: RequestId(request_id),
                })
                .expect("event receiver is alive");
        }

        assert_eq!(bridge.drain_events(2).count(), 2);
        assert_eq!(bridge.drain_events(2).count(), 1);
    }
}
