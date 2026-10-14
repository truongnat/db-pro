//! Tests for `../audit.rs` (moved out of the source file).
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
            // cc-scan:allow LINE_TOO_LONG — literal must not wrap
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
                    // cc-scan:allow LINE_TOO_LONG — literal must not wrap
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
