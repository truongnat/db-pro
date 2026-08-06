pub mod backup_engine;
pub mod connection_repository;
pub mod db_connector;
pub mod dialect;
pub mod introspection_cache;
pub mod query_history_repository;
pub mod run_config_repository;
pub mod saved_query_repository;
pub mod secret_store;
pub mod settings_repository;
pub mod user_manager;
pub mod workspace_repository;

pub use backup_engine::BackupEngine;
pub use connection_repository::ConnectionRepository;
pub use db_connector::DbConnector;
pub use dialect::SqlDialect;
pub use introspection_cache::IntrospectionCache;
pub use query_history_repository::QueryHistoryRepository;
pub use run_config_repository::RunConfigRepository;
pub use saved_query_repository::SavedQueryRepository;
pub use secret_store::SecretStore;
pub use settings_repository::SettingsRepository;
pub use user_manager::UserManager;
pub use workspace_repository::WorkspaceRepository;

#[cfg(test)]
pub use connection_repository::MockConnectionRepository;
#[cfg(test)]
pub use db_connector::MockDbConnector;
#[cfg(test)]
pub use introspection_cache::MockIntrospectionCache;
#[cfg(test)]
pub use query_history_repository::MockQueryHistoryRepository;
#[cfg(test)]
pub use run_config_repository::MockRunConfigRepository;
#[cfg(test)]
pub use saved_query_repository::MockSavedQueryRepository;
#[cfg(test)]
pub use secret_store::MockSecretStore;
#[cfg(test)]
pub use user_manager::MockUserManager;
