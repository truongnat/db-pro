//! Composition state for the query feature.
//!
//! The editor, execution policy, output projection, saved-query library and
//! document session share one lifecycle. Keeping them behind one aggregate
//! makes that ownership explicit at the application boundary.

use super::{QueryEditorState, QueryExecutionPolicyState, QueryLibraryState, QueryOutputState, QuerySessionState};

#[derive(Default)]
pub(crate) struct QueryFeatureState {
    pub(super) editor: QueryEditorState,
    pub(super) execution: QueryExecutionPolicyState,
    pub(super) library: QueryLibraryState,
    pub(super) output: QueryOutputState,
    pub(super) session: QuerySessionState,
}
