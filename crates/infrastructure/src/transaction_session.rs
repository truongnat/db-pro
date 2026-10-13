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
mod tests {
    use super::*;
    use db_pro_core::application::with_query_session;
    use std::sync::atomic::{AtomicI32, Ordering};

    async fn lease(slot: SessionSlot<i32>, control: Option<TransactionControl>) -> SessionLease<i32> {
        checkout(slot, || async { Ok(1) }, control).await.expect("checkout")
    }

    fn next_value(next: &AtomicI32) -> i32 {
        next.fetch_add(1, Ordering::SeqCst)
    }

    #[tokio::test]
    async fn successful_begin_keeps_a_fresh_connection() {
        let slot = SessionSlot::default();
        let mut lease = lease(slot.clone(), Some(TransactionControl::Begin)).await;
        lease.succeed();
        drop(lease);
        assert_eq!(slot.take_current(), Some(1));
    }

    #[tokio::test]
    async fn failed_begin_drops_the_connection() {
        let slot = SessionSlot::default();
        let lease = lease(slot.clone(), Some(TransactionControl::Begin)).await;
        drop(lease);
        assert!(slot.take_current().is_none());
    }

    #[tokio::test]
    async fn successful_commit_releases_the_held_connection() {
        let slot = SessionSlot::default();
        let mut opened = lease(slot.clone(), Some(TransactionControl::Begin)).await;
        opened.succeed();
        drop(opened);

        let mut committed = checkout(slot.clone(), || async { Ok(2) }, Some(TransactionControl::Commit))
            .await
            .expect("checkout held");
        assert_eq!(*committed.conn(), 1);
        committed.succeed();
        drop(committed);
        assert!(slot.take_current().is_none());
    }

    #[tokio::test]
    async fn failed_statement_keeps_the_open_transaction() {
        let slot = SessionSlot::default();
        let mut opened = lease(slot.clone(), Some(TransactionControl::Begin)).await;
        opened.succeed();
        drop(opened);

        let failed = checkout(slot.clone(), || async { Ok(2) }, None)
            .await
            .expect("checkout held");
        drop(failed);
        assert_eq!(slot.take_current(), Some(1));
    }

    #[tokio::test]
    async fn two_tabs_do_not_share_a_connection() {
        let slot = SessionSlot::default();
        let next = AtomicI32::new(1);

        with_query_session(Some("tab-a"), async {
            let value = next_value(&next);
            let mut opened = checkout(
                slot.clone(),
                || async move { Ok(value) },
                Some(TransactionControl::Begin),
            )
            .await
            .expect("begin a");
            opened.succeed();
        })
        .await;

        with_query_session(Some("tab-b"), async {
            let value = next_value(&next);
            let mut opened = checkout(
                slot.clone(),
                || async move { Ok(value) },
                Some(TransactionControl::Begin),
            )
            .await
            .expect("begin b");
            assert_eq!(*opened.conn(), 2);
            opened.succeed();
        })
        .await;

        with_query_session(Some("tab-a"), async {
            let mut joined = checkout_if_held(slot.clone(), || async { Ok(99) })
                .await
                .expect("join a")
                .expect("tab-a is open");
            assert_eq!(*joined.conn(), 1);
            drop(joined);
            let mut again = checkout_if_held(slot.clone(), || async { Ok(99) })
                .await
                .expect("join a again")
                .expect("a failed script keeps the connection");
            assert_eq!(*again.conn(), 1);
        })
        .await;

        let anonymous = checkout_if_held(slot.clone(), || async { Ok(99) })
            .await
            .expect("anonymous");
        assert!(anonymous.is_none());

        with_query_session(Some("tab-a"), async {
            let mut committed = checkout(slot.clone(), || async { Ok(99) }, Some(TransactionControl::Commit))
                .await
                .expect("commit a");
            committed.succeed();
        })
        .await;

        with_query_session(Some("tab-b"), async {
            let mut still_open = checkout_if_held(slot.clone(), || async { Ok(99) })
                .await
                .expect("tab-b")
                .expect("committing tab-a leaves tab-b open");
            assert_eq!(*still_open.conn(), 2);
        })
        .await;
    }
}
