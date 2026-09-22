//! State owned by the foreign-data-wrapper administration surface.

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
