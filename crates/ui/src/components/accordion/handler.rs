use std::collections::BTreeSet;

/// Returns whether a header activation should update accordion state.
pub fn should_activate_header(activated: bool, disabled: bool) -> bool {
    activated && !disabled
}

/// Applies a header click to single-expansion accordion state.
pub fn toggle_single(selected_id: &mut Option<String>, item_id: &str, collapsible: bool) {
    if selected_id.as_deref() == Some(item_id) {
        if collapsible {
            *selected_id = None;
        }
    } else {
        *selected_id = Some(item_id.to_owned());
    }
}

/// Applies multi-expansion selection changes back to the caller's open set.
pub fn sync_multi_open_set(
    open_set: &mut BTreeSet<String>,
    item_id: &str,
    was_open: bool,
    now_selected: Option<&str>,
    disabled: bool,
) {
    if disabled {
        return;
    }
    if was_open && now_selected.is_none() {
        open_set.remove(item_id);
    } else if !was_open && now_selected.is_some() {
        open_set.insert(item_id.to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_activation_requires_enabled_item() {
        assert!(should_activate_header(true, false));
        assert!(!should_activate_header(true, true));
        assert!(!should_activate_header(false, false));
    }

    #[test]
    fn keyboard_activation_decision_respects_disabled_state() {
        let keyboard_pressed = true;
        assert!(should_activate_header(keyboard_pressed, false));
        assert!(!should_activate_header(keyboard_pressed, true));
    }

    #[test]
    fn single_toggle_respects_collapsible_setting() {
        let mut selected = Some("a".to_owned());
        toggle_single(&mut selected, "a", false);
        assert_eq!(selected.as_deref(), Some("a"));
        toggle_single(&mut selected, "a", true);
        assert_eq!(selected, None);
        toggle_single(&mut selected, "b", false);
        assert_eq!(selected.as_deref(), Some("b"));
    }

    #[test]
    fn multi_open_set_syncs_open_and_close_transitions() {
        let mut set = BTreeSet::new();
        set.insert("item-1".to_string());

        // Closing an open item
        sync_multi_open_set(&mut set, "item-1", true, None, false);
        assert!(!set.contains("item-1"));

        // Opening a closed item
        sync_multi_open_set(&mut set, "item-2", false, Some("item-2"), false);
        assert!(set.contains("item-2"));

        // No change
        sync_multi_open_set(&mut set, "item-2", true, Some("item-2"), false);
        assert!(set.contains("item-2"));
    }

    #[test]
    fn multi_toggle_preserves_disabled_item_state() {
        let mut set = BTreeSet::from(["item-1".to_string()]);

        sync_multi_open_set(&mut set, "item-1", true, None, true);
        assert!(set.contains("item-1"));

        sync_multi_open_set(&mut set, "item-2", false, Some("item-2"), true);
        assert!(!set.contains("item-2"));
    }
}
