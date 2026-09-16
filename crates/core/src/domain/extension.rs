//! Native Plugin / Extension SDK model (#256).
//!
//! First iteration is an **in-process, capability-gated** registry — not an
//! arbitrary native-code marketplace. Unsigned third-party `.so`/dylib loading
//! stays deferred until a sandbox/trust policy exists.
//!
//! Extensions declare a manifest, required host version, permissions, and
//! contributions (commands, optional agent tools, sidebar metadata). Load
//! failures are isolated: the host continues with the failed extension skipped.

use serde::{Deserialize, Serialize};

/// Host API major.minor the SDK currently exposes.
pub const HOST_API_VERSION: &str = "1.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    /// Minimum host API version required (`major.minor`).
    pub min_host_api: String,
    pub permissions: Vec<ExtensionPermission>,
    pub contributions: ExtensionContributions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionPermission {
    RegisterCommand,
    RegisterAgentTool,
    ContributeSidebar,
    ReadSchema,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExtensionContributions {
    pub commands: Vec<ExtensionCommand>,
    pub agent_tools: Vec<ExtensionAgentTool>,
    pub sidebar_items: Vec<ExtensionSidebarItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionCommand {
    pub id: String,
    pub title: String,
    /// Optional capability id from the #234 provider contract this command needs.
    pub required_capability: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionAgentTool {
    pub id: String,
    pub title: String,
    /// Destructive tools must still route through canonical confirmation.
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionSidebarItem {
    pub id: String,
    pub title: String,
    pub activity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionLoadStatus {
    Loaded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionLoadReport {
    pub id: String,
    pub status: ExtensionLoadStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionLoadError {
    IncompatibleHost { required: String, host: String },
    MissingPermission(ExtensionPermission),
    InvalidManifest(String),
    DuplicateId(String),
}

impl std::fmt::Display for ExtensionLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IncompatibleHost { required, host } => {
                write!(f, "extension requires host API {required}, host is {host}")
            }
            Self::MissingPermission(p) => write!(f, "missing permission {p:?}"),
            Self::InvalidManifest(m) => write!(f, "invalid manifest: {m}"),
            Self::DuplicateId(id) => write!(f, "duplicate extension id `{id}`"),
        }
    }
}

/// Compare `major.minor` host versions. Returns true when `host >= required`.
pub fn host_api_satisfies(host: &str, required: &str) -> bool {
    fn parse(v: &str) -> Option<(u32, u32)> {
        let mut parts = v.trim().split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next().unwrap_or("0").parse().ok()?;
        Some((major, minor))
    }
    match (parse(host), parse(required)) {
        (Some((hm, hn)), Some((rm, rn))) => hm > rm || (hm == rm && hn >= rn),
        _ => false,
    }
}

/// Built-in example extension used for conformance tests and docs.
pub fn example_diagnostics_extension() -> ExtensionManifest {
    ExtensionManifest {
        id: "dbpro.example.diagnostics".into(),
        name: "Example Diagnostics Helper".into(),
        version: "0.1.0".into(),
        min_host_api: "1.0".into(),
        permissions: vec![
            ExtensionPermission::RegisterCommand,
            ExtensionPermission::ContributeSidebar,
        ],
        contributions: ExtensionContributions {
            commands: vec![ExtensionCommand {
                id: "example.open_diagnostics_hint".into(),
                title: "Open diagnostics setup hint".into(),
                required_capability: None,
            }],
            agent_tools: Vec::new(),
            sidebar_items: vec![ExtensionSidebarItem {
                id: "example.diagnostics_hint".into(),
                title: "Diagnostics Hint".into(),
                activity: "monitor".into(),
            }],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_version_compare() {
        assert!(host_api_satisfies("1.0", "1.0"));
        assert!(host_api_satisfies("1.2", "1.0"));
        assert!(!host_api_satisfies("1.0", "1.1"));
        assert!(!host_api_satisfies("0.9", "1.0"));
    }

    #[test]
    fn example_manifest_serializes() {
        let json = serde_json::to_string(&example_diagnostics_extension()).unwrap();
        assert!(json.contains("dbpro.example.diagnostics"));
    }
}
