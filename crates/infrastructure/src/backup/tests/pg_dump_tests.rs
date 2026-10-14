//! Tests for `../pg_dump.rs` (moved out of the source file).
use super::*;

#[test]
fn ensure_tool_available_rejects_missing_binary() {
    let err = PgDumpEngine::ensure_tool_available("db-pro-definitely-missing-tool-xyz").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("not found on PATH"), "{msg}");
    assert!(msg.contains("Install PostgreSQL client tools"), "{msg}");
}

#[test]
fn ensure_tool_available_finds_common_unix_binary() {
    // `sh` is present on unix CI images; proves the PATH search path works.
    assert!(PgDumpEngine::ensure_tool_available("sh").is_ok());
}

/// #244 (E-2) asks for the argv shape to be asserted without a live server. This pins the
/// **chosen** behaviour (option (b) of the issue): the restore is not wrapped in a transaction,
/// so the app must say so on failure rather than silently implying an atomic restore.
#[test]
// cc-scan:allow HUGE_FUNCTION — argv-assertion test builds the full config inline; splitting fragments the scenario
fn restore_argv_has_no_transaction_boundary_and_is_documented_as_such() {
    let config = ConnectionConfig {
        name: "fixture".to_owned(),
        host: "127.0.0.1".to_owned(),
        port: 55432,
        database: "dbpro_fixture".to_owned(),
        username: "dbpro".to_owned(),
        driver: db_pro_core::domain::connection::DriverType::Postgres,
        // cc-scan:allow DUPLICATE_BLOCK — coincidental boilerplate, not a real clone
        ssl_mode: db_pro_core::domain::connection::SslMode::Disable,
        ssh_tunnel: None,
        ssh_profile_id: None,
        ssl_root_cert_path: None,
        ssl_client_cert_path: None,
        ssl_client_key_path: None,
        query_timeout_ms: 30_000,
        max_rows: 500,
        color: None,
        tags: Vec::new(),
        group: None,
        favorite: false,
        environment: Default::default(),
        readonly: false,
    };
    let plain = RestoreOptions {
        connection_id: "c1".to_owned(),
        input_path: "/tmp/dump.sql".to_owned(),
        format: BackupFormat::Plain,
    };
    let custom = RestoreOptions {
        connection_id: "c1".to_owned(),
        input_path: "/tmp/dump.dump".to_owned(),
        format: BackupFormat::Custom,
    };

    let plain_args = PgDumpEngine::restore_command(&plain, &config, "pw")
        .as_std()
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let custom_args = PgDumpEngine::restore_command(&custom, &config, "pw")
        .as_std()
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    assert_eq!(plain_args.first().map(String::as_str), Some("-f"));
    assert_eq!(plain_args.get(1).map(String::as_str), Some("/tmp/dump.sql"));
    assert_eq!(custom_args.first().map(String::as_str), Some("/tmp/dump.dump"));
    for args in [&plain_args, &custom_args] {
        assert!(
            !args.iter().any(|arg| arg.contains("single-transaction")),
            "option (b) is chosen: the restore must not silently become transactional: {args:?}"
        );
        assert!(
            !args.iter().any(|arg| arg == "--exit-on-error"),
            "option (b) is chosen: no exit-on-error flag either: {args:?}"
        );
        assert!(args.windows(2).any(|pair| pair == ["-d", "dbpro_fixture"]), "{args:?}");
        assert!(
            !args.iter().any(|arg| arg.contains("pw")),
            "password must not appear in argv (use PGPASSWORD env): {args:?}"
        );
    }

    let plain_env = PgDumpEngine::restore_command(&plain, &config, "pw")
        .as_std()
        .get_envs()
        .find_map(|(key, value)| {
            (key.to_string_lossy() == "PGPASSWORD").then(|| value.map(|v| v.to_string_lossy().into_owned()))
        });
    assert_eq!(plain_env, Some(Some("pw".to_owned())));
}

#[tokio::test]
async fn external_command_timeout_returns_query_timeout() {
    let mut command = Command::new("sh");
    command.args(["-c", "sleep 1"]);

    let error = PgDumpEngine::run_command(&mut command, 50, "test command")
        .await
        .expect_err("a sleeping command must exceed the test timeout");

    assert!(matches!(error, DbError::QueryTimeout { timeout_ms: 50 }));
}

#[tokio::test]
async fn backup_output_reservation_rejects_existing_file() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join(format!("db-pro-pg-backup-{}.dump", uuid::Uuid::new_v4()));
    tokio::fs::write(&path, b"existing backup").await?;

    let error = match PgDumpEngine::reserve_backup_output(&path).await {
        Ok(()) => return Err("existing backup output was reserved".into()),
        Err(error) => error,
    };

    assert!(matches!(error, DbError::Validation(message) if message.contains("already exists")));
    tokio::fs::remove_file(path).await?;
    Ok(())
}
