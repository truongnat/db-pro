use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DbEngine {
    Sqlite,
    Postgres,
    Mysql,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInput {
    pub id: Option<String>,
    pub name: String,
    pub engine: DbEngine,
    pub path: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionProfile {
    pub id: String,
    pub name: String,
    pub engine: DbEngine,
    pub path: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionListItem {
    pub id: String,
    pub name: String,
    pub engine: DbEngine,
    pub target: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequest {
    pub connection_id: String,
    pub sql: String,
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub offset: Option<u64>,
    pub timeout_ms: Option<u64>,
    pub quick_filter: Option<String>,
    pub filter_columns: Option<Vec<String>>,
    pub sort_column: Option<String>,
    pub sort_direction: Option<SortDirection>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub affected_rows: u64,
    pub execution_ms: u128,
    pub message: String,
    pub schema_changed: bool,
    pub is_row_query: bool,
    pub page_size: u32,
    pub page_offset: u64,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigatorTree {
    pub schemas: Vec<NavigatorSchema>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigatorSchema {
    pub name: String,
    pub tables: Vec<NavigatorObject>,
    pub views: Vec<NavigatorObject>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigatorObject {
    pub name: String,
    pub kind: NavigatorObjectKind,
    pub columns: Vec<NavigatorColumn>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NavigatorObjectKind {
    Table,
    View,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigatorColumn {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
}

impl ConnectionListItem {
    pub fn from_profile(profile: &ConnectionProfile) -> Self {
        let target = match profile.engine {
            DbEngine::Sqlite => profile
                .path
                .clone()
                .unwrap_or_else(|| "(missing path)".to_string()),
            DbEngine::Postgres | DbEngine::Mysql => {
                let host = profile
                    .host
                    .clone()
                    .unwrap_or_else(|| "localhost".to_string());
                let database = profile.database.clone().unwrap_or_else(|| "db".to_string());
                format!("{host}/{database}")
            }
        };

        Self {
            id: profile.id.clone(),
            name: profile.name.clone(),
            engine: profile.engine.clone(),
            target,
        }
    }
}
