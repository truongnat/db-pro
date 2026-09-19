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
mod tests {
    use super::*;

    #[test]
    fn replace_replaces_the_saved_connection_read_model() {
        let mut catalog = ConnectionCatalogState::default();
        catalog.replace(vec![UiConnectionSummary {
            id: "conn-1".to_owned(),
            name: "Local".to_owned(),
            host: "localhost".to_owned(),
            port: 5432,
            database: "app".to_owned(),
            username: "postgres".to_owned(),
            driver: "PostgreSQL".to_owned(),
            ssl_mode: crate::UiSslMode::Require,
            readonly: false,
            tags: Vec::new(),
            group: None,
            favorite: false,
            environment: "Development".to_owned(),
        }]);

        assert_eq!(
            catalog.find("conn-1").map(|connection| connection.name.as_str()),
            Some("Local")
        );
        assert!(catalog.find("missing").is_none());
    }
}
