//! Tests for `../transaction_session.rs` (moved out of the source file).
use super::*;
use db_pro_core::application::with_query_session;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;

impl DiscardOnDrop for i32 {
    fn discard_on_drop(&mut self) {}
}

struct DiscardProbe {
    dropped: Arc<AtomicBool>,
}

impl DiscardOnDrop for DiscardProbe {
    fn discard_on_drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

fn probe() -> (DiscardProbe, Arc<AtomicBool>) {
    let dropped = Arc::new(AtomicBool::new(false));
    (
        DiscardProbe {
            dropped: Arc::clone(&dropped),
        },
        dropped,
    )
}

async fn lease(slot: SessionSlot<i32>, control: Option<TransactionControl>) -> SessionLease<i32> {
    checkout(slot, || async { Ok(1) }, control).await.expect("checkout")
}

#[tokio::test]
async fn abandoned_session_scoped_run_discards_the_connection() {
    let slot = SessionSlot::default();
    let (probe, dropped) = probe();
    let mut lease = checkout(slot, || async move { Ok(probe) }, None)
        .await
        .expect("checkout");
    lease.discard_if_abandoned();
    drop(lease);
    assert!(
        dropped.load(Ordering::SeqCst),
        "a run abandoned before restoring its session state must close the connection"
    );
}

#[tokio::test]
async fn completed_session_scoped_run_keeps_the_connection() {
    let slot = SessionSlot::default();
    let (probe, dropped) = probe();
    let mut lease = checkout(slot, || async move { Ok(probe) }, None)
        .await
        .expect("checkout");
    lease.discard_if_abandoned();
    lease.allow_reuse();
    drop(lease);
    assert!(
        !dropped.load(Ordering::SeqCst),
        "a restored run must return its connection to the pool"
    );
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
