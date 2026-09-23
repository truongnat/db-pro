//! Read-only monitoring snapshot presentation.

use super::*;

pub(super) fn draw_health_and_local(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    snapshot: &db_pro_core::domain::monitoring::MonitoringSnapshot,
) {
    let health = db_pro_core::domain::health_advisor::analyze_health(
        snapshot,
        snapshot.workload.as_ref(),
        &db_pro_core::domain::health_advisor::HealthAdvisorConfig::default(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0),
    );
    section_label(ui, "HEALTH ADVISOR", theme);
    ui.add_space(SPACE_SM);
    ui.label(
        RichText::new(format!(
            "{} · snapshot @ {} ms",
            health.message, health.snapshot_fetched_at_ms
        ))
        .small()
        .color(theme.text_muted),
    );
    ui.label(
        RichText::new("Deterministic heuristics only — never auto-mutates the database.")
            .small()
            .color(theme.text_muted),
    );
    ui.add_space(SPACE_SM);
    draw_health_findings(ui, theme, &health);
    ui.add_space(SPACE_MD);
    draw_local_state(ui, theme, snapshot);
}

fn draw_health_findings(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    health: &db_pro_core::domain::health_advisor::HealthReport,
) {
    if health.findings.is_empty() {
        ui.label(
            RichText::new("No findings for the current snapshot.")
                .small()
                .color(theme.text_secondary),
        );
        return;
    }
    for finding in health.findings.iter().take(25) {
        let color = match finding.severity {
            db_pro_core::domain::health_advisor::HealthSeverity::Critical => theme.danger,
            db_pro_core::domain::health_advisor::HealthSeverity::Warning => theme.warning,
            db_pro_core::domain::health_advisor::HealthSeverity::Info => theme.accent,
        };
        card_frame(theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                badge(ui, finding.severity.as_label(), color, theme.text_primary);
                ui.label(RichText::new(&finding.title).strong().color(theme.text_primary));
            });
            ui.label(
                RichText::new(format!("affected: {}", finding.affected))
                    .small()
                    .color(theme.text_secondary),
            );
            ui.label(
                RichText::new(format!("evidence: {}", finding.evidence))
                    .small()
                    .monospace()
                    .color(theme.text_muted),
            );
            ui.label(RichText::new(&finding.explanation).small().color(theme.text_secondary));
            ui.label(
                RichText::new(format!("suggest: {}", finding.suggested_action))
                    .small()
                    .color(theme.text_primary),
            );
        });
        ui.add_space(SPACE_SM);
    }
}

fn draw_local_state(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    snapshot: &db_pro_core::domain::monitoring::MonitoringSnapshot,
) {
    let Some(local) = &snapshot.local else {
        return;
    };
    section_label(ui, "LOCAL STATE", theme);
    ui.add_space(SPACE_SM);
    ui.label(RichText::new(&local.note).small().color(theme.text_muted));
    ui.label(format!(
        "journal={} · pages={:?} · page_size={:?} · freelist={:?} · ~bytes={:?}",
        local.journal_mode.as_deref().unwrap_or("?"),
        local.page_count,
        local.page_size,
        local.freelist_count,
        local.file_size_bytes
    ));
}

pub(super) fn draw_server_stats(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    snapshot: &db_pro_core::domain::monitoring::MonitoringSnapshot,
) {
    draw_server_summary(ui, theme, snapshot);
    draw_blocking_locks(ui, theme, snapshot);
    draw_relation_sizes(ui, theme, snapshot);
}

fn draw_server_summary(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    snapshot: &db_pro_core::domain::monitoring::MonitoringSnapshot,
) {
    let Some(server) = &snapshot.server else {
        return;
    };
    ui.add_space(SPACE_MD);
    section_label(ui, "SERVER", theme);
    ui.add_space(SPACE_SM);
    if let Some(version) = &server.version {
        ui.label(RichText::new(version).small().color(theme.text_secondary));
    }
    ui.label(
        RichText::new(format!(
            "db={} · connections={:?}/{:?} · size={}",
            server.current_database.as_deref().unwrap_or("?"),
            server.current_connections,
            server.max_connections,
            server
                .database_size_bytes
                .map(db_pro_core::domain::monitoring::format_bytes_exact)
                .unwrap_or_else(|| "—".into())
        ))
        .small()
        .color(theme.text_muted),
    );
}

fn draw_blocking_locks(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    snapshot: &db_pro_core::domain::monitoring::MonitoringSnapshot,
) {
    let blocking = snapshot.blocking_locks();
    if blocking.is_empty() {
        return;
    }
    ui.add_space(SPACE_MD);
    section_label(ui, "LOCKS / BLOCKERS", theme);
    ui.add_space(SPACE_SM);
    for lock in blocking.into_iter().take(30) {
        ui.label(
            RichText::new(format!(
                "pid {} {} · blocker={:?} · {} · {}",
                lock.locked_pid,
                if lock.granted { "granted" } else { "waiting" },
                lock.blocker_pid,
                lock.mode.as_deref().unwrap_or("?"),
                lock.relation.as_deref().unwrap_or("?")
            ))
            .small()
            .monospace()
            .color(theme.text_secondary),
        );
    }
}

fn draw_relation_sizes(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    snapshot: &db_pro_core::domain::monitoring::MonitoringSnapshot,
) {
    if snapshot.relation_sizes.is_empty() {
        return;
    }
    ui.add_space(SPACE_MD);
    section_label(ui, "SIZE / STATS", theme);
    ui.add_space(SPACE_SM);
    for relation in snapshot.relation_sizes.iter().take(20) {
        ui.label(
            RichText::new(format!(
                "{}.{} ({}) · {} · seq={:?} idx={:?} dead={:?}",
                relation.schema,
                relation.name,
                relation.kind,
                db_pro_core::domain::monitoring::format_bytes_exact(relation.total_bytes),
                relation.seq_scan,
                relation.idx_scan,
                relation.n_dead_tup
            ))
            .small()
            .color(theme.text_secondary),
        );
    }
}
