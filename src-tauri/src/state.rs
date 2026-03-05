use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use tokio::sync::Notify;

use crate::models::ConnectionProfile;

pub struct QueryCancellation {
    cancelled: AtomicBool,
    notify: Notify,
}

impl QueryCancellation {
    pub fn new() -> Self {
        Self {
            cancelled: AtomicBool::new(false),
            notify: Notify::new(),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }
        self.notify.notified().await;
    }
}

pub struct AppState {
    pub connections: RwLock<HashMap<String, ConnectionProfile>>,
    pub query_cancellations: Mutex<HashMap<String, Arc<QueryCancellation>>>,
    pub storage_path: PathBuf,
}

impl AppState {
    pub fn new(storage_path: PathBuf, connections: HashMap<String, ConnectionProfile>) -> Self {
        Self {
            connections: RwLock::new(connections),
            query_cancellations: Mutex::new(HashMap::new()),
            storage_path,
        }
    }
}
