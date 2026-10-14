/// Converts the builder's independent axis flags into egui's axis tuple.
/// Keeping this decision here prevents the presentation layer from duplicating
/// the ordering convention required by `egui::ScrollArea::new`.
pub fn scroll_axes(horizontal: bool, vertical: bool) -> [bool; 2] {
    [horizontal, vertical]
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
