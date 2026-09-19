//! Feature-owned selection state for query result/output tabs.

use super::OutputTab;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct QueryOutputState {
    pub(crate) active_tab: OutputTab,
    pub(crate) tabs_by_document: HashMap<String, OutputTab>,
}

impl Default for QueryOutputState {
    fn default() -> Self {
        Self {
            active_tab: OutputTab::Results,
            tabs_by_document: HashMap::new(),
        }
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
}
