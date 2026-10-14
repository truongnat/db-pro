//! Tests for `../schema_workbench_surface_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn workbench_surface_keeps_mode_selection_typed() {
        assert_eq!(
            SchemaWorkbenchSurfaceAction::SelectMode(SchemaWorkbenchMode::Docs),
            SchemaWorkbenchSurfaceAction::SelectMode(SchemaWorkbenchMode::Docs)
        );
        assert_eq!(
            SchemaWorkbenchSurfaceAction::SelectMode(SchemaWorkbenchMode::History),
            SchemaWorkbenchSurfaceAction::SelectMode(SchemaWorkbenchMode::History)
        );
    }