use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDiff {
    pub tables_only_in_source: Vec<String>,
    pub tables_only_in_target: Vec<String>,
    pub column_diffs: Vec<TableColumnDiff>,
    pub indexes_only_in_source: Vec<String>,
    pub indexes_only_in_target: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumnDiff {
    pub schema: String,
    pub table: String,
    pub columns_only_in_source: Vec<String>,
    pub columns_only_in_target: Vec<String>,
    pub type_mismatches: Vec<ColumnTypeMismatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnTypeMismatch {
    pub column: String,
    pub source_type: String,
    pub target_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDiff {
    pub schema: String,
    pub table: String,
    pub source_row_count: i64,
    pub target_row_count: i64,
    pub row_count_diff: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDependency {
    pub object_type: String,
    pub object_name: String,
    pub depends_on_type: String,
    pub depends_on_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    pub schema: String,
    pub table: String,
    pub partition_strategy: String,
    pub partitions: Vec<PartitionChild>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionChild {
    pub name: String,
    pub bound_expr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TablespaceInfo {
    pub name: String,
    pub owner: String,
    pub location: String,
}
