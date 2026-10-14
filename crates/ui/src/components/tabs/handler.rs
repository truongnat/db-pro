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
#[path = "tests/handler_tests.rs"]
mod tests;
