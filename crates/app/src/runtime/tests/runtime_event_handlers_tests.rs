//! Tests for `../runtime_event_handlers.rs` (moved out of the source file).
use super::*;
use crate::app::schema_compare::UiSchemaSnapshot;
use crate::{RequestId, UiSchemaSummary, UiTableSummary};

fn summary(tables: &[&str]) -> UiSchemaSummary {
    UiSchemaSummary {
        schemas: vec!["public".to_owned()],
        tables: tables.iter().map(|table| format!("public.{table}")).collect(),
        table_details: tables
            .iter()
            .map(|table| UiTableSummary {
                schema: "public".to_owned(),
                name: (*table).to_owned(),
                row_count: None,
                columns: Vec::new(),
                foreign_keys: Vec::new(),
            })
            .collect(),
        ..Default::default()
    }
}

fn app_waiting_for_refresh(request_id: RequestId) -> DbProApp {
    let mut app = DbProApp::default();
    app.schema.compare.schema_snapshot = Some(UiSchemaSnapshot::from_summary("baseline", &summary(&["orders"])));
    app.schema.compare.pending_diff_after_refresh = true;
    app.schema.explorer.schema_request = Some(request_id);
    app
}

/// A refresh that raced a disconnect leaves a pending diff behind; its late
/// reply must not be adopted as the new baseline nor complete the diff.
#[test]
fn stale_schema_reply_does_not_complete_pending_diff() {
    let mut app = app_waiting_for_refresh(RequestId(7));

    app.on_schema_loaded(RequestId(8), summary(&["orders", "invoices"]));

    assert!(app.schema.compare.pending_diff_after_refresh);
    assert!(app.schema.compare.schema_diff.is_none());
    assert!(app.schema.compare.schema_diff_current.is_none());
    assert_eq!(app.schema.explorer.schema_request, Some(RequestId(7)));
}

#[test]
fn matching_schema_reply_completes_pending_diff_once() {
    let mut app = app_waiting_for_refresh(RequestId(7));

    app.on_schema_loaded(RequestId(7), summary(&["orders", "invoices"]));

    assert!(!app.schema.compare.pending_diff_after_refresh);
    assert!(app.schema.compare.schema_diff.is_some());
    assert_eq!(
        app.schema
            .compare
            .schema_diff_current
            .as_ref()
            .map(|snapshot| snapshot.tables.len()),
        Some(2)
    );

    app.on_schema_loaded(RequestId(7), summary(&["orders"]));
    assert!(app.schema.compare.schema_diff.is_some());
}

/// A failure for a request the explorer is not waiting on must leave the
/// pending diff armed for its own refresh.
#[test]
fn unrelated_schema_failure_keeps_pending_diff() {
    let mut app = app_waiting_for_refresh(RequestId(7));

    assert!(!app.handle_schema_request_failure(RequestId(9), "boom"));

    assert!(app.schema.compare.pending_diff_after_refresh);
    assert_eq!(app.schema.explorer.schema_request, Some(RequestId(7)));
}

#[test]
fn matching_schema_failure_cancels_pending_diff() {
    let mut app = app_waiting_for_refresh(RequestId(7));

    assert!(app.handle_schema_request_failure(RequestId(7), "boom"));

    assert!(!app.schema.compare.pending_diff_after_refresh);
    assert_eq!(app.schema.explorer.schema_request, None);
    assert_eq!(app.schema.explorer.schema_error.as_deref(), Some("boom"));
}
