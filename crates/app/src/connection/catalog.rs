// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use crate::UiConnectionSummary;

/// Read model of saved connections exposed to explorer and workspace surfaces.
#[derive(Debug, Default)]
pub(crate) struct ConnectionCatalogState {
    #[cfg(test)]
    pub(in crate::app) connections: Vec<UiConnectionSummary>,
    #[cfg(not(test))]
    connections: Vec<UiConnectionSummary>,
}

impl ConnectionCatalogState {
    pub(crate) fn len(&self) -> usize {
        self.connections.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.connections.is_empty()
    }

    pub(crate) fn get(&self, index: usize) -> Option<&UiConnectionSummary> {
        self.connections.get(index)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &UiConnectionSummary> {
        self.connections.iter()
    }

    #[cfg(test)]
    pub(crate) fn connections_mut(&mut self) -> &mut Vec<UiConnectionSummary> {
        &mut self.connections
    }

    pub(crate) fn replace(&mut self, connections: Vec<UiConnectionSummary>) {
        self.connections = connections;
    }

    pub(crate) fn find(&self, connection_id: &str) -> Option<&UiConnectionSummary> {
        self.connections
            .iter()
            .find(|connection| connection.id == connection_id)
    }
}

#[cfg(test)]
#[path = "tests/catalog_tests.rs"]
mod tests;

