//! State and effect planning for the audit surface.

#[derive(Default)]
pub(super) struct AuditState {
    pub(super) audit_page: Option<db_pro_core::domain::audit::AuditPage>,
    pub(super) audit_page_request: Option<(u64, String)>,
    pub(super) audit_error: Option<String>,
    pub(super) audit_filter_text: String,
    pub(super) audit_filter_database: String,
    pub(super) audit_filter_username: String,
    pub(super) audit_filter_severity: String,
    pub(super) audit_bookmarks: std::collections::HashSet<String>,
    pub(super) audit_selected: std::collections::HashSet<String>,
    pub(super) audit_export_preview: Option<String>,
}

impl AuditState {
    pub(super) fn audit_filter(&self) -> db_pro_core::domain::audit::AuditFilter {
        db_pro_core::domain::audit::AuditFilter {
            text: self.audit_filter_text.clone(),
            database: self.audit_filter_database.clone(),
            username: self.audit_filter_username.clone(),
            severity: self.audit_filter_severity.clone(),
            command_tag: String::new(),
        }
    }

    pub(super) fn build_export_preview(&mut self) -> Result<(usize, String), String> {
        let Some(page) = &self.audit_page else {
            return Err("Load an audit page before exporting".to_owned());
        };
        let selected: Vec<_> = page
            .events
            .iter()
            .filter(|event| self.audit_selected.contains(&event.id) || self.audit_bookmarks.contains(&event.id))
            .cloned()
            .collect();
        if selected.is_empty() {
            return Err("Select or bookmark events to export".to_owned());
        }
        let selected_count = selected.len();
        self.audit_export_preview = Some(db_pro_core::application::AuditService::export_selected(&selected));
        self.audit_error = None;
        Ok((selected_count, page.export_warning.clone()))
    }
}

#[cfg(test)]
#[path = "tests/audit_state_tests.rs"]
mod tests;

