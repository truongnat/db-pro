use crate::domain::connection::ConnectionId;
use crate::domain::cross_connection::SchemaDiff;
use crate::domain::error::DbError;
use crate::domain::schema::IntrospectResult;

use super::SchemaService;
use schema_diff_compare::compare_introspect_results;

#[path = "schema_diff_compare.rs"]
mod schema_diff_compare;

#[cfg(test)]
use schema_diff_compare::{display_qualified_name, qualify_key};

pub fn compare_schemas(source: &IntrospectResult, target: &IntrospectResult) -> SchemaDiff {
    compare_introspect_results(source, target)
}

impl SchemaService {
    pub async fn diff_schemas(
        &self,
        source_id: &ConnectionId,
        target_id: &ConnectionId,
    ) -> Result<SchemaDiff, DbError> {
        let source = self.introspect(source_id, false).await?;
        let target = self.introspect(target_id, false).await?;
        Ok(compare_introspect_results(&source, &target))
    }
}

#[cfg(test)]
#[path = "tests/schema_diff_tests.rs"]
mod tests;
