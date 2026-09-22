//! Pure mutation-request planning for the schema workbench.

use super::schema_workbench::{
    nonempty_opt, parse_column_defs, split_csv, ConstraintKindUi, SchemaWorkbenchMode, SchemaWorkbenchState,
};
use db_pro_core::domain::object_mutation::*;

impl SchemaWorkbenchState {
    pub(crate) fn build_mutation_request(
        &self,
        action: ObjectAction,
        driver: String,
    ) -> Result<ObjectMutationRequest, String> {
        let schema = self.schema.clone();
        let name = self.name.clone();
        if name.trim().is_empty() && !matches!(self.mode, SchemaWorkbenchMode::Dependencies | SchemaWorkbenchMode::Docs)
        {
            return Err("Name is required".into());
        }
        let options = MutationOptions {
            cascade: self.cascade,
            ..MutationOptions::default()
        };

        let (definition, kind, parent) = match self.mode {
            SchemaWorkbenchMode::Table => {
                let columns = if !self.table_columns.is_empty() {
                    self.table_columns
                        .iter()
                        .map(|col| {
                            let mut data_type = col.data_type.clone();
                            if col.auto_increment && driver.eq_ignore_ascii_case("postgres") {
                                if data_type.eq_ignore_ascii_case("INTEGER") || data_type.eq_ignore_ascii_case("INT") {
                                    data_type = "SERIAL".into();
                                } else if data_type.eq_ignore_ascii_case("BIGINT") {
                                    data_type = "BIGSERIAL".into();
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
                    method: None,
                    predicate: None,
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
                            on_update: None,
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
                    start: self.start.parse().ok(),
                    increment: self.increment.parse().ok(),
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
                        kind: if self.parent_table.is_empty() {
                            ObjectKind::Table
                        } else {
                            ObjectKind::Column
                        },
                        schema: Some(schema.clone()),
                        name: name.clone(),
                        parent: nonempty_opt(&self.parent_table),
                    },
                    comment: nonempty_opt(&self.comment_text),
                }),
                ObjectKind::Comment,
                nonempty_opt(&self.parent_table),
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
            SchemaWorkbenchMode::Dependencies | SchemaWorkbenchMode::Docs => (ObjectDefinition::Empty, ObjectKind::Table, None),
        };

        Ok(ObjectMutationRequest {
            action,
            target: Some(ObjectRef {
                kind,
                schema: Some(schema),
                name,
                parent,
            }),
            definition,
            options,
            driver,
        })
    }
}
