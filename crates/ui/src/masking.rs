//! Pure masking preview construction for the database management surface.

use db_pro_core::domain::masking::{preview_masking, ColumnMask, MaskRule, MaskingPreview, MaskingProfile};

pub(super) fn build_preview(columns_csv: &str, rule: MaskRule, keyed: bool) -> MaskingPreview {
    let columns: Vec<String> = columns_csv
        .split(',')
        .map(str::trim)
        .filter(|column| !column.is_empty())
        .map(str::to_owned)
        .collect();
    let headers = if columns.is_empty() {
        vec!["email".into(), "phone".into(), "id".into()]
    } else {
        let mut headers = columns.clone();
        if !headers.iter().any(|column| column == "id") {
            headers.push("id".into());
        }
        headers
    };
    let sample_row = |email: &str, phone: &str, fallback_prefix: &str| {
        headers
            .iter()
            .map(|header| match header.as_str() {
                "email" => email.to_owned(),
                "phone" => phone.to_owned(),
                "id" => "42".to_owned(),
                _ => format!("{fallback_prefix}_{header}"),
            })
            .collect::<Vec<_>>()
    };
    let sample = vec![
        sample_row("ada@example.com", "1234567890", "val"),
        sample_row("grace@example.com", "0987654321", "val2"),
    ];
    let profile = MaskingProfile {
        name: "preview".into(),
        schema: String::new(),
        table: String::new(),
        columns: columns
            .into_iter()
            .map(|column| ColumnMask {
                column,
                rule,
                replacement: "[masked]".into(),
                keep_prefix: 2,
                keep_suffix: 2,
            })
            .collect(),
        keyed,
        key_id: "local-dev".into(),
    };
    let key_material = if keyed { "db-pro-local-masking-key" } else { "" };
    preview_masking(&headers, &sample, &profile, key_material)
}

#[cfg(test)]
mod tests {
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
}
