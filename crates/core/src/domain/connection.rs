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

const MAX_CONNECTION_NAME_LEN: usize = 128;
const MAX_MAX_ROWS: u64 = 100_000;

impl ConnectionConfig {
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if self.name.trim().is_empty() {
            errors.push(ValidationError {
                field: "name".into(),
                message: "Connection name is required".into(),
            });
        } else if self.name.len() > MAX_CONNECTION_NAME_LEN {
            errors.push(ValidationError {
                field: "name".into(),
                message: format!("Connection name must be at most {MAX_CONNECTION_NAME_LEN} characters"),
            });
        }

        match self.driver {
            DriverType::Postgres => {
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
                if self.username.trim().is_empty() {
                    errors.push(ValidationError {
                        field: "username".into(),
                        message: "Username is required".into(),
                    });
                }
            }
            DriverType::SQLite => {}
        }

        if self.database.trim().is_empty() {
            errors.push(ValidationError {
                field: "database".into(),
                message: "Database is required".into(),
            });
        }

        if self.query_timeout_ms == 0 {
            errors.push(ValidationError {
                field: "query_timeout_ms".into(),
                message: "Query timeout must be greater than 0".into(),
            });
        }

        if self.max_rows == 0 || self.max_rows > MAX_MAX_ROWS {
            errors.push(ValidationError {
                field: "max_rows".into(),
                message: format!("max_rows must be between 1 and {MAX_MAX_ROWS}"),
            });
        }

        if let Some(ref ssh) = self.ssh_tunnel {
            if ssh.host.trim().is_empty() {
                errors.push(ValidationError {
                    field: "ssh_tunnel.host".into(),
                    message: "SSH tunnel host is required".into(),
                });
            }
            if ssh.port == 0 {
                errors.push(ValidationError {
                    field: "ssh_tunnel.port".into(),
                    message: "SSH tunnel port must be between 1 and 65535".into(),
                });
            }
            if ssh.user.trim().is_empty() {
                errors.push(ValidationError {
                    field: "ssh_tunnel.user".into(),
                    message: "SSH tunnel user is required".into(),
                });
            }
            if ssh.private_key_path.trim().is_empty() {
                errors.push(ValidationError {
                    field: "ssh_tunnel.private_key_path".into(),
                    message: "SSH tunnel private key path is required".into(),
                });
            }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: ConnectionId,
    pub config: ConnectionConfig,
    pub secret_ref: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Connection {
    pub fn new(config: ConnectionConfig) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: ConnectionId::new(),
            config,
            secret_ref: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_secret_ref(mut self, secret_ref: String) -> Self {
        self.secret_ref = Some(secret_ref);
        self
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
            driver: DriverType::Postgres,
            ssl_mode: SslMode::Disable,
            ssh_tunnel: None,
            query_timeout_ms: 30_000,
            max_rows: 500,
        };
        let errors = config.validate().unwrap_err();
        assert_eq!(errors.len(), 5);
    }

    #[test]
    fn sqlite_does_not_require_host_port_username() {
        let config = ConnectionConfig {
            name: "local-db".into(),
            host: "".into(),
            port: 0,
            database: "/tmp/test.db".into(),
            username: "".into(),
            driver: DriverType::SQLite,
            ssl_mode: SslMode::Disable,
            ssh_tunnel: None,
            query_timeout_ms: 30_000,
            max_rows: 500,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_zero_query_timeout() {
        let mut config = valid_config();
        config.query_timeout_ms = 0;
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.field == "query_timeout_ms"));
    }

    #[test]
    fn validate_max_rows_zero() {
        let mut config = valid_config();
        config.max_rows = 0;
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.field == "max_rows"));
    }

    #[test]
    fn validate_max_rows_exceedses_limit() {
        let mut config = valid_config();
        config.max_rows = MAX_MAX_ROWS + 1;
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.field == "max_rows"));
    }

    #[test]
    fn validate_name_too_long() {
        let mut config = valid_config();
        config.name = "x".repeat(MAX_CONNECTION_NAME_LEN + 1);
        let errors = config.validate().unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "name");
    }

    #[test]
    fn validate_ssh_tunnel_empty_fields() {
        let mut config = valid_config();
        config.ssh_tunnel = Some(SshTunnelConfig {
            host: "".into(),
            port: 0,
            user: "".into(),
            private_key_path: "".into(),
        });
        let errors = config.validate().unwrap_err();
        let ssh_fields: Vec<&str> = errors
            .iter()
            .filter(|e| e.field.starts_with("ssh_tunnel."))
            .map(|e| e.field.as_str())
            .collect();
        assert_eq!(ssh_fields.len(), 4);
    }
}
