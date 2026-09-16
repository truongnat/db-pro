# Findings — UI08 Dialog, Form, and Confirmation Consistency

1. Modals and confirmation dialogs in `query_dialogs_view.rs` and `connection_view.rs` were still relying on legacy helper functions (`danger_button`, `compact_button`, `secondary_button_with_icon`, `primary_button_with_icon`).
2. Standardizing to canonical `Button` with appropriate `variant` (`Destructive`, `Default`, `Secondary`, `Ghost`) and `size` (`Sm`) ensures clear visual risk hierarchy and consistent keyboard focusability.
