//! State owned by the synthetic-data surface.

pub(super) struct SyntheticDataState {
    pub(super) synthetic_table: String,
    pub(super) synthetic_row_count: String,
    pub(super) synthetic_seed: String,
    pub(super) synthetic_null_pct: String,
    pub(super) synthetic_preview: Option<db_pro_core::domain::synthetic_data::SyntheticPreview>,
    pub(super) synthetic_error: Option<String>,
    pub(super) synthetic_production_confirm: bool,
}

impl Default for SyntheticDataState {
    fn default() -> Self {
        Self {
            synthetic_table: String::new(),
            synthetic_row_count: "10".to_owned(),
            synthetic_seed: "42".to_owned(),
            synthetic_null_pct: "0".to_owned(),
            synthetic_preview: None,
            synthetic_error: None,
            synthetic_production_confirm: false,
        }
    }
}
