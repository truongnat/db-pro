use serde::{Deserialize, Serialize};

/// Credentials associated with a connection, stored separately from metadata.
///
/// `ConnectionConfig` holds non-secret metadata (host, port, database, username, driver).
/// `ConnectionSecret` holds sensitive credentials that must never appear in:
/// - workspace persistence
/// - recent connection lists
/// - logs or diagnostics bundles
/// - frontend localStorage
/// - error messages
///
/// The secret is referenced by a `secret_ref` key in the `SecretStore` (e.g. OS keychain).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSecret {
    /// Database password.
    pub password: Option<String>,
    /// SSH tunnel password (if different from database password).
    pub ssh_password: Option<String>,
    /// Private key passphrase for SSH tunnel.
    pub ssh_passphrase: Option<String>,
}

impl ConnectionSecret {
    /// Create an empty secret (no credentials).
    pub fn empty() -> Self {
        Self {
            password: None,
            ssh_password: None,
            ssh_passphrase: None,
        }
    }

    /// Create with only a database password.
    pub fn with_password(password: impl Into<String>) -> Self {
        Self {
            password: Some(password.into()),
            ssh_password: None,
            ssh_passphrase: None,
        }
    }

    /// Whether this secret contains any credentials at all.
    pub fn has_any(&self) -> bool {
        self.password.is_some() || self.ssh_password.is_some() || self.ssh_passphrase.is_some()
    }

    /// Redact all fields for safe display in logs/diagnostics.
    /// Returns a copy with all values replaced by `"[REDACTED]"` if present.
    pub fn redacted(&self) -> RedactedSecret {
        RedactedSecret {
            has_password: self.password.is_some(),
            has_ssh_password: self.ssh_password.is_some(),
            has_ssh_passphrase: self.ssh_passphrase.is_some(),
        }
    }
}

impl Default for ConnectionSecret {
    fn default() -> Self {
        Self::empty()
    }
}

/// A redacted view of `ConnectionSecret` safe for logging and diagnostics.
///
/// Only reveals whether credentials *exist*, never their values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactedSecret {
    pub has_password: bool,
    pub has_ssh_password: bool,
    pub has_ssh_passphrase: bool,
}

impl std::fmt::Display for RedactedSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        if self.has_password {
            parts.push("password=***");
        }
        if self.has_ssh_password {
            parts.push("ssh_password=***");
        }
        if self.has_ssh_passphrase {
            parts.push("ssh_passphrase=***");
        }
        if parts.is_empty() {
            write!(f, "(no secrets)")
        } else {
            write!(f, "{}", parts.join(", "))
        }
    }
}

/// Key format for storing connection secrets in the `SecretStore`.
///
/// Format: `connection:<connection_id>:<field>`
pub fn secret_key(connection_id: &str, field: &str) -> String {
    format!("connection:{connection_id}:{field}")
}

/// Well-known secret field names.
pub mod fields {
    pub const PASSWORD: &str = "password";
    pub const SSH_PASSWORD: &str = "ssh_password";
    pub const SSH_PASSPHRASE: &str = "ssh_passphrase";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_secret_has_none() {
        let s = ConnectionSecret::empty();
        assert!(!s.has_any());
    }

    #[test]
    fn with_password_has_any() {
        let s = ConnectionSecret::with_password("secret123");
        assert!(s.has_any());
    }

    #[test]
    fn redacted_hides_values() {
        let s = ConnectionSecret {
            password: Some("hunter2".into()),
            ssh_password: Some("ssh_pass".into()),
            ssh_passphrase: None,
        };
        let r = s.redacted();
        assert!(r.has_password);
        assert!(r.has_ssh_password);
        assert!(!r.has_ssh_passphrase);
    }

    #[test]
    fn redacted_display_format() {
        let s = ConnectionSecret::with_password("secret");
        let display = s.redacted().to_string();
        assert!(display.contains("password=***"));
        assert!(!display.contains("secret"));
    }

    #[test]
    fn redacted_empty_display() {
        let s = ConnectionSecret::empty();
        assert_eq!(s.redacted().to_string(), "(no secrets)");
    }

    #[test]
    fn secret_key_format() {
        assert_eq!(
            secret_key("abc-123", fields::PASSWORD),
            "connection:abc-123:password"
        );
    }
}
