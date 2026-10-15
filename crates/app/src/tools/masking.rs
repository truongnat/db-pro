//! Pure masking preview construction for the database management surface.

use db_pro_core::domain::masking::{preview_masking, ColumnMask, MaskRule, MaskingPreview, MaskingProfile};


/// Per-install key material for keyed masking (#395).
///
/// Keyed masking is only safe when the key is secret; a literal compiled into
/// the binary makes every install share the same key and the masked output
/// publicly reproducible. The key lives at `<data dir>/masking.key`, created
/// on first use with owner-only permissions, and never leaves the machine.
pub(super) fn local_masking_key_material() -> Result<String, String> {
    let dir = masking_state_dir().ok_or_else(|| "cannot resolve the local data directory".to_owned())?;
    std::fs::create_dir_all(&dir).map_err(|err| format!("cannot create {}: {err}", dir.display()))?;
    let path = dir.join("masking.key");
    match std::fs::read_to_string(&path) {
        Ok(key) if !key.trim().is_empty() => return Ok(key.trim().to_owned()),
        Ok(_) | Err(_) => {}
    }
    let key = uuid::Uuid::new_v4().to_string();
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&path)
        .map_err(|err| format!("cannot write {}: {err}", path.display()))?;
    use std::io::Write;
    file.write_all(key.as_bytes())
        .map_err(|err| format!("cannot write {}: {err}", path.display()))?;
    Ok(key)
}

/// Mirrors the platform data dir resolution in `native-app` (`DB_PRO_DATA_DIR`
/// override, then the per-user platform directory) so the masking key lives
/// beside the rest of DB Pro's local state.
fn masking_state_dir() -> Option<std::path::PathBuf> {
    let non_empty = |key: &str| std::env::var_os(key).filter(|value| !value.is_empty());
    if let Some(dir) = non_empty("DB_PRO_DATA_DIR") {
        return Some(std::path::PathBuf::from(dir));
    }
    if cfg!(target_os = "macos") {
        non_empty("HOME").map(|home| {
            std::path::PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("db-pro")
        })
    } else if cfg!(target_os = "windows") {
        non_empty("APPDATA").map(|appdata| std::path::PathBuf::from(appdata).join("db-pro"))
    } else {
        non_empty("XDG_DATA_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| non_empty("HOME").map(|home| std::path::PathBuf::from(home).join(".local").join("share")))
            .map(|data_root| data_root.join("db-pro"))
    }
}
// cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
pub(super) fn build_preview(columns_csv: &str, rule: MaskRule, keyed: bool) -> MaskingPreview {
    let columns: Vec<String> = columns_csv
        .split(',')
        .map(str::trim)
        .filter(|column| !column.is_empty())
        .map(str::to_owned)
        .collect();
    let headers = if columns.is_empty() {
        vec!["email".into(), "phone".into(), "id".into()]
    } else {
        let mut headers = columns.clone();
        if !headers.iter().any(|column| column == "id") {
            headers.push("id".into());
        }
        headers
    };
    let sample_row = |email: &str, phone: &str, fallback_prefix: &str| {
        headers
            .iter()
            .map(|header| match header.as_str() {
                "email" => email.to_owned(),
                "phone" => phone.to_owned(),
                "id" => "42".to_owned(),
                _ => format!("{fallback_prefix}_{header}"),
            })
            .collect::<Vec<_>>()
    };
    let sample = vec![
        sample_row("ada@example.com", "1234567890", "val"),
        sample_row("grace@example.com", "0987654321", "val2"),
    ];
    let profile = MaskingProfile {
        name: "preview".into(),
        schema: String::new(),
        table: String::new(),
        columns: columns
            .into_iter()
            .map(|column| ColumnMask {
                column,
                rule,
                replacement: "[masked]".into(),
                keep_prefix: 2,
                keep_suffix: 2,
            })
            .collect(),
        keyed,
        key_id: "local-dev".into(),
    };
    let key_material = if keyed { "db-pro-local-masking-key" } else { "" };
    preview_masking(&headers, &sample, &profile, key_material)
}

#[cfg(test)]
#[path = "tests/masking_tests.rs"]
mod tests;

