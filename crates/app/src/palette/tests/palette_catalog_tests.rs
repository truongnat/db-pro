//! Tests for `../palette_catalog.rs` (moved out of the source file).
use super::*;

/// Quick Open merges navigation/activity groups with the command catalog; the
/// same action must never appear twice in one result list.
#[test]
fn quick_open_lists_each_action_once() {
    let items = quick_open_items();
    let mut actions: Vec<&PaletteAction> = items.iter().map(|item| &item.action).collect();
    let total = actions.len();
    actions.sort_by_key(|action| format!("{action:?}"));
    actions.dedup();
    assert_eq!(actions.len(), total, "duplicate actions in {:?}", items.len());

    assert!(items.iter().any(|item| item.action == PaletteAction::Agent));
    assert!(items.iter().any(|item| item.action == PaletteAction::Diagram));
    // The command catalog still contributes entries only it provides.
    assert!(items.iter().any(|item| item.action == PaletteAction::RunQuery));
    assert!(items.iter().any(|item| item.action == PaletteAction::FormatSql));
}
