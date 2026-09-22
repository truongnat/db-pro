//! Schema Workbench — mutation payload construction.

use super::schema_workbench::{
    nonempty_opt, parse_column_defs, split_csv, ConstraintKindUi, SchemaWorkbenchMode, SchemaWorkbenchState,
};
use db_pro_core::domain::object_mutation::*;

impl SchemaWorkbenchState {
    pub(super) fn build_mutation_request(
        &self,
        action: ObjectAction,
        driver: &str,
    ) -> Result<ObjectMutationRequest, String> {
        let schema = self.schema.trim().to_owned();
        let name = self.name.trim().to_owned();
        if name.is_empty()
            && !matches!(
                self.mode,
                SchemaWorkbenchMode::SchemaDb
                    | SchemaWorkbenchMode::Comment
                    | SchemaWorkbenchMode::Dependencies
                    | SchemaWorkbenchMode::Docs
                    | SchemaWorkbenchMode::History
            )
        {
            return Err("Object name cannot be empty".to_owned());
        }

        let (definition, kind, parent) = match self.mode {
            SchemaWorkbenchMode::Table => {
                let columns = if !self.table_columns.is_empty() {
                    self.table_columns
                        .iter()
                        .map(|col| {
                            let mut data_type = col.data_type.clone();
                            if col.auto_increment {
                                if driver.to_ascii_lowercase().contains("postgres") {
                                    data_type = if data_type.to_ascii_uppercase().contains("BIG") {
                                        "BIGSERIAL".into()
                                    } else {
                                        "SERIAL".into()
                                    };
                                } else if driver.to_ascii_lowercase().contains("sqlite") {
                                    data_type = "INTEGER PRIMARY KEY AUTOINCREMENT".into();
                                } else if driver.to_ascii_lowercase().contains("mysql") {
                                    data_type = format!("{data_type} AUTO_INCREMENT");
                                }
                            }
                            ColumnDefinition {
                                schema: schema.clone(),
                                table: name.clone(),
                                name: col.name.clone(),
                                data_type,
                                nullable: col.nullable && !col.is_pk,
                                default: nonempty_opt(&col.default_expr),
                                is_pk: col.is_pk,
                                new_name: None,
                            }
                        })
                        .collect()
                } else {
                    parse_column_defs(&schema, &name, &self.columns_csv)?
                };
                (
                    ObjectDefinition::Table(TableDefinition {
                        schema: schema.clone(),
                        name: name.clone(),
                        columns,
                    }),
                    ObjectKind::Table,
                    None,
                )
            }
            SchemaWorkbenchMode::Column => (
                ObjectDefinition::Column(ColumnDefinition {
                    schema: schema.clone(),
                    table: self.parent_table.clone(),
                    name: name.clone(),
                    data_type: self.data_type.clone(),
                    nullable: self.nullable,
                    default: nonempty_opt(&self.default_expr),
                    is_pk: self.is_pk,
                    new_name: nonempty_opt(&self.new_name),
                }),
                ObjectKind::Column,
                Some(self.parent_table.clone()),
            ),
            SchemaWorkbenchMode::View => {
                let definition = ViewDefinition {
                    schema: schema.clone(),
                    name: name.clone(),
                    select_sql: self.select_sql.clone(),
                    materialized: self.materialized,
                    replace: false,
                };
                let kind = if self.materialized {
                    ObjectKind::MaterializedView
                } else {
                    ObjectKind::View
                };
                let definition = if self.materialized {
                    ObjectDefinition::MaterializedView(definition)
                } else {
                    ObjectDefinition::View(definition)
                };
                (definition, kind, None)
            }
            SchemaWorkbenchMode::Index => (
                ObjectDefinition::Index(IndexDefinition {
                    schema: schema.clone(),
                    table: self.parent_table.clone(),
                    name: name.clone(),
                    columns: split_csv(&self.columns_csv),
                    unique: self.unique,
                    method: nonempty_opt(&self.index_method),
                    predicate: nonempty_opt(&self.index_predicate),
                }),
                ObjectKind::Index,
                Some(self.parent_table.clone()),
            ),
            SchemaWorkbenchMode::Constraint => {
                let columns = split_csv(&self.columns_csv);
                match self.constraint_kind {
                    ConstraintKindUi::PrimaryKey => (
                        ObjectDefinition::PrimaryKey(NamedColumnsDefinition {
                            schema: schema.clone(),
                            table: self.parent_table.clone(),
                            name: name.clone(),
                            columns,
                        }),
                        ObjectKind::PrimaryKey,
                        Some(self.parent_table.clone()),
                    ),
                    ConstraintKindUi::Unique => (
                        ObjectDefinition::UniqueConstraint(NamedColumnsDefinition {
                            schema: schema.clone(),
                            table: self.parent_table.clone(),
                            name: name.clone(),
                            columns,
                        }),
                        ObjectKind::UniqueConstraint,
                        Some(self.parent_table.clone()),
                    ),
                    ConstraintKindUi::Check => (
                        ObjectDefinition::CheckConstraint(CheckDefinition {
                            schema: schema.clone(),
                            table: self.parent_table.clone(),
                            name: name.clone(),
                            expression: self.expression.clone(),
                        }),
                        ObjectKind::CheckConstraint,
                        Some(self.parent_table.clone()),
                    ),
                    ConstraintKindUi::ForeignKey => (
                        ObjectDefinition::ForeignKey(ForeignKeyDefinition {
                            schema: schema.clone(),
                            table: self.parent_table.clone(),
                            name: name.clone(),
                            columns,
                            ref_schema: self.ref_schema.clone(),
                            ref_table: self.ref_table.clone(),
                            ref_columns: split_csv(&self.ref_columns_csv),
                            on_delete: nonempty_opt(&self.on_delete),
                            on_update: nonempty_opt(&self.on_update),
                        }),
                        ObjectKind::ForeignKey,
                        Some(self.parent_table.clone()),
                    ),
                }
            }
            SchemaWorkbenchMode::Trigger => (
                ObjectDefinition::Trigger(TriggerDefinition {
                    schema: schema.clone(),
                    table: self.parent_table.clone(),
                    name: name.clone(),
                    timing: self.timing.clone(),
                    event: self.event.clone(),
                    body: self.body.clone(),
                }),
                ObjectKind::Trigger,
                Some(self.parent_table.clone()),
            ),
            SchemaWorkbenchMode::Sequence => (
                ObjectDefinition::Sequence(SequenceDefinition {
                    schema: schema.clone(),
                    name: name.clone(),
                    start: self.start.parse::<i64>().ok(),
                    increment: self.increment.parse::<i64>().ok(),
                    min_value: None,
                    max_value: None,
                    cache: None,
                    cycle: self.cycle,
                }),
                ObjectKind::Sequence,
                None,
            ),
            SchemaWorkbenchMode::Type => (
                ObjectDefinition::EnumType(EnumTypeDefinition {
                    schema: schema.clone(),
                    name: name.clone(),
                    values: split_csv(&self.enum_values_csv),
                }),
                ObjectKind::EnumType,
                None,
            ),
            SchemaWorkbenchMode::SchemaDb => (
                ObjectDefinition::Schema(SchemaDefinition {
                    name: name.clone(),
                    new_name: None,
                    owner: None,
                }),
                ObjectKind::Schema,
                None,
            ),
            SchemaWorkbenchMode::Extension => (
                ObjectDefinition::Extension(ExtensionDefinition {
                    name: name.clone(),
                    schema: nonempty_opt(&self.extension_schema),
                    version: None,
                    cascade: self.cascade,
                }),
                ObjectKind::Extension,
                None,
            ),
            SchemaWorkbenchMode::Comment => (
                ObjectDefinition::Comment(CommentDefinition {
                    object: ObjectRef {
                        kind: ObjectKind::Table,
                        schema: nonempty_opt(&schema),
                        name: if self.parent_table.is_empty() {
                            name.clone()
                        } else {
                            self.parent_table.clone()
                        },
                        parent: None,
                    },
                    comment: nonempty_opt(&self.comment_text),
                }),
                ObjectKind::Comment,
                None,
            ),
            SchemaWorkbenchMode::Partition => (
                ObjectDefinition::Partition(PartitionDefinition {
                    schema: schema.clone(),
                    parent_table: self.parent_table.clone(),
                    name: name.clone(),
                    strategy: "RANGE".into(),
                    bound_expression: self.partition_bound.clone(),
                }),
                ObjectKind::Partition,
                Some(self.parent_table.clone()),
            ),
            SchemaWorkbenchMode::Dependencies | SchemaWorkbenchMode::Docs | SchemaWorkbenchMode::History => {
                (ObjectDefinition::Empty, ObjectKind::Table, None)
            }
        };

        let target = if name.is_empty() {
            None
        } else {
            Some(ObjectRef {
                kind,
                schema: nonempty_opt(&schema),
                name,
                parent,
            })
        };

        Ok(ObjectMutationRequest {
            action,
            target,
            definition,
            options: MutationOptions {
                cascade: self.cascade,
                if_exists: true,
                if_not_exists: true,
                dry_run: false,
            },
            driver: driver.to_owned(),
        })
    }
}
