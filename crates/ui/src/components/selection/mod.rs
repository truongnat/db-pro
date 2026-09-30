mod config;
mod handler;
mod ui;

pub use ui::{Checkbox, Radio, Slider, Switch};

#[cfg(test)]
mod tests {
    use super::handler::{checkbox_row_height, switch_toggle_requested};

    #[test]
    fn row_height_accounts_for_description() {
        assert_eq!(checkbox_row_height(false), 20.0);
        assert_eq!(checkbox_row_height(true), 36.0);
    }

    #[test]
    fn switch_toggle_requires_enabled_click_or_keyboard() {
        assert!(switch_toggle_requested(true, true, false));
        assert!(switch_toggle_requested(true, false, true));
        assert!(!switch_toggle_requested(false, true, true));
    }
}
