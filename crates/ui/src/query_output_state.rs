//! Feature-owned selection state for query result/output tabs.

use super::OutputTab;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct QueryOutputState {
    pub(super) active_tab: OutputTab,
    pub(super) tabs_by_document: HashMap<String, OutputTab>,
}

impl Default for QueryOutputState {
    fn default() -> Self {
        Self {
            active_tab: OutputTab::Results,
            tabs_by_document: HashMap::new(),
        }
    }
}

impl QueryOutputState {
    pub(super) fn tab_for_document(&self, document_id: &str) -> OutputTab {
        self.tabs_by_document
            .get(document_id)
            .copied()
            .unwrap_or(OutputTab::Results)
    }

    pub(crate) fn active_tab_for_document(&self, document_id: Option<&str>) -> OutputTab {
        document_id
            .map(|document_id| self.tab_for_document(document_id))
            .unwrap_or(OutputTab::Results)
    }

    pub(super) fn set_active_for_document(&mut self, document_id: &str, tab: OutputTab) {
        self.active_tab = tab;
        self.tabs_by_document.insert(document_id.to_owned(), tab);
    }

    pub(crate) fn set_active_for_optional_document(&mut self, document_id: Option<&str>, tab: OutputTab) {
        if let Some(document_id) = document_id {
            self.set_active_for_document(document_id, tab);
        } else {
            self.active_tab = tab;
        }
    }

    pub(super) fn set_for_document(&mut self, document_id: &str, tab: OutputTab) {
        self.tabs_by_document.insert(document_id.to_owned(), tab);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_state_defaults_to_results_without_document_overrides() {
        let state = QueryOutputState::default();

        assert_eq!(state.active_tab, OutputTab::Results);
        assert!(state.tabs_by_document.is_empty());
    }

    #[test]
    fn document_tab_overrides_are_owned_by_output_state() {
        let mut state = QueryOutputState::default();

        state.set_active_for_document("query-1", OutputTab::Explain);
        assert_eq!(state.tab_for_document("query-1"), OutputTab::Explain);
        assert_eq!(state.active_tab, OutputTab::Explain);

        state.set_for_document("query-2", OutputTab::Messages);
        assert_eq!(state.tab_for_document("query-2"), OutputTab::Messages);
        assert_eq!(state.active_tab, OutputTab::Explain);
    }
}
