//! Composition state for the table editor feature.
//!
//! Keeping table metadata, grid interaction, and staged mutations under one
//! feature owner prevents the application shell from presenting them as
//! unrelated global concerns.

use super::{TableDataQueryState, TableDataState, TableEditingState, TableMutationState, TableState};

#[derive(Default)]
pub(crate) struct TableEditorState {
    pub(super) data: TableDataState,
    pub(super) data_query: TableDataQueryState,
    pub(super) editing: TableEditingState,
    pub(super) mutation: TableMutationState,
    pub(super) state: TableState,
}
