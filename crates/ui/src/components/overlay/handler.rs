/// Decides whether an open overlay should dismiss after a pointer interaction.
pub(crate) fn should_close_on_outside_click(open: bool, any_click: bool, outside: bool) -> bool {
    open && any_click && outside
}
