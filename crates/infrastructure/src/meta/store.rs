use db_pro_core::domain::error::DbError;

use super::schema::SCHEMA;
use crate::sqlite::actor::{SqliteActor, SqliteHandle};

#[derive(Clone)]
pub struct SQLiteMetaStore {
    pub(crate) actor: SqliteHandle,
}

impl SQLiteMetaStore {
    pub async fn new(db_path: &str) -> Result<Self, DbError> {
        let actor = SqliteActor::spawn(db_path)?;
        actor.execute_statement(SCHEMA.into()).await?;
        Ok(Self { actor })
    }

    pub fn handle(&self) -> &SqliteHandle {
        &self.actor
    }
}
