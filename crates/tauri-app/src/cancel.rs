use std::collections::HashMap;
use std::sync::Mutex;

use tokio::sync::oneshot;

pub struct CancelRegistry {
    inner: Mutex<HashMap<String, oneshot::Sender<()>>>,
}

impl CancelRegistry {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, connection_id: &str) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(connection_id.to_string(), tx);
        rx
    }

    pub fn cancel(&self, connection_id: &str) -> bool {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(tx) = guard.remove(connection_id) {
            let _ = tx.send(());
            true
        } else {
            false
        }
    }

    pub fn unregister(&self, connection_id: &str) {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).remove(connection_id);
    }
}

impl Default for CancelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
