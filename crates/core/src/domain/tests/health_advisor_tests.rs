//! Tests for `../health_advisor.rs` (moved out of the source file).
use super::*;
use crate::domain::monitoring::{
    MonitorLock, MonitorSession, MonitoringSnapshot, RelationSizeStat, ServerSummary, StatStatementSort,
    StatStatementsSnapshot,
};

fn base_snapshot() -> MonitoringSnapshot {
    MonitoringSnapshot {
        connection_id: "c1".into(),
        driver: "Postgres".into(),
        sessions: Vec::new(),
        locks: Vec::new(),
        lock_error: None,
        relation_sizes: Vec::new(),
        server: None,
        local: None,
        workload: None,
        fetched_at_ms: 1_000,
        message: "ok".into(),
    }
}

#[test]
fn idle_in_transaction_emits_warning_with_evidence() {
    let mut snap = base_snapshot();
    snap.sessions.push(MonitorSession {
        backend_id: 9,
        database: Some("db".into()),
        username: Some("u".into()),
        application_name: None,
        client_addr: None,
        state: Some("idle in transaction".into()),
        wait_event_type: None,
        wait_event: None,
        query_text: Some("SELECT 1".into()),
        query_duration_ms: None,
        backend_start: None,
        xact_start: None,
        query_start: None,
        xact_age_ms: Some(120_000),
        backend_age_ms: Some(200_000),
        idle_in_transaction: true,
        is_current: false,
    });
    let report = analyze_health(&snap, None, &HealthAdvisorConfig::default(), 2_000);
    assert!(report.findings.iter().any(|f| f.id.starts_with("idle_in_xact:")));
    assert!(report.findings.iter().all(|f| !f.evidence.is_empty()));
}

#[test]
fn no_finding_without_evidence_thresholds() {
    let snap = base_snapshot();
    let report = analyze_health(&snap, None, &HealthAdvisorConfig::default(), 1_500);
    assert!(report.findings.is_empty() || report.findings.iter().all(|f| f.id == "stale_snapshot"));
}

#[test]
fn blocking_lock_and_saturation_and_dead_tuples() {
    let mut snap = base_snapshot();
    snap.locks.push(MonitorLock {
        locked_pid: 1,
        blocker_pid: Some(2),
        relation: Some("public.t".into()),
        lock_type: Some("relation".into()),
        mode: Some("AccessExclusiveLock".into()),
        granted: false,
        waiting_ms: Some(500),
    });
    snap.server = Some(ServerSummary {
        version: None,
        current_database: Some("db".into()),
        max_connections: Some(100),
        current_connections: Some(90),
        database_size_bytes: None,
    });
    snap.relation_sizes.push(RelationSizeStat {
        schema: "public".into(),
        name: "big".into(),
        kind: "table".into(),
        total_bytes: 50 * 1024 * 1024,
        table_bytes: 40 * 1024 * 1024,
        index_bytes: 10 * 1024 * 1024,
        seq_scan: Some(10_000),
        idx_scan: Some(100),
        n_live_tup: Some(1_000),
        n_dead_tup: Some(800),
    });
    let report = analyze_health(&snap, None, &HealthAdvisorConfig::default(), 1_500);
    assert!(report.findings.iter().any(|f| f.id.starts_with("blocking_lock:")));
    assert!(report.findings.iter().any(|f| f.id == "connection_saturation"));
    assert!(report.findings.iter().any(|f| f.id.starts_with("dead_tuples:")));
    assert!(report.findings.iter().any(|f| f.id.starts_with("seq_scan:")));
}

#[test]
fn missing_extension_finding_is_actionable() {
    let snap = base_snapshot();
    let workload = StatStatementsSnapshot {
        extension_present: false,
        extension_version: None,
        message: "not installed".into(),
        statements: Vec::new(),
        sort: StatStatementSort::TotalTime,
        fetched_at_ms: 1_000,
    };
    let report = analyze_health(&snap, Some(&workload), &HealthAdvisorConfig::default(), 1_500);
    assert!(report.findings.iter().any(|f| f.id == "missing_pg_stat_statements"));
}
