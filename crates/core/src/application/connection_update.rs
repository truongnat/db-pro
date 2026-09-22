use crate::domain::connection::{Connection, ConnectionConfig, ConnectionId};
use crate::domain::error::DbError;

use super::{validate_config, ConnectionService};

pub(super) async fn execute(
    service: &ConnectionService,
    id: &ConnectionId,
    config: ConnectionConfig,
    password: Option<&str>,
) -> Result<(), DbError> {
    let request = UpdateRequest { id, config, password };
    let mut update = PreparedUpdate::load(service, request).await?;

    update.apply_database_secret(service).await?;
    if let Err(error) = update.apply_ssh_secret(service).await {
        update.restore_secrets(service).await;
        return Err(error);
    }
    update.clear_ssh_password();

    if let Err(error) = service.repo.save(&update.connection).await {
        update.restore_secrets(service).await;
        return Err(error);
    }

    if service.registry.is_active(id) {
        if let Err(error) = service.disconnect(id).await {
            restore_persisted_connection(service, &update).await;
            update.restore_secrets(service).await;
            return Err(error);
        }
    }

    service.invalidate_introspection_cache(id).await;
    Ok(())
}

struct UpdateRequest<'a> {
    id: &'a ConnectionId,
    config: ConnectionConfig,
    password: Option<&'a str>,
}

struct PreparedUpdate {
    previous: Connection,
    connection: Connection,
    database_secret: DatabaseSecretChange,
    ssh_secret: SshSecretChange,
}

impl PreparedUpdate {
    async fn load(service: &ConnectionService, request: UpdateRequest<'_>) -> Result<Self, DbError> {
        validate_config(&request.config)?;
        let previous = service
            .repo
            .get(request.id)
            .await?
            .ok_or_else(|| DbError::NotFound(format!("connection {}", request.id)))?;
        let database_secret = DatabaseSecretChange::plan(service, request.id, &previous, &request).await?;
        let ssh_secret = SshSecretChange::plan(service, request.id, &previous, &request.config).await?;

        let mut connection = previous.clone();
        connection.config = request.config;
        connection.updated_at = chrono::Utc::now();

        Ok(Self {
            previous,
            connection,
            database_secret,
            ssh_secret,
        })
    }

    async fn apply_database_secret(&mut self, service: &ConnectionService) -> Result<(), DbError> {
        if self.database_secret.requires_database_secret {
            if let Some(password) = self.database_secret.new_password.as_deref() {
                if let Err(error) = service.secrets.store_secret(&self.database_secret.key, password).await {
                    service
                        .restore_secret(
                            &self.database_secret.key,
                            self.database_secret.previous_password.as_deref(),
                        )
                        .await;
                    return Err(error);
                }
                self.connection.secret_ref = Some(self.database_secret.key.clone());
            }
        } else {
            self.connection.secret_ref = None;
            if self.database_secret.changed {
                if let Err(error) = service.secrets.delete_secret(&self.database_secret.key).await {
                    service
                        .restore_secret(
                            &self.database_secret.key,
                            self.database_secret.previous_password.as_deref(),
                        )
                        .await;
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    async fn apply_ssh_secret(&self, service: &ConnectionService) -> Result<(), DbError> {
        if let Some(password) = self.ssh_secret.new_password.as_deref() {
            return service.secrets.store_secret(&self.ssh_secret.key, password).await;
        }
        if let Some(password) = self.ssh_secret.migrated_password.as_deref() {
            return service.secrets.store_secret(&self.ssh_secret.key, password).await;
        }
        if self.ssh_secret.removed {
            return service.secrets.delete_secret(&self.ssh_secret.key).await;
        }
        Ok(())
    }

    async fn restore_secrets(&self, service: &ConnectionService) {
        if self.database_secret.changed {
            service
                .restore_secret(
                    &self.database_secret.key,
                    self.database_secret.previous_password.as_deref(),
                )
                .await;
        }
        if self.ssh_secret.changed {
            service
                .restore_secret(&self.ssh_secret.key, self.ssh_secret.previous_secret.as_deref())
                .await;
        }
    }

    fn clear_ssh_password(&mut self) {
        if let Some(ssh_tunnel) = self.connection.config.ssh_tunnel.as_mut() {
            ssh_tunnel.password = None;
        }
    }
}

struct DatabaseSecretChange {
    key: String,
    requires_database_secret: bool,
    changed: bool,
    previous_password: Option<String>,
    new_password: Option<String>,
}

impl DatabaseSecretChange {
    async fn plan(
        service: &ConnectionService,
        id: &ConnectionId,
        previous: &Connection,
        request: &UpdateRequest<'_>,
    ) -> Result<Self, DbError> {
        let previous_requires_secret = ConnectionService::requires_database_secret(&previous.config);
        let requires_database_secret = ConnectionService::requires_database_secret(&request.config);
        if requires_database_secret
            && !previous_requires_secret
            && request.password.is_none()
            && previous.secret_ref.is_none()
        {
            return Err(DbError::AuthFailed(
                "database password is required when switching to PostgreSQL".into(),
            ));
        }

        let key = previous
            .secret_ref
            .clone()
            .unwrap_or_else(|| ConnectionService::secret_key(id));
        let changed = (requires_database_secret && request.password.is_some())
            || (!requires_database_secret && (previous_requires_secret || previous.secret_ref.is_some()));
        let previous_password = if changed {
            service.secrets.retrieve_secret(&key).await?
        } else {
            None
        };

        Ok(Self {
            key,
            requires_database_secret,
            changed,
            previous_password,
            new_password: request.password.map(str::to_owned),
        })
    }
}

struct SshSecretChange {
    key: String,
    changed: bool,
    previous_secret: Option<String>,
    new_password: Option<String>,
    migrated_password: Option<String>,
    removed: bool,
}

impl SshSecretChange {
    async fn plan(
        service: &ConnectionService,
        id: &ConnectionId,
        previous: &Connection,
        config: &ConnectionConfig,
    ) -> Result<Self, DbError> {
        let previous_password = previous
            .config
            .ssh_tunnel
            .as_ref()
            .and_then(|ssh_tunnel| ssh_tunnel.password.clone());
        let new_password = config
            .ssh_tunnel
            .as_ref()
            .and_then(|ssh_tunnel| ssh_tunnel.password.clone());
        let previous_has_ssh = previous.config.ssh_tunnel.is_some();
        let new_has_ssh = config.ssh_tunnel.is_some();
        let migrated_password = (new_has_ssh && new_password.is_none())
            .then_some(previous_password.clone())
            .flatten();
        let removed = !new_has_ssh && previous_has_ssh;
        let changed = removed || new_password.is_some() || migrated_password.is_some();
        let key = ConnectionService::ssh_secret_key(id);
        let previous_secret = if changed && previous_has_ssh {
            service.secrets.retrieve_secret(&key).await?
        } else {
            None
        };

        Ok(Self {
            key,
            changed,
            previous_secret,
            new_password,
            migrated_password,
            removed,
        })
    }
}

async fn restore_persisted_connection(service: &ConnectionService, update: &PreparedUpdate) {
    if let Err(error) = service.repo.save(&update.previous).await {
        tracing::error!("failed to restore previous connection after disconnect failure: {error}");
    }
}
