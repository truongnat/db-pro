use db_pro_core::domain::connection::DriverType;

const MAX_QUALIFIED_IDENTIFIER_PARTS: usize = 3;

pub(super) fn build_export_query(driver_label: &str, table: &str) -> Result<String, String> {
    let driver = parse_driver(driver_label)?;
    let table = quote_qualified_identifier(driver, table)?;

    Ok(match driver {
        DriverType::SqlServer => format!("SELECT TOP 1000 * FROM {table}"),
        DriverType::Postgres | DriverType::SQLite | DriverType::Mysql => {
            format!("SELECT * FROM {table} LIMIT 1000")
        }
    })
}

pub(super) fn build_maintenance_query(
    driver_label: &str,
    operation: &str,
    target: Option<&str>,
) -> Result<String, String> {
    let driver = parse_driver(driver_label)?;
    let operation = operation.trim().to_ascii_lowercase();
    let target = target
        .map(|value| quote_qualified_identifier(driver, value))
        .transpose()?;

    match (driver, operation.as_str(), target.as_deref()) {
        (DriverType::Postgres, "vacuum", target) => Ok(format_optional_target("VACUUM", target)),
        (DriverType::Postgres, "analyze", target) => Ok(format_optional_target("ANALYZE", target)),
        (DriverType::SQLite, "vacuum", None) => Ok("VACUUM".to_owned()),
        (DriverType::SQLite, "vacuum", Some(_)) => Err("SQLite VACUUM does not accept a table target".to_owned()),
        (DriverType::SQLite, "analyze", target) => Ok(format_optional_target("ANALYZE", target)),
        (DriverType::Mysql, "analyze", Some(target)) => Ok(format!("ANALYZE TABLE {target}")),
        (DriverType::Mysql, "analyze", None) => Err("MySQL ANALYZE requires a table target".to_owned()),
        (DriverType::Mysql, "vacuum", _) => Err("MySQL does not support VACUUM".to_owned()),
        (DriverType::SqlServer, _, _) => Err("SQL Server maintenance tasks are not supported".to_owned()),
        (_, _, _) => Err(format!("unsupported maintenance operation: {operation}")),
    }
}

pub(super) fn validate_export_format(format: &str) -> Result<&'static str, String> {
    match format.trim().to_ascii_lowercase().as_str() {
        "csv" => Ok("csv"),
        "tsv" => Ok("tsv"),
        "json" => Ok("json"),
        _ => Err("export format must be csv, tsv, or json".to_owned()),
    }
}

fn parse_driver(label: &str) -> Result<DriverType, String> {
    match label.trim().to_ascii_lowercase().as_str() {
        "postgres" | "postgresql" => Ok(DriverType::Postgres),
        "sqlite" => Ok(DriverType::SQLite),
        "mysql" => Ok(DriverType::Mysql),
        "sql server" | "sqlserver" | "mssql" => Ok(DriverType::SqlServer),
        _ => Err(format!("unsupported database driver: {label}")),
    }
}

fn quote_qualified_identifier(driver: DriverType, raw: &str) -> Result<String, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Err("identifier is required".to_owned());
    }

    let parts: Vec<_> = value.split('.').map(str::trim).collect();
    if parts.len() > MAX_QUALIFIED_IDENTIFIER_PARTS {
        return Err("identifier may contain at most three qualified parts".to_owned());
    }
    if parts.iter().any(|part| part.is_empty()) {
        return Err("identifier contains an empty qualified part".to_owned());
    }
    if parts.iter().any(|part| part.chars().any(char::is_control)) {
        return Err("identifier contains a control character".to_owned());
    }

    Ok(parts
        .into_iter()
        .map(|part| quote_identifier_part(driver, part))
        .collect::<Vec<_>>()
        .join("."))
}

fn quote_identifier_part(driver: DriverType, part: &str) -> String {
    match driver {
        DriverType::Mysql => format!("`{}`", part.replace('`', "``")),
        DriverType::SqlServer => format!("[{}]", part.replace(']', "]]")),
        DriverType::Postgres | DriverType::SQLite => format!("\"{}\"", part.replace('"', "\"\"")),
    }
}

fn format_optional_target(operation: &str, target: Option<&str>) -> String {
    match target {
        Some(target) => format!("{operation} {target}"),
        None => operation.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_qualified_identifiers_without_allowing_sql_injection() {
        let sql = build_export_query("PostgreSQL", "public.users\"; DROP TABLE audit;--").unwrap();

        assert_eq!(
            sql,
            "SELECT * FROM \"public\".\"users\"\"; DROP TABLE audit;--\" LIMIT 1000"
        );
    }

    #[test]
    fn uses_provider_specific_export_limit_syntax() {
        assert_eq!(
            build_export_query("SQL Server", "dbo.users").unwrap(),
            "SELECT TOP 1000 * FROM [dbo].[users]"
        );
        assert_eq!(
            build_export_query("MySQL", "users").unwrap(),
            "SELECT * FROM `users` LIMIT 1000"
        );
    }

    #[test]
    fn gates_unsupported_maintenance_by_provider() {
        assert_eq!(
            build_maintenance_query("MySQL", "analyze", Some("app.users")).unwrap(),
            "ANALYZE TABLE `app`.`users`"
        );
        assert_eq!(
            build_maintenance_query("SQLite", "vacuum", Some("users")),
            Err("SQLite VACUUM does not accept a table target".to_owned())
        );
        assert_eq!(
            build_maintenance_query("SQL Server", "analyze", None),
            Err("SQL Server maintenance tasks are not supported".to_owned())
        );
    }

    #[test]
    fn validates_export_format() {
        assert_eq!(validate_export_format(" JSON "), Ok("json"));
        assert!(validate_export_format("xml").is_err());
    }
}
