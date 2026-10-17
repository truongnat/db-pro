#![allow(clippy::unnecessary_to_owned)]

#[macro_use]
extern crate rust_i18n;

// Shared locale catalog — files live in crates/ui/locales; the macro resolves
// the path relative to this crate's manifest dir.
i18n!("../ui/locales", fallback = "en");

mod agent;
mod app;
mod policy;
mod result_grid;

// Lifts the whole db-pro-ui public surface (components, tokens, theme,
// Ui* protocol types, TaskBridge, query/editor/diagram/chart widgets) into
// `crate::` so feature modules keep resolving `use crate::X` unchanged.
pub(crate) use db_pro_ui::*;

#[cfg(test)]
#[path = "app/test_frame.rs"]
pub(crate) mod test_frame;

// `app::*` also surfaces `app::result_grid_view::GridSelectionLookup`, pinned for
// `benches/result_grid_benchmarks.rs` (a bench is a separate crate and can only
// reach the lookup through the crate root).
pub use agent::{
    respond as respond_to_agent, AgentContext, AgentMessage, AgentProvider, AgentProviderError, AgentProviderInfo,
    AgentProviderKind, AgentProviderState, AgentRole, OfflineAgentProvider,
};
pub use app::result_grid_view::GridSelectionLookup;
pub use app::{ComponentGalleryState, DbProApp};
pub(crate) use policy::ColumnWritePolicy;
pub use result_grid::{
    cell_text, cell_text_as_str, compare_ui_cells, displayed_row_number, filtered_sorted_indexes, grid_column_window,
    grid_keyboard_selection, GridColumnWindow, GridProjectionCache, GridProjectionKey,
};
