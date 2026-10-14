//! Aggregate state for schema navigation, design and comparison workspaces.

use super::schema_workbench::SchemaWorkbenchState;
use super::{DiagramState, SchemaCompareState, SchemaExplorerState};

#[derive(Default)]
pub(crate) struct SchemaWorkspaceState {
    pub(super) explorer: SchemaExplorerState,
    pub(super) workbench: SchemaWorkbenchState,
    pub(super) compare: SchemaCompareState,
    pub(super) diagram: DiagramState,
}
