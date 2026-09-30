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
mod tests {
    use super::{accessible_label, character_count};

    #[test]
    fn counts_unicode_characters_for_textarea_feedback() {
        assert_eq!(character_count("café", 8), "4/8");
    }

    #[test]
    fn accessible_label_falls_back_in_expected_order() {
        assert_eq!(accessible_label(Some("name"), Some("Label"), "Hint"), "name");
        assert_eq!(accessible_label(None, Some("Label"), "Hint"), "Label");
        assert_eq!(accessible_label(None, None, "Hint"), "Hint");
    }
}
