//! ER Design Mode draft model (#226) — no live DB mutation until explicit apply.

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use db_pro_core::{
    application::ObjectMutationService,
    domain::object_mutation::{
        ColumnDefinition, ForeignKeyDefinition, MutationOptions, ObjectAction, ObjectDefinition, ObjectMutationPreview,
        ObjectMutationRequest, ObjectRef, TableDefinition,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftColumn {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_pk: bool,
    pub is_unique: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftTable {
    pub schema: String,
    pub name: String,
    pub columns: Vec<DraftColumn>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftForeignKey {
    pub name: String,
    pub from_schema: String,
    pub from_table: String,
    pub from_columns: Vec<String>,
    pub to_schema: String,
    pub to_table: String,
    pub to_columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DesignDraft {
    pub tables: Vec<DraftTable>,
    pub foreign_keys: Vec<DraftForeignKey>,
    pub schema_fingerprint: String,
}

#[derive(Debug, Clone, Default)]
pub struct DesignModeState {
    pub enabled: bool,
    pub draft: DesignDraft,
    undo: Vec<DesignDraft>,
    redo: Vec<DesignDraft>,
    pub preview_sql: String,
    pub preview_fingerprint: String,
    pub preview_effects: Vec<String>,
    pub error: Option<String>,
    pub apply_confirm: bool,
    /// In-flight design Apply. The draft stays until this DDL request succeeds.
    pub apply_request: Option<u64>,
}

impl DesignModeState {
    pub fn push_undo(&mut self) {
        self.undo.push(self.draft.clone());
        self.redo.clear();
        if self.undo.len() > 64 {
            self.undo.remove(0);
        }
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.undo.pop() {
            self.redo.push(self.draft.clone());
            self.draft = prev;
            self.clear_preview();
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(self.draft.clone());
            self.draft = next;
            self.clear_preview();
        }
    }

    pub fn clear_preview(&mut self) {
        self.preview_sql.clear();
        self.preview_fingerprint.clear();
        self.preview_effects.clear();
        self.error = None;
        self.apply_confirm = false;
    }

    pub fn discard(&mut self) {
        // A manual discard is not waiting on the in-flight DDL.
        self.apply_request = None;
        self.push_undo();
        let fp = self.draft.schema_fingerprint.clone();
        self.draft = DesignDraft {
            schema_fingerprint: fp,
            ..DesignDraft::default()
        };
        self.clear_preview();
    }

    pub fn add_draft_table(&mut self, schema: &str, name: &str) {
        if name.trim().is_empty() {
            self.error = Some("draft table name required".into());
            return;
        }
        self.push_undo();
        self.draft.tables.push(DraftTable {
            schema: schema.to_owned(),
            name: name.trim().to_owned(),
            columns: vec![DraftColumn {
                name: "id".into(),
                data_type: "integer".into(),
                nullable: false,
                is_pk: true,
                is_unique: false,
            }],
        });
        self.clear_preview();
    }

    pub fn add_column(&mut self, table_index: usize, column: DraftColumn) {
        if table_index >= self.draft.tables.len() {
            return;
        }
        self.push_undo();
        self.draft.tables[table_index].columns.push(column);
        self.clear_preview();
    }

    pub fn add_fk(&mut self, fk: DraftForeignKey) {
        self.push_undo();
        self.draft.foreign_keys.push(fk);
        self.clear_preview();
    }

    pub fn set_schema_fingerprint(&mut self, fingerprint: String) {
        self.draft.schema_fingerprint = fingerprint;
    }

    pub fn fingerprint_stale(&self, live: &str) -> bool {
        !self.draft.schema_fingerprint.is_empty() && self.draft.schema_fingerprint != live
    }
}

pub fn schema_fingerprint_from_names(names: &[String]) -> String {
    let mut hasher = DefaultHasher::new();
    for name in names {
        name.hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

pub fn plan_design_draft(draft: &DesignDraft, driver: &str) -> Result<Vec<ObjectMutationPreview>, String> {
    if draft.tables.is_empty() && draft.foreign_keys.is_empty() {
        return Err("draft is empty".into());
    }
    let mut previews = Vec::new();
    for table in &draft.tables {
        let columns = table
            .columns
            .iter()
            .map(|c| ColumnDefinition {
                schema: table.schema.clone(),
                table: table.name.clone(),
                name: c.name.clone(),
                data_type: c.data_type.clone(),
                nullable: c.nullable,
                default: None,
                is_pk: c.is_pk,
                new_name: None,
                origin_name: None,
            })
            .collect();
        let request = ObjectMutationRequest {
            action: ObjectAction::Create,
            target: Some(ObjectRef {
                kind: db_pro_core::domain::object_mutation::ObjectKind::Table,
                schema: Some(table.schema.clone()).filter(|s| !s.is_empty()),
                name: table.name.clone(),
                parent: None,
            }),
            definition: ObjectDefinition::Table(TableDefinition {
                schema: table.schema.clone(),
                name: table.name.clone(),
                columns,
                baseline: Vec::new(),
            }),
            options: MutationOptions {
                if_not_exists: true,
                ..MutationOptions::default()
            },
            driver: driver.to_owned(),
        };
        let preview = ObjectMutationService::plan_for_driver(&request).map_err(|e| e.to_string())?;
        if let Some(reason) = preview.unsupported_reason {
            return Err(reason);
        }
        previews.push(preview);
    }
    for fk in &draft.foreign_keys {
        let request = ObjectMutationRequest {
            action: ObjectAction::Create,
            target: Some(ObjectRef {
                kind: db_pro_core::domain::object_mutation::ObjectKind::ForeignKey,
                schema: Some(fk.from_schema.clone()).filter(|s| !s.is_empty()),
                name: fk.name.clone(),
                parent: Some(fk.from_table.clone()),
            }),
            definition: ObjectDefinition::ForeignKey(ForeignKeyDefinition {
                schema: fk.from_schema.clone(),
                table: fk.from_table.clone(),
                name: fk.name.clone(),
                columns: fk.from_columns.clone(),
                ref_schema: fk.to_schema.clone(),
                ref_table: fk.to_table.clone(),
                ref_columns: fk.to_columns.clone(),
                on_delete: None,
                on_update: None,
            }),
            options: MutationOptions::default(),
            driver: driver.to_owned(),
        };
        let preview = ObjectMutationService::plan_for_driver(&request).map_err(|e| e.to_string())?;
        if let Some(reason) = preview.unsupported_reason {
            return Err(reason);
        }
        previews.push(preview);
    }
    Ok(previews)
}

pub fn merge_preview_sql(previews: &[ObjectMutationPreview]) -> (String, String, Vec<String>) {
    let mut sql = String::new();
    let mut effects = Vec::new();
    let mut hasher = DefaultHasher::new();
    for preview in previews {
        for stmt in &preview.statements {
            sql.push_str(stmt);
            if !stmt.trim_end().ends_with(';') {
                sql.push(';');
            }
            sql.push('\n');
            stmt.hash(&mut hasher);
        }
        effects.extend(preview.effects.clone());
    }
    (sql, format!("{:016x}", hasher.finish()), effects)
}

#[cfg(test)]
#[path = "tests/design_mode_tests.rs"]
mod tests;
