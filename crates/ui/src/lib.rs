#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "en");

mod chart;
pub mod components;
#[cfg(debug_assertions)]
pub mod dev_tools;
pub mod diagram;
pub mod editor;
pub mod i18n;
pub mod query;
pub mod runtime;
#[cfg(test)]
#[path = "test_frame.rs"]
pub(crate) mod test_frame;
mod theme;

pub use chart::{ChartAggregation, ChartConfig, ChartEngine, ChartPoint, ChartProjection, ChartRenderer, ChartType};
pub use components::*;
// Named re-export for component helpers added after the legacy glob, per the
// `lib.rs` API-contract rule (a new public item must be listed explicitly).
pub use components::common::popup_content_frame;
pub use i18n::UiLanguage;
pub use runtime::{
    RequestId, TaskBridge, UiCell, UiCheckConstraint, UiColumn, UiCommand, UiConnectionDraft, UiConnectionSummary,
    UiDependencyDirection, UiDependencyKind, UiDriver, UiEvent, UiFunctionSummary, UiQueryError,
    UiQueryExecutionOutput, UiQueryFolderSummary, UiQueryHistoryEntry, UiQueryHistoryStatus, UiQueryResult,
    UiRoutineParameter, UiSavedQuerySummary, UiSchemaColumn, UiSchemaForeignKey, UiSchemaSummary, UiSslMode,
    UiStatementOutput, UiTableColumn, UiTableDataFilter, UiTableDataSort, UiTableDependency, UiTableFilterOperator,
    UiTableForeignKey, UiTableIndex, UiTableInfo, UiTableMutation, UiTableSummary, UiTriggerSummary, UiViewSummary,
    MAX_CELL_TEXT_BYTES,
};
pub use theme::tokens;
pub use theme::DbProTheme;
