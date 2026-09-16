//! Deterministic database health advisor (#259).
//!
//! Findings are heuristics over canonical monitoring snapshots — never AI, never
//! automatic mutations.

use serde::{Deserialize, Serialize};

use super::monitoring::{MonitoringSnapshot, StatStatementsSnapshot};

/// Severity for a health finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthSeverity {
    Info,
    Warning,
    Critical,
}

impl HealthSeverity {
    pub fn as_label(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}

/// Where the UI should navigate when acting on a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthNavigateTo {
    MonitorSessions,
    MonitorLocks,
    MonitorWorkload,
    MonitorSizes,
    MonitorMaintenance,
    QueryEditor,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthFinding {
    pub id: String,
    pub severity: HealthSeverity,
    pub title: String,
    pub evidence: String,
    pub affected: String,
    pub explanation: String,
    pub suggested_action: String,
    pub navigate_to: HealthNavigateTo,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthReport {
    pub connection_id: String,
    pub driver: String,
    pub snapshot_fetched_at_ms: u64,
    pub findings: Vec<HealthFinding>,
    pub message: String,
}

/// Tunable thresholds (defaults match conservative production heuristics).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthAdvisorConfig {
    pub long_xact_ms: u64,
    pub idle_in_xact_ms: u64,
    pub connection_saturation_pct: u64,
    pub dead_tup_ratio: f64,
    pub seq_scan_ratio: f64,
    pub min_relation_bytes_for_seq_hint: u64,
    pub snapshot_stale_ms: u64,
}

impl Default for HealthAdvisorConfig {
    fn default() -> Self {
        Self {
            long_xact_ms: 5 * 60 * 1000,
            idle_in_xact_ms: 60 * 1000,
            connection_saturation_pct: 85,
            dead_tup_ratio: 0.2,
            seq_scan_ratio: 10.0,
            min_relation_bytes_for_seq_hint: 10 * 1024 * 1024,
            snapshot_stale_ms: 60_000,
        }
    }
}

/// Build a deterministic health report from a monitoring snapshot (+ optional workload).
pub fn analyze_health(
    snapshot: &MonitoringSnapshot,
    workload: Option<&StatStatementsSnapshot>,
    config: &HealthAdvisorConfig,
    now_ms: u64,
) -> HealthReport {
    let mut findings = Vec::new();

    if snapshot.fetched_at_ms > 0 && now_ms.saturating_sub(snapshot.fetched_at_ms) > config.snapshot_stale_ms {
        findings.push(HealthFinding {
            id: "stale_snapshot".into(),
            severity: HealthSeverity::Info,
            title: "Monitoring snapshot may be stale".into(),
            evidence: format!(
                "fetched_at_ms={} · age={} ms · threshold={} ms",
                snapshot.fetched_at_ms,
                now_ms.saturating_sub(snapshot.fetched_at_ms),
                config.snapshot_stale_ms
            ),
            affected: snapshot.connection_id.clone(),
            explanation: "Recommendations reflect the last refreshed monitoring snapshot.".into(),
            suggested_action: "Refresh Monitor before acting on findings.".into(),
            navigate_to: HealthNavigateTo::MonitorSessions,
        });
    }

    for session in &snapshot.sessions {
        if session.idle_in_transaction {
            let age = session.xact_age_ms.unwrap_or(0);
            if age >= config.idle_in_xact_ms {
                findings.push(HealthFinding {
                    id: format!("idle_in_xact:{}", session.backend_id),
                    severity: if age >= config.long_xact_ms {
                        HealthSeverity::Critical
                    } else {
                        HealthSeverity::Warning
                    },
                    title: "Idle in transaction".into(),
                    evidence: format!("xact_age_ms={age} · state={:?}", session.state),
                    affected: format!("pid {}", session.backend_id),
                    explanation: "An open transaction is idle and may hold locks or prevent vacuum.".into(),
                    suggested_action: "Inspect the session in Monitor; commit/rollback, or cancel only after review."
                        .into(),
                    navigate_to: HealthNavigateTo::MonitorSessions,
                });
            }
        } else if let Some(age) = session.xact_age_ms {
            if age >= config.long_xact_ms {
                findings.push(HealthFinding {
                    id: format!("long_xact:{}", session.backend_id),
                    severity: HealthSeverity::Warning,
                    title: "Long-running transaction".into(),
                    evidence: format!("xact_age_ms={age}"),
                    affected: format!("pid {}", session.backend_id),
                    explanation: "Long open transactions increase bloat risk and lock contention.".into(),
                    suggested_action: "Review the SQL in Monitor; do not terminate blindly.".into(),
                    navigate_to: HealthNavigateTo::MonitorSessions,
                });
            }
        }
    }

    for lock in snapshot.blocking_locks() {
        findings.push(HealthFinding {
            id: format!("blocking_lock:{}:{:?}", lock.locked_pid, lock.blocker_pid),
            severity: HealthSeverity::Warning,
            title: "Blocking lock relationship".into(),
            evidence: format!(
                "locked_pid={} · blocker={:?} · mode={:?} · relation={:?}",
                lock.locked_pid, lock.blocker_pid, lock.mode, lock.relation
            ),
            affected: format!("pid {}", lock.locked_pid),
            explanation: "A session is waiting on (or holding) a lock that blocks others.".into(),
            suggested_action: "Inspect blocker/waiter in Monitor Locks; never auto-terminate.".into(),
            navigate_to: HealthNavigateTo::MonitorLocks,
        });
    }

    if let Some(server) = &snapshot.server {
        if let (Some(cur), Some(max)) = (server.current_connections, server.max_connections) {
            if max > 0 {
                let pct = cur.saturating_mul(100) / max;
                if pct >= config.connection_saturation_pct {
                    findings.push(HealthFinding {
                        id: "connection_saturation".into(),
                        severity: if pct >= 95 {
                            HealthSeverity::Critical
                        } else {
                            HealthSeverity::Warning
                        },
                        title: "Connection saturation".into(),
                        evidence: format!("{cur}/{max} connections ({pct}%)"),
                        affected: server
                            .current_database
                            .clone()
                            .unwrap_or_else(|| snapshot.connection_id.clone()),
                        explanation: "Near max_connections increases risk of refused clients.".into(),
                        suggested_action: "Review idle sessions and pool sizing; avoid raising max blindly.".into(),
                        navigate_to: HealthNavigateTo::MonitorSessions,
                    });
                }
            }
        }
    }

    for rel in &snapshot.relation_sizes {
        if let (Some(dead), Some(live)) = (rel.n_dead_tup, rel.n_live_tup) {
            let denom = live.saturating_add(dead).max(1) as f64;
            let ratio = dead as f64 / denom;
            if dead > 100 && ratio >= config.dead_tup_ratio {
                findings.push(HealthFinding {
                    id: format!("dead_tuples:{}.{}", rel.schema, rel.name),
                    severity: HealthSeverity::Warning,
                    title: "High dead tuple ratio".into(),
                    evidence: format!("dead={dead} live={live} ratio={ratio:.2}"),
                    affected: format!("{}.{}", rel.schema, rel.name),
                    explanation: "Dead tuples suggest vacuum/analyze may be overdue.".into(),
                    suggested_action: "Preview ANALYZE/VACUUM from Monitor Maintenance after reviewing load.".into(),
                    navigate_to: HealthNavigateTo::MonitorMaintenance,
                });
            }
        }

        if let (Some(seq), Some(idx)) = (rel.seq_scan, rel.idx_scan) {
            if rel.total_bytes >= config.min_relation_bytes_for_seq_hint && idx > 0 {
                let ratio = seq as f64 / idx.max(1) as f64;
                if seq > 100 && ratio >= config.seq_scan_ratio {
                    findings.push(HealthFinding {
                        id: format!("seq_scan:{}.{}", rel.schema, rel.name),
                        severity: HealthSeverity::Info,
                        title: "High sequential scan ratio on large relation".into(),
                        evidence: format!(
                            "seq_scan={seq} idx_scan={idx} ratio={ratio:.1} size={}",
                            super::monitoring::format_bytes_exact(rel.total_bytes)
                        ),
                        affected: format!("{}.{}", rel.schema, rel.name),
                        explanation: "Hint only — large tables scanned sequentially may need index review.".into(),
                        suggested_action: "Inspect indexes / EXPLAIN in Query; do not auto-create indexes.".into(),
                        navigate_to: HealthNavigateTo::MonitorSizes,
                    });
                }
            }
        }
    }

    match workload {
        Some(w) if !w.extension_present => {
            findings.push(HealthFinding {
                id: "missing_pg_stat_statements".into(),
                severity: HealthSeverity::Info,
                title: "pg_stat_statements not installed".into(),
                evidence: w.message.clone(),
                affected: snapshot.connection_id.clone(),
                explanation: "Workload top-query analysis requires the extension (never auto-installed).".into(),
                suggested_action: "Ask a DBA to CREATE EXTENSION pg_stat_statements if appropriate.".into(),
                navigate_to: HealthNavigateTo::MonitorWorkload,
            });
        }
        Some(w) if w.statements.is_empty() && w.extension_present => {
            findings.push(HealthFinding {
                id: "empty_pg_stat_statements".into(),
                severity: HealthSeverity::Info,
                title: "No pg_stat_statements rows yet".into(),
                evidence: w.message.clone(),
                affected: snapshot.connection_id.clone(),
                explanation: "Stats may be freshly reset or the workload is idle.".into(),
                suggested_action: "Revisit after traffic; open Workload after refresh.".into(),
                navigate_to: HealthNavigateTo::MonitorWorkload,
            });
        }
        _ => {}
    }

    // Prefer critical → warning → info for display.
    findings.sort_by_key(|f| match f.severity {
        HealthSeverity::Critical => 0,
        HealthSeverity::Warning => 1,
        HealthSeverity::Info => 2,
    });

    let message = if findings.is_empty() {
        "No health findings from the current snapshot".into()
    } else {
        format!("{} finding(s)", findings.len())
    };

    HealthReport {
        connection_id: snapshot.connection_id.clone(),
        driver: snapshot.driver.clone(),
        snapshot_fetched_at_ms: snapshot.fetched_at_ms,
        findings,
        message,
    }
}

#[cfg(test)]
mod tests {
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
}
