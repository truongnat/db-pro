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
mod unicode_search;

pub use chart::{ChartAggregation, ChartConfig, ChartEngine, ChartPoint, ChartProjection, ChartRenderer, ChartType};
pub use components::*;
pub use i18n::UiLanguage;
pub use runtime::{
    RequestId, TaskBridge, UiCell, UiCheckConstraint, UiColumn, UiCommand, UiConnectionDraft, UiConnectionSummary,
    UiDependencyDirection, UiDependencyKind, UiDriver, UiEvent, UiFunctionSummary, UiQueryError,
    UiQueryExecutionOutput, UiQueryFolderSummary, UiQueryHistoryEntry, UiQueryHistoryStatus, UiQueryResult,
    UiRoutineParameter, UiSavedQuerySummary, UiSchemaColumn, UiSchemaForeignKey, UiSchemaSummary, UiSslMode,
    UiStatementOutput, UiTableColumn, UiTableDataFilter, UiTableDataSort, UiTableDependency, UiTableFilterOperator,
    UiTableForeignKey, UiTableIndex, UiTableInfo, UiTableMutation, UiTableSummary, UiTriggerSummary, UiViewSummary,
};
pub use theme::tokens;
pub use theme::DbProTheme;
