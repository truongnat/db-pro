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
        let mut map = self.inner.write().unwrap_or_else(|e| e.into_inner());
        map.insert(id, handle);
    }

    /// Atomically register `handle` for `id`, or return the existing handle
    /// if another caller already registered it.
    pub fn register_or_get(&self, id: ConnectionId, handle: ConnectionHandle) -> RegisterResult {
        let mut map = self.inner.write().unwrap_or_else(|e| e.into_inner());
        if let Some(&existing) = map.get(&id) {
            return RegisterResult::Existing(existing);
        }
        map.insert(id, handle);
        RegisterResult::Inserted
    }

    pub fn unregister(&self, id: &ConnectionId) -> Option<ConnectionHandle> {
        let mut map = self.inner.write().unwrap_or_else(|e| e.into_inner());
        map.remove(id)
    }

    pub fn get(&self, id: &ConnectionId) -> Option<ConnectionHandle> {
        let map = self.inner.read().unwrap_or_else(|e| e.into_inner());
        map.get(id).copied()
    }

    pub fn is_active(&self, id: &ConnectionId) -> bool {
        let map = self.inner.read().unwrap_or_else(|e| e.into_inner());
        map.contains_key(id)
    }

    pub fn active_count(&self) -> usize {
        let map = self.inner.read().unwrap_or_else(|e| e.into_inner());
        map.len()
    }
}

impl Default for ConnectionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "tests/registry_tests.rs"]
mod tests;
