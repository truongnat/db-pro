//! Tests for `../saved_task_sql.rs` (moved out of the source file).
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