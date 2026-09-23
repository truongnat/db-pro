use super::mapper::draft_to_domain_config;
use crate::{RequestId, UiCommand, UiConnectionDraft, UiDriver, UiSslMode};
use db_pro_core::domain::connection::SshProfile;
use db_pro_core::domain::connection_diagnostics::{
    probe_network_stages, with_auth_result, ConnectionDiagnosticsReport,
};

/// Validate that the draft has all required fields and valid ports.
pub fn validate_connection_draft(draft: &UiConnectionDraft) -> Result<(), String> {
    if draft.name.trim().is_empty() || draft.database.trim().is_empty() {
        return Err("Name and database are required".to_owned());
    }
    if matches!(draft.driver, UiDriver::Postgres | UiDriver::Mysql | UiDriver::SqlServer)
        && draft.port.parse::<u16>().is_err()
    {
        return Err("Port must be a number between 1 and 65535".to_owned());
    }
    if draft.ssh_tunnel_enabled {
        if draft.ssh_host.trim().is_empty()
            || draft.ssh_user.trim().is_empty()
            || draft.ssh_private_key.trim().is_empty()
        {
            return Err("SSH host, user and private key are required".to_owned());
        }
        if draft.ssh_port.parse::<u16>().is_err() {
            return Err("SSH port must be a number between 1 and 65535".to_owned());
        }
    }
    Ok(())
}

/// Apply a driver selection onto a draft, setting conventional default ports and TLS defaults.
pub fn select_driver(draft: &mut UiConnectionDraft, driver: UiDriver) {
    if driver != draft.driver {
        match driver {
            UiDriver::Postgres => {
                draft.ssl_mode = UiSslMode::Require;
                if draft.port == "3306" || draft.port == "1433" || draft.port.is_empty() {
                    draft.port = "5432".to_owned();
                }
            }
            UiDriver::Mysql => {
                draft.ssl_mode = UiSslMode::Require;
                if draft.port == "5432" || draft.port == "1433" || draft.port.is_empty() {
                    draft.port = "3306".to_owned();
                }
            }
            UiDriver::SqlServer => {
                draft.ssl_mode = UiSslMode::Require;
                if draft.port == "5432" || draft.port == "3306" || draft.port.is_empty() {
                    draft.port = "1433".to_owned();
                }
            }
            UiDriver::Sqlite => {}
        }
    }
    draft.driver = driver;
}

/// Save the current SSH fields in a draft as a new reusable [`SshProfile`].
pub fn save_ssh_profile(profiles: &mut Vec<SshProfile>, draft: &UiConnectionDraft) -> Result<String, String> {
    if draft.ssh_host.trim().is_empty() || draft.ssh_user.trim().is_empty() {
        return Err("SSH host and user are required to save a profile".to_owned());
    }
    let id = uuid::Uuid::new_v4().to_string();
    let profile = SshProfile {
        id: id.clone(),
        name: format!("{}@{}", draft.ssh_user.trim(), draft.ssh_host.trim()),
        host: draft.ssh_host.clone(),
        port: draft.ssh_port.parse().unwrap_or(22),
        user: draft.ssh_user.clone(),
        private_key_path: draft.ssh_private_key.clone(),
        password: None,
    };
    profiles.push(profile);
    Ok(id)
}

/// Apply a saved [`SshProfile`] onto a draft.
pub fn apply_ssh_profile_to_draft(draft: &mut UiConnectionDraft, profile: &SshProfile) {
    draft.ssh_tunnel_enabled = true;
    draft.ssh_host = profile.host.clone();
    draft.ssh_port = profile.port.to_string();
    draft.ssh_user = profile.user.clone();
    draft.ssh_private_key = profile.private_key_path.clone();
    draft.ssh_profile_id = profile.id.clone();
}

/// Construct the appropriate [`UiCommand`] for testing or saving a connection.
pub fn build_connection_command(
    draft: UiConnectionDraft,
    editing_id: Option<String>,
    request_id: RequestId,
    save: bool,
) -> UiCommand {
    if save {
        if let Some(connection_id) = editing_id {
            UiCommand::UpdateConnection {
                request_id,
                connection_id,
                draft,
            }
        } else {
            UiCommand::CreateConnection { request_id, draft }
        }
    } else {
        UiCommand::TestConnection { request_id, draft }
    }
}

/// Build the runtime command for switching to a saved connection.
pub fn build_connect_command(request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::Connect {
        request_id,
        connection_id,
    }
}

/// Build the initial saved-connection catalog request.
pub fn build_list_connections_command(request_id: RequestId) -> UiCommand {
    UiCommand::ListConnections { request_id }
}

/// Run network stage probes and produce diagnostics report.
pub fn probe_draft_diagnostics(
    draft: &UiConnectionDraft,
    auth_ok: bool,
    auth_message: &str,
) -> ConnectionDiagnosticsReport {
    let config = draft_to_domain_config(draft);
    let report = probe_network_stages(&config);
    with_auth_result(report, auth_ok, auth_message)
}

#[cfg(test)]
mod tests {
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
}
