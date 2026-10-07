use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// `eframe::Storage` key for per-connection ER layout snapshots.
pub const ER_LAYOUTS_STORAGE_KEY: &str = "dbpro.native.er-layouts-v1";

/// Persisted diagram viewport + manual node positions for one connection.
/// Positions are keyed by `schema.name` (see [`super::model::er_table_key`]) so
/// they survive graph rebuilds and schema refreshes; entries for dropped tables
/// are ignored on restore.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErLayoutSnapshot {
    #[serde(default)]
    pub positions: HashMap<String, [f32; 2]>,
    #[serde(default)]
    pub zoom: Option<f32>,
    #[serde(default)]
    pub pan: Option<[f32; 2]>,
}
