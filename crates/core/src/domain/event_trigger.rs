//! PostgreSQL event trigger models (#231) — distinct from table triggers.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventTriggerInfo {
    pub name: String,
    pub event: String,
    /// O=origin, D=disabled, R=replica, A=always
    pub enabled: String,
    pub enabled_label: String,
    pub function_signature: String,
    pub owner: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct EventTriggerInventory {
    pub connection_id: String,
    pub triggers: Vec<EventTriggerInfo>,
    pub message: String,
}

pub fn enabled_label(code: &str) -> String {
    match code {
        "O" => "origin".into(),
        "D" => "disabled".into(),
        "R" => "replica".into(),
        "A" => "always".into(),
        other => other.to_owned(),
    }
}

pub fn validate_ident(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 63 {
        return Err("invalid identifier".into());
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("identifier must be [A-Za-z0-9_]".into());
    }
    if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return Err("identifier cannot start with a digit".into());
    }
    Ok(())
}

pub fn quote_ident(name: &str) -> Result<String, String> {
    validate_ident(name)?;
    Ok(format!("\"{name}\""))
}

pub fn validate_event(event: &str) -> Result<(), String> {
    match event.to_ascii_lowercase().as_str() {
        "ddl_command_start" | "ddl_command_end" | "sql_drop" | "table_rewrite" => Ok(()),
        _ => Err("event must be ddl_command_start, ddl_command_end, sql_drop, or table_rewrite".into()),
    }
}

pub fn validate_function_ref(func: &str) -> Result<(), String> {
    if func.is_empty() || func.len() > 128 {
        return Err("invalid function reference".into());
    }
    if !func
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '(' || c == ')' || c == ',')
    {
        return Err("function must be a simple schema.func or func() reference".into());
    }
    Ok(())
}

fn parse_tags(raw: &str) -> Result<Vec<String>, String> {
    let tags: Vec<String> = raw
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_uppercase())
        .collect();
    for tag in &tags {
        if !tag.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ' ') {
            return Err(format!("invalid tag `{tag}`"));
        }
    }
    Ok(tags)
}

pub fn preview_create_event_trigger(
    name: &str,
    event: &str,
    function_ref: &str,
    tags_csv: &str,
) -> Result<String, String> {
    let name = quote_ident(name)?;
    validate_event(event)?;
    validate_function_ref(function_ref)?;
    let event = event.to_ascii_lowercase();
    let tags = parse_tags(tags_csv)?;
    let when = if tags.is_empty() {
        String::new()
    } else {
        let quoted = tags.iter().map(|t| format!("'{t}'")).collect::<Vec<_>>().join(", ");
        format!(" WHEN TAG IN ({quoted})")
    };
    Ok(format!(
        "CREATE EVENT TRIGGER {name} ON {event}{when} EXECUTE FUNCTION {function_ref};"
    ))
}

pub fn preview_drop_event_trigger(name: &str) -> Result<String, String> {
    let name = quote_ident(name)?;
    Ok(format!("DROP EVENT TRIGGER IF EXISTS {name};"))
}

pub fn preview_alter_event_trigger(name: &str, mode: &str) -> Result<String, String> {
    let name = quote_ident(name)?;
    let clause = match mode.to_ascii_lowercase().as_str() {
        "enable" | "origin" => "ENABLE",
        "disable" => "DISABLE",
        "replica" => "ENABLE REPLICA",
        "always" => "ENABLE ALWAYS",
        _ => return Err("mode must be enable, disable, replica, or always".into()),
    };
    Ok(format!("ALTER EVENT TRIGGER {name} {clause};"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_with_tags() {
        let sql = preview_create_event_trigger(
            "audit_ddl",
            "ddl_command_end",
            "public.note_ddl()",
            "CREATE TABLE, ALTER TABLE",
        )
        .unwrap();
        assert!(sql.contains("CREATE EVENT TRIGGER \"audit_ddl\" ON ddl_command_end"));
        assert!(sql.contains("WHEN TAG IN ('CREATE TABLE', 'ALTER TABLE')"));
        assert!(sql.contains("EXECUTE FUNCTION public.note_ddl()"));
    }

    #[test]
    fn alter_and_drop() {
        assert_eq!(
            preview_alter_event_trigger("t1", "disable").unwrap(),
            "ALTER EVENT TRIGGER \"t1\" DISABLE;"
        );
        assert_eq!(
            preview_drop_event_trigger("t1").unwrap(),
            "DROP EVENT TRIGGER IF EXISTS \"t1\";"
        );
    }

    #[test]
    fn rejects_bad_event() {
        assert!(validate_event("after_insert").is_err());
    }
}
