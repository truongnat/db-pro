use db_pro_core::domain::cross_connection::{ObjectDependency, PartitionChild, PartitionInfo, TablespaceInfo};
use db_pro_core::domain::error::DbError;
use sqlx::PgPool;

pub async fn get_object_dependencies(pool: &PgPool, schema: &str, object_name: &str) -> Result<Vec<ObjectDependency>, DbError> {
    let sql = r#"
        SELECT
            source_class::regclass::text AS object_type,
            source_name,
            target_class::regclass::text AS depends_on_type,
            target_name
        FROM (
            SELECT
                d.classid::regclass AS source_class,
                CASE
                    WHEN d.classid = 'pg_class'::regclass THEN d.objid::regclass::text
                    WHEN d.classid = 'pg_proc'::regclass THEN d.objid::regproc::text
                    ELSE d.objid::text
                END AS source_name,
                d.refclassid::regclass AS target_class,
                CASE
                    WHEN d.refclassid = 'pg_class'::regclass THEN d.refobjid::regclass::text
                    WHEN d.refclassid = 'pg_proc'::regclass THEN d.refobjid::regproc::text
                    ELSE d.refobjid::text
                END AS target_name
            FROM pg_depend d
            JOIN pg_namespace n ON n.oid = (
                CASE
                    WHEN d.classid = 'pg_class'::regclass THEN (d.objid::regclass).relnamespace
                    ELSE 0
                END
            )
            WHERE d.deptype IN ('n', 'a')
        ) sub
        WHERE source_name LIKE $1 || '.%'
        ORDER BY object_type, source_name
    "#;

    let pattern = format!("{schema}.{object_name}");
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(sql)
        .bind(&pattern)
        .fetch_all(pool)
        .await
        .map_err(|e| DbError::Internal(format!("get dependencies: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|(object_type, object_name, depends_on_type, depends_on_name)| ObjectDependency {
            object_type,
            object_name,
            depends_on_type,
            depends_on_name,
        })
        .collect())
}

pub async fn list_partitions(pool: &PgPool) -> Result<Vec<PartitionInfo>, DbError> {
    let sql = r#"
        SELECT
            n.nspname AS schema,
            c.relname AS table_name,
            CASE p.partstrat
                WHEN 'l' THEN 'list'
                WHEN 'r' THEN 'range'
                WHEN 'h' THEN 'hash'
                ELSE 'unknown'
            END AS strategy
        FROM pg_partitioned_table p
        JOIN pg_class c ON c.oid = p.partrelid
        JOIN pg_namespace n ON n.oid = c.relnamespace
        ORDER BY n.nspname, c.relname
    "#;

    let parents: Vec<(String, String, String)> = sqlx::query_as(sql)
        .fetch_all(pool)
        .await
        .map_err(|e| DbError::Internal(format!("list partitions: {e}")))?;

    let mut result = Vec::new();
    for (schema, table, strategy) in parents {
        let child_sql = r#"
            SELECT
                c.relname AS partition_name,
                pg_get_expr(c.relpartbound, c.oid) AS bound_expr
            FROM pg_inherits i
            JOIN pg_class c ON c.oid = i.inhrelid
            JOIN pg_class parent ON parent.oid = i.inhparent
            JOIN pg_namespace n ON n.oid = parent.relnamespace
            WHERE n.nspname = $1 AND parent.relname = $2
            ORDER BY c.relname
        "#;

        let children: Vec<(String, String)> = sqlx::query_as(child_sql)
            .bind(&schema)
            .bind(&table)
            .fetch_all(pool)
            .await
            .map_err(|e| DbError::Internal(format!("list partition children: {e}")))?;

        result.push(PartitionInfo {
            schema,
            table,
            partition_strategy: strategy,
            partitions: children
                .into_iter()
                .map(|(name, bound_expr)| PartitionChild { name, bound_expr })
                .collect(),
        });
    }

    Ok(result)
}

pub async fn list_tablespaces(pool: &PgPool) -> Result<Vec<TablespaceInfo>, DbError> {
    let sql = r#"
        SELECT
            spcname AS name,
            pg_catalog.pg_get_userbyid(spcowner) AS owner,
            pg_tablespace_location(oid) AS location
        FROM pg_tablespace
        ORDER BY spcname
    "#;

    let rows: Vec<(String, String, String)> = sqlx::query_as(sql)
        .fetch_all(pool)
        .await
        .map_err(|e| DbError::Internal(format!("list tablespaces: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|(name, owner, location)| TablespaceInfo {
            name,
            owner,
            location,
        })
        .collect())
}

pub async fn rename_schema_object(
    pool: &PgPool,
    object_type: &str,
    schema: &str,
    old_name: &str,
    new_name: &str,
) -> Result<(), DbError> {
    let qualified_old = format!(
        "\"{}\".\"{}\"",
        schema.replace('"', "\"\""),
        old_name.replace('"', "\"\"")
    );
    let safe_new = format!("\"{}\"", new_name.replace('"', "\"\""));

    let sql = match object_type {
        "table" => format!("ALTER TABLE {qualified_old} RENAME TO {safe_new}"),
        "view" => format!("ALTER VIEW {qualified_old} RENAME TO {safe_new}"),
        "index" => format!("ALTER INDEX {qualified_old} RENAME TO {safe_new}"),
        "sequence" => format!("ALTER SEQUENCE {qualified_old} RENAME TO {safe_new}"),
        "column" => {
            return Err(DbError::Validation(
                "use ALTER TABLE ... RENAME COLUMN for column renames".into(),
            ));
        }
        _ => {
            return Err(DbError::Validation(format!(
                "unsupported object type: {object_type}"
            )));
        }
    };

    sqlx::query(&sql)
        .execute(pool)
        .await
        .map_err(|e| DbError::Internal(format!("rename object: {e}")))?;

    Ok(())
}
