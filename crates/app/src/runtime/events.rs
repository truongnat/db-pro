// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use super::*;
#[cfg(test)]
use crate::RequestId;

pub(super) struct QueryHistoryRecord {
    pub(super) sql: String,
    pub(super) connection_id: Option<String>,
    pub(super) schema: Option<String>,
    pub(super) started_at: String,
    pub(super) status: UiQueryHistoryStatus,
    pub(super) duration_ms: u64,
    pub(super) row_count: Option<u64>,
    pub(super) affected_rows: Option<u64>,
    pub(super) error_code: Option<String>,
    pub(super) error_summary: Option<String>,
}

impl DbProApp {
    pub(super) fn apply_runtime_events(&mut self) -> bool {
        let events: Vec<UiEvent> = self
            .task_bridge
            .drain_events(crate::runtime::MAX_RUNTIME_EVENTS_PER_FRAME)
            .collect();
        let batch_was_full = events.len() == crate::runtime::MAX_RUNTIME_EVENTS_PER_FRAME;
        for event in events {
            self.apply_runtime_event(event);
        }
        batch_was_full
    }

    // Query completion/history/prediction: dedicated query event reducers.
    // Shortcuts / dispatch: `events_query_dispatch.rs`.
}

#[cfg(test)]
#[path = "tests/events_row_reload_tests.rs"]
mod row_reload_tests;

