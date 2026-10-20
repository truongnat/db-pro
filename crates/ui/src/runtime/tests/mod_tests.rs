//! Tests for `../mod.rs` (moved out of the source file).
use std::sync::mpsc;

use super::*;

/// New-connection drafts must not pre-fill identity, credentials, host, or
/// port — every value comes from the user.
#[test]
fn default_draft_starts_blank() {
    let draft = UiConnectionDraft::default();

    assert!(draft.name.is_empty());
    assert!(draft.host.is_empty());
    assert!(draft.port.is_empty());
    assert!(draft.database.is_empty());
    assert!(draft.username.is_empty());
    assert!(draft.password.is_empty());
}

/// The neutral parts of the draft are the same in every build profile.
#[test]
fn default_draft_neutral_fields_are_profile_independent() {
    let draft = UiConnectionDraft::default();

    assert!(draft.port.is_empty());
    assert_eq!(draft.driver, UiDriver::Postgres);
    assert_eq!(draft.ssl_mode, UiSslMode::Disable);
    assert!(!draft.readonly);
    assert!(!draft.ssh_tunnel_enabled);
    assert!(draft.ssh_port.is_empty());
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
