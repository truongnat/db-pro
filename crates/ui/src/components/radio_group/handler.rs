use egui::Vec2;

use super::config::{HORIZONTAL_ITEM_GAP, VERTICAL_ITEM_GAP};

/// Returns the item spacing vector depending on layout orientation.
pub fn radio_group_item_spacing(horizontal: bool) -> Vec2 {
    if horizontal {
        Vec2::new(HORIZONTAL_ITEM_GAP, 0.0)
    } else {
        Vec2::new(0.0, VERTICAL_ITEM_GAP)
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
mod tests {
    use super::*;

    #[test]
    fn radio_group_item_spacing_matches_orientation() {
        assert_eq!(radio_group_item_spacing(true), Vec2::new(HORIZONTAL_ITEM_GAP, 0.0));
        assert_eq!(radio_group_item_spacing(false), Vec2::new(0.0, VERTICAL_ITEM_GAP));
    }

    #[test]
    fn keyboard_navigation_wraps_and_skips_disabled_options() {
        let disabled = [false, true, false];
        assert_eq!(next_enabled_option_index(&disabled, 0, true), Some(2));
        assert_eq!(next_enabled_option_index(&disabled, 2, true), Some(0));
        assert_eq!(next_enabled_option_index(&disabled, 0, false), Some(2));
        assert_eq!(next_enabled_option_index(&[true, true], 0, true), None);
        assert_eq!(next_enabled_option_index(&[], 0, true), None);
        assert_eq!(next_enabled_option_index(&disabled, 3, true), None);
    }

    #[test]
    fn handle_radio_option_click_only_triggers_when_unselected_and_enabled() {
        let val = 42;

        // Valid selection change
        assert_eq!(handle_radio_option_click(true, false, false, &val), Some(42));

        // Already selected - no change event
        assert_eq!(handle_radio_option_click(true, true, false, &val), None);

        // Disabled option clicked - no change event
        assert_eq!(handle_radio_option_click(true, false, true, &val), None);

        // Not clicked
        assert_eq!(handle_radio_option_click(false, false, false, &val), None);
    }
}
