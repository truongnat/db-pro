use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(Uuid);

impl ConnectionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DriverType {
    Postgres,
    SQLite,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SslMode {
    #[default]
    Disable,
    Require,
    VerifyCa,
    VerifyFull,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshTunnelConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub private_key_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    #[serde(skip)]
    pub encrypted_password: Vec<u8>,
    pub driver: DriverType,
    #[serde(default)]
    pub ssl_mode: SslMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_tunnel: Option<SshTunnelConfig>,
    #[serde(default = "default_query_timeout")]
    pub query_timeout_ms: u64,
    #[serde(default = "default_max_rows")]
    pub max_rows: u64,
}

fn default_query_timeout() -> u64 {
    30_000
}

fn default_max_rows() -> u64 {
    500
}

impl ConnectionConfig {
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if self.name.trim().is_empty() {
            errors.push(ValidationError {
                field: "name".into(),
                message: "Connection name is required".into(),
            });
        }

        if self.host.trim().is_empty() {
            errors.push(ValidationError {
                field: "host".into(),
                message: "Host is required".into(),
            });
        }

        if self.port == 0 {
            errors.push(ValidationError {
                field: "port".into(),
                message: "Port must be between 1 and 65535".into(),
            });
        }

        if self.database.trim().is_empty() {
            errors.push(ValidationError {
                field: "database".into(),
                message: "Database is required".into(),
            });
        }

        if self.username.trim().is_empty() {
            errors.push(ValidationError {
                field: "username".into(),
                message: "Username is required".into(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub id: ConnectionId,
    pub config: ConnectionConfig,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Connection {
    pub fn new(config: ConnectionConfig) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: ConnectionId::new(),
            config,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionHandle(pub u64);

impl ConnectionHandle {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> ConnectionConfig {
        ConnectionConfig {
            name: "test".into(),
            host: "localhost".into(),
            port: 5432,
            database: "testdb".into(),
            username: "user".into(),
            encrypted_password: vec![],
            driver: DriverType::Postgres,
            ssl_mode: SslMode::Disable,
            ssh_tunnel: None,
            query_timeout_ms: 30_000,
            max_rows: 500,
        }
    }

    #[test]
    fn connection_id_parse_valid_uuid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = ConnectionId::parse(uuid_str).unwrap();
        assert_eq!(id.to_string(), uuid_str);
    }

    #[test]
    fn connection_id_parse_invalid_uuid() {
        let result = ConnectionId::parse("not-a-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn connection_config_validate_valid() {
        let config = valid_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn connection_config_validate_empty_name() {
        let mut config = valid_config();
        config.name = "".into();
        let errors = config.validate().unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "name");
    }

    #[test]
    fn connection_config_validate_multiple_errors() {
        let config = ConnectionConfig {
            name: "".into(),
            host: "".into(),
            port: 0,
            database: "".into(),
            username: "".into(),
            encrypted_password: vec![],
            driver: DriverType::Postgres,
            ssl_mode: SslMode::Disable,
            ssh_tunnel: None,
            query_timeout_ms: 30_000,
            max_rows: 500,
        };
        let errors = config.validate().unwrap_err();
        assert_eq!(errors.len(), 5);
    }
}
