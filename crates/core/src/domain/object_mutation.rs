//! Typed schema/object mutation domain (#183 / Phase A).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectKind {
    Table,
    Column,
    View,
    MaterializedView,
    Index,
    PrimaryKey,
    ForeignKey,
    UniqueConstraint,
    CheckConstraint,
    Trigger,
    Sequence,
    EnumType,
    DomainType,
    CompositeType,
    /// Stored function or procedure (#192).
    Routine,
    /// Row-level security policy (#219).
    RlsPolicy,
    Schema,
    Database,
    Extension,
    Comment,
    Partition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectAction {
    Create,
    Alter,
    Drop,
    Rename,
    Refresh,
    Enable,
    Disable,
    GenerateDdl,
    Comment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectRef {
    pub kind: ObjectKind,
    pub schema: Option<String>,
    pub name: String,
    pub parent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MutationOptions {
    pub cascade: bool,
    pub if_exists: bool,
    pub if_not_exists: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ObjectDefinition {
    Table(TableDefinition),
    Column(ColumnDefinition),
    View(ViewDefinition),
    MaterializedView(ViewDefinition),
    Index(IndexDefinition),
    ForeignKey(ForeignKeyDefinition),
    UniqueConstraint(NamedColumnsDefinition),
    CheckConstraint(CheckDefinition),
    PrimaryKey(NamedColumnsDefinition),
    Trigger(TriggerDefinition),
    Sequence(SequenceDefinition),
    EnumType(EnumTypeDefinition),
    DomainType(DomainTypeDefinition),
    Schema(SchemaDefinition),
    Database(DatabaseDefinition),
    Extension(ExtensionDefinition),
    Comment(CommentDefinition),
    Partition(PartitionDefinition),
    Routine(RoutineDefinition),
    /// Create/alter/drop a single RLS policy (#219).
    RlsPolicy(RlsPolicyDefinition),
    /// Enable/disable/force table RLS (#219).
    TableRls(TableRlsDefinition),
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableDefinition {
    pub schema: String,
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub schema: String,
    pub table: String,
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub is_pk: bool,
    pub new_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewDefinition {
    pub schema: String,
    pub name: String,
    pub select_sql: String,
    pub materialized: bool,
    pub replace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexDefinition {
    pub schema: String,
    pub table: String,
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub method: Option<String>,
    pub predicate: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedColumnsDefinition {
    pub schema: String,
    pub table: String,
    pub name: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForeignKeyDefinition {
    pub schema: String,
    pub table: String,
    pub name: String,
    pub columns: Vec<String>,
    pub ref_schema: String,
    pub ref_table: String,
    pub ref_columns: Vec<String>,
    pub on_delete: Option<String>,
    pub on_update: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckDefinition {
    pub schema: String,
    pub table: String,
    pub name: String,
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerDefinition {
    pub schema: String,
    pub table: String,
    pub name: String,
    pub timing: String,
    pub event: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceDefinition {
    pub schema: String,
    pub name: String,
    pub start: Option<i64>,
    pub increment: Option<i64>,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub cache: Option<i64>,
    pub cycle: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumTypeDefinition {
    pub schema: String,
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainTypeDefinition {
    pub schema: String,
    pub name: String,
    pub base_type: String,
    pub not_null: bool,
    pub default: Option<String>,
    pub check: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaDefinition {
    pub name: String,
    pub new_name: Option<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseDefinition {
    pub name: String,
    pub owner: Option<String>,
    pub template: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionDefinition {
    pub name: String,
    pub schema: Option<String>,
    pub version: Option<String>,
    pub cascade: bool,
}

/// Stored function / procedure create-or-replace / drop (#192).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutineDefinition {
    pub schema: String,
    pub name: String,
    /// FUNCTION or PROCEDURE
    pub routine_type: String,
    /// Overload identity arguments for DROP (from pg_get_function_identity_arguments).
    pub identity_arguments: String,
    /// Full CREATE [OR REPLACE] FUNCTION/PROCEDURE body for create/alter.
    pub definition_sql: String,
    pub replace: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentDefinition {
    pub object: ObjectRef,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionDefinition {
    pub schema: String,
    pub parent_table: String,
    pub name: String,
    pub strategy: String,
    pub bound_expression: String,
}

/// CREATE/ALTER/DROP POLICY payload. Expression strings are never rewritten.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RlsPolicyDefinition {
    pub schema: String,
    pub table: String,
    pub name: String,
    pub permissive: bool,
    /// ALL / SELECT / INSERT / UPDATE / DELETE
    pub command: String,
    /// Empty → PUBLIC.
    pub roles: Vec<String>,
    pub using_expr: Option<String>,
    pub with_check_expr: Option<String>,
    /// Optional rename target for ALTER POLICY … RENAME TO.
    pub new_name: Option<String>,
}

/// ALTER TABLE … [NO] [FORCE] ROW LEVEL SECURITY payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableRlsDefinition {
    pub schema: String,
    pub table: String,
    /// When true, toggles FORCE ROW LEVEL SECURITY; otherwise toggles RLS itself.
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectMutationRequest {
    pub action: ObjectAction,
    pub target: Option<ObjectRef>,
    pub definition: ObjectDefinition,
    pub options: MutationOptions,
    pub driver: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectMutationPreview {
    pub statements: Vec<String>,
    pub safety: String,
    pub long_running: bool,
    pub effects: Vec<String>,
    pub fingerprint: String,
    pub unsupported_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectMutationResult {
    pub applied: Vec<String>,
    pub refreshed: Vec<ObjectRef>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectDependencyEdge {
    pub from_kind: ObjectKind,
    pub from_schema: Option<String>,
    pub from_name: String,
    pub to_kind: ObjectKind,
    pub to_schema: Option<String>,
    pub to_name: String,
    pub relation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutationError {
    Unsupported { capability: String, reason: String },
    Validation { field: String, message: String },
    Build(String),
}

impl std::fmt::Display for MutationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported { capability, reason } => {
                write!(f, "unsupported ({capability}): {reason}")
            }
            Self::Validation { field, message } => write!(f, "validation ({field}): {message}"),
            Self::Build(message) => write!(f, "build: {message}"),
        }
    }
}

impl std::error::Error for MutationError {}
