use super::*;

impl DbProApp {
    pub(super) fn draw_monitor_activity(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "MONITOR", self.theme);
        ui.add_space(SPACE_SM);

        let connected =
            self.connection.lifecycle.is_connected() && self.connection.lifecycle.active_connection_id().is_some();
        let driver = self.active_driver().to_owned();
        let name = self.active_connection_name().to_owned();

        egui::Frame {
            fill: self.theme.surface_elevated,
            inner_margin: egui::Margin::same(SPACE_MD),
            rounding: egui::Rounding::same(RADIUS_MD),
            stroke: egui::Stroke::new(STROKE_THIN, self.theme.border_subtle),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (dot, label) = if connected {
                    (self.theme.success, "Connected")
                } else {
                    (self.theme.text_muted, "Disconnected")
                };
                status_dot(ui, dot, connected, false, self.theme);
                ui.label(RichText::new(label).strong().color(self.theme.text_primary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if connected && secondary_button_with_icon(ui, Icon::RefreshCw, "Refresh", self.theme).clicked() {
                        self.request_monitoring_snapshot();
                    }
                    ui.checkbox(&mut self.management.monitoring.monitoring_poll, "Auto-refresh");
                });
            });
            ui.add_space(SPACE_SM);
            if connected {
                ui.label(
                    RichText::new(format!("{name} · {driver}"))
                        .small()
                        .color(self.theme.text_secondary),
                );
            } else {
                ui.label(
                    RichText::new("Connect from Explorer to monitor sessions.")
                        .small()
                        .color(self.theme.text_muted),
                );
            }
        });

        if connected && self.management.monitoring.monitoring_poll {
            let due = self
                .management
                .monitoring
                .monitoring_last_poll
                .map(|t| t.elapsed() >= std::time::Duration::from_secs(5))
                .unwrap_or(true);
            if due {
                self.request_monitoring_snapshot();
            }
        }

        ui.add_space(SPACE_MD);
        if let Some(error) = &self.management.monitoring.monitoring_error {
            ui.colored_label(self.theme.warning, error);
            ui.add_space(SPACE_SM);
        }

        self.draw_monitor_snapshot(ui, connected);

        self.draw_monitor_confirmations(ui);
    }

    fn draw_monitor_snapshot(&mut self, ui: &mut egui::Ui, connected: bool) {
        if let Some(snapshot) = self.management.monitoring.monitoring_snapshot.clone() {
            self.draw_monitor_health_and_local(ui, &snapshot);
            self.draw_monitor_sessions(ui, &snapshot);
            self.draw_monitor_server_stats(ui, &snapshot);
            self.draw_monitor_workload(ui, &snapshot);

            self.draw_audit_activity(ui);

            self.draw_fdw_activity(ui);

            self.draw_replication_activity(ui);

            self.draw_event_trigger_activity(ui);

            self.draw_pg_settings_activity(ui);

            maintenance_activity_view::draw_maintenance_activity(ui, self.theme, &mut self.management.monitoring);
        } else if connected {
            ui.label(
                RichText::new("Refresh to load sessions (or wait for auto-refresh).")
                    .small()
                    .color(self.theme.text_muted),
            );
        }
    }
    fn draw_monitor_health_and_local(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &db_pro_core::domain::monitoring::MonitoringSnapshot,
    ) {
        let health = db_pro_core::domain::health_advisor::analyze_health(
            snapshot,
            snapshot.workload.as_ref(),
            &db_pro_core::domain::health_advisor::HealthAdvisorConfig::default(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        );
        section_label(ui, "HEALTH ADVISOR", self.theme);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new(format!(
                "{} · snapshot @ {} ms",
                health.message, health.snapshot_fetched_at_ms
            ))
            .small()
            .color(self.theme.text_muted),
        );
        ui.label(
            RichText::new("Deterministic heuristics only — never auto-mutates the database.")
                .small()
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_SM);
        if health.findings.is_empty() {
            ui.label(
                RichText::new("No findings for the current snapshot.")
                    .small()
                    .color(self.theme.text_secondary),
            );
        } else {
            for finding in health.findings.iter().take(25) {
                let color = match finding.severity {
                    db_pro_core::domain::health_advisor::HealthSeverity::Critical => self.theme.danger,
                    db_pro_core::domain::health_advisor::HealthSeverity::Warning => self.theme.warning,
                    db_pro_core::domain::health_advisor::HealthSeverity::Info => self.theme.accent,
                };
                card_frame(self.theme).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        badge(ui, finding.severity.as_label(), color, self.theme.text_primary);
                        ui.label(RichText::new(&finding.title).strong().color(self.theme.text_primary));
                    });
                    ui.label(
                        RichText::new(format!("affected: {}", finding.affected))
                            .small()
                            .color(self.theme.text_secondary),
                    );
                    ui.label(
                        RichText::new(format!("evidence: {}", finding.evidence))
                            .small()
                            .monospace()
                            .color(self.theme.text_muted),
                    );
                    ui.label(
                        RichText::new(&finding.explanation)
                            .small()
                            .color(self.theme.text_secondary),
                    );
                    ui.label(
                        RichText::new(format!("suggest: {}", finding.suggested_action))
                            .small()
                            .color(self.theme.text_primary),
                    );
                });
                ui.add_space(SPACE_SM);
            }
        }
        ui.add_space(SPACE_MD);

        if let Some(local) = &snapshot.local {
            section_label(ui, "LOCAL STATE", self.theme);
            ui.add_space(SPACE_SM);
            ui.label(RichText::new(&local.note).small().color(self.theme.text_muted));
            ui.label(format!(
                "journal={} · pages={:?} · page_size={:?} · freelist={:?} · ~bytes={:?}",
                local.journal_mode.as_deref().unwrap_or("?"),
                local.page_count,
                local.page_size,
                local.freelist_count,
                local.file_size_bytes
            ));
        }
    }

    fn draw_monitor_sessions(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &db_pro_core::domain::monitoring::MonitoringSnapshot,
    ) {
        section_label(ui, "SESSIONS", self.theme);
        ui.add_space(SPACE_SM);
        ui.checkbox(
            &mut self.management.monitoring.monitoring_filter_active_only,
            "Active queries only",
        );
        ui.add_space(SPACE_SM);

        let idle_xacts = snapshot.idle_in_transaction_sessions();
        if !idle_xacts.is_empty() {
            section_label(ui, "IDLE IN TRANSACTION", self.theme);
            ui.add_space(SPACE_SM);
            for session in idle_xacts.into_iter().take(20) {
                ui.label(
                    RichText::new(format!(
                        "pid {} · xact_age={:?} ms · backend_age={:?} ms · {}",
                        session.backend_id,
                        session.xact_age_ms,
                        session.backend_age_ms,
                        session.username.as_deref().unwrap_or("?")
                    ))
                    .small()
                    .color(self.theme.warning),
                );
            }
            ui.add_space(SPACE_MD);
        }

        let sessions: Vec<_> = if self.management.monitoring.monitoring_filter_active_only {
            snapshot.active_queries().into_iter().cloned().collect()
        } else {
            snapshot.sessions.clone()
        };

        if sessions.is_empty() {
            ui.label(
                RichText::new(if snapshot.sessions.is_empty() {
                    snapshot.message.as_str()
                } else {
                    "No active queries right now."
                })
                .small()
                .color(self.theme.text_muted),
            );
        } else {
            for session in sessions {
                card_frame(self.theme).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("pid {}", session.backend_id))
                                .strong()
                                .monospace()
                                .color(self.theme.text_primary),
                        );
                        if session.is_current {
                            badge(ui, "current", self.theme.surface_active, self.theme.text_secondary);
                        }
                        ui.label(
                            RichText::new(session.state.clone().unwrap_or_else(|| "—".into()))
                                .small()
                                .color(self.theme.text_secondary),
                        );
                        if let Some(ms) = session.query_duration_ms {
                            ui.label(
                                RichText::new(format!("query {ms} ms"))
                                    .small()
                                    .color(self.theme.text_muted),
                            );
                        }
                        if let Some(ms) = session.xact_age_ms {
                            ui.label(
                                RichText::new(format!("xact {ms} ms"))
                                    .small()
                                    .color(self.theme.text_muted),
                            );
                        }
                        if session.idle_in_transaction {
                            badge(ui, "idle-in-xact", self.theme.warning, self.theme.text_primary);
                        }
                    });
                    ui.label(
                        RichText::new(format!(
                            "{} · {} · {}",
                            session.username.as_deref().unwrap_or("?"),
                            session.database.as_deref().unwrap_or("?"),
                            session.application_name.as_deref().unwrap_or("-")
                        ))
                        .small()
                        .color(self.theme.text_secondary),
                    );
                    if let Some(query) = &session.query_text {
                        let short = if query.len() > 120 {
                            format!("{}…", &query.chars().take(119).collect::<String>())
                        } else {
                            query.clone()
                        };
                        ui.label(RichText::new(short).monospace().small().color(self.theme.text_primary));
                        ui.horizontal(|ui| {
                            if ghost_button_with_icon(ui, Icon::FileCode2, "Open SQL", self.theme).clicked() {
                                self.set_active_query_text(query.clone());
                                self.workspace.active_tab = WorkspaceTab::Query;
                                self.workspace.activity = Activity::Explorer;
                            }
                            if !session.is_current
                                && secondary_button_with_icon(ui, Icon::Ban, "Cancel", self.theme).clicked()
                            {
                                if let Some(connection_id) =
                                    self.connection.lifecycle.active_connection_id().map(str::to_owned)
                                {
                                    let request_id = self.task_bridge.next_request_id();
                                    self.dispatch_command(self.management.monitoring.cancel_backend_command(
                                        request_id,
                                        connection_id,
                                        session.backend_id,
                                    ));
                                }
                            }
                            if !session.is_current && danger_button(ui, "Terminate", self.theme).clicked() {
                                self.management.monitoring.monitoring_terminate_confirm = Some(session.backend_id);
                            }
                        });
                    }
                });
                ui.add_space(SPACE_SM);
            }
        }
    }

    fn draw_monitor_server_stats(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &db_pro_core::domain::monitoring::MonitoringSnapshot,
    ) {
        if let Some(server) = &snapshot.server {
            ui.add_space(SPACE_MD);
            section_label(ui, "SERVER", self.theme);
            ui.add_space(SPACE_SM);
            if let Some(version) = &server.version {
                ui.label(RichText::new(version).small().color(self.theme.text_secondary));
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
                .color(self.theme.text_muted),
            );
        }

        let blocking = snapshot.blocking_locks();
        if !blocking.is_empty() {
            ui.add_space(SPACE_MD);
            section_label(ui, "LOCKS / BLOCKERS", self.theme);
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
                    .color(self.theme.text_secondary),
                );
            }
        }

        if !snapshot.relation_sizes.is_empty() {
            ui.add_space(SPACE_MD);
            section_label(ui, "SIZE / STATS", self.theme);
            ui.add_space(SPACE_SM);
            for rel in snapshot.relation_sizes.iter().take(20) {
                ui.label(
                    RichText::new(format!(
                        "{}.{} ({}) · {} · seq={:?} idx={:?} dead={:?}",
                        rel.schema,
                        rel.name,
                        rel.kind,
                        db_pro_core::domain::monitoring::format_bytes_exact(rel.total_bytes),
                        rel.seq_scan,
                        rel.idx_scan,
                        rel.n_dead_tup
                    ))
                    .small()
                    .color(self.theme.text_secondary),
                );
            }
        }
    }

    fn draw_monitor_workload(
        &mut self,
        ui: &mut egui::Ui,
        snapshot: &db_pro_core::domain::monitoring::MonitoringSnapshot,
    ) {
        if let Some(workload) = &snapshot.workload {
            ui.add_space(SPACE_MD);
            section_label(ui, "TOP QUERIES (pg_stat_statements)", self.theme);
            ui.add_space(SPACE_SM);
            if !workload.extension_present {
                ui.colored_label(self.theme.warning, &workload.message);
            } else {
                if let Some(ver) = &workload.extension_version {
                    ui.label(
                        RichText::new(format!("extension v{ver} · {}", workload.message))
                            .small()
                            .color(self.theme.text_muted),
                    );
                }
                ui.horizontal_wrapped(|ui| {
                    use db_pro_core::domain::monitoring::StatStatementSort;
                    for sort in [
                        StatStatementSort::TotalTime,
                        StatStatementSort::MeanTime,
                        StatStatementSort::Calls,
                        StatStatementSort::Rows,
                    ] {
                        let selected = self.management.monitoring.monitoring_stat_sort == sort;
                        if ui.selectable_label(selected, sort.as_label()).clicked() {
                            self.management.monitoring.monitoring_stat_sort = sort;
                            self.request_monitoring_workload();
                        }
                    }
                    if danger_button(ui, "Reset stats…", self.theme).clicked() {
                        self.management.monitoring.monitoring_reset_stats_confirm = true;
                    }
                });
                ui.add_space(SPACE_XS);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Filter").small().color(self.theme.text_muted));
                    ui.text_edit_singleline(&mut self.management.monitoring.monitoring_workload_filter);
                });
                let filter = self
                    .management
                    .monitoring
                    .monitoring_workload_filter
                    .to_ascii_lowercase();
                let prev_by_id: std::collections::HashMap<Option<i64>, f64> = self
                    .management
                    .monitoring
                    .monitoring_workload_prev
                    .as_ref()
                    .map(|prev| prev.statements.iter().map(|s| (s.queryid, s.total_time_ms)).collect())
                    .unwrap_or_default();
                let rows: Vec<_> = workload
                    .statements
                    .iter()
                    .filter(|s| {
                        filter.is_empty()
                            || s.query.to_ascii_lowercase().contains(&filter)
                            || s.database
                                .as_deref()
                                .unwrap_or("")
                                .to_ascii_lowercase()
                                .contains(&filter)
                            || s.username
                                .as_deref()
                                .unwrap_or("")
                                .to_ascii_lowercase()
                                .contains(&filter)
                    })
                    .take(40)
                    .collect();
                if rows.is_empty() {
                    ui.label(
                        RichText::new("No statements match the current filter.")
                            .small()
                            .color(self.theme.text_muted),
                    );
                }
                for stmt in rows {
                    card_frame(self.theme).show(ui, |ui| {
                        let delta = prev_by_id
                            .get(&stmt.queryid)
                            .map(|prev| stmt.total_time_ms - prev)
                            .filter(|d| d.abs() > 0.01);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "calls {} · total {:.1} ms · mean {:.1} ms · rows {}",
                                    stmt.calls, stmt.total_time_ms, stmt.mean_time_ms, stmt.rows
                                ))
                                .small()
                                .strong()
                                .color(self.theme.text_primary),
                            );
                            if let Some(delta) = delta {
                                ui.label(
                                    RichText::new(format!("Δ total {delta:+.1} ms"))
                                        .small()
                                        .color(self.theme.accent),
                                );
                            }
                        });
                        ui.label(
                            RichText::new(format!(
                                "{} · {} · shared hit/read {}/{} · temp r/w {}/{}",
                                stmt.username.as_deref().unwrap_or("?"),
                                stmt.database.as_deref().unwrap_or("?"),
                                stmt.shared_blks_hit,
                                stmt.shared_blks_read,
                                stmt.temp_blks_read,
                                stmt.temp_blks_written
                            ))
                            .small()
                            .color(self.theme.text_muted),
                        );
                        let short = if stmt.query.len() > 160 {
                            format!("{}…", &stmt.query.chars().take(159).collect::<String>())
                        } else {
                            stmt.query.clone()
                        };
                        ui.label(RichText::new(short).monospace().small().color(self.theme.text_primary));
                        if ghost_button_with_icon(ui, Icon::FileCode2, "Open SQL", self.theme).clicked() {
                            self.set_active_query_text(stmt.query.clone());
                            self.workspace.active_tab = WorkspaceTab::Query;
                            self.workspace.activity = Activity::Explorer;
                        }
                    });
                    ui.add_space(SPACE_SM);
                }
            }
        }
    }

    fn draw_monitor_confirmations(&mut self, ui: &mut egui::Ui) {
        if let Some(backend_id) = self.management.monitoring.monitoring_terminate_confirm {
            egui::Window::new("Terminate session?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(format!(
                        "Terminate PostgreSQL backend pid {backend_id}? This disconnects the client."
                    ));
                    ui.horizontal(|ui| {
                        if danger_button(ui, "Terminate", self.theme).clicked() {
                            if let Some(connection_id) =
                                self.connection.lifecycle.active_connection_id().map(str::to_owned)
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(self.management.monitoring.terminate_backend_command(
                                    request_id,
                                    connection_id,
                                    backend_id,
                                ));
                            }
                            self.management.monitoring.monitoring_terminate_confirm = None;
                        }
                        if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                            self.management.monitoring.monitoring_terminate_confirm = None;
                        }
                    });
                });
        }

        if let Some(action) = self.management.monitoring.monitoring_maintenance_confirm {
            egui::Window::new("Run maintenance?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(format!(
                        "Run {} on the active database? Long-running VACUUM can take locks.",
                        action.as_label()
                    ));
                    ui.horizontal(|ui| {
                        if danger_button(ui, action.as_label(), self.theme).clicked() {
                            if let Some(connection_id) =
                                self.connection.lifecycle.active_connection_id().map(str::to_owned)
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(self.management.monitoring.maintenance_command(
                                    request_id,
                                    connection_id,
                                    action,
                                ));
                            }
                            self.management.monitoring.monitoring_maintenance_confirm = None;
                        }
                        if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                            self.management.monitoring.monitoring_maintenance_confirm = None;
                        }
                    });
                });
        }

        if self.management.monitoring.monitoring_reset_stats_confirm {
            egui::Window::new("Reset pg_stat_statements?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label(
                        "This clears all accumulated statement statistics on the server. \
                     It is an administrative action and cannot be undone.",
                    );
                    ui.horizontal(|ui| {
                        if danger_button(ui, "Reset statistics", self.theme).clicked() {
                            if let Some(connection_id) =
                                self.connection.lifecycle.active_connection_id().map(str::to_owned)
                            {
                                let request_id = self.task_bridge.next_request_id();
                                self.dispatch_command(
                                    self.management
                                        .monitoring
                                        .reset_statements_command(request_id, connection_id),
                                );
                            }
                            self.management.monitoring.monitoring_reset_stats_confirm = false;
                        }
                        if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                            self.management.monitoring.monitoring_reset_stats_confirm = false;
                        }
                    });
                });
        }
    }

    fn request_monitoring_workload(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.monitoring.workload_command(request_id, connection_id));
    }

    fn request_monitoring_snapshot(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        self.management.monitoring.monitoring_last_poll = Some(std::time::Instant::now());
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.monitoring.snapshot_command(request_id, connection_id));
    }
}
