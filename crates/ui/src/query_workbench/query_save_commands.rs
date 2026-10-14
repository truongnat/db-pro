//! Runtime protocol mapping for saved-query effects.

use super::{QueryLibraryState, RequestId, UiCommand};

pub(super) fn list_queries_command(request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::ListSavedQueries {
        request_id,
        connection_id,
    }
}

pub(super) fn list_folders_command(request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::ListQueryFolders {
        request_id,
        connection_id,
    }
}

pub(super) fn create_folder_command(
    library: &QueryLibraryState,
    request_id: RequestId,
    connection_id: String,
) -> Result<UiCommand, String> {
    Ok(UiCommand::CreateQueryFolder {
        request_id,
        connection_id,
        name: library.required_folder_name()?,
    })
}

pub(super) fn save_query_command(prepared: &super::query_save_actions::PreparedQuerySave) -> UiCommand {
    UiCommand::SaveQuery {
        request_id: prepared.request_id,
        connection_id: prepared.connection_id.clone(),
        saved_query_id: prepared.saved_query_id.clone(),
        name: prepared.name.clone(),
        sql: prepared.sql.clone(),
        folder: prepared.folder.clone(),
    }
}

pub(super) fn rename_query_command(request_id: RequestId, id: String, name: String) -> UiCommand {
    UiCommand::RenameSavedQuery { request_id, id, name }
}

pub(super) fn delete_query_command(request_id: RequestId, id: String) -> UiCommand {
    UiCommand::DeleteSavedQuery { request_id, id }
}
