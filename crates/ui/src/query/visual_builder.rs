//! Visual SELECT query builder (#247) — generate SQL + limited round-trip import.
//!
//! Generates SQL only; never executes. Unsupported constructs refuse lossy import.

use sqlparser::ast::{
    BinaryOperator, Expr, FunctionArg, FunctionArgExpr, FunctionArguments, GroupByExpr, JoinConstraint, JoinOperator,
    ObjectName, Query, Select, SelectItem, SetExpr, Statement, TableFactor, TableWithJoins,
};
use sqlparser::dialect::{GenericDialect, PostgreSqlDialect, SQLiteDialect};
use sqlparser::parser::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderDialect {
    Postgres,
    Sqlite,
}

impl BuilderDialect {
    pub fn quote(&self, ident: &str) -> String {
        let escaped = ident.replace('"', "\"\"");
        format!("\"{escaped}\"")
    }

    pub fn qualify(&self, schema: &str, table: &str) -> String {
        if schema.is_empty() {
            self.quote(table)
        } else {
            format!("{}.{}", self.quote(schema), self.quote(table))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderTable {
    pub schema: String,
    pub name: String,
    pub alias: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderColumn {
    pub table_alias: String,
    pub column: String,
    pub alias: Option<String>,
    pub aggregate: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderJoin {
    pub table: BuilderTable,
    pub join_type: String,
    pub left_alias: String,
    pub left_column: String,
    pub right_alias: String,
    pub right_column: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderPredicate {
    pub left_alias: String,
    pub left_column: String,
    pub op: String,
    pub value: String,
    pub is_null_check: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VisualQueryModel {
    pub tables: Vec<BuilderTable>,
    pub columns: Vec<BuilderColumn>,
    pub joins: Vec<BuilderJoin>,
    pub where_clauses: Vec<BuilderPredicate>,
    pub group_by: Vec<(String, String)>,
    pub having: Vec<BuilderPredicate>,
    pub order_by: Vec<(String, String, bool)>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub select_star: bool,
}

impl VisualQueryModel {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn next_alias(&self, table: &str) -> String {
        let base: String = table
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .take(3)
            .collect::<String>()
            .to_ascii_lowercase();
        let base = if base.is_empty() { "t".to_owned() } else { base };
        let mut n = 1usize;
        loop {
            let candidate = if n == 1 { base.clone() } else { format!("{base}{n}") };
            if !self.tables.iter().any(|t| t.alias == candidate)
                && !self.joins.iter().any(|j| j.table.alias == candidate)
            {
                return candidate;
            }
            n += 1;
        }
    }

    pub fn add_table(&mut self, schema: &str, name: &str) -> String {
        let alias = self.next_alias(name);
        self.tables.push(BuilderTable {
            schema: schema.to_owned(),
            name: name.to_owned(),
            alias: alias.clone(),
        });
        alias
    }

    pub fn generate_sql(&self, dialect: BuilderDialect) -> Result<String, String> {
        if self.tables.is_empty() {
            return Err("add at least one table".into());
        }
        let primary = &self.tables[0];
        let mut sql = String::from("SELECT ");
        if self.select_star || self.columns.is_empty() {
            sql.push('*');
        } else {
            let parts: Vec<String> = self
                .columns
                .iter()
                .map(|c| {
                    let expr = if let Some(agg) = &c.aggregate {
                        format!(
                            "{}({}.{})",
                            agg.to_ascii_uppercase(),
                            dialect.quote(&c.table_alias),
                            dialect.quote(&c.column)
                        )
                    } else {
                        format!("{}.{}", dialect.quote(&c.table_alias), dialect.quote(&c.column))
                    };
                    if let Some(alias) = &c.alias {
                        format!("{expr} AS {}", dialect.quote(alias))
                    } else {
                        expr
                    }
                })
                .collect();
            sql.push_str(&parts.join(", "));
        }
        sql.push_str(&format!(
            "\nFROM {} AS {}",
            dialect.qualify(&primary.schema, &primary.name),
            dialect.quote(&primary.alias)
        ));
        for join in &self.joins {
            let jt = join.join_type.to_ascii_uppercase();
            sql.push_str(&format!(
                "\n{jt} JOIN {} AS {} ON {}.{} = {}.{}",
                dialect.qualify(&join.table.schema, &join.table.name),
                dialect.quote(&join.table.alias),
                dialect.quote(&join.left_alias),
                dialect.quote(&join.left_column),
                dialect.quote(&join.right_alias),
                dialect.quote(&join.right_column)
            ));
        }
        if !self.where_clauses.is_empty() {
            sql.push_str("\nWHERE ");
            sql.push_str(&render_predicates(&self.where_clauses, dialect)?);
        }
        if !self.group_by.is_empty() {
            let parts: Vec<String> = self
                .group_by
                .iter()
                .map(|(a, c)| format!("{}.{}", dialect.quote(a), dialect.quote(c)))
                .collect();
            sql.push_str(&format!("\nGROUP BY {}", parts.join(", ")));
        }
        if !self.having.is_empty() {
            sql.push_str("\nHAVING ");
            sql.push_str(&render_predicates(&self.having, dialect)?);
        }
        if !self.order_by.is_empty() {
            let parts: Vec<String> = self
                .order_by
                .iter()
                .map(|(a, c, desc)| {
                    format!(
                        "{}.{}{}",
                        dialect.quote(a),
                        dialect.quote(c),
                        if *desc { " DESC" } else { " ASC" }
                    )
                })
                .collect();
            sql.push_str(&format!("\nORDER BY {}", parts.join(", ")));
        }
        if let Some(limit) = self.limit {
            sql.push_str(&format!("\nLIMIT {limit}"));
        }
        if let Some(offset) = self.offset {
            sql.push_str(&format!("\nOFFSET {offset}"));
        }
        sql.push(';');
        Ok(sql)
    }
}

fn render_predicates(preds: &[BuilderPredicate], dialect: BuilderDialect) -> Result<String, String> {
    let mut parts = Vec::new();
    for p in preds {
        let left = format!("{}.{}", dialect.quote(&p.left_alias), dialect.quote(&p.left_column));
        if p.is_null_check {
            let op = p.op.to_ascii_uppercase();
            if op != "IS NULL" && op != "IS NOT NULL" {
                return Err(format!("unsupported null check op `{op}`"));
            }
            parts.push(format!("{left} {op}"));
        } else {
            let op = normalize_op(&p.op)?;
            parts.push(format!("{left} {op} {}", quote_literal(&p.value)));
        }
    }
    Ok(parts.join(" AND "))
}

fn normalize_op(op: &str) -> Result<String, String> {
    match op.trim() {
        "=" | "<>" | "!=" | "<" | "<=" | ">" | ">=" | "LIKE" | "like" | "ILIKE" | "ilike" => {
            Ok(op.trim().to_ascii_uppercase().replace("!=", "<>"))
        }
        other => Err(format!("unsupported predicate op `{other}`")),
    }
}

fn quote_literal(value: &str) -> String {
    if value.eq_ignore_ascii_case("null") {
        return "NULL".into();
    }
    if value.parse::<i64>().is_ok() || value.parse::<f64>().is_ok() {
        return value.to_owned();
    }
    let trimmed = value.trim();
    if (trimmed.starts_with('\'') && trimmed.ends_with('\'')) || (trimmed.starts_with('"') && trimmed.ends_with('"')) {
        return trimmed.to_owned();
    }
    format!("'{}'", value.replace('\'', "''"))
}

/// Import a supported SELECT subset. Returns an explanatory error for unsupported SQL.
pub fn try_import_select(sql: &str, dialect: BuilderDialect) -> Result<VisualQueryModel, String> {
    let statements = match dialect {
        BuilderDialect::Postgres => Parser::parse_sql(&PostgreSqlDialect {}, sql),
        BuilderDialect::Sqlite => Parser::parse_sql(&SQLiteDialect {}, sql),
    }
    .or_else(|_| Parser::parse_sql(&GenericDialect {}, sql))
    .map_err(|e| format!("parse failed: {e}"))?;

    if statements.len() != 1 {
        return Err("only a single SELECT statement can be imported".into());
    }
    let Statement::Query(query) = &statements[0] else {
        return Err("only SELECT queries are supported".into());
    };
    import_query(query)
}

fn import_query(query: &Query) -> Result<VisualQueryModel, String> {
    if query.with.is_some() {
        return Err("WITH/CTE is not supported by the visual builder".into());
    }
    let SetExpr::Select(select) = query.body.as_ref() else {
        return Err("UNION/INTERSECT/EXCEPT are not supported by the visual builder".into());
    };
    if select.distinct.is_some() {
        return Err("DISTINCT is not supported by the visual builder".into());
    }
    if !select.lateral_views.is_empty() || select.top.is_some() || select.into.is_some() {
        return Err("unsupported SELECT options".into());
    }
    let mut model = VisualQueryModel::default();
    import_from(&mut model, select)?;
    import_projection(&mut model, select)?;
    if let Some(selection) = &select.selection {
        model.where_clauses = import_and_predicates(selection)?;
    }
    match &select.group_by {
        GroupByExpr::Expressions(exprs, _) => {
            for expr in exprs {
                model.group_by.push(import_column_ref(expr)?);
            }
        }
        GroupByExpr::All(_) => return Err("GROUP BY ALL is not supported".into()),
    }
    if let Some(having) = &select.having {
        model.having = import_and_predicates(having)?;
    }
    if let Some(order_by) = &query.order_by {
        for order in &order_by.exprs {
            let (alias, col) = import_column_ref(&order.expr)?;
            model.order_by.push((alias, col, order.asc == Some(false)));
        }
    }
    if let Some(limit) = &query.limit {
        model.limit = Some(import_u64(limit)?);
    }
    if let Some(offset) = &query.offset {
        model.offset = Some(import_u64(&offset.value)?);
    }
    Ok(model)
}

fn import_from(model: &mut VisualQueryModel, select: &Select) -> Result<(), String> {
    if select.from.len() != 1 {
        return Err("FROM must reference exactly one primary table (use JOINs for more)".into());
    }
    let TableWithJoins { relation, joins } = &select.from[0];
    let primary = import_table_factor(relation)?;
    model.tables.push(primary);
    for join in joins {
        let join_type = match &join.join_operator {
            JoinOperator::Inner(_) => "INNER",
            JoinOperator::LeftOuter(_) => "LEFT",
            JoinOperator::RightOuter(_) => "RIGHT",
            JoinOperator::FullOuter(_) => "FULL",
            other => return Err(format!("unsupported join type: {other:?}")),
        };
        let table = import_table_factor(&join.relation)?;
        let constraint = match &join.join_operator {
            JoinOperator::Inner(c)
            | JoinOperator::LeftOuter(c)
            | JoinOperator::RightOuter(c)
            | JoinOperator::FullOuter(c) => c,
            _ => unreachable!(),
        };
        let JoinConstraint::On(expr) = constraint else {
            return Err("only ON join constraints are supported".into());
        };
        let (left_alias, left_column, right_alias, right_column) = import_eq_columns(expr)?;
        model.joins.push(BuilderJoin {
            table,
            join_type: join_type.into(),
            left_alias,
            left_column,
            right_alias,
            right_column,
        });
    }
    Ok(())
}

fn import_projection(model: &mut VisualQueryModel, select: &Select) -> Result<(), String> {
    if select.projection.len() == 1 {
        if let SelectItem::Wildcard(_) = &select.projection[0] {
            model.select_star = true;
            return Ok(());
        }
    }
    for item in &select.projection {
        match item {
            SelectItem::UnnamedExpr(expr) => {
                model.columns.push(import_select_expr(expr, None)?);
            }
            SelectItem::ExprWithAlias { expr, alias } => {
                model.columns.push(import_select_expr(expr, Some(alias.value.clone()))?);
            }
            SelectItem::Wildcard(_) | SelectItem::QualifiedWildcard(_, _) => {
                return Err("mixed * projection is not supported".into());
            }
        }
    }
    Ok(())
}

fn import_select_expr(expr: &Expr, alias: Option<String>) -> Result<BuilderColumn, String> {
    match expr {
        Expr::CompoundIdentifier(parts) if parts.len() == 2 => Ok(BuilderColumn {
            table_alias: parts[0].value.clone(),
            column: parts[1].value.clone(),
            alias,
            aggregate: None,
        }),
        Expr::Function(func) => {
            let name = object_name_last(&func.name)?;
            let FunctionArguments::List(list) = &func.args else {
                return Err(format!("aggregate `{name}` must take one column argument"));
            };
            let args: Vec<&Expr> = list
                .args
                .iter()
                .filter_map(|a| match a {
                    FunctionArg::Unnamed(FunctionArgExpr::Expr(e)) => Some(e),
                    _ => None,
                })
                .collect();
            if args.len() != 1 {
                return Err(format!("aggregate `{name}` must take one column argument"));
            }
            let (table_alias, column) = import_column_ref(args[0])?;
            Ok(BuilderColumn {
                table_alias,
                column,
                alias,
                aggregate: Some(name.to_ascii_uppercase()),
            })
        }
        _ => Err("only table.column or AGG(table.column) projections are supported".into()),
    }
}

fn import_and_predicates(expr: &Expr) -> Result<Vec<BuilderPredicate>, String> {
    match expr {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            let mut out = import_and_predicates(left)?;
            out.extend(import_and_predicates(right)?);
            Ok(out)
        }
        Expr::IsNull(inner) => {
            let (left_alias, left_column) = import_column_ref(inner)?;
            Ok(vec![BuilderPredicate {
                left_alias,
                left_column,
                op: "IS NULL".into(),
                value: String::new(),
                is_null_check: true,
            }])
        }
        Expr::IsNotNull(inner) => {
            let (left_alias, left_column) = import_column_ref(inner)?;
            Ok(vec![BuilderPredicate {
                left_alias,
                left_column,
                op: "IS NOT NULL".into(),
                value: String::new(),
                is_null_check: true,
            }])
        }
        Expr::Like {
            negated: false,
            any: false,
            expr,
            pattern,
            ..
        } => {
            let (left_alias, left_column) = import_column_ref(expr)?;
            Ok(vec![BuilderPredicate {
                left_alias,
                left_column,
                op: "LIKE".into(),
                value: import_literal(pattern)?,
                is_null_check: false,
            }])
        }
        Expr::ILike {
            negated: false,
            any: false,
            expr,
            pattern,
            ..
        } => {
            let (left_alias, left_column) = import_column_ref(expr)?;
            Ok(vec![BuilderPredicate {
                left_alias,
                left_column,
                op: "ILIKE".into(),
                value: import_literal(pattern)?,
                is_null_check: false,
            }])
        }
        Expr::BinaryOp { left, op, right } => {
            let (left_alias, left_column) = import_column_ref(left)?;
            let op = match op {
                BinaryOperator::Eq => "=",
                BinaryOperator::NotEq => "<>",
                BinaryOperator::Lt => "<",
                BinaryOperator::LtEq => "<=",
                BinaryOperator::Gt => ">",
                BinaryOperator::GtEq => ">=",
                other => return Err(format!("unsupported predicate operator {other:?}")),
            };
            Ok(vec![BuilderPredicate {
                left_alias,
                left_column,
                op: op.into(),
                value: import_literal(right)?,
                is_null_check: false,
            }])
        }
        _ => Err("only AND-combined simple predicates are supported".into()),
    }
}

fn import_eq_columns(expr: &Expr) -> Result<(String, String, String, String), String> {
    let Expr::BinaryOp {
        left,
        op: BinaryOperator::Eq,
        right,
    } = expr
    else {
        return Err("join ON must be a single equality".into());
    };
    let (la, lc) = import_column_ref(left)?;
    let (ra, rc) = import_column_ref(right)?;
    Ok((la, lc, ra, rc))
}

fn import_column_ref(expr: &Expr) -> Result<(String, String), String> {
    match expr {
        Expr::CompoundIdentifier(parts) if parts.len() == 2 => Ok((parts[0].value.clone(), parts[1].value.clone())),
        _ => Err("column references must be alias.column".into()),
    }
}

fn import_literal(expr: &Expr) -> Result<String, String> {
    match expr {
        Expr::Value(v) => Ok(v.to_string()),
        Expr::Identifier(id) => Ok(id.value.clone()),
        _ => Err("predicate right-hand side must be a literal".into()),
    }
}

fn import_u64(expr: &Expr) -> Result<u64, String> {
    match expr {
        Expr::Value(v) => v
            .to_string()
            .parse::<u64>()
            .map_err(|_| "expected integer literal".into()),
        _ => Err("expected integer literal".into()),
    }
}

fn import_table_factor(factor: &TableFactor) -> Result<BuilderTable, String> {
    let TableFactor::Table { name, alias, .. } = factor else {
        return Err("only plain table references are supported".into());
    };
    let (schema, table) = split_object_name(name)?;
    let alias = alias
        .as_ref()
        .map(|a| a.name.value.clone())
        .unwrap_or_else(|| table.clone());
    Ok(BuilderTable {
        schema,
        name: table,
        alias,
    })
}

fn split_object_name(name: &ObjectName) -> Result<(String, String), String> {
    match name.0.as_slice() {
        [table] => Ok((String::new(), table.value.clone())),
        [schema, table] => Ok((schema.value.clone(), table.value.clone())),
        _ => Err("table name must be table or schema.table".into()),
    }
}

fn object_name_last(name: &ObjectName) -> Result<String, String> {
    name.0
        .last()
        .map(|p| p.value.clone())
        .ok_or_else(|| "empty function name".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_join_select() {
        let mut model = VisualQueryModel::default();
        model.add_table("public", "users");
        model.columns.push(BuilderColumn {
            table_alias: "use".into(),
            column: "id".into(),
            alias: None,
            aggregate: None,
        });
        model.joins.push(BuilderJoin {
            table: BuilderTable {
                schema: "public".into(),
                name: "orders".into(),
                alias: "ord".into(),
            },
            join_type: "INNER".into(),
            left_alias: "use".into(),
            left_column: "id".into(),
            right_alias: "ord".into(),
            right_column: "user_id".into(),
        });
        model.where_clauses.push(BuilderPredicate {
            left_alias: "use".into(),
            left_column: "active".into(),
            op: "=".into(),
            value: "true".into(),
            is_null_check: false,
        });
        model.limit = Some(50);
        let sql = model.generate_sql(BuilderDialect::Postgres).unwrap();
        assert!(sql.contains("FROM \"public\".\"users\" AS \"use\""));
        assert!(sql.contains("INNER JOIN \"public\".\"orders\" AS \"ord\""));
        assert!(sql.contains("LIMIT 50"));
    }

    #[test]
    fn round_trips_supported_select() {
        let sql = r#"
SELECT "u"."id", COUNT("o"."id") AS "order_count"
FROM "public"."users" AS "u"
INNER JOIN "public"."orders" AS "o" ON "u"."id" = "o"."user_id"
WHERE "u"."active" = true
GROUP BY "u"."id"
ORDER BY "u"."id" ASC
LIMIT 10;
"#;
        let model = try_import_select(sql, BuilderDialect::Postgres).unwrap();
        assert_eq!(model.tables.len(), 1);
        assert_eq!(model.joins.len(), 1);
        assert_eq!(model.columns.len(), 2);
        assert_eq!(model.limit, Some(10));
        let again = model.generate_sql(BuilderDialect::Postgres).unwrap();
        let model2 = try_import_select(&again, BuilderDialect::Postgres).unwrap();
        assert_eq!(model2.tables[0].name, "users");
        assert_eq!(model2.joins[0].table.name, "orders");
    }

    #[test]
    fn rejects_union() {
        let err = try_import_select("SELECT 1 UNION SELECT 2", BuilderDialect::Postgres).unwrap_err();
        assert!(err.to_ascii_lowercase().contains("union") || err.contains("not supported"));
    }
}
