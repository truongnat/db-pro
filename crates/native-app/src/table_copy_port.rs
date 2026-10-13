//! UI copy port backed by the open connections in the runtime registry.

use std::sync::Arc;

use db_pro_core::application::TableCopyConnector;
use db_pro_core::domain::connection::{ConnectionHandle, ConnectionId};
use db_pro_core::domain::query::{QueryParam, QueryResult};
use db_pro_core::ports::DbConnector;
use db_pro_runtime::DbProRuntime;

pub(crate) fn live(runtime: Arc<DbProRuntime>, tokio: tokio::runtime::Handle) -> Arc<dyn TableCopyConnector> {
    Arc::new(LiveCopy { runtime, tokio })
}

struct LiveCopy {
    runtime: Arc<DbProRuntime>,
    tokio: tokio::runtime::Handle,
}

impl LiveCopy {
    fn handle(&self, connection_id: &str) -> Result<ConnectionHandle, String> {
        let id = ConnectionId::parse(connection_id).map_err(|_| format!("connection {connection_id} is not open"))?;
        self.runtime
            .registry()
            .get(&id)
            .ok_or_else(|| format!("connection {connection_id} is not open"))
    }

    fn dialect(&self, connection_id: &str) -> Result<Box<dyn db_pro_core::ports::SqlDialect>, String> {
        let handle = self.handle(connection_id)?;
        self.runtime
            .connector()
            .dialect(&handle)
            .map_err(|error| error.to_string())
    }
}

impl TableCopyConnector for LiveCopy {
    fn quote(&self, connection_id: &str, identifier: &str) -> Result<String, String> {
        Ok(self.dialect(connection_id)?.quote_identifier(identifier))
    }

    fn placeholder(&self, connection_id: &str, index: usize) -> Result<String, String> {
        Ok(self.dialect(connection_id)?.placeholder(index))
    }

    fn query(&self, connection_id: &str, sql: &str) -> Result<QueryResult, String> {
        let handle = self.handle(connection_id)?;
        let connector = self.runtime.connector();
        let sql = sql.to_owned();
        self.tokio.block_on(async move {
            connector
                .query(&handle, &sql, &[])
                .await
                .map_err(|error| error.to_string())
        })
    }

    fn execute(&self, connection_id: &str, sql: &str, params: &[QueryParam]) -> Result<u64, String> {
        let handle = self.handle(connection_id)?;
        let connector = self.runtime.connector();
        let sql = sql.to_owned();
        let params = params.to_vec();
        self.tokio.block_on(async move {
            connector
                .execute(&handle, &sql, &params)
                .await
                .map_err(|error| error.to_string())
        })
    }
}
