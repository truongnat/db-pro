//! State owned by the masking surface.

pub(super) struct MaskingState {
    pub(super) masking_columns_csv: String,
    pub(super) masking_rule: db_pro_core::domain::masking::MaskRule,
    pub(super) masking_keyed: bool,
    pub(super) masking_preview: Option<db_pro_core::domain::masking::MaskingPreview>,
    pub(super) masking_error: Option<String>,
}

impl Default for MaskingState {
    fn default() -> Self {
        Self {
            masking_columns_csv: "email,phone".to_owned(),
            masking_rule: db_pro_core::domain::masking::MaskRule::PartialReveal,
            masking_keyed: true,
            masking_preview: None,
            masking_error: None,
        }
    }
}
