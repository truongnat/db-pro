//! Tests for `../masking.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn preview_builder_adds_id_and_masks_requested_columns() {
        let preview = build_preview("email", MaskRule::Redact, false);

        assert_eq!(preview.columns, vec!["email", "id"]);
        assert_eq!(preview.original[0][0], "ada@example.com");
        assert_eq!(preview.masked[0][0], "***");
        assert_eq!(preview.masked[0][1], "42");
    }

    #[test]
    fn empty_selection_uses_stable_preview_headers() {
        let preview = build_preview("", MaskRule::Fixed, false);

        assert_eq!(preview.columns, vec!["email", "phone", "id"]);
        assert!(preview.message.contains("preview 2 rows"));
    }