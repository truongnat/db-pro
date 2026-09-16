use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationRisk {
    Safe,
    Mutating,
    Destructive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationOpKind {
    CreateTable,
    DropTable,
    AddColumn,
    DropColumn,
    AlterColumnType,
    CreateIndex,
    DropIndex,
    CreateView,
    DropView,
    CreateRoutine,
    DropRoutine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationOperation {
    pub id: String,
    pub kind: MigrationOpKind,
    pub schema: String,
    pub object: String,
    pub sql: String,
    pub risk: MigrationRisk,
    pub dependencies: Vec<String>,
    pub provider_supported: bool,
    pub unsupported_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub driver: String,
    pub operations: Vec<MigrationOperation>,
    pub fingerprint: String,
    pub warnings: Vec<String>,
    pub has_destructive: bool,
}
