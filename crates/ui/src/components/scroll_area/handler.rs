/// Converts the builder's independent axis flags into egui's axis tuple.
/// Keeping this decision here prevents the presentation layer from duplicating
/// the ordering convention required by `egui::ScrollArea::new`.
pub fn scroll_axes(horizontal: bool, vertical: bool) -> [bool; 2] {
    [horizontal, vertical]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_order_matches_egui_scroll_area() {
        assert_eq!(scroll_axes(false, true), [false, true]);
        assert_eq!(scroll_axes(true, false), [true, false]);
        assert_eq!(scroll_axes(true, true), [true, true]);
    }
}
