//! Database activity / audit event model (#258).
//!
//! Source contract (PostgreSQL-first, explicit configuration only):
//! - Connection tag `audit:csvlog=<path>` points at a readable PostgreSQL CSV
//!   log file produced when `logging_collector=on` and `log_destination`
//!   includes `csvlog`. DB Pro never enables server logging automatically.
//! - Optional extension probe for `pgaudit` provides setup guidance only;
//!   pgaudit itself writes into the same server log stream.
//! - Application/agent logs are a separate surface and must not be mixed here.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Connection tag prefix for the configured CSV log path.
pub const AUDIT_CSVLOG_TAG_PREFIX: &str = "audit:csvlog=";

/// How the audit source is currently available.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSourceKind {
    /// User-configured PostgreSQL CSV log file.
    PostgresCsvLog,
    /// No readable source; UI must show `guidance`.
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditSourceStatus {
    pub kind: AuditSourceKind,
    pub path: Option<String>,
    /// True when `CREATE EXTENSION pgaudit` is present (guidance only).
    pub pgaudit_extension_present: bool,
    pub guidance: String,
}

impl AuditSourceStatus {
    pub fn unavailable(guidance: impl Into<String>) -> Self {
        Self {
            kind: AuditSourceKind::Unavailable,
            path: None,
            pgaudit_extension_present: false,
            guidance: guidance.into(),
        }
    }
}

/// One normalized audit / activity event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Stable local identity for bookmark/export (not a server id).
    pub id: String,
    pub timestamp: Option<String>,
    pub username: Option<String>,
    pub database: Option<String>,
    pub severity: Option<String>,
    pub command_tag: Option<String>,
    pub application_name: Option<String>,
    pub client_addr: Option<String>,
    pub message: String,
    pub query: Option<String>,
    pub detail: Option<String>,
    /// True when message/query were redacted before leaving the service.
    pub redacted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuditFilter {
    pub text: String,
    pub database: String,
    pub username: String,
    pub severity: String,
    pub command_tag: String,
}

impl AuditFilter {
    pub fn matches(&self, event: &AuditEvent) -> bool {
        let contains = |needle: &str, hay: Option<&str>, fallback: &str| {
            let n = needle.trim();
            if n.is_empty() {
                return true;
            }
            hay.unwrap_or(fallback)
                .to_ascii_lowercase()
                .contains(&n.to_ascii_lowercase())
        };
        if !contains(&self.database, event.database.as_deref(), "") {
            return false;
        }
        if !contains(&self.username, event.username.as_deref(), "") {
            return false;
        }
        if !contains(&self.severity, event.severity.as_deref(), "") {
            return false;
        }
        if !contains(&self.command_tag, event.command_tag.as_deref(), "") {
            return false;
        }
        let t = self.text.trim();
        if t.is_empty() {
            return true;
        }
        let t = t.to_ascii_lowercase();
        event.message.to_ascii_lowercase().contains(&t)
            || event.query.as_deref().unwrap_or("").to_ascii_lowercase().contains(&t)
            || event.detail.as_deref().unwrap_or("").to_ascii_lowercase().contains(&t)
            || event
                .application_name
                .as_deref()
                .unwrap_or("")
                .to_ascii_lowercase()
                .contains(&t)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditPage {
    pub source: AuditSourceStatus,
    pub events: Vec<AuditEvent>,
    /// Bytes offset from end used for this page (for incremental follow-ups).
    pub scanned_bytes: u64,
    pub truncated: bool,
    pub export_warning: String,
}

/// Resolve `audit:csvlog=` from connection tags.
pub fn csvlog_path_from_tags(tags: &[String]) -> Option<PathBuf> {
    tags.iter().find_map(|tag| {
        let trimmed = tag.trim();
        trimmed
            .strip_prefix(AUDIT_CSVLOG_TAG_PREFIX)
            .or_else(|| trimmed.strip_prefix("audit:csvlog:"))
            .map(|p| PathBuf::from(p.trim()))
            .filter(|p| !p.as_os_str().is_empty())
    })
}

/// Redact obvious credential material from free-form log text.
/// Deterministic, dependency-free heuristics (not a full secret scanner).
pub fn redact_secrets(input: &str) -> (String, bool) {
    let mut out = input.to_string();
    let mut changed = false;
    for key in ["password=", "pwd=", "passphrase=", "PASSWORD=", "PWD="] {
        if let Some(idx) = find_ci(&out, key) {
            let start = idx + key.len();
            let end = out[start..]
                .find(|c: char| c.is_whitespace() || c == ',' || c == ';')
                .map(|i| start + i)
                .unwrap_or(out.len());
            if end > start {
                out.replace_range(start..end, "***");
                changed = true;
            }
        }
    }
    if let Some(idx) = find_ci(&out, "bearer ") {
        let start = idx + "bearer ".len();
        let end = out[start..]
            .find(char::is_whitespace)
            .map(|i| start + i)
            .unwrap_or(out.len());
        if end > start {
            out.replace_range(start..end, "***");
            changed = true;
        }
    }
    // postgresql://user:secret@host → postgresql://user:***@host
    for scheme in ["postgresql://", "postgres://", "mysql://"] {
        if let Some(scheme_at) = find_ci(&out, scheme) {
            let after = scheme_at + scheme.len();
            if let Some(colon) = out[after..].find(':') {
                let user_end = after + colon;
                let secret_start = user_end + 1;
                if let Some(at) = out[secret_start..].find('@') {
                    let secret_end = secret_start + at;
                    if secret_end > secret_start {
                        out.replace_range(secret_start..secret_end, "***");
                        changed = true;
                    }
                }
            }
        }
    }
    (out, changed)
}

fn find_ci(haystack: &str, needle: &str) -> Option<usize> {
    haystack.to_ascii_lowercase().find(&needle.to_ascii_lowercase())
}

fn field(cols: &[String], idx: usize) -> Option<String> {
    cols.get(idx).map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Parse one PostgreSQL CSV log line into an [`AuditEvent`].
pub fn parse_postgres_csv_line(line: &str) -> Option<AuditEvent> {
    let cols = parse_csv_line(line)?;
    if cols.len() < 14 {
        return None;
    }
    let timestamp = field(&cols, 0);
    let username = field(&cols, 1);
    let database = field(&cols, 2);
    let pid = field(&cols, 3).unwrap_or_default();
    let client_addr = field(&cols, 4);
    let session_line = field(&cols, 6).unwrap_or_default();
    let command_tag = field(&cols, 7);
    let severity = field(&cols, 11);
    let raw_message = field(&cols, 13).unwrap_or_default();
    let detail = field(&cols, 14);
    let raw_query = field(&cols, 19);
    let application_name = field(&cols, 22);

    let (message, msg_redacted) = redact_secrets(&raw_message);
    let (query, query_redacted) = match raw_query {
        Some(q) => {
            let (r, c) = redact_secrets(&q);
            (Some(r), c)
        }
        None => (None, false),
    };
    let (detail, detail_redacted) = match detail {
        Some(d) => {
            let (r, c) = redact_secrets(&d);
            (Some(r), c)
        }
        None => (None, false),
    };
    let redacted = msg_redacted || query_redacted || detail_redacted;
    let id = format!(
        "{}|{}|{}|{}",
        timestamp.as_deref().unwrap_or(""),
        pid,
        session_line,
        message.chars().take(48).collect::<String>()
    );
    Some(AuditEvent {
        id,
        timestamp,
        username,
        database,
        severity,
        command_tag,
        application_name,
        client_addr,
        message,
        query,
        detail,
        redacted,
    })
}

/// Minimal CSV line splitter that understands double-quoted fields.
pub fn parse_csv_line(line: &str) -> Option<Vec<String>> {
    let mut cols = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes {
                    if chars.peek() == Some(&'"') {
                        cur.push('"');
                        chars.next();
                    } else {
                        in_quotes = false;
                    }
                } else {
                    in_quotes = true;
                }
            }
            ',' if !in_quotes => {
                cols.push(std::mem::take(&mut cur));
            }
            _ => cur.push(ch),
        }
    }
    if in_quotes {
        return None;
    }
    cols.push(cur);
    Some(cols)
}

/// Read up to `limit` newest matching events by scanning backward from EOF.
/// Never materializes the entire file into memory.
pub fn read_csvlog_page(
    path: &Path,
    filter: &AuditFilter,
    limit: usize,
    max_scan_bytes: u64,
) -> Result<AuditPage, String> {
    let mut file = File::open(path).map_err(|e| format!("cannot open audit csvlog: {e}"))?;
    let len = file
        .seek(SeekFrom::End(0))
        .map_err(|e| format!("cannot seek audit csvlog: {e}"))?;
    let scan = len.min(max_scan_bytes.max(4_096));
    let start = len.saturating_sub(scan);
    file.seek(SeekFrom::Start(start))
        .map_err(|e| format!("cannot seek audit csvlog start: {e}"))?;

    let mut buf = Vec::with_capacity(scan as usize);
    file.take(scan)
        .read_to_end(&mut buf)
        .map_err(|e| format!("cannot read audit csvlog: {e}"))?;

    // If we started mid-line, drop the partial first line.
    let text = String::from_utf8_lossy(&buf);
    let body = if start > 0 {
        text.split_once('\n').map(|(_, rest)| rest).unwrap_or("")
    } else {
        text.as_ref()
    };

    let mut matched = Vec::new();
    for line in body.lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            continue;
        }
        if let Some(event) = parse_postgres_csv_line(line) {
            if filter.matches(&event) {
                matched.push(event);
            }
        }
    }

    let truncated = start > 0 || matched.len() > limit;
    let events = if matched.len() > limit {
        matched.split_off(matched.len() - limit)
    } else {
        matched
    };

    Ok(AuditPage {
        source: AuditSourceStatus {
            kind: AuditSourceKind::PostgresCsvLog,
            path: Some(path.display().to_string()),
            pgaudit_extension_present: false,
            guidance: "Reading configured PostgreSQL CSV log (newest matching rows, bounded scan).".into(),
        },
        events,
        scanned_bytes: scan,
        truncated,
        export_warning: "Exported audit rows may still contain sensitive SQL literals. Review before sharing.".into(),
    })
}

/// Export selected events as CSV with redacted message/query fields.
pub fn export_events_csv(events: &[AuditEvent]) -> String {
    let mut out =
        String::from("timestamp,username,database,severity,command_tag,application_name,message,query,redacted\n");
    for e in events {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            csv_escape(e.timestamp.as_deref().unwrap_or("")),
            csv_escape(e.username.as_deref().unwrap_or("")),
            csv_escape(e.database.as_deref().unwrap_or("")),
            csv_escape(e.severity.as_deref().unwrap_or("")),
            csv_escape(e.command_tag.as_deref().unwrap_or("")),
            csv_escape(e.application_name.as_deref().unwrap_or("")),
            csv_escape(&e.message),
            csv_escape(e.query.as_deref().unwrap_or("")),
            e.redacted,
        ));
    }
    out
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Default setup guidance when no source is configured.
pub fn unavailable_guidance(driver_label: &str, pgaudit_present: bool) -> String {
    let mut msg = format!(
        "No database audit source configured for {driver_label}. \
         DB Pro does not enable server logging automatically. \
         To view PostgreSQL CSV logs, set connection tag `{AUDIT_CSVLOG_TAG_PREFIX}/path/to/logfile.csv` \
         after enabling `logging_collector` and `csvlog` on the server."
    );
    if pgaudit_present {
        msg.push_str(
            " Extension `pgaudit` is installed; its records still land in the server log stream — \
             point the csvlog tag at that file.",
        );
    } else {
        msg.push_str(" Optional: install `pgaudit` (DBA-managed) for richer audit detail in the same log.");
    }
    msg
}

/// Validate that a path looks like a readable file (exists + is file).
pub fn validate_csvlog_path(path: &Path) -> Result<(), String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("audit csvlog not readable: {e}"))?;
    if !meta.is_file() {
        return Err("audit csvlog path is not a file".into());
    }
    let _ = File::open(path).map_err(|e| format!("audit csvlog cannot be opened: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn redact_password_and_uri() {
        let (out, changed) = redact_secrets("password=s3cret postgresql://u:hunter2@host/db");
        assert!(changed);
        assert!(!out.contains("s3cret"));
        assert!(!out.contains("hunter2"));
        assert!(out.contains("***"));
    }

    #[test]
    fn parse_csv_quoted_commas() {
        let cols = parse_csv_line(
            r#"2024-01-01 00:00:00.000 UTC,"u","db",1,"127.0.0.1",s,1,"SELECT",t,v,0,"LOG","00000","hello, world",,,,,,,,,,"app","client",,"0""#,
        )
        .unwrap();
        assert_eq!(cols[13], "hello, world");
    }

    #[test]
    fn page_reads_newest_without_full_materialization() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("db-pro-audit-{}.csv", std::process::id()));
        {
            let mut tmp = File::create(&path).unwrap();
            for i in 0..20 {
                writeln!(
                    tmp,
                    r#"2024-01-01 00:00:{i:02}.000 UTC,"u","db",{i},"127.0.0.1",s,{i},"SELECT",t,v,0,"LOG","00000","msg {i}",,,,,,,,,,"app","client",,"0""#
                )
                .unwrap();
            }
            tmp.flush().unwrap();
        }
        let page = read_csvlog_page(&path, &AuditFilter::default(), 5, 64 * 1024).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(page.events.len(), 5);
        assert!(page.events.last().unwrap().message.contains("msg 19"));
        assert_eq!(page.source.kind, AuditSourceKind::PostgresCsvLog);
    }

    #[test]
    fn filter_and_tag_parse() {
        let path = csvlog_path_from_tags(&["favorite".into(), "audit:csvlog=/var/log/pg/log.csv".into()]);
        assert_eq!(path.unwrap(), PathBuf::from("/var/log/pg/log.csv"));
        let ev = AuditEvent {
            id: "1".into(),
            timestamp: None,
            username: Some("alice".into()),
            database: Some("app".into()),
            severity: Some("LOG".into()),
            command_tag: Some("SELECT".into()),
            application_name: None,
            client_addr: None,
            message: "ok".into(),
            query: Some("select 1".into()),
            detail: None,
            redacted: false,
        };
        assert!(AuditFilter {
            username: "ali".into(),
            ..AuditFilter::default()
        }
        .matches(&ev));
        assert!(!AuditFilter {
            database: "other".into(),
            ..AuditFilter::default()
        }
        .matches(&ev));
    }

    #[test]
    fn export_csv_includes_header() {
        let csv = export_events_csv(&[]);
        assert!(csv.starts_with("timestamp,"));
    }
}
