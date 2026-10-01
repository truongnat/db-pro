#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct NavigationKeys {
    pub arrow_left: bool,
    pub arrow_up: bool,
    pub arrow_right: bool,
    pub arrow_down: bool,
    pub activate: bool,
}

/// Resolves pointer and keyboard signals into the next selected tab index.
pub(super) fn next_selection(
    clicked: Option<usize>,
    focused: Option<usize>,
    tab_count: usize,
    keys: NavigationKeys,
) -> Option<usize> {
    if tab_count == 0 {
        return None;
    }

    if let Some(focused) = focused.filter(|index| *index < tab_count) {
        if keys.arrow_left || keys.arrow_up {
            return Some((focused + tab_count - 1) % tab_count);
        }
        if keys.arrow_right || keys.arrow_down {
            return Some((focused + 1) % tab_count);
        }
        if keys.activate {
            return Some(focused);
        }
    }

    clicked.filter(|index| *index < tab_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_wraps_and_ignores_empty_or_out_of_range_tracks() {
        let left = NavigationKeys {
            arrow_left: true,
            ..Default::default()
        };
        let right = NavigationKeys {
            arrow_right: true,
            ..Default::default()
        };
        assert_eq!(next_selection(None, Some(0), 3, left), Some(2));
        assert_eq!(next_selection(None, Some(2), 3, right), Some(0));
        assert_eq!(next_selection(None, Some(0), 0, right), None);
        assert_eq!(next_selection(None, Some(3), 3, right), None);
    }

    #[test]
    fn keyboard_navigation_precedes_click_and_activation_keeps_focus() {
        let right = NavigationKeys {
            arrow_right: true,
            ..Default::default()
        };
        let activate = NavigationKeys {
            activate: true,
            ..Default::default()
        };
        assert_eq!(next_selection(Some(1), Some(0), 3, right), Some(1));
        assert_eq!(next_selection(None, Some(2), 3, activate), Some(2));
        assert_eq!(next_selection(Some(1), None, 3, right), Some(1));
    }
}
