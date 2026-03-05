use keyring::{Entry, Error};

const SERVICE_NAME: &str = "io.dbpro.app.connection";

pub fn save_password(connection_id: &str, password: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, connection_id)
        .map_err(|err| format!("Failed to access system keychain entry: {err}"))?;

    entry
        .set_password(password)
        .map_err(|err| format!("Failed to store password in system keychain: {err}"))
}

pub fn load_password(connection_id: &str) -> Result<String, String> {
    let entry = Entry::new(SERVICE_NAME, connection_id)
        .map_err(|err| format!("Failed to access system keychain entry: {err}"))?;

    entry.get_password().map_err(|err| match err {
        Error::NoEntry => "No saved password found for this connection".to_string(),
        _ => format!("Failed to read password from system keychain: {err}"),
    })
}

pub fn delete_password(connection_id: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, connection_id)
        .map_err(|err| format!("Failed to access system keychain entry: {err}"))?;

    match entry.delete_password() {
        Ok(_) => Ok(()),
        Err(Error::NoEntry) => Ok(()),
        Err(err) => Err(format!(
            "Failed to remove password from system keychain: {err}"
        )),
    }
}
