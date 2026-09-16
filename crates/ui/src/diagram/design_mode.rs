//! ER Design Mode draft model (#226) — no live DB mutation until explicit apply.

use db_pro_core::application::ObjectMutationService;
use db_pro_core::domain::object_mutation::{
    ColumnDefinition, ForeignKeyDefinition, MutationOptions, ObjectAction, ObjectDefinition, ObjectMutationPreview,
    ObjectMutationRequest, ObjectRef, TableDefinition,
};
use db_pro_core::ports::SqlDialect;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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

pub fn plan_design_draft(
    draft: &DesignDraft,
    driver: &str,
    dialect: &dyn SqlDialect,
) -> Result<Vec<ObjectMutationPreview>, String> {
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
            }),
            options: MutationOptions {
                if_not_exists: true,
                ..MutationOptions::default()
            },
            driver: driver.to_owned(),
        };
        let preview = ObjectMutationService::plan(&request, dialect).map_err(|e| e.to_string())?;
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
        let preview = ObjectMutationService::plan(&request, dialect).map_err(|e| e.to_string())?;
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
mod tests {
    use super::*;

    struct QuoteDialect;
    impl SqlDialect for QuoteDialect {
        fn quote_identifier(&self, name: &str) -> String {
            format!("\"{name}\"")
        }
        fn placeholder(&self, n: usize) -> String {
            format!("${n}")
        }
    }

    #[test]
    fn undo_redo_is_deterministic() {
        let mut state = DesignModeState::default();
        state.add_draft_table("public", "orders");
        assert_eq!(state.draft.tables.len(), 1);
        state.undo();
        assert!(state.draft.tables.is_empty());
        state.redo();
        assert_eq!(state.draft.tables[0].name, "orders");
    }

    #[test]
    fn plan_creates_table_sql() {
        let mut draft = DesignDraft {
            schema_fingerprint: "abc".into(),
            ..DesignDraft::default()
        };
        draft.tables.push(DraftTable {
            schema: "public".into(),
            name: "draft_items".into(),
            columns: vec![
                DraftColumn {
                    name: "id".into(),
                    data_type: "integer".into(),
                    nullable: false,
                    is_pk: true,
                    is_unique: false,
                },
                DraftColumn {
                    name: "title".into(),
                    data_type: "text".into(),
                    nullable: false,
                    is_pk: false,
                    is_unique: false,
                },
            ],
        });
        let previews = plan_design_draft(&draft, "PostgreSQL", &QuoteDialect).unwrap();
        let (sql, fp, _) = merge_preview_sql(&previews);
        assert!(sql.to_ascii_lowercase().contains("create table"));
        assert!(sql.contains("draft_items"));
        assert!(!fp.is_empty());
    }

    #[test]
    fn stale_fingerprint_detected() {
        let mut state = DesignModeState::default();
        state.set_schema_fingerprint("old".into());
        assert!(state.fingerprint_stale("new"));
        assert!(!state.fingerprint_stale("old"));
    }
}
