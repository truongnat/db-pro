use async_trait::async_trait;

use crate::domain::connection::ConnectionHandle;
use crate::domain::error::DbError;
use crate::domain::rls::TableRlsState;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait RlsManager: Send + Sync {
    async fn table_rls_state(
        &self,
        handle: &ConnectionHandle,
        schema: &str,
        table: &str,
    ) -> Result<TableRlsState, DbError>;
}
