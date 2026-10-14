//! Hold one pooled connection per query tab while that tab's transaction is open.
//!
//! The slot is taken out for the statement and put back in `Drop`, so a cancelled
//! future does not leave the open transaction on a random pool connection. The key
//! is captured at checkout: `Drop` must not read the task-local again.

use db_pro_core::application::active_query_session;
use db_pro_core::domain::error::DbError;
use db_pro_core::domain::transaction_control::TransactionControl;
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

pub(crate) struct SessionSlot<C> {
    inner: Arc<Mutex<HashMap<String, C>>>,
}

impl<C> Default for SessionSlot<C> {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<C> Clone for SessionSlot<C> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<C> SessionSlot<C> {
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, C>> {
        self.inner.lock().unwrap_or_else(|error| error.into_inner())
    }

    fn session_key() -> String {
        active_query_session().unwrap_or_default()
    }

    /// The current tab already has a connection checked out.
    pub(crate) fn is_held(&self) -> bool {
        self.lock().contains_key(&Self::session_key())
    }

    fn take(&self, key: &str) -> Option<C> {
        self.lock().remove(key)
    }

    fn store(&self, key: &str, conn: C) {
        self.lock().insert(key.to_owned(), conn);
    }

    #[cfg(test)]
    fn take_current(&self) -> Option<C> {
        let key = Self::session_key();
        self.take(&key)
    }
}

pub(crate) struct SessionLease<C> {
    slot: SessionSlot<C>,
    key: String,
    conn: Option<C>,
    was_held: bool,
    control: Option<TransactionControl>,
    ok: bool,
}

impl<C> SessionLease<C> {
    pub(crate) fn conn(&mut self) -> &mut C {
        self.conn.as_mut().expect("session lease still holds its connection")
    }

    pub(crate) fn was_held(&self) -> bool {
        self.was_held
    }

    pub(crate) fn succeed(&mut self) {
        self.ok = true;
    }
}

impl<C> Drop for SessionLease<C> {
    fn drop(&mut self) {
        let Some(conn) = self.conn.take() else {
            return;
        };
        let keep = match (self.control, self.ok) {
            (Some(TransactionControl::Begin), true) => true,
            (Some(TransactionControl::Commit | TransactionControl::Rollback), true) => false,
            _ => self.was_held,
        };
        if keep {
            self.slot.store(&self.key, conn);
        }
    }
}

pub(crate) async fn checkout<C, F, Fut>(
    slot: SessionSlot<C>,
    acquire: F,
    control: Option<TransactionControl>,
) -> Result<SessionLease<C>, DbError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<C, DbError>>,
{
    let key = SessionSlot::<C>::session_key();
    let (conn, was_held) = if let Some(conn) = slot.take(&key) {
        (conn, true)
    } else {
        (acquire().await?, false)
    };
    Ok(SessionLease {
        slot,
        key,
        conn: Some(conn),
        was_held,
        control,
        ok: false,
    })
}

/// Reuse the tab's open transaction. A tab with nothing held, including schema
/// apply and the table grid, gets `None` and must open its own transaction.
pub(crate) async fn checkout_if_held<C, F, Fut>(
    slot: SessionSlot<C>,
    acquire: F,
) -> Result<Option<SessionLease<C>>, DbError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<C, DbError>>,
{
    if !slot.is_held() {
        return Ok(None);
    }
    let lease = checkout(slot, acquire, None).await?;
    if lease.was_held() {
        Ok(Some(lease))
    } else {
        drop(lease);
        Ok(None)
    }
}

#[cfg(test)]
#[path = "tests/transaction_session_tests.rs"]
mod tests;
