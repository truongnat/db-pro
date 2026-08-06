use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BackupFormat {
    Plain,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupOptions {
    pub connection_id: String,
    pub output_path: String,
    pub format: BackupFormat,
    #[serde(default)]
    pub schemas: Vec<String>,
    #[serde(default)]
    pub tables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreOptions {
    pub connection_id: String,
    pub input_path: String,
    pub format: BackupFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub output_path: String,
    pub size_bytes: u64,
}
