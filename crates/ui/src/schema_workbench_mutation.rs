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
                let columns = parse_column_defs(&schema, &name, &self.columns_csv)?;
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
            SchemaWorkbenchMode::Comment => {
                let parent = nonempty_opt(&self.parent_table);
                let kind = if parent.is_some() {
                    ObjectKind::Column
                } else {
                    ObjectKind::Table
                };
                (
                    ObjectDefinition::Comment(CommentDefinition {
                        object: ObjectRef {
                            kind,
                            schema: Some(schema.clone()),
                            name: name.clone(),
                            parent: parent.clone(),
                        },
                        comment: nonempty_opt(&self.comment_text),
                    }),
                    ObjectKind::Comment,
                    parent,
                )
            }
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
            SchemaWorkbenchMode::Dependencies | SchemaWorkbenchMode::Docs => {
                return Err("This mode does not produce DDL".into());
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_mutation_request_requires_a_name() {
        let state = SchemaWorkbenchState::default();

        assert!(matches!(
            state.build_mutation_request(ObjectAction::Create, "PostgreSQL".to_owned()),
            Err(error) if error == "Name is required"
        ));
    }

    #[test]
    fn table_mutation_request_preserves_typed_definition_and_driver() {
        let state = SchemaWorkbenchState {
            name: "users".to_owned(),
            columns_csv: "id INTEGER, email TEXT".to_owned(),
            ..Default::default()
        };

        let request = state
            .build_mutation_request(ObjectAction::Create, "SQLite".to_owned())
            .expect("table request should be valid");

        assert_eq!(request.driver, "SQLite");
        assert_eq!(
            request.target.as_ref().map(|target| target.name.as_str()),
            Some("users")
        );
        assert!(matches!(request.definition, ObjectDefinition::Table(_)));
    }

    #[test]
    fn documentation_mode_is_not_a_mutation_request() {
        let state = SchemaWorkbenchState {
            mode: SchemaWorkbenchMode::Docs,
            ..Default::default()
        };

        assert!(matches!(
            state.build_mutation_request(ObjectAction::GenerateDdl, "PostgreSQL".to_owned()),
            Err(error) if error == "This mode does not produce DDL"
        ));
    }
}
