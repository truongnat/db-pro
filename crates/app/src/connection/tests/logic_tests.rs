//! Tests for `../logic.rs` (moved out of the source file).
    use super::build_connect_command;
    use crate::{RequestId, UiCommand};

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