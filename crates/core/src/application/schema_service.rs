use std::sync::Arc;

use crate::domain::connection::{ConnectionConfig, ConnectionId, DriverType};
use crate::domain::error::DbError;
use crate::domain::safety::{validate_against_policy, ConnectionSafetyPolicy};
use crate::domain::schema::{CheckConstraint, ForeignKey, IndexOrigin, IntrospectResult, TableInfo, Trigger};
use crate::ports::{ConnectionRepository, DbConnector, IntrospectionCache};

use super::registry::ConnectionRegistry;
use super::sql_policy::reject_multi_statement;

pub struct SchemaService {
    connector: Box<dyn DbConnector>,
    cache: Box<dyn IntrospectionCache>,
    registry: Arc<ConnectionRegistry>,
    connections: Box<dyn ConnectionRepository>,
}

impl SchemaService {
    pub fn new(
        connector: Box<dyn DbConnector>,
        cache: Box<dyn IntrospectionCache>,
        registry: Arc<ConnectionRegistry>,
        connections: Box<dyn ConnectionRepository>,
    ) -> Self {
        Self {
            connector,
            cache,
            registry,
            connections,
        }
    }

    async fn safety_policy_for(&self, connection_id: &ConnectionId) -> Result<ConnectionSafetyPolicy, DbError> {
        let config = self.connection_config(connection_id).await?;
        if config.readonly {
            Ok(ConnectionSafetyPolicy::read_only())
        } else {
            Ok(ConnectionSafetyPolicy::full_access())
        }
    }

    async fn connection_config(&self, connection_id: &ConnectionId) -> Result<ConnectionConfig, DbError> {
        self.connections
            .get_config(connection_id)
            .await?
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} not found")))
    }

    pub async fn introspect(
        &self,
        connection_id: &ConnectionId,
        force_refresh: bool,
    ) -> Result<IntrospectResult, DbError> {
        if !force_refresh {
            match self.cache.get(connection_id).await {
                Ok(Some(cached)) => {
                    if self.cached_schema_still_usable(connection_id, &cached).await {
                        return Ok(cached);
                    }
                    tracing::warn!(
                        connection_id = %connection_id,
                        "discarding stale introspection cache that no longer matches the database"
                    );
                    if let Err(invalidate_error) = self.cache.invalidate(connection_id).await {
                        tracing::warn!(
                            connection_id = %connection_id,
                            error = %invalidate_error,
                            "failed to discard stale introspection cache"
                        );
                    }
                }
                Ok(None) => {}
                Err(error) => {
                    tracing::warn!(
                        connection_id = %connection_id,
                        error = %error,
                        "discarding invalid introspection cache"
                    );
                    if let Err(invalidate_error) = self.cache.invalidate(connection_id).await {
                        tracing::warn!(
                            connection_id = %connection_id,
                            error = %invalidate_error,
                            "failed to discard invalid introspection cache"
                        );
                    }
                }
            }
        }

        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        let result = self.connector.introspect(&handle).await?;

        if let Err(e) = self.cache.save(connection_id, &result).await {
            tracing::warn!("failed to cache introspection: {e}");
        }

        Ok(result)
    }

    /// Returns false when a cached SQLite schema would mislead the Explorer.
    ///
    /// Truncated or deleted `.sqlite` files previously kept serving a warm cache
    /// (`force_refresh=false` on connect), so the tree showed tables the query
    /// engine could no longer resolve.
    async fn cached_schema_still_usable(&self, connection_id: &ConnectionId, cached: &IntrospectResult) -> bool {
        let Ok(config) = self.connection_config(connection_id).await else {
            // Config lookup failed — keep the cache rather than blocking the UI.
            return true;
        };
        if config.driver != DriverType::SQLite {
            return true;
        }
        sqlite_file_matches_cached_schema(&config.database, cached)
    }

    pub async fn get_table_info(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
    ) -> Result<TableInfo, DbError> {
        let introspect = self.introspect(connection_id, false).await?;

        let tbl = introspect
            .tables
            .iter()
            .find(|t| t.schema == schema && t.name == table)
            .ok_or_else(|| DbError::NotFound(format!("table {schema}.{table}")))?
            .clone();

        let columns: Vec<_> = introspect
            .columns
            .into_iter()
            .filter(|c| c.schema == schema && c.table_name == table)
            .collect();

        let primary_key = introspect
            .primary_keys
            .into_iter()
            .find(|pk| pk.schema == schema && pk.table_name == table);

        let indexes: Vec<_> = introspect
            .indexes
            .into_iter()
            .filter(|i| i.schema == schema && i.table_name == table)
            .collect();

        let foreign_keys: Vec<_> = introspect
            .foreign_keys
            .iter()
            .filter(|fk| fk.from_table == table && fk.schema == schema)
            .cloned()
            .collect();
        let check_constraints: Vec<_> = introspect
            .check_constraints
            .iter()
            .filter(|c| c.schema == schema && c.table_name == table)
            .cloned()
            .collect();

        let mut dependencies = Vec::new();

        // 1. Outgoing Foreign Keys (This table depends on other tables)
        for fk in &foreign_keys {
            dependencies.push(crate::domain::schema::TableDependency {
                name: fk.to_table.clone(),
                schema: fk.to_schema.clone(),
                kind: crate::domain::schema::DependencyKind::Table,
                direction: crate::domain::schema::DependencyDirection::DependsOn,
                details: format!(
                    "Foreign key {} ({}) → {}.{}({})",
                    fk.name,
                    fk.from_columns.join(", "),
                    fk.to_schema,
                    fk.to_table,
                    fk.to_columns.join(", ")
                ),
            });
        }

        // 2. Incoming Foreign Keys (Other tables depend on this table)
        for fk in &introspect.foreign_keys {
            if fk.to_schema == schema && fk.to_table == table && !(fk.schema == schema && fk.from_table == table) {
                dependencies.push(crate::domain::schema::TableDependency {
                    name: fk.from_table.clone(),
                    schema: fk.schema.clone(),
                    kind: crate::domain::schema::DependencyKind::Table,
                    direction: crate::domain::schema::DependencyDirection::DependedBy,
                    details: format!(
                        "Referenced by foreign key {} ({}.{}) → ({})",
                        fk.name,
                        fk.schema,
                        fk.from_table,
                        fk.to_columns.join(", ")
                    ),
                });
            }
        }

        // 3. Views referencing this table (Incoming dependency)
        for view in &introspect.views {
            let def_lower = view.definition.to_lowercase();
            let table_lower = table.to_lowercase();
            let qualified_lower = format!("{}.{}", schema.to_lowercase(), table_lower);
            if def_lower.contains(&qualified_lower) || def_lower.contains(&table_lower) {
                dependencies.push(crate::domain::schema::TableDependency {
                    name: view.name.clone(),
                    schema: view.schema.clone(),
                    kind: crate::domain::schema::DependencyKind::View,
                    direction: crate::domain::schema::DependencyDirection::DependedBy,
                    details: format!(
                        "View {}.{} references table in query definition",
                        view.schema, view.name
                    ),
                });
            }
        }

        // 4. Triggers on this table (Incoming dependency)
        for trigger in &introspect.triggers {
            if trigger.schema == schema && trigger.table_name == table {
                dependencies.push(crate::domain::schema::TableDependency {
                    name: trigger.name.clone(),
                    schema: trigger.schema.clone(),
                    kind: crate::domain::schema::DependencyKind::Trigger,
                    direction: crate::domain::schema::DependencyDirection::DependedBy,
                    details: format!(
                        "Trigger {} ({} {}) attached to table",
                        trigger.name, trigger.timing, trigger.event
                    ),
                });
            }
        }

        // 5. Functions referencing this table (Incoming dependency)
        for func in &introspect.functions {
            let def_lower = func.definition.to_lowercase();
            let table_lower = table.to_lowercase();
            if !func.definition.is_empty()
                && (def_lower.contains(&format!("{schema}.{table}").to_lowercase()) || def_lower.contains(&table_lower))
            {
                dependencies.push(crate::domain::schema::TableDependency {
                    name: func.name.clone(),
                    schema: func.schema.clone(),
                    kind: crate::domain::schema::DependencyKind::Function,
                    direction: crate::domain::schema::DependencyDirection::DependedBy,
                    details: format!(
                        "Routine {}.{} ({}) references this table",
                        func.schema, func.name, func.routine_type
                    ),
                });
            }
        }

        // 6. Sequences used by column defaults (Outgoing dependency)
        for col in &columns {
            if let Some(ref def) = col.default {
                let def_lower = def.to_lowercase();
                if def_lower.contains("nextval") || def_lower.contains("_seq") {
                    dependencies.push(crate::domain::schema::TableDependency {
                        name: col.name.clone(),
                        schema: schema.to_owned(),
                        kind: crate::domain::schema::DependencyKind::Sequence,
                        direction: crate::domain::schema::DependencyDirection::DependsOn,
                        details: format!("Column {} default sequence expression: {}", col.name, def),
                    });
                }
            }
        }

        let mut seen_deps = std::collections::HashSet::new();
        let mut deduplicated_dependencies = Vec::with_capacity(dependencies.len());
        for dep in dependencies {
            if seen_deps.insert((dep.kind, dep.direction, dep.schema.clone(), dep.name.clone())) {
                deduplicated_dependencies.push(dep);
            }
        }

        Ok(TableInfo {
            table: tbl,
            columns,
            primary_key,
            indexes,
            foreign_keys,
            check_constraints,
            dependencies: deduplicated_dependencies,
        })
    }

    pub async fn get_table_ddl(
        &self,
        connection_id: &ConnectionId,
        schema: &str,
        table: &str,
    ) -> Result<String, DbError> {
        let introspect = self.introspect(connection_id, false).await?;

        // Check if this is a view first — views use their stored definition.
        if let Some(view) = introspect.views.iter().find(|v| v.schema == schema && v.name == table) {
            return Ok(format!("{};\n", view.definition));
        }

        let tbl = introspect
            .tables
            .iter()
            .find(|t| t.schema == schema && t.name == table)
            .ok_or_else(|| DbError::NotFound(format!("table or view {schema}.{table}")))?
            .clone();

        let check_constraints: Vec<_> = introspect
            .check_constraints
            .iter()
            .filter(|constraint| constraint.schema == schema && constraint.table_name == table)
            .cloned()
            .collect();

        let columns: Vec<_> = introspect
            .columns
            .into_iter()
            .filter(|c| c.schema == schema && c.table_name == table)
            .collect();

        let primary_key = introspect
            .primary_keys
            .into_iter()
            .find(|pk| pk.schema == schema && pk.table_name == table);

        let indexes: Vec<_> = introspect
            .indexes
            .into_iter()
            .filter(|i| i.schema == schema && i.table_name == table)
            .collect();

        let foreign_keys: Vec<_> = introspect
            .foreign_keys
            .into_iter()
            .filter(|fk| fk.from_table == table && fk.schema == schema)
            .collect();

        let triggers: Vec<_> = introspect
            .triggers
            .into_iter()
            .filter(|tr| tr.table_name == table && tr.schema == schema)
            .collect();

        let info = TableInfo {
            table: tbl,
            columns,
            primary_key,
            indexes,
            foreign_keys,
            check_constraints: check_constraints.clone(),
            dependencies: Vec::new(),
        };

        let driver = self.connection_config(connection_id).await?.driver;
        let mut ddl = build_create_table_ddl(&info, driver, &check_constraints);
        for trigger in &triggers {
            ddl.push_str(&format_trigger_ddl(trigger));
            ddl.push('\n');
        }

        Ok(ddl)
    }

    pub async fn execute_ddl(&self, connection_id: &ConnectionId, sql: &str) -> Result<u64, DbError> {
        reject_multi_statement(sql)?;

        // Enforce safety policy: readonly connections cannot execute DDL.
        let policy = self.safety_policy_for(connection_id).await?;
        validate_against_policy(sql, &policy).map_err(DbError::QueryFailed)?;

        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        let affected = self.connector.execute(&handle, sql, &[]).await?;

        if let Err(e) = self.cache.invalidate(connection_id).await {
            tracing::warn!("failed to invalidate cache after DDL: {e}");
        }

        Ok(affected)
    }

    pub async fn execute_ddl_batch(&self, connection_id: &ConnectionId, statements: &[String]) -> Result<u64, DbError> {
        let policy = self.safety_policy_for(connection_id).await?;
        for sql in statements {
            reject_multi_statement(sql)?;
            validate_against_policy(sql, &policy).map_err(DbError::QueryFailed)?;
        }

        let handle = self
            .registry
            .get(connection_id)
            .ok_or_else(|| DbError::ConnectionFailed(format!("connection {connection_id} is not active")))?;

        let affected = self.connector.execute_batch(&handle, statements).await?;

        if let Err(e) = self.cache.invalidate(connection_id).await {
            tracing::warn!("failed to invalidate cache after batch DDL: {e}");
        }

        Ok(affected)
    }

    pub async fn invalidate_cache(&self, connection_id: &ConnectionId) -> Result<(), DbError> {
        self.cache.invalidate(connection_id).await
    }
}

struct ForeignKeyDdlGroup<'a> {
    name: &'a str,
    from_columns: Vec<&'a str>,
    to_table: &'a str,
    to_columns: Vec<&'a str>,
    to_schema: &'a str,
    on_update: &'a str,
    on_delete: &'a str,
    match_option: &'a str,
    deferrable: bool,
    initially_deferred: bool,
}

fn group_foreign_keys_for_ddl(foreign_keys: &[ForeignKey]) -> Vec<ForeignKeyDdlGroup<'_>> {
    foreign_keys
        .iter()
        .map(|fk| ForeignKeyDdlGroup {
            name: &fk.name,
            from_columns: fk.from_columns.iter().map(|s| s.as_str()).collect(),
            to_table: &fk.to_table,
            to_columns: fk.to_columns.iter().map(|s| s.as_str()).collect(),
            to_schema: &fk.to_schema,
            on_update: &fk.on_update,
            on_delete: &fk.on_delete,
            match_option: &fk.match_option,
            deferrable: fk.deferrable,
            initially_deferred: fk.initially_deferred,
        })
        .collect()
}

fn qualify_name(schema: &str, name: &str) -> String {
    if schema.is_empty() {
        quote_identifier(name)
    } else {
        format!("{}.{}", quote_identifier(schema), quote_identifier(name))
    }
}

fn build_create_table_ddl(info: &TableInfo, driver: DriverType, check_constraints: &[CheckConstraint]) -> String {
    let qualified = qualify_name_for_driver(driver, &info.table.schema, &info.table.name);
    let foreign_keys = group_foreign_keys_for_ddl(&info.foreign_keys);
    let definitions = table_definitions(info, driver, check_constraints, &foreign_keys);
    let mut ddl = format!("CREATE TABLE {qualified} (\n{}\n);\n", definitions.join(",\n"));

    append_index_ddl(&mut ddl, info, driver);
    if driver == DriverType::Postgres {
        append_postgres_foreign_keys(&mut ddl, &qualified, &foreign_keys);
    }

    ddl
}

fn table_definitions(
    info: &TableInfo,
    driver: DriverType,
    check_constraints: &[CheckConstraint],
    foreign_keys: &[ForeignKeyDdlGroup<'_>],
) -> Vec<String> {
    let mut definitions: Vec<_> = info.columns.iter().map(format_column_definition).collect();

    if let Some(ref pk) = info.primary_key {
        definitions.push(format!(
            "    PRIMARY KEY ({})",
            quote_columns(&pk.columns.iter().map(String::as_str).collect::<Vec<_>>())
        ));
    }
    definitions.extend(check_constraints.iter().map(format_check_constraint));

    if driver == DriverType::SQLite {
        definitions.extend(foreign_keys.iter().map(format_sqlite_foreign_key));
        definitions.extend(
            info.indexes
                .iter()
                .filter(|index| index.origin == IndexOrigin::UniqueConstraint)
                .map(format_sqlite_unique_constraint),
        );
    }

    definitions
}

fn format_column_definition(column: &crate::domain::schema::Column) -> String {
    let mut definition = format!("    {} {}", quote_identifier(&column.name), column.data_type);
    if column.is_identity {
        definition.push_str(" GENERATED BY DEFAULT AS IDENTITY");
    } else if column.is_generated {
        if let Some(ref expression) = column.default {
            definition.push_str(&format!(" GENERATED ALWAYS AS ({expression}) STORED"));
        }
    }
    if !column.nullable {
        definition.push_str(" NOT NULL");
    }
    if !column.is_identity && !column.is_generated {
        if let Some(ref default) = column.default {
            definition.push_str(&format!(" DEFAULT {default}"));
        }
    }
    definition
}

fn format_check_constraint(constraint: &CheckConstraint) -> String {
    format!(
        "    CONSTRAINT {} {}",
        quote_identifier(&constraint.name),
        check_constraint_definition(constraint),
    )
}

fn format_sqlite_foreign_key(foreign_key: &ForeignKeyDdlGroup<'_>) -> String {
    let to_qualified = qualify_name_for_driver(DriverType::SQLite, foreign_key.to_schema, foreign_key.to_table);
    let from_columns = quote_columns(&foreign_key.from_columns);
    let to_columns = quote_columns(&foreign_key.to_columns);
    let actions = format_foreign_key_actions(foreign_key);
    format!(
        "    CONSTRAINT {} FOREIGN KEY ({from_columns}) REFERENCES {to_qualified} ({to_columns}){actions}",
        quote_identifier(foreign_key.name),
    )
}

fn format_foreign_key_actions(foreign_key: &ForeignKeyDdlGroup<'_>) -> String {
    let mut clauses = String::new();
    if !foreign_key.match_option.is_empty()
        && foreign_key.match_option != "SIMPLE"
        && foreign_key.match_option != "NONE"
    {
        clauses.push_str(&format!(" MATCH {}", foreign_key.match_option));
    }
    if !foreign_key.on_update.is_empty() && foreign_key.on_update != "NO ACTION" {
        clauses.push_str(&format!(" ON UPDATE {}", foreign_key.on_update));
    }
    if !foreign_key.on_delete.is_empty() && foreign_key.on_delete != "NO ACTION" {
        clauses.push_str(&format!(" ON DELETE {}", foreign_key.on_delete));
    }
    if foreign_key.deferrable {
        clauses.push_str(" DEFERRABLE");
        if foreign_key.initially_deferred {
            clauses.push_str(" INITIALLY DEFERRED");
        }
    }
    clauses
}

fn format_sqlite_unique_constraint(index: &crate::domain::schema::Index) -> String {
    let columns = quote_columns(&index.columns.iter().map(String::as_str).collect::<Vec<_>>());
    format!("    UNIQUE ({columns})")
}

fn append_index_ddl(ddl: &mut String, info: &TableInfo, driver: DriverType) {
    let index_target = qualify_name_for_driver(driver, &info.table.schema, &info.table.name);
    for index in &info.indexes {
        if index.primary || (driver == DriverType::SQLite && index.origin == IndexOrigin::UniqueConstraint) {
            continue;
        }
        if driver == DriverType::Postgres && !index.definition.is_empty() {
            ddl.push_str(&index.definition);
            ddl.push_str(";\n");
            continue;
        }
        let unique = if index.unique { "UNIQUE " } else { "" };
        let index_name = qualify_name_for_driver(driver, &index.schema, &index.name);
        let cols = quote_columns(&index.columns.iter().map(String::as_str).collect::<Vec<_>>());
        ddl.push_str(&format!(
            "CREATE {unique}INDEX {index_name} ON {index_target} ({cols});\n"
        ));
    }
}

fn append_postgres_foreign_keys(ddl: &mut String, qualified_table: &str, foreign_keys: &[ForeignKeyDdlGroup<'_>]) {
    for foreign_key in foreign_keys {
        let to_qualified = qualify_name(foreign_key.to_schema, foreign_key.to_table);
        let from_columns = quote_columns(&foreign_key.from_columns);
        let to_columns = quote_columns(&foreign_key.to_columns);
        ddl.push_str(&format!(
            "ALTER TABLE {qualified_table} ADD CONSTRAINT {} FOREIGN KEY ({from_columns}) REFERENCES {to_qualified} ({to_columns}){};\n",
            quote_identifier(foreign_key.name),
            format_foreign_key_actions(foreign_key),
        ));
    }
}

fn qualify_name_for_driver(driver: DriverType, schema: &str, name: &str) -> String {
    if driver == DriverType::SQLite {
        quote_identifier(name)
    } else {
        qualify_name(schema, name)
    }
}

fn quote_columns(columns: &[&str]) -> String {
    columns
        .iter()
        .map(|column| quote_identifier(column))
        .collect::<Vec<_>>()
        .join(", ")
}

fn check_constraint_definition(constraint: &CheckConstraint) -> String {
    if constraint
        .definition
        .trim_start()
        .to_ascii_uppercase()
        .starts_with("CHECK")
    {
        constraint.definition.clone()
    } else {
        format!("CHECK ({})", constraint.definition)
    }
}

fn format_trigger_ddl(trigger: &Trigger) -> String {
    // SQLite: definition is the full CREATE TRIGGER SQL from sqlite_master.
    // PostgreSQL: definition is action_statement (EXECUTE FUNCTION ...),
    //   so emit the function definition first, then the CREATE TRIGGER.
    if trigger.definition.to_ascii_uppercase().starts_with("CREATE TRIGGER") {
        format!("{};\n", trigger.definition)
    } else {
        let qualified = qualify_name(&trigger.schema, &trigger.table_name);
        let trigger_stmt = format!(
            "CREATE TRIGGER {}\n  {} {} ON {}\n  {};\n",
            quote_identifier(&trigger.name),
            trigger.timing,
            trigger.event,
            qualified,
            trigger.definition
        );
        if !trigger.function_def.is_empty() {
            let mut ddl = String::new();
            ddl.push_str(&trigger.function_def);
            if !trigger.function_def.ends_with('\n') {
                ddl.push('\n');
            }
            ddl.push('\n');
            ddl.push_str(&trigger_stmt);
            ddl
        } else {
            trigger_stmt
        }
    }
}

fn quote_identifier(name: &str) -> String {
    let escaped = name.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

/// Whether a SQLite path is concrete enough to fingerprint against the cache.
///
/// Unit tests often use a bare name like `testdb`; those stay cache-eligible
/// without touching the filesystem.
fn is_concrete_sqlite_path(database: &str) -> bool {
    let path = std::path::Path::new(database);
    path.is_absolute()
        || path.extension().is_some_and(|ext| {
            matches!(
                ext.to_str().unwrap_or_default().to_ascii_lowercase().as_str(),
                "sqlite" | "sqlite3" | "db" | "db3"
            )
        })
}

fn cached_schema_is_empty(cached: &IntrospectResult) -> bool {
    cached.tables.is_empty() && cached.views.is_empty() && cached.triggers.is_empty()
}

fn sqlite_file_matches_cached_schema(database: &str, cached: &IntrospectResult) -> bool {
    if !is_concrete_sqlite_path(database) {
        return true;
    }
    match std::fs::metadata(database) {
        Ok(meta) => {
            // Empty / truncated file cannot honestly describe a non-empty tree.
            if meta.len() == 0 {
                return cached_schema_is_empty(cached);
            }
            true
        }
        Err(_) => cached_schema_is_empty(cached),
    }
}

#[cfg(test)]
#[path = "schema_service_tests.rs"]
mod tests;
