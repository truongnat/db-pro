//! State owned by the foreign-data-wrapper administration surface.

use super::{RequestId, UiCommand};

pub(super) struct FdwState {
    pub(super) fdw_inventory: Option<db_pro_core::domain::fdw::FdwInventory>,
    pub(super) fdw_error: Option<String>,
    pub(super) fdw_create_name: String,
    pub(super) fdw_create_wrapper: String,
    pub(super) fdw_create_host: String,
    pub(super) fdw_create_dbname: String,
    pub(super) fdw_create_port: String,
    pub(super) fdw_ddl_preview: Option<String>,
    pub(super) fdw_drop_confirm: Option<String>,
}

impl Default for FdwState {
    fn default() -> Self {
        Self {
            fdw_inventory: None,
            fdw_error: None,
            fdw_create_name: String::new(),
            fdw_create_wrapper: "postgres_fdw".to_owned(),
            fdw_create_host: String::new(),
            fdw_create_dbname: String::new(),
            fdw_create_port: "5432".to_owned(),
            fdw_ddl_preview: None,
            fdw_drop_confirm: None,
        }
    }
}

impl FdwState {
    pub(super) fn list_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::ListFdwInventory {
            request_id,
            connection_id,
        }
    }

    pub(super) fn create_command(&self, request_id: RequestId, connection_id: String) -> UiCommand {
        UiCommand::CreateFdwServer {
            request_id,
            connection_id,
            name: self.fdw_create_name.clone(),
            fdw: self.fdw_create_wrapper.clone(),
            host: self.fdw_create_host.clone(),
            dbname: self.fdw_create_dbname.clone(),
            port: self.fdw_create_port.clone(),
            confirmed: true,
        }
    }

    pub(super) fn drop_command(
        &self,
        request_id: RequestId,
        connection_id: String,
        name: String,
        cascade: bool,
    ) -> UiCommand {
        UiCommand::DropFdwServer {
            request_id,
            connection_id,
            name,
            cascade,
            confirmed: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FdwState, RequestId, UiCommand};

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
            state.create_command(RequestId(5), "source".to_owned()),
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
}
