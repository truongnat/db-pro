//! Tests for `../registry.rs` (moved out of the source file).
use super::*;

fn handle(id: u64) -> ConnectionHandle {
    ConnectionHandle(id)
}

fn conn_id() -> ConnectionId {
    ConnectionId::new()
}

#[test]
fn register_and_get() {
    let reg = ConnectionRegistry::new();
    let id = conn_id();
    let h = handle(42);
    reg.register(id, h);
    assert_eq!(reg.get(&id), Some(h));
}

#[test]
fn get_unregistered_returns_none() {
    let reg = ConnectionRegistry::new();
    assert_eq!(reg.get(&conn_id()), None);
}

#[test]
fn unregister_returns_handle() {
    let reg = ConnectionRegistry::new();
    let id = conn_id();
    let h = handle(7);
    reg.register(id, h);
    assert_eq!(reg.unregister(&id), Some(h));
    assert_eq!(reg.get(&id), None);
}

#[test]
fn is_active_tracks_state() {
    let reg = ConnectionRegistry::new();
    let id = conn_id();
    assert!(!reg.is_active(&id));
    reg.register(id, handle(1));
    assert!(reg.is_active(&id));
    reg.unregister(&id);
    assert!(!reg.is_active(&id));
}

#[test]
fn active_count() {
    let reg = ConnectionRegistry::new();
    assert_eq!(reg.active_count(), 0);
    let id1 = conn_id();
    let id2 = conn_id();
    reg.register(id1, handle(1));
    reg.register(id2, handle(2));
    assert_eq!(reg.active_count(), 2);
    reg.unregister(&id1);
    assert_eq!(reg.active_count(), 1);
}

#[test]
fn register_or_get_first_wins() {
    let reg = ConnectionRegistry::new();
    let id = conn_id();
    assert!(matches!(reg.register_or_get(id, handle(1)), RegisterResult::Inserted));
    match reg.register_or_get(id, handle(2)) {
        RegisterResult::Existing(h) => assert_eq!(h, handle(1)),
        _ => panic!("expected Existing"),
    }
    assert_eq!(reg.get(&id), Some(handle(1)));
}
