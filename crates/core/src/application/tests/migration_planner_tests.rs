//! Tests for `../migration_planner.rs` (moved out of the source file).
use super::*;
use crate::domain::cross_connection::{ColumnTypeMismatch, SchemaDiff, TableColumnDiff};

fn sample_diff() -> SchemaDiff {
    SchemaDiff {
        tables_only_in_source: vec!["public.new_table".into()],
        tables_only_in_target: vec!["public.old_table".into()],
        column_diffs: vec![TableColumnDiff {
            schema: "public".into(),
            table: "orders".into(),
            columns_only_in_source: vec!["tenant_id".into()],
            columns_only_in_target: vec!["legacy_flag".into()],
            type_mismatches: vec![ColumnTypeMismatch {
                column: "amount".into(),
                source_type: "numeric".into(),
                target_type: "integer".into(),
            }],
        }],
        indexes_only_in_source: vec!["public.idx_orders_tenant".into()],
        indexes_only_in_target: vec!["public.idx_orders_legacy".into()],
        index_definition_mismatches: Vec::new(),
        foreign_keys_only_in_source: Vec::new(),
        foreign_keys_only_in_target: Vec::new(),
        views_only_in_source: Vec::new(),
        views_only_in_target: Vec::new(),
        view_body_mismatches: Vec::new(),
    }
}

#[test]
fn plans_dependency_order_create_before_drop() {
    let plan = MigrationPlanner::plan_from_schema_diff(&sample_diff(), "postgresql");
    let kinds: Vec<_> = plan.operations.iter().map(|op| op.kind).collect();
    let create_pos = kinds.iter().position(|k| *k == MigrationOpKind::CreateTable).unwrap();
    let drop_pos = kinds.iter().position(|k| *k == MigrationOpKind::DropTable).unwrap();
    assert!(create_pos < drop_pos);
    assert!(plan.has_destructive);
    assert!(!plan.fingerprint.is_empty());
    assert!(MigrationPlanner::verify_fingerprint(&plan, &plan.fingerprint));
}

#[test]
fn sqlite_marks_type_alter_unsupported() {
    let plan = MigrationPlanner::plan_from_schema_diff(&sample_diff(), "sqlite");
    let alter = plan
        .operations
        .iter()
        .find(|op| op.kind == MigrationOpKind::AlterColumnType)
        .expect("alter");
    assert!(!alter.provider_supported);
}

#[test]
fn non_destructive_sql_excludes_drops() {
    let plan = MigrationPlanner::plan_from_schema_diff(&sample_diff(), "postgresql");
    let sql = MigrationPlanner::non_destructive_sql(&plan);
    assert!(!sql.to_ascii_uppercase().contains("DROP TABLE"));
    assert!(!sql.to_ascii_uppercase().contains("DROP COLUMN"));
}

#[test]
fn migration_sql_uses_introspected_columns() {
    use crate::domain::schema::{Column, Index, IntrospectResult, Table};

    let mut source = IntrospectResult::empty();
    source.tables.push(Table {
        name: "widgets".into(),
        schema: "public".into(),
        row_count: None,
    });
    source.tables.push(Table {
        name: "orders".into(),
        schema: "public".into(),
        row_count: None,
    });
    source.columns.push(Column {
        name: "id".into(),
        data_type: "integer".into(),
        nullable: false,
        is_primary_key: true,
        table_name: "widgets".into(),
        schema: "public".into(),
        ..Default::default()
    });
    source.columns.push(Column {
        name: "label".into(),
        data_type: "text".into(),
        nullable: true,
        default: Some("'new'".into()),
        table_name: "widgets".into(),
        schema: "public".into(),
        ..Default::default()
    });
    source.columns.push(Column {
        name: "id".into(),
        data_type: "integer".into(),
        nullable: false,
        is_primary_key: true,
        table_name: "orders".into(),
        schema: "public".into(),
        ..Default::default()
    });
    source.columns.push(Column {
        name: "note".into(),
        data_type: "text".into(),
        nullable: false,
        table_name: "orders".into(),
        schema: "public".into(),
        ..Default::default()
    });
    source.indexes.push(Index {
        name: "widgets_label_idx".into(),
        columns: vec!["label".into()],
        table_name: "widgets".into(),
        schema: "public".into(),
        ..Default::default()
    });

    let mut target = IntrospectResult::empty();
    target.tables.push(Table {
        name: "orders".into(),
        schema: "public".into(),
        row_count: None,
    });
    target.columns.push(Column {
        name: "id".into(),
        data_type: "integer".into(),
        nullable: false,
        is_primary_key: true,
        table_name: "orders".into(),
        schema: "public".into(),
        ..Default::default()
    });

    let plan = MigrationPlanner::plan_from_introspection(&source, &target, "postgresql");
    let sql = MigrationPlanner::non_destructive_sql(&plan);
    assert!(sql.contains("CREATE TABLE \"public\".\"widgets\""));
    assert!(sql.contains("\"id\" integer NOT NULL"));
    assert!(sql.contains("\"label\" text DEFAULT 'new'"));
    assert!(sql.contains("PRIMARY KEY (\"id\")"));
    assert!(sql.contains("ALTER TABLE \"public\".\"orders\" ADD COLUMN \"note\" text NOT NULL"));
    assert!(sql.contains("CREATE INDEX \"public\".\"widgets_label_idx\" ON \"public\".\"widgets\" (\"label\")"));
    assert!(!crate::application::placeholder_remains(&sql));
}

#[test]
fn migration_quotes_identifiers_for_the_driver() {
    use crate::domain::schema::{Column, IntrospectResult, Table};

    let mut source = IntrospectResult::empty();
    source.tables.push(Table {
        name: "widgets".into(),
        schema: "app".into(),
        row_count: None,
    });
    source.columns.push(Column {
        name: "id".into(),
        data_type: "int".into(),
        nullable: false,
        is_primary_key: true,
        table_name: "widgets".into(),
        schema: "app".into(),
        ..Default::default()
    });
    let mysql = MigrationPlanner::non_destructive_sql(&MigrationPlanner::plan_from_introspection(
        &source,
        &IntrospectResult::empty(),
        "mysql",
    ));
    assert!(mysql.contains("CREATE TABLE `app`.`widgets`"), "{mysql}");
    assert!(mysql.contains("`id` int"), "{mysql}");
    let sqlserver = MigrationPlanner::non_destructive_sql(&MigrationPlanner::plan_from_introspection(
        &source,
        &IntrospectResult::empty(),
        "sqlserver",
    ));
    assert!(sqlserver.contains("CREATE TABLE [app].[widgets]"), "{sqlserver}");
    assert!(sqlserver.contains("[id] int"), "{sqlserver}");
}

#[test]
fn missing_introspection_is_not_applicable_sql() {
    let plan = MigrationPlanner::plan_from_schema_diff(&sample_diff(), "postgresql");
    let create = plan
        .operations
        .iter()
        .find(|op| op.kind == MigrationOpKind::CreateTable)
        .expect("create");
    assert!(!create.provider_supported);
    let sql = MigrationPlanner::non_destructive_sql(&plan);
    assert!(!crate::application::placeholder_remains(&sql));
    assert!(!sql.contains("CREATE TABLE"));
}

fn source_with_column(data_type: &str, default: Option<&str>) -> IntrospectResult {
    use crate::domain::schema::{Column, IntrospectResult, Table};

    let mut source = IntrospectResult::empty();
    source.tables.push(Table {
        name: "widgets".into(),
        schema: "public".into(),
        row_count: None,
    });
    source.columns.push(Column {
        name: "payload".into(),
        data_type: data_type.into(),
        nullable: true,
        default: default.map(str::to_owned),
        table_name: "widgets".into(),
        schema: "public".into(),
        ..Default::default()
    });
    source
}

#[test]
fn sqlite_declared_type_cannot_smuggle_a_second_statement() {
    // SQLite stores the declared type verbatim, so a crafted table can put
    // arbitrary text into `PRAGMA table_info.data_type`.
    let source = source_with_column("INTEGER); DROP TABLE users; --", None);
    let plan = MigrationPlanner::plan_from_introspection(&source, &IntrospectResult::empty(), "sqlite");
    let create = plan
        .operations
        .iter()
        .find(|op| op.kind == MigrationOpKind::CreateTable)
        .expect("create");
    assert!(!create.provider_supported);
    let reason = create.unsupported_reason.as_deref().unwrap_or_default();
    assert!(reason.contains("cannot be emitted safely"), "{reason}");
    let sql = MigrationPlanner::preview_sql(&plan, true);
    assert!(!sql.to_ascii_uppercase().contains("DROP TABLE USERS"), "{sql}");
}

#[test]
fn sqlite_default_cannot_smuggle_a_second_statement() {
    let source = source_with_column("integer", Some("0; DROP TABLE users; --"));
    let plan = MigrationPlanner::plan_from_introspection(&source, &IntrospectResult::empty(), "sqlite");
    let create = plan
        .operations
        .iter()
        .find(|op| op.kind == MigrationOpKind::CreateTable)
        .expect("create");
    assert!(!create.provider_supported);
    let reason = create.unsupported_reason.as_deref().unwrap_or_default();
    assert!(reason.contains("default expression"), "{reason}");
    let sql = MigrationPlanner::preview_sql(&plan, true);
    assert!(!sql.to_ascii_uppercase().contains("DROP TABLE USERS"), "{sql}");
}

#[test]
fn type_parameterized_names_and_quoted_defaults_remain_supported() {
    let source = source_with_column("numeric(12, 2)", Some("'it''s fine'"));
    let plan = MigrationPlanner::plan_from_introspection(&source, &IntrospectResult::empty(), "postgresql");
    let create = plan
        .operations
        .iter()
        .find(|op| op.kind == MigrationOpKind::CreateTable)
        .expect("create");
    assert!(create.provider_supported, "{:?}", create.unsupported_reason);
    assert!(create.sql.contains("\"payload\" numeric(12, 2) DEFAULT 'it''s fine'"));
}

#[test]
fn unsafe_source_type_is_not_emitted_for_alter_column() {
    let mut diff = sample_diff();
    diff.column_diffs[0].type_mismatches[0].source_type = "numeric; DROP TABLE users; --".into();
    let plan = MigrationPlanner::plan_from_schema_diff(&diff, "postgresql");
    let alter = plan
        .operations
        .iter()
        .find(|op| op.kind == MigrationOpKind::AlterColumnType)
        .expect("alter");
    assert!(!alter.provider_supported);
    let sql = MigrationPlanner::preview_sql(&plan, true);
    assert!(!sql.to_ascii_uppercase().contains("DROP TABLE USERS"), "{sql}");
}
