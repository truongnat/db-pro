pub(crate) fn checkbox_row_height(has_description: bool) -> f32 {
    super::config::COMPACT_ROW_HEIGHT
        + if has_description {
            super::config::DESCRIPTION_HEIGHT
        } else {
            0.0
        }
}

pub(crate) fn switch_toggle_requested(enabled: bool, clicked: bool, keyboard_toggle: bool) -> bool {
    enabled && (clicked || keyboard_toggle)
}
