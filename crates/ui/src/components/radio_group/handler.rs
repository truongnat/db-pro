use egui::Vec2;

use super::config::{HORIZONTAL_ITEM_GAP, NO_ITEM_GAP, VERTICAL_ITEM_GAP};

/// Returns the item spacing vector depending on layout orientation.
pub fn radio_group_item_spacing(horizontal: bool) -> Vec2 {
    if horizontal {
        Vec2::new(HORIZONTAL_ITEM_GAP, NO_ITEM_GAP)
    } else {
        Vec2::new(NO_ITEM_GAP, VERTICAL_ITEM_GAP)
    }
}

/// Finds the next enabled option, wrapping at either end of the radio group.
pub fn next_enabled_option_index(disabled: &[bool], current: usize, forward: bool) -> Option<usize> {
    let option_count = disabled.len();
    if option_count == 0 || current >= option_count {
        return None;
    }

    (1..=option_count).find_map(|step| {
        let index = if forward {
            (current + step) % option_count
        } else {
            (current + option_count - (step % option_count)) % option_count
        };
        (!disabled[index]).then_some(index)
    })
}

/// Evaluates whether an option click should mutate selection and produce a changed event.
pub fn handle_radio_option_click<T: Clone + PartialEq>(
    clicked: bool,
    is_selected: bool,
    disabled: bool,
    value: &T,
) -> Option<T> {
    if clicked && !is_selected && !disabled {
        Some(value.clone())
    } else {
        None
    }
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
