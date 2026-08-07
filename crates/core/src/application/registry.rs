use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::domain::connection::{ConnectionHandle, ConnectionId};

pub enum RegisterResult {
    Inserted,
    Existing(ConnectionHandle),
}

#[derive(Clone)]
pub struct ConnectionRegistry {
    inner: Arc<RwLock<HashMap<ConnectionId, ConnectionHandle>>>,
}

impl ConnectionRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(&self, id: ConnectionId, handle: ConnectionHandle) {
        let mut map = self.inner.write().expect("registry lock poisoned");
        map.insert(id, handle);
    }

    /// Atomically register `handle` for `id`, or return the existing handle
    /// if another caller already registered it.
    pub fn register_or_get(&self, id: ConnectionId, handle: ConnectionHandle) -> RegisterResult {
        let mut map = self.inner.write().expect("registry lock poisoned");
        if let Some(&existing) = map.get(&id) {
            return RegisterResult::Existing(existing);
        }
        map.insert(id, handle);
        RegisterResult::Inserted
    }

    pub fn unregister(&self, id: &ConnectionId) -> Option<ConnectionHandle> {
        let mut map = self.inner.write().expect("registry lock poisoned");
        map.remove(id)
    }

    pub fn get(&self, id: &ConnectionId) -> Option<ConnectionHandle> {
        let map = self.inner.read().expect("registry lock poisoned");
        map.get(id).copied()
    }

    pub fn is_active(&self, id: &ConnectionId) -> bool {
        let map = self.inner.read().expect("registry lock poisoned");
        map.contains_key(id)
    }

    pub fn active_count(&self) -> usize {
        let map = self.inner.read().expect("registry lock poisoned");
        map.len()
    }
}

impl Default for ConnectionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
}
