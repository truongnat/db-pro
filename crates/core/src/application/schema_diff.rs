use crate::domain::connection::ConnectionId;
use crate::domain::cross_connection::SchemaDiff;
use crate::domain::error::DbError;

use super::SchemaService;
use schema_diff_compare::compare_introspect_results;

#[path = "schema_diff_compare.rs"]
mod schema_diff_compare;

#[cfg(test)]
use crate::domain::schema::IntrospectResult;
#[cfg(test)]
use schema_diff_compare::{display_qualified_name, qualify_key};

impl SchemaService {
    pub async fn diff_schemas(
        &self,
        source_id: &ConnectionId,
        target_id: &ConnectionId,
    ) -> Result<SchemaDiff, DbError> {
        let source = self.introspect(source_id, false).await?;
        let target = self.introspect(target_id, false).await?;
        Ok(compare_introspect_results(&source, &target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::schema::{Column, Index, IndexOrigin, Table};

    #[test]
    fn qualified_name_preserves_dotted_schema_and_table_names() {
        let qualified = qualify_key("tenant.prod", "orders.archive");
        assert_eq!(qualified, ("tenant.prod".into(), "orders.archive".into()));
        assert_eq!(display_qualified_name(&qualified), "\"tenant.prod\".\"orders.archive\"");
    }

    #[test]
    fn empty_schema_dotted_table_is_compared_by_its_literal_name() {
        let mut source = IntrospectResult::empty();
        source.tables.push(Table {
            name: "my.table".into(),
            schema: "".into(),
            row_count: None,
        });
        source.columns.push(Column {
            name: "id".into(),
            data_type: "INTEGER".into(),
            nullable: false,
            default: None,
            is_primary_key: true,
            table_name: "my.table".into(),
            schema: "".into(),
            ..Default::default()
        });

        let mut target = source.clone();
        target.columns[0].data_type = "TEXT".into();

        let diff = compare_introspect_results(&source, &target);
        assert_eq!(diff.column_diffs.len(), 1);
        assert_eq!(diff.column_diffs[0].schema, "");
        assert_eq!(diff.column_diffs[0].table, "my.table");
        assert_eq!(diff.column_diffs[0].type_mismatches[0].column, "id");
    }

    #[test]
    fn dotted_schema_names_do_not_collide_with_dotted_table_names() {
        let mut source = IntrospectResult::empty();
        source.tables.push(Table {
            name: "orders".into(),
            schema: "tenant.prod".into(),
            row_count: None,
        });

        let mut target = IntrospectResult::empty();
        target.tables.push(Table {
            name: "prod.orders".into(),
            schema: "tenant".into(),
            row_count: None,
        });

        let diff = compare_introspect_results(&source, &target);
        assert_eq!(diff.tables_only_in_source, vec!["\"tenant.prod\".orders"]);
        assert_eq!(diff.tables_only_in_target, vec!["tenant.\"prod.orders\""]);
    }

    #[test]
    fn schema_diff_orders_set_based_results_deterministically() {
        let mut source = IntrospectResult::empty();
        source.tables = vec![
            Table {
                name: "users".into(),
                schema: "public".into(),
                row_count: None,
            },
            Table {
                name: "accounts".into(),
                schema: "public".into(),
                row_count: None,
            },
        ];
        source.indexes = vec![
            Index {
                name: "users_z_idx".into(),
                columns: vec![],
                unique: false,
                origin: IndexOrigin::User,
                table_name: "users".into(),
                schema: "public".into(),
                ..Default::default()
            },
            Index {
                name: "users_a_idx".into(),
                columns: vec![],
                unique: false,
                origin: IndexOrigin::User,
                table_name: "users".into(),
                schema: "public".into(),
                ..Default::default()
            },
        ];

        let mut target = IntrospectResult::empty();
        target.tables.push(Table {
            name: "orders".into(),
            schema: "public".into(),
            row_count: None,
        });

        let diff = compare_introspect_results(&source, &target);
        assert_eq!(diff.tables_only_in_source, vec!["public.accounts", "public.users"]);
        assert_eq!(diff.tables_only_in_target, vec!["public.orders"]);
        assert_eq!(
            diff.indexes_only_in_source,
            vec!["public.users_a_idx", "public.users_z_idx"]
        );
    }
}
