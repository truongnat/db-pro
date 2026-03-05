use std::sync::Arc;

use crate::state::{AppState, QueryCancellation};

pub fn register(state: &AppState, connection_id: &str) -> Result<Arc<QueryCancellation>, String> {
    let mut guard = state
        .query_cancellations
        .lock()
        .map_err(|_| "Failed to access running query state".to_string())?;

    let cancellation = Arc::new(QueryCancellation::new());
    if let Some(previous) = guard.insert(connection_id.to_string(), cancellation.clone()) {
        previous.cancel();
    }

    Ok(cancellation)
}

pub fn clear(
    state: &AppState,
    connection_id: &str,
    expected: &Arc<QueryCancellation>,
) -> Result<(), String> {
    let mut guard = state
        .query_cancellations
        .lock()
        .map_err(|_| "Failed to access running query state".to_string())?;

    if let Some(current) = guard.get(connection_id) {
        if Arc::ptr_eq(current, expected) {
            guard.remove(connection_id);
        }
    }

    Ok(())
}

pub fn cancel_for_connection(state: &AppState, connection_id: &str) -> Result<bool, String> {
    let cancellation = state
        .query_cancellations
        .lock()
        .map_err(|_| "Failed to access running query state".to_string())?
        .get(connection_id)
        .cloned();

    if let Some(token) = cancellation {
        token.cancel();
        return Ok(true);
    }

    Ok(false)
}
