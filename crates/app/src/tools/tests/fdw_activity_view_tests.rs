//! Tests for `../fdw_activity_view.rs` (moved out of the source file).
    use super::{create_fdw_server_command, fdw_state::FdwCreateApply, RequestId, UiCommand};

    #[test]
    fn create_command_copies_pending_payload_and_confirms_ddl() {
        let pending = FdwCreateApply {
            name: "analytics".to_owned(),
            fdw: "postgres_fdw".to_owned(),
            host: "db.internal".to_owned(),
            dbname: "analytics".to_owned(),
            port: "5432".to_owned(),
            sql: "CREATE SERVER analytics FOREIGN DATA WRAPPER postgres_fdw".to_owned(),
        };

         assert!(matches!(
            create_fdw_server_command(&pending, RequestId(5), "source".to_owned()),
             UiCommand::CreateFdwServer {
                 request_id: RequestId(5),
                 connection_id,
                 name,
                 fdw,
                 host,
                 dbname,
                 port,
                 confirmed: true,
             } if connection_id == "source"
                 && name == "analytics"
                 && fdw == "postgres_fdw"
                 && host == "db.internal"
                 && dbname == "analytics"
                 && port == "5432"
         ));
    }

    #[test]
    fn create_confirm_holds_the_previewed_sql() {
        let pending = FdwCreateApply {
            name: "lake".to_owned(),
            sql: "CREATE SERVER lake".to_owned(),
            ..FdwCreateApply {
                name: String::new(),
                fdw: String::new(),
                host: String::new(),
                dbname: String::new(),
                port: String::new(),
                sql: String::new(),
            }
        };
        assert_eq!(pending.sql, "CREATE SERVER lake");
    }