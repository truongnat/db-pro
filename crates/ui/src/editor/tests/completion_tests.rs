//! Tests for `../completion.rs` (moved out of the source file).
use super::*;

#[test]
// cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
fn test_completion_state_navigation() {
    let mut state = CompletionState::new();
    let items = vec![
        CompletionItem {
            label: "users".to_owned(),
            insert_text: "users".to_owned(),
            kind: CompletionItemKind::Table,
            detail: None,
            documentation: None,
            replacement_range: (0, 2),
            sort_score: 1,
        },
        CompletionItem {
            label: "user_roles".to_owned(),
            insert_text: "user_roles".to_owned(),
            kind: CompletionItemKind::Table,
            detail: None,
            documentation: None,
            replacement_range: (0, 2),
            sort_score: 2,
        },
        CompletionItem {
            label: "user_profiles".to_owned(),
            insert_text: "user_profiles".to_owned(),
            kind: CompletionItemKind::Table,
            detail: None,
            documentation: None,
            replacement_range: (0, 2),
            sort_score: 3,
        },
    ];

    state.open(
        2,
        7,
        Pos2::new(10.0, 20.0),
        "us".to_owned(),
        items,
        CompletionTriggerKind::Automatic,
    );
    assert!(state.is_open);
    assert!(state.can_apply_to_version(7));
    assert!(!state.can_apply_to_version(8));
    assert_eq!(state.selected_index, 0);
    assert_eq!(state.current_item().unwrap().label, "users");

    state.select_next();
    assert_eq!(state.selected_index, 1);
    assert_eq!(state.current_item().unwrap().label, "user_roles");

    state.select_prev();
    assert_eq!(state.selected_index, 0);

    state.select_prev(); // wraps to last
    assert_eq!(state.selected_index, 2);

    state.select_next(); // wraps to first
    assert_eq!(state.selected_index, 0);

    state.close();
    assert!(!state.is_open);
    assert!(state.items.is_empty());
}
