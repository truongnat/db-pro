//! Input behavior decisions that do not require an egui painter or widget tree.

/// Returns the counter text for a textarea, keeping the displayed value consistent with the
/// same character-count rule used by the component's public `max_chars` option.
pub(crate) fn character_count(value: &str, max_chars: usize) -> String {
    format!("{}/{}", value.chars().count(), max_chars)
}

/// Chooses the accessible label for a text input, preferring an explicit label over its hint.
pub(crate) fn accessible_label<'a>(
    access_label: Option<&'a str>,
    label: Option<&'a str>,
    placeholder: &'a str,
) -> &'a str {
    access_label.or(label).unwrap_or(placeholder)
}

#[cfg(test)]
#[path = "tests/handler_tests.rs"]
mod tests;
