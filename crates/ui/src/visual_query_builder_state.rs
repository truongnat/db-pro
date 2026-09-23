//! State and transitions for the visual SELECT query builder.

use crate::query::visual_builder::{
    try_import_select, BuilderColumn, BuilderDialect, BuilderJoin, BuilderPredicate, BuilderTable, VisualQueryModel,
};
use crate::UiTableSummary;

/// Owns the visual builder draft, its form inputs, and the generated preview.
///
/// The view renders this state, while the app remains responsible only for
/// connecting it to the active query document and schema catalog.
#[derive(Debug)]
pub(crate) struct VisualQueryBuilderState {
    pub(super) open: bool,
    pub(super) model: VisualQueryModel,
    pub(super) sql_preview: String,
    pub(super) error: Option<String>,
    pub(super) add_table: String,
    pub(super) join_table: String,
    pub(super) join_left: String,
    pub(super) join_right: String,
    pub(super) col_ref: String,
    pub(super) col_alias: String,
    pub(super) col_agg: String,
    pub(super) where_left: String,
    pub(super) where_op: String,
    pub(super) where_value: String,
    pub(super) order: String,
    pub(super) order_desc: bool,
    pub(super) limit: String,
    pub(super) offset: String,
}

impl Default for VisualQueryBuilderState {
    fn default() -> Self {
        Self {
            open: false,
            model: VisualQueryModel::default(),
            sql_preview: String::new(),
            error: None,
            add_table: String::new(),
            join_table: String::new(),
            join_left: String::new(),
            join_right: String::new(),
            col_ref: String::new(),
            col_alias: String::new(),
            col_agg: String::new(),
            where_left: String::new(),
            where_op: "=".to_owned(),
            where_value: String::new(),
            order: String::new(),
            order_desc: false,
            limit: "100".to_owned(),
            offset: String::new(),
        }
    }
}

impl VisualQueryBuilderState {
    pub(super) fn dialect_for_driver(driver: &str) -> BuilderDialect {
        if driver.to_ascii_lowercase().contains("sqlite") {
            BuilderDialect::Sqlite
        } else {
            BuilderDialect::Postgres
        }
    }

    pub(super) fn clear(&mut self) {
        self.model.clear();
        self.error = None;
        self.sql_preview.clear();
    }

    pub(super) fn refresh_preview(&mut self, dialect: BuilderDialect) {
        match self.model.generate_sql(dialect) {
            Ok(sql) => {
                self.sql_preview = sql;
                self.error = None;
            }
            Err(err) => {
                self.sql_preview.clear();
                if self.model.tables.is_empty() {
                    self.error = None;
                } else {
                    self.error = Some(err);
                }
            }
        }
    }

    pub(super) fn apply_limit_offset(&mut self) {
        self.model.limit = self.limit.parse().ok();
        self.model.offset = self.offset.parse().ok();
    }

    pub(super) fn remove_table(&mut self, index: usize, dialect: BuilderDialect) {
        self.model.tables.remove(index);
        self.refresh_preview(dialect);
    }

    pub(super) fn remove_join(&mut self, index: usize, dialect: BuilderDialect) {
        self.model.joins.remove(index);
        self.refresh_preview(dialect);
    }

    pub(super) fn remove_column(&mut self, index: usize, dialect: BuilderDialect) {
        self.model.columns.remove(index);
        self.refresh_preview(dialect);
    }

    pub(super) fn remove_predicate(&mut self, index: usize, dialect: BuilderDialect) {
        self.model.where_clauses.remove(index);
        self.refresh_preview(dialect);
    }

    pub(super) fn add_selected_table(&mut self, dialect: BuilderDialect) {
        if self.add_table.is_empty() {
            return;
        }
        let raw = self.add_table.clone();
        let (schema, name) = split_schema_table(raw.strip_prefix("view:").unwrap_or(&raw));
        self.model.add_table(&schema, &name);
        self.add_table.clear();
        self.refresh_preview(dialect);
    }

    pub(super) fn add_join(&mut self, dialect: BuilderDialect) {
        let (schema, name) = split_schema_table(&self.join_table);
        let Some((left_alias, left_column)) = split_alias_col(&self.join_left) else {
            self.error = Some("join left must be alias.column".into());
            return;
        };
        let Some((right_alias, right_column)) = split_alias_col(&self.join_right) else {
            self.error = Some("join right must be alias.column".into());
            return;
        };
        let alias = self.model.next_alias(&name);
        self.model.joins.push(BuilderJoin {
            table: BuilderTable {
                schema,
                name,
                alias: alias.clone(),
            },
            join_type: "INNER".into(),
            left_alias,
            left_column,
            right_alias: if right_alias.is_empty() { alias } else { right_alias },
            right_column,
        });
        self.refresh_preview(dialect);
    }

    pub(super) fn suggest_fk_join(&mut self, table_details: &[UiTableSummary], dialect: BuilderDialect) {
        let Some(primary) = self.model.tables.first().cloned() else {
            self.error = Some("add a primary table first".into());
            return;
        };
        let Some(detail) = table_details
            .iter()
            .find(|table| table.name == primary.name && (primary.schema.is_empty() || table.schema == primary.schema))
        else {
            self.error = Some("primary table metadata not loaded".into());
            return;
        };
        let Some(fk) = detail.foreign_keys.first() else {
            self.error = Some("no FK discovered on primary table".into());
            return;
        };
        let alias = self.model.next_alias(&fk.to_table);
        self.model.joins.push(BuilderJoin {
            table: BuilderTable {
                schema: fk.to_schema.clone(),
                name: fk.to_table.clone(),
                alias: alias.clone(),
            },
            join_type: "INNER".into(),
            left_alias: primary.alias,
            left_column: fk.from_columns.first().cloned().unwrap_or_default(),
            right_alias: alias,
            right_column: fk.to_columns.first().cloned().unwrap_or_default(),
        });
        self.error = None;
        self.refresh_preview(dialect);
    }

    pub(super) fn add_column(&mut self, dialect: BuilderDialect) {
        let Some((table_alias, column)) = split_alias_col(&self.col_ref) else {
            self.error = Some("column must be alias.column".into());
            return;
        };
        let alias = optional_trimmed(&self.col_alias);
        let aggregate = optional_trimmed(&self.col_agg).map(|value| value.to_ascii_uppercase());
        self.model.columns.push(BuilderColumn {
            table_alias,
            column,
            alias,
            aggregate,
        });
        self.model.select_star = false;
        self.refresh_preview(dialect);
    }

    pub(super) fn add_where(&mut self, dialect: BuilderDialect) {
        let Some((left_alias, left_column)) = split_alias_col(&self.where_left) else {
            self.error = Some("WHERE left must be alias.column".into());
            return;
        };
        let op = if self.where_op.trim().is_empty() {
            "=".to_owned()
        } else {
            self.where_op.trim().to_owned()
        };
        let is_null_check = op.eq_ignore_ascii_case("IS NULL") || op.eq_ignore_ascii_case("IS NOT NULL");
        self.model.where_clauses.push(BuilderPredicate {
            left_alias,
            left_column,
            op,
            value: self.where_value.clone(),
            is_null_check,
        });
        self.refresh_preview(dialect);
    }

    pub(super) fn add_order(&mut self, dialect: BuilderDialect) {
        let Some((alias, col)) = split_alias_col(&self.order) else {
            self.error = Some("ORDER BY must be alias.column".into());
            return;
        };
        self.model.order_by.push((alias, col, self.order_desc));
        self.refresh_preview(dialect);
    }

    pub(super) fn import_sql(&mut self, sql: &str, dialect: BuilderDialect) -> Result<(), String> {
        let model = try_import_select(sql, dialect)?;
        self.model = model;
        self.error = None;
        self.refresh_preview(dialect);
        Ok(())
    }
}

fn optional_trimmed(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then(|| value.trim().to_owned())
}

fn split_schema_table(raw: &str) -> (String, String) {
    if let Some((schema, table)) = raw.split_once('.') {
        (schema.to_owned(), table.to_owned())
    } else {
        (String::new(), raw.to_owned())
    }
}

fn split_alias_col(raw: &str) -> Option<(String, String)> {
    let (alias, col) = raw.split_once('.')?;
    if alias.is_empty() || col.is_empty() {
        return None;
    }
    Some((alias.to_owned(), col.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_has_safe_form_defaults() {
        let state = VisualQueryBuilderState::default();

        assert!(!state.open);
        assert_eq!(state.where_op, "=");
        assert_eq!(state.limit, "100");
        assert!(state.model.tables.is_empty());
    }

    #[test]
    fn add_table_and_preview_are_kept_in_one_transition() {
        let mut state = VisualQueryBuilderState {
            add_table: "public.users".into(),
            ..Default::default()
        };

        state.add_selected_table(BuilderDialect::Postgres);

        assert!(state.error.is_none());
        assert_eq!(state.model.tables[0].alias, "use");
        assert!(state.sql_preview.contains("FROM \"public\".\"users\""));
        assert!(state.add_table.is_empty());
    }
}
