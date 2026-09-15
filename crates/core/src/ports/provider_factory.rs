use crate::domain::capabilities::DatabaseCapabilities;
use crate::domain::connection::ConnectionConfig;
use crate::domain::error::DbError;
use crate::ports::DbConnector;

/// Factory for constructing a provider connector and its capability contract.
///
/// Each concrete provider implements this trait once and registers the
/// implementation with [`crate::infrastructure::connector::CompositeConnector`].
/// The composite dispatches to the factory that matches the connection's
/// driver type, so adding a new engine does not touch the dispatch match.
#[async_trait::async_trait]
pub trait ProviderFactory: Send + Sync {
    /// The driver this factory serves.
    fn driver(&self) -> crate::domain::connection::DriverType;

    /// The capabilities the provider advertises for the given connection
    /// config. The composite exposes this via its own `capabilities()`
    /// method so the UI and application layers can branch on capability
    /// instead of driver type.
    fn capabilities(&self, config: &ConnectionConfig) -> DatabaseCapabilities;

    /// Construct the connector that will execute queries for this driver.
    fn build(&self) -> Box<dyn DbConnector>;

    /// Probe connectivity with the given config without establishing a
    /// long-lived connection. Returns `Ok(())` on success, a structured
    /// error otherwise — used by the "Test Connection" action.
    async fn test_connection(&self, config: &ConnectionConfig, password: &str) -> Result<(), DbError>;
}
