use db_pro_core::domain::error::DbError;
use db_pro_core::domain::schema::*;
use sqlx::mysql::MySqlPool;
use sqlx::Row;

pub struct MySqlIntrospect;

impl MySqlIntrospect {
    pub async fn introspect(pool: &MySqlPool) -> Result<IntrospectResult, DbError> {
        let database: String = sqlx::query_scalar("SELECT DATABASE()")
            .fetch_one(pool)
            .await
            .map_err(|e| DbError::QueryFailed(format!("MySQL introspect failed: {}", e)))?;

        let schema = Schema { name: database.clone() };

        // Tables
        let table_rows =
            sqlx::query("SELECT table_name, table_type FROM information_schema.tables WHERE table_schema = ?")
                .bind(&database)
                .fetch_all(pool)
                .await
                .map_err(|e| DbError::QueryFailed(format!("MySQL introspect tables failed: {}", e)))?;

        let mut tables = Vec::new();
        let mut table_names = Vec::new();
        for row in &table_rows {
            let name: String = row.get("table_name");
            table_names.push(name.clone());
            tables.push(Table {
                name,
                schema: database.clone(),
                row_count: None,
            });
        }

        // Columns
        let column_rows = sqlx::query(
            "SELECT table_name, column_name, data_type, is_nullable, column_default, extra, column_key, ordinal_position 
             FROM information_schema.columns 
             WHERE table_schema = ? 
             ORDER BY table_name, ordinal_position"
        )
        .bind(&database)
        .fetch_all(pool)
        .await
        .map_err(|e| DbError::QueryFailed(format!("MySQL introspect columns failed: {}", e)))?;

        let mut columns = Vec::new();
        for row in &column_rows {
            let col = Column {
                name: row.get("column_name"),
                data_type: row.get("data_type"),
                ordinal: row.get::<u32, _>("ordinal_position") as usize,
                nullable: row.get::<String, _>("is_nullable") == "YES",
                default: row.get::<Option<String>, _>("column_default"),
                is_primary_key: row.get::<String, _>("column_key") == "PRI",
                is_unique: row.get::<String, _>("column_key") == "UNI",
                is_identity: row.get::<String, _>("extra").contains("auto_increment"),
                is_generated: row.get::<String, _>("extra").contains("GENERATED"),
                collation: None,
                table_name: row.get("table_name"),
                schema: database.clone(),
            };
            columns.push(col);
        }

        // Primary keys
        let pk_rows = sqlx::query(
            "SELECT table_name, column_name, constraint_name 
             FROM information_schema.key_column_usage 
             WHERE table_schema = ? AND constraint_name = 'PRIMARY'
             ORDER BY table_name, ordinal_position",
        )
        .bind(&database)
        .fetch_all(pool)
        .await
        .map_err(|e| DbError::QueryFailed(format!("MySQL introspect PK failed: {}", e)))?;

        let mut pk_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for row in &pk_rows {
            let table: String = row.get("table_name");
            let col: String = row.get("column_name");
            pk_map.entry(table).or_default().push(col);
        }
        let primary_keys = pk_map
            .into_iter()
            .map(|(table_name, columns)| PrimaryKey {
                constraint_name: format!("{table_name}_pk"),
                columns,
                table_name,
                schema: database.clone(),
            })
            .collect();

        // Indexes
        let index_rows = sqlx::query(
            "SELECT table_name, index_name, column_name, non_unique, index_type 
             FROM information_schema.statistics 
             WHERE table_schema = ? 
             ORDER BY table_name, index_name, seq_in_index",
        )
        .bind(&database)
        .fetch_all(pool)
        .await
        .map_err(|e| DbError::QueryFailed(format!("MySQL introspect indexes failed: {}", e)))?;

        let mut index_groups: std::collections::HashMap<String, (Vec<String>, bool, String)> =
            std::collections::HashMap::new();
        for row in &index_rows {
            let idx_name: String = row.get("index_name");
            let col: String = row.get("column_name");
            let non_unique: i32 = row.get("non_unique");
            let idx_type: String = row.get("index_type");
            let entry = index_groups
                .entry(idx_name)
                .or_insert_with(|| (Vec::new(), non_unique == 0, idx_type));
            entry.0.push(col);
        }
        let indexes = index_groups
            .into_iter()
            .map(|(name, (columns, unique, method))| {
                let is_primary = name == "PRIMARY";
                Index {
                    name,
                    columns,
                    unique: unique && !is_primary,
                    method,
                    primary: is_primary,
                    include_columns: Vec::new(),
                    predicate: None,
                    definition: String::new(),
                    origin: IndexOrigin::User,
                    table_name: String::new(),
                    schema: database.clone(),
                }
            })
            .collect();

        // Foreign keys
        let fk_rows = sqlx::query(
            "SELECT 
                tc.table_name, 
                kcu.column_name,
                kcu.referenced_table_name,
                kcu.referenced_column_name,
                rc.constraint_name,
                rc.update_rule,
                rc.delete_rule
             FROM information_schema.table_constraints tc
             JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name
             JOIN information_schema.referential_constraints rc ON tc.constraint_name = rc.constraint_name
             WHERE tc.table_schema = ? AND tc.constraint_type = 'FOREIGN KEY'
             ORDER BY tc.constraint_name, kcu.ordinal_position",
        )
        .bind(&database)
        .fetch_all(pool)
        .await
        .map_err(|e| DbError::QueryFailed(format!("MySQL introspect FK failed: {}", e)))?;

        #[allow(clippy::type_complexity)]
        let mut fk_groups: std::collections::HashMap<
            String,
            (Vec<String>, Vec<String>, String, String, String, String),
        > = std::collections::HashMap::new();
        for row in &fk_rows {
            let constraint: String = row.get("constraint_name");
            let entry = fk_groups.entry(constraint).or_insert_with(|| {
                (
                    Vec::new(),
                    Vec::new(),
                    row.get("table_name"),
                    row.get("referenced_table_name"),
                    row.get("update_rule"),
                    row.get("delete_rule"),
                )
            });
            entry.0.push(row.get("column_name"));
            entry.1.push(row.get("referenced_column_name"));
        }
        let foreign_keys = fk_groups
            .into_iter()
            .map(
                |(name, (from_cols, to_cols, from_table, to_table, on_update, on_delete))| ForeignKey {
                    name,
                    from_table,
                    from_columns: from_cols,
                    to_table,
                    to_columns: to_cols,
                    schema: database.clone(),
                    to_schema: database.clone(),
                    on_update,
                    on_delete,
                    match_option: String::new(),
                    deferrable: false,
                    initially_deferred: false,
                },
            )
            .collect();

        // Views
        let view_rows =
            sqlx::query("SELECT table_name, view_definition FROM information_schema.views WHERE table_schema = ?")
                .bind(&database)
                .fetch_all(pool)
                .await
                .map_err(|e| DbError::QueryFailed(format!("MySQL introspect views failed: {}", e)))?;

        let views = view_rows
            .iter()
            .map(|row| View {
                name: row.get("table_name"),
                schema: database.clone(),
                definition: row.get::<Option<String>, _>("view_definition").unwrap_or_default(),
            })
            .collect();

        // Triggers
        let trigger_rows = sqlx::query(
            "SELECT trigger_name, event_manipulation, event_object_table, action_statement, action_timing 
             FROM information_schema.triggers 
             WHERE trigger_schema = ?",
        )
        .bind(&database)
        .fetch_all(pool)
        .await
        .map_err(|e| DbError::QueryFailed(format!("MySQL introspect triggers failed: {}", e)))?;

        let triggers = trigger_rows
            .iter()
            .map(|row| Trigger {
                name: row.get("trigger_name"),
                table_name: row.get("event_object_table"),
                schema: database.clone(),
                timing: row.get("action_timing"),
                event: row.get("event_manipulation"),
                definition: row.get::<Option<String>, _>("action_statement").unwrap_or_default(),
                function_def: String::new(),
                enabled: true,
            })
            .collect();

        // Functions
        let routine_rows = sqlx::query(
            "SELECT routine_name, routine_type, data_type, routine_definition 
             FROM information_schema.routines 
             WHERE routine_schema = ?",
        )
        .bind(&database)
        .fetch_all(pool)
        .await
        .map_err(|e| DbError::QueryFailed(format!("MySQL introspect routines failed: {}", e)))?;

        let functions = routine_rows
            .iter()
            .map(|row| Function {
                name: row.get("routine_name"),
                schema: database.clone(),
                routine_type: row.get("routine_type"),
                data_type: row.get::<Option<String>, _>("data_type").unwrap_or_default(),
                definition: row.get::<Option<String>, _>("routine_definition").unwrap_or_default(),
            })
            .collect();

        Ok(IntrospectResult {
            schemas: vec![schema],
            tables,
            columns,
            primary_keys,
            indexes,
            foreign_keys,
            check_constraints: Vec::new(),
            views,
            triggers,
            functions,
        })
    }
}
