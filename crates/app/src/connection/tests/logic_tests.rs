//! Tests for `../logic.rs` (moved out of the source file).
    use super::*;
    use crate::{RequestId, UiCommand, UiConnectionDraft, UiDriver};

    fn valid_network_draft() -> UiConnectionDraft {
        UiConnectionDraft {
            name: "conn".to_owned(),
            host: "db.example.com".to_owned(),
            database: "appdb".to_owned(),
            username: "alice".to_owned(),
            driver: UiDriver::Postgres,
            ..Default::default()
        }
    }

    #[test]
    fn network_draft_requires_host() {
        let mut draft = valid_network_draft();
        draft.host = "   ".to_owned();
        assert_eq!(
            validate_connection_draft(&draft),
            Err("Host is required".to_owned())
        );
    }

    #[test]
    fn network_draft_requires_username() {
        let mut draft = valid_network_draft();
        draft.username = String::new();
        assert_eq!(
            validate_connection_draft(&draft),
            Err("Username is required".to_owned())
        );
    }

    #[test]
    fn network_draft_with_host_and_username_passes() {
        assert_eq!(validate_connection_draft(&valid_network_draft()), Ok(()));
    }

    #[test]
    fn sqlite_draft_is_exempt_from_host_and_username() {
        let draft = UiConnectionDraft {
            name: "local file".to_owned(),
            database: "app.db".to_owned(),
            driver: UiDriver::Sqlite,
            host: String::new(),
            username: String::new(),
            port: String::new(),
            ..Default::default()
        };
        assert_eq!(validate_connection_draft(&draft), Ok(()));
    }

    #[test]
    fn connect_command_keeps_connection_identity_explicit() {
        assert!(matches!(
            build_connect_command(RequestId(7), "conn-1".to_owned()),
            UiCommand::Connect {
                request_id: RequestId(7),
                connection_id,
            } if connection_id == "conn-1"
        ));
    }