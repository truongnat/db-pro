/// Decides whether an open overlay should dismiss after a pointer interaction.
pub(crate) fn should_close_on_outside_click(open: bool, any_click: bool, outside: bool) -> bool {
    open && any_click && outside
}

/// Measured row width for a [`DropdownItem`]: icon slot + label + optional
/// shortcut badge + both paddings. The popup clamps its width to the widest row
/// so the panel hugs content instead of spanning the `Area`'s screen width.
pub(crate) fn dropdown_item_content_width(ui: &egui::Ui, item: &super::ui::DropdownItem<'_>) -> f32 {
    let label_w = ui
        .painter()
        .layout_no_wrap(
            item.label.to_string(),
            egui::FontId::proportional(super::config::DROPDOWN_LABEL_SIZE),
            egui::Color32::WHITE,
        )
        .size()
        .x;
    let icon_w = if item.icon.is_some() {
        super::config::DROPDOWN_ITEM_ICON_OFFSET
    } else {
        0.0
    };
    let shortcut_w = if item.shortcut.is_some() {
        super::config::DROPDOWN_ITEM_TRAILING_RESERVE
    } else {
        0.0
    };
    super::config::DROPDOWN_ITEM_HORIZONTAL_PADDING * 2.0 + icon_w + label_w + shortcut_w
}
