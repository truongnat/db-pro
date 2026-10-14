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
#[path = "tests/handler_tests.rs"]
mod tests;
