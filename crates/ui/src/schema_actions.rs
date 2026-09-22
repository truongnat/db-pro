//! Runtime command mapping for schema requests.

use super::{RequestId, UiCommand};

pub(super) fn introspect_schema_command(
    request_id: RequestId,
    connection_id: String,
    force_refresh: bool,
) -> UiCommand {
    UiCommand::IntrospectSchema {
        request_id,
        connection_id,
        force_refresh,
    }
}
