//! Tests for `../fdw_activity_view.rs` (moved out of the source file).
    use super::{create_fdw_server_command, FdwState, RequestId, UiCommand};

    #[test]
    fn create_command_copies_form_state_and_confirms_ddl() {
        let state = FdwState {
            fdw_create_name: "analytics".to_owned(),
            fdw_create_wrapper: "postgres_fdw".to_owned(),
            fdw_create_host: "db.internal".to_owned(),
            fdw_create_dbname: "analytics".to_owned(),
            fdw_create_port: "5432".to_owned(),
            ..FdwState::default()
        };

        assert!(matches!(
            create_fdw_server_command(&state, RequestId(5), "source".to_owned()),
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