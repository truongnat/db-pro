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
                    ui.checkbox(&mut self.monitoring.monitoring_poll, "Auto-refresh");
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

        if connected && self.monitoring.monitoring_poll {
            let due = self
                .monitoring
                .monitoring_last_poll
                .map(|t| t.elapsed() >= std::time::Duration::from_secs(5))
                .unwrap_or(true);
            if due {
                self.request_monitoring_snapshot();
            }
        }

        ui.add_space(SPACE_MD);
        if let Some(error) = &self.monitoring.monitoring_error {
            ui.colored_label(self.theme.warning, error);
            ui.add_space(SPACE_SM);
        }

        if let Some(snapshot) = self.monitoring.monitoring_snapshot.clone() {
            let health = db_pro_core::domain::health_advisor::analyze_health(
                &snapshot,
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

            section_label(ui, "SESSIONS", self.theme);
            ui.add_space(SPACE_SM);
            ui.checkbox(
                &mut self.monitoring.monitoring_filter_active_only,
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

            let sessions: Vec<_> = if self.monitoring.monitoring_filter_active_only {
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
                                        self.dispatch_command(self.monitoring.cancel_backend_command(
                                            request_id,
                                            connection_id,
                                            session.backend_id,
                                        ));
                                    }
                                }
                                if !session.is_current && danger_button(ui, "Terminate", self.theme).clicked() {
                                    self.monitoring.monitoring_terminate_confirm = Some(session.backend_id);
                                }
                            });
                        }
                    });
                    ui.add_space(SPACE_SM);
                }
            }

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
                            let selected = self.monitoring.monitoring_stat_sort == sort;
                            if ui.selectable_label(selected, sort.as_label()).clicked() {
                                self.monitoring.monitoring_stat_sort = sort;
                                self.request_monitoring_workload();
                            }
                        }
                        if danger_button(ui, "Reset stats…", self.theme).clicked() {
                            self.monitoring.monitoring_reset_stats_confirm = true;
                        }
                    });
                    ui.add_space(SPACE_XS);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Filter").small().color(self.theme.text_muted));
                        ui.text_edit_singleline(&mut self.monitoring.monitoring_workload_filter);
                    });
                    let filter = self.monitoring.monitoring_workload_filter.to_ascii_lowercase();
                    let prev_by_id: std::collections::HashMap<Option<i64>, f64> = self
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

            ui.add_space(SPACE_MD);
            section_label(ui, "DATABASE AUDIT / ACTIVITY LOG", self.theme);
            ui.add_space(SPACE_SM);
            ui.label(
                RichText::new(
                    "Database audit (server CSV log) — distinct from application Diagnostics. \
                     Configure tag audit:csvlog=/path/to/logfile.csv. Logging is never auto-enabled.",
                )
                .small()
                .color(self.theme.text_muted),
            );
            ui.horizontal(|ui| {
                if secondary_button_with_icon(ui, Icon::RefreshCw, "Load audit page", self.theme).clicked() {
                    self.request_audit_page();
                }
                if secondary_button_with_icon(ui, Icon::Download, "Export selected", self.theme).clicked() {
                    self.export_selected_audit_events();
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Text").small().color(self.theme.text_muted));
                ui.add(
                    egui::TextEdit::singleline(&mut self.audit.audit_filter_text)
                        .desired_width(120.0)
                        .hint_text("message/query"),
                );
                ui.label(RichText::new("DB").small().color(self.theme.text_muted));
                ui.add(egui::TextEdit::singleline(&mut self.audit.audit_filter_database).desired_width(80.0));
                ui.label(RichText::new("User").small().color(self.theme.text_muted));
                ui.add(egui::TextEdit::singleline(&mut self.audit.audit_filter_username).desired_width(80.0));
                ui.label(RichText::new("Severity").small().color(self.theme.text_muted));
                ui.add(egui::TextEdit::singleline(&mut self.audit.audit_filter_severity).desired_width(60.0));
            });
            if let Some(error) = &self.audit.audit_error {
                ui.colored_label(self.theme.danger, error);
            }
            if let Some(page) = self.audit.audit_page.clone() {
                ui.label(
                    RichText::new(format!(
                        "{} · scanned {} bytes · truncated={}",
                        page.source.guidance, page.scanned_bytes, page.truncated
                    ))
                    .small()
                    .color(self.theme.text_secondary),
                );
                if let Some(path) = &page.source.path {
                    ui.label(RichText::new(format!("source: {path}")).small().monospace());
                }
                if page.source.kind == db_pro_core::domain::audit::AuditSourceKind::Unavailable {
                    ui.colored_label(self.theme.warning, &page.source.guidance);
                }
                for event in page.events.iter().take(80) {
                    let bookmarked = self.audit.audit_bookmarks.contains(&event.id);
                    let selected = self.audit.audit_selected.contains(&event.id);
                    card_frame(self.theme).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut sel = selected;
                            if ui.checkbox(&mut sel, "").changed() {
                                if sel {
                                    self.audit.audit_selected.insert(event.id.clone());
                                } else {
                                    self.audit.audit_selected.remove(&event.id);
                                }
                            }
                            ui.label(
                                RichText::new(format!(
                                    "{} · {} · {} · {}",
                                    event.timestamp.as_deref().unwrap_or("-"),
                                    event.severity.as_deref().unwrap_or("-"),
                                    event.username.as_deref().unwrap_or("-"),
                                    event.database.as_deref().unwrap_or("-"),
                                ))
                                .small()
                                .strong(),
                            );
                            if event.redacted {
                                ui.colored_label(self.theme.warning, "redacted");
                            }
                            if bookmarked {
                                ui.colored_label(self.theme.accent, "★");
                            }
                        });
                        ui.label(RichText::new(&event.message).small().color(self.theme.text_primary));
                        if let Some(q) = &event.query {
                            let short = if q.len() > 160 {
                                format!("{}…", &q.chars().take(159).collect::<String>())
                            } else {
                                q.clone()
                            };
                            ui.label(RichText::new(short).monospace().small().color(self.theme.text_muted));
                            if ghost_button_with_icon(ui, Icon::FileCode2, "Open SQL", self.theme).clicked() {
                                self.set_active_query_text(q.clone());
                                self.workspace.active_tab = WorkspaceTab::Query;
                                self.workspace.activity = Activity::Explorer;
                            }
                        }
                        ui.horizontal(|ui| {
                            let label = if bookmarked { "Unbookmark" } else { "Bookmark" };
                            if ghost_button_with_icon(ui, Icon::Bookmark, label, self.theme).clicked() {
                                if bookmarked {
                                    self.audit.audit_bookmarks.remove(&event.id);
                                } else {
                                    self.audit.audit_bookmarks.insert(event.id.clone());
                                }
                            }
                        });
                    });
                    ui.add_space(SPACE_XS);
                }
                if let Some(preview) = &self.audit.audit_export_preview {
                    ui.label(
                        RichText::new(page.export_warning.clone())
                            .small()
                            .color(self.theme.warning),
                    );
                    ui.label(
                        RichText::new(preview.chars().take(400).collect::<String>())
                            .monospace()
                            .small()
                            .color(self.theme.text_muted),
                    );
                }
            }

            ui.add_space(SPACE_MD);
            section_label(ui, "FOREIGN DATA (FDW)", self.theme);
            ui.add_space(SPACE_SM);
            ui.label(
                RichText::new("PostgreSQL-only · passwords/options redacted · CREATE EXTENSION never auto-run")
                    .small()
                    .color(self.theme.text_muted),
            );
            if secondary_button_with_icon(ui, Icon::RefreshCw, "Load FDW inventory", self.theme).clicked() {
                self.request_fdw_inventory();
            }
            if let Some(error) = &self.fdw.fdw_error {
                ui.colored_label(self.theme.danger, error);
            }
            if let Some(inv) = self.fdw.fdw_inventory.clone() {
                ui.label(RichText::new(&inv.message).small().color(self.theme.text_secondary));
                if let Some(hint) = &inv.extension_hint {
                    ui.colored_label(self.theme.warning, hint);
                }
                for w in inv.wrappers.iter().take(20) {
                    ui.label(
                        RichText::new(format!("wrapper {} · handler={:?}", w.name, w.handler))
                            .small()
                            .monospace(),
                    );
                }
                for s in inv.servers.iter().take(30) {
                    card_frame(self.theme).show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("server {} · fdw={}", s.name, s.fdw_name))
                                .strong()
                                .monospace(),
                        );
                        let opts = s
                            .options
                            .iter()
                            .map(|o| format!("{}={}", o.key, o.display_value()))
                            .collect::<Vec<_>>()
                            .join(", ");
                        if !opts.is_empty() {
                            ui.label(RichText::new(opts).small().color(self.theme.text_muted));
                        }
                        ui.horizontal(|ui| {
                            if ghost_button_with_icon(ui, Icon::FileCode2, "Preview DROP", self.theme).clicked() {
                                // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Drop
                                self.fdw.fdw_ddl_preview =
                                    db_pro_core::domain::fdw::preview_drop_server(&s.name, true).ok();
                            }
                            if danger_button(ui, "Drop…", self.theme).clicked() {
                                self.fdw.fdw_drop_confirm = Some(s.name.clone());
                            }
                        });
                    });
                    ui.add_space(SPACE_XS);
                }
                for m in inv.user_mappings.iter().take(30) {
                    let opts = m
                        .options
                        .iter()
                        .map(|o| format!("{}={}", o.key, o.display_value()))
                        .collect::<Vec<_>>()
                        .join(", ");
                    ui.label(
                        RichText::new(format!("mapping {}@{} · {}", m.user_name, m.server_name, opts))
                            .small()
                            .monospace()
                            .color(self.theme.text_secondary),
                    );
                }
                for t in inv.foreign_tables.iter().take(40) {
                    ui.label(
                        RichText::new(format!("foreign {}.{} → {}", t.schema, t.name, t.server_name))
                            .small()
                            .monospace(),
                    );
                }
            }

            ui.add_space(SPACE_SM);
            ui.label(RichText::new("Create foreign server").small().strong());
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.fdw.fdw_create_name).hint_text("server name"));
                ui.add(egui::TextEdit::singleline(&mut self.fdw.fdw_create_wrapper).hint_text("fdw"));
            });
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.fdw.fdw_create_host).hint_text("host"));
                ui.add(egui::TextEdit::singleline(&mut self.fdw.fdw_create_dbname).hint_text("dbname"));
                ui.add(egui::TextEdit::singleline(&mut self.fdw.fdw_create_port).hint_text("port"));
            });
            ui.horizontal(|ui| {
                if ghost_button_with_icon(ui, Icon::FileCode2, "Preview CREATE", self.theme).clicked() {
                    // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Create
                    self.fdw.fdw_ddl_preview = db_pro_core::domain::fdw::preview_create_server(
                        &self.fdw.fdw_create_name,
                        &self.fdw.fdw_create_wrapper,
                        &self.fdw.fdw_create_host,
                        &self.fdw.fdw_create_dbname,
                        &self.fdw.fdw_create_port,
                    )
                    .ok();
                }
                if secondary_button(ui, "Create (confirm)", self.theme).clicked() {
                    self.create_fdw_server_confirmed();
                }
            });

            if let Some(preview) = self.fdw.fdw_ddl_preview.clone() {
                egui::Window::new("FDW DDL preview")
                    .collapsible(false)
                    .resizable(true)
                    .default_width(480.0)
                    .show(ui.ctx(), |ui| {
                        ui.label(RichText::new(preview).monospace());
                        if secondary_button(ui, "Close", self.theme).clicked() {
                            self.fdw.fdw_ddl_preview = None;
                        }
                    });
            }
            if let Some(name) = self.fdw.fdw_drop_confirm.clone() {
                egui::Window::new("Drop foreign server?")
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.label(format!(
                            "Drop server `{name}` CASCADE? This removes dependent foreign tables/mappings."
                        ));
                        ui.horizontal(|ui| {
                            if danger_button(ui, "Drop CASCADE", self.theme).clicked() {
                                self.drop_fdw_server_confirmed(&name, true);
                            }
                            if secondary_button(ui, "Cancel", self.theme).clicked() {
                                self.fdw.fdw_drop_confirm = None;
                            }
                        });
                    });
            }

            ui.add_space(SPACE_MD);
            section_label(ui, "LOGICAL REPLICATION", self.theme);
            ui.add_space(SPACE_SM);
            ui.label(
                RichText::new(
                    "PostgreSQL-only · subscription conninfo redacted · CREATE SUBSCRIPTION not offered (secrets)",
                )
                .small()
                .color(self.theme.text_muted),
            );
            if secondary_button_with_icon(ui, Icon::RefreshCw, "Load replication inventory", self.theme).clicked() {
                self.request_replication_inventory();
            }
            if let Some(error) = &self.replication.replication_error {
                ui.colored_label(self.theme.danger, error);
            }
            if let Some(inv) = self.replication.replication_inventory.clone() {
                ui.label(RichText::new(&inv.message).small().color(self.theme.text_secondary));
                for pub_info in inv.publications.iter().take(40) {
                    card_frame(self.theme).show(ui, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "publication {} · all_tables={} · owner={:?}",
                                pub_info.name, pub_info.all_tables, pub_info.owner
                            ))
                            .strong()
                            .monospace(),
                        );
                        if !pub_info.tables.is_empty() {
                            ui.label(
                                RichText::new(format!("tables: {}", pub_info.tables.join(", ")))
                                    .small()
                                    .color(self.theme.text_muted),
                            );
                        }
                        ui.horizontal(|ui| {
                            if ghost_button_with_icon(ui, Icon::FileCode2, "Preview DROP", self.theme).clicked() {
                                // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Drop
                                self.replication.replication_ddl_preview =
                                    db_pro_core::domain::replication::preview_drop_publication(&pub_info.name).ok();
                            }
                            if danger_button(ui, "Drop…", self.theme).clicked() {
                                self.replication.replication_drop_publication = Some(pub_info.name.clone());
                            }
                        });
                    });
                    ui.add_space(SPACE_XS);
                }
                for sub in inv.subscriptions.iter().take(40) {
                    card_frame(self.theme).show(ui, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "subscription {} · enabled={} · slot={:?}",
                                sub.name, sub.enabled, sub.slot_name
                            ))
                            .strong()
                            .monospace(),
                        );
                        ui.label(
                            RichText::new(format!(
                                "pubs={} · conninfo={}",
                                sub.publications.join(","),
                                sub.conninfo_redacted
                            ))
                            .small()
                            .color(self.theme.text_muted),
                        );
                        ui.horizontal(|ui| {
                            if ghost_button_with_icon(ui, Icon::FileCode2, "Preview DROP", self.theme).clicked() {
                                // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Drop
                                self.replication.replication_ddl_preview =
                                    db_pro_core::domain::replication::preview_drop_subscription(&sub.name).ok();
                            }
                            if danger_button(ui, "Drop…", self.theme).clicked() {
                                self.replication.replication_drop_subscription = Some(sub.name.clone());
                            }
                        });
                    });
                    ui.add_space(SPACE_XS);
                }
                for slot in inv.slots.iter().take(40) {
                    ui.label(
                        RichText::new(format!(
                            "slot {} · type={:?} · active={} · restart={:?}",
                            slot.slot_name, slot.slot_type, slot.active, slot.restart_lsn
                        ))
                        .small()
                        .monospace()
                        .color(self.theme.text_secondary),
                    );
                }
            }

            ui.add_space(SPACE_SM);
            ui.label(RichText::new("Create publication (FOR ALL TABLES)").small().strong());
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.replication.replication_create_name)
                        .hint_text("publication name"),
                );
                if ghost_button_with_icon(ui, Icon::FileCode2, "Preview CREATE", self.theme).clicked() {
                    // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Create
                    self.replication.replication_ddl_preview =
                        db_pro_core::domain::replication::preview_create_publication_all(
                            &self.replication.replication_create_name,
                        )
                        .ok();
                }
                if secondary_button(ui, "Create (confirm)", self.theme).clicked() {
                    self.create_publication_confirmed();
                }
            });

            if let Some(preview) = self.replication.replication_ddl_preview.clone() {
                egui::Window::new("Replication DDL preview")
                    .collapsible(false)
                    .resizable(true)
                    .default_width(480.0)
                    .show(ui.ctx(), |ui| {
                        ui.label(RichText::new(preview).monospace());
                        if secondary_button(ui, "Close", self.theme).clicked() {
                            self.replication.replication_ddl_preview = None;
                        }
                    });
            }
            if let Some(name) = self.replication.replication_drop_publication.clone() {
                egui::Window::new("Drop publication?")
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.label(format!("Drop publication `{name}`?"));
                        ui.horizontal(|ui| {
                            if danger_button(ui, "Drop", self.theme).clicked() {
                                self.drop_publication_confirmed(&name);
                            }
                            if secondary_button(ui, "Cancel", self.theme).clicked() {
                                self.replication.replication_drop_publication = None;
                            }
                        });
                    });
            }
            if let Some(name) = self.replication.replication_drop_subscription.clone() {
                egui::Window::new("Drop subscription?")
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.label(format!(
                            "Drop subscription `{name}`? Conninfo is never shown or logged."
                        ));
                        ui.horizontal(|ui| {
                            if danger_button(ui, "Drop", self.theme).clicked() {
                                self.drop_subscription_confirmed(&name);
                            }
                            if secondary_button(ui, "Cancel", self.theme).clicked() {
                                self.replication.replication_drop_subscription = None;
                            }
                        });
                    });
            }

            ui.add_space(SPACE_MD);
            section_label(ui, "EVENT TRIGGERS", self.theme);
            ui.add_space(SPACE_SM);
            ui.label(
                RichText::new(
                    "PostgreSQL-only · database-level DDL hooks · not table/row triggers · create requires existing function",
                )
                .small()
                .color(self.theme.text_muted),
            );
            if secondary_button_with_icon(ui, Icon::RefreshCw, "Load event triggers", self.theme).clicked() {
                self.request_event_triggers();
            }
            if let Some(error) = &self.event_trigger.event_trigger_error {
                ui.colored_label(self.theme.danger, error);
            }
            if let Some(inv) = self.event_trigger.event_trigger_inventory.clone() {
                ui.label(RichText::new(&inv.message).small().color(self.theme.text_secondary));
                for trig in inv.triggers.iter().take(50) {
                    card_frame(self.theme).show(ui, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} · on {} · {} · fn={}",
                                trig.name, trig.event, trig.enabled_label, trig.function_signature
                            ))
                            .strong()
                            .monospace(),
                        );
                        if !trig.tags.is_empty() {
                            ui.label(
                                RichText::new(format!("tags: {}", trig.tags.join(", ")))
                                    .small()
                                    .color(self.theme.text_muted),
                            );
                        }
                        ui.horizontal(|ui| {
                            if ghost_button_with_icon(ui, Icon::FileCode2, "Preview DROP", self.theme).clicked() {
                                // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Drop
                                self.event_trigger.event_trigger_ddl_preview =
                                    db_pro_core::domain::event_trigger::preview_drop_event_trigger(&trig.name).ok();
                            }
                            if ghost_button(ui, "Disable", self.theme).clicked() {
                                self.alter_event_trigger_confirmed(&trig.name, "disable");
                            }
                            if ghost_button(ui, "Enable", self.theme).clicked() {
                                self.alter_event_trigger_confirmed(&trig.name, "enable");
                            }
                            if danger_button(ui, "Drop…", self.theme).clicked() {
                                self.event_trigger.event_trigger_drop_confirm = Some(trig.name.clone());
                            }
                        });
                    });
                    ui.add_space(SPACE_XS);
                }
            }

            ui.add_space(SPACE_SM);
            ui.label(RichText::new("Create event trigger").small().strong());
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.event_trigger.event_trigger_create_name).hint_text("name"));
                ui.add(
                    egui::TextEdit::singleline(&mut self.event_trigger.event_trigger_create_event).hint_text("event"),
                );
            });
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.event_trigger.event_trigger_create_function)
                        .hint_text("schema.func()"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.event_trigger.event_trigger_create_tags)
                        .hint_text("tags CSV optional"),
                );
            });
            ui.horizontal(|ui| {
                if ghost_button_with_icon(ui, Icon::FileCode2, "Preview CREATE", self.theme).clicked() {
                    // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking Create
                    self.event_trigger.event_trigger_ddl_preview =
                        db_pro_core::domain::event_trigger::preview_create_event_trigger(
                            &self.event_trigger.event_trigger_create_name,
                            &self.event_trigger.event_trigger_create_event,
                            &self.event_trigger.event_trigger_create_function,
                            &self.event_trigger.event_trigger_create_tags,
                        )
                        .ok();
                }
                if secondary_button(ui, "Create (confirm)", self.theme).clicked() {
                    self.create_event_trigger_confirmed();
                }
            });

            if let Some(preview) = self.event_trigger.event_trigger_ddl_preview.clone() {
                egui::Window::new("Event trigger DDL preview")
                    .collapsible(false)
                    .resizable(true)
                    .default_width(520.0)
                    .show(ui.ctx(), |ui| {
                        ui.label(RichText::new(preview).monospace());
                        if secondary_button(ui, "Close", self.theme).clicked() {
                            self.event_trigger.event_trigger_ddl_preview = None;
                        }
                    });
            }
            if let Some(name) = self.event_trigger.event_trigger_drop_confirm.clone() {
                egui::Window::new("Drop event trigger?")
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.label(format!(
                            "Drop event trigger `{name}`? This changes global DDL hook behavior."
                        ));
                        ui.horizontal(|ui| {
                            if danger_button(ui, "Drop", self.theme).clicked() {
                                self.drop_event_trigger_confirmed(&name);
                            }
                            if secondary_button(ui, "Cancel", self.theme).clicked() {
                                self.event_trigger.event_trigger_drop_confirm = None;
                            }
                        });
                    });
            }

            ui.add_space(SPACE_MD);
            section_label(ui, "SERVER SETTINGS (pg_settings)", self.theme);
            ui.add_space(SPACE_SM);
            ui.label(
                RichText::new(
                    "PostgreSQL-only · session SET/RESET for user-context GUCs · ALTER SYSTEM is preview-only",
                )
                .small()
                .color(self.theme.text_muted),
            );
            ui.horizontal(|ui| {
                if secondary_button_with_icon(ui, Icon::RefreshCw, "Load settings", self.theme).clicked() {
                    self.request_pg_settings();
                }
            });
            ui.add_space(SPACE_XS);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Filter").small().color(self.theme.text_muted));
                ui.text_edit_singleline(&mut self.pg_settings.pg_settings_filter);
            });
            if let Some(error) = &self.pg_settings.pg_settings_error {
                ui.colored_label(self.theme.danger, error);
            }
            if let Some(snapshot) = self.pg_settings.pg_settings.clone() {
                ui.label(
                    RichText::new(format!(
                        "{} · fetched @ {} ms",
                        snapshot.message, snapshot.fetched_at_ms
                    ))
                    .small()
                    .color(self.theme.text_muted),
                );
                let filter = self.pg_settings.pg_settings_filter.to_ascii_lowercase();
                let rows: Vec<_> = snapshot
                    .settings
                    .iter()
                    .filter(|s| {
                        filter.is_empty()
                            || s.name.to_ascii_lowercase().contains(&filter)
                            || s.category.to_ascii_lowercase().contains(&filter)
                            || s.source.to_ascii_lowercase().contains(&filter)
                    })
                    .take(60)
                    .collect();
                for setting in rows {
                    card_frame(self.theme).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&setting.name)
                                    .strong()
                                    .monospace()
                                    .color(self.theme.text_primary),
                            );
                            if setting.pending_restart {
                                badge(ui, "pending restart", self.theme.warning, self.theme.text_primary);
                            }
                            if setting.sensitive {
                                badge(ui, "redacted", self.theme.surface_active, self.theme.text_secondary);
                            }
                        });
                        ui.label(
                            RichText::new(format!(
                                "{} · context={} · source={} · {}",
                                setting.category,
                                setting.context,
                                setting.source,
                                setting.display_setting()
                            ))
                            .small()
                            .color(self.theme.text_secondary),
                        );
                        if let Some(desc) = &setting.short_desc {
                            ui.label(RichText::new(desc).small().color(self.theme.text_muted));
                        }
                        ui.label(
                            RichText::new(setting.mutability_reason())
                                .small()
                                .color(self.theme.text_muted),
                        );
                        ui.horizontal(|ui| {
                            if setting.session_mutable() && !setting.sensitive {
                                if ghost_button_with_icon(ui, Icon::Pencil, "Edit session", self.theme).clicked() {
                                    self.pg_settings.pg_settings_edit_name = setting.name.clone();
                                    self.pg_settings.pg_settings_edit_value = setting.setting.clone();
                                }
                                if secondary_button(ui, "RESET", self.theme).clicked() {
                                    self.reset_pg_setting_session(&setting.name);
                                }
                            }
                            if !setting.sensitive
                                && ghost_button_with_icon(ui, Icon::FileCode2, "Preview ALTER SYSTEM", self.theme)
                                    .clicked()
                            {
                                // allow: preview is best-effort — preview generation error (name validation) only hides preview without blocking ALTER SYSTEM
                                self.pg_settings.pg_settings_preview =
                                    db_pro_core::domain::pg_settings::preview_alter_system(
                                        &setting.name,
                                        &setting.setting,
                                    )
                                    .ok();
                            }
                        });
                    });
                    ui.add_space(SPACE_SM);
                }
            }

            if !self.pg_settings.pg_settings_edit_name.is_empty() {
                egui::Window::new(format!("SET SESSION · {}", self.pg_settings.pg_settings_edit_name))
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ui.ctx(), |ui| {
                        ui.text_edit_singleline(&mut self.pg_settings.pg_settings_edit_value);
                        ui.horizontal(|ui| {
                            if secondary_button(ui, "Apply SET", self.theme).clicked() {
                                let name = self.pg_settings.pg_settings_edit_name.clone();
                                let value = self.pg_settings.pg_settings_edit_value.clone();
                                self.set_pg_setting_session(&name, &value);
                                self.pg_settings.pg_settings_edit_name.clear();
                            }
                            if ghost_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                                self.pg_settings.pg_settings_edit_name.clear();
                            }
                        });
                    });
            }

            if let Some(preview) = self.pg_settings.pg_settings_preview.clone() {
                egui::Window::new("ALTER SYSTEM preview")
                    .collapsible(false)
                    .resizable(true)
                    .default_width(480.0)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ui.ctx(), |ui| {
                        ui.label(RichText::new(&preview.note).small().color(self.theme.warning));
                        ui.label(RichText::new(&preview.sql).monospace());
                        if secondary_button(ui, "Close", self.theme).clicked() {
                            self.pg_settings.pg_settings_preview = None;
                        }
                    });
            }

            ui.add_space(SPACE_MD);
            section_label(ui, "MAINTENANCE", self.theme);
            ui.add_space(SPACE_SM);
            ui.label(
                RichText::new("Actions run through MonitoringService — SQL is never built in the UI.")
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.horizontal_wrapped(|ui| {
                use db_pro_core::domain::monitoring::MaintenanceAction;
                for action in [
                    MaintenanceAction::Analyze,
                    MaintenanceAction::Vacuum,
                    MaintenanceAction::VacuumAnalyze,
                ] {
                    if secondary_button(ui, action.as_label(), self.theme).clicked() {
                        self.monitoring.monitoring_maintenance_confirm = Some(action);
                    }
                }
            });
        } else if connected {
            ui.label(
                RichText::new("Refresh to load sessions (or wait for auto-refresh).")
                    .small()
                    .color(self.theme.text_muted),
            );
        }

        if let Some(backend_id) = self.monitoring.monitoring_terminate_confirm {
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
                                self.dispatch_command(self.monitoring.terminate_backend_command(
                                    request_id,
                                    connection_id,
                                    backend_id,
                                ));
                            }
                            self.monitoring.monitoring_terminate_confirm = None;
                        }
                        if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                            self.monitoring.monitoring_terminate_confirm = None;
                        }
                    });
                });
        }

        if let Some(action) = self.monitoring.monitoring_maintenance_confirm {
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
                                self.dispatch_command(self.monitoring.maintenance_command(
                                    request_id,
                                    connection_id,
                                    action,
                                ));
                            }
                            self.monitoring.monitoring_maintenance_confirm = None;
                        }
                        if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                            self.monitoring.monitoring_maintenance_confirm = None;
                        }
                    });
                });
        }

        if self.monitoring.monitoring_reset_stats_confirm {
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
                                    self.monitoring.reset_statements_command(request_id, connection_id),
                                );
                            }
                            self.monitoring.monitoring_reset_stats_confirm = false;
                        }
                        if secondary_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                            self.monitoring.monitoring_reset_stats_confirm = false;
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
        self.dispatch_command(self.monitoring.workload_command(request_id, connection_id));
    }

    fn request_audit_page(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.audit.audit_error = Some("Connect a database first".into());
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.audit.events_load_command(request_id, connection_id));
    }

    fn export_selected_audit_events(&mut self) {
        match self.audit.build_export_preview() {
            Ok((selected_count, export_warning)) => {
                self.feedback.runtime_message =
                    format!("Audit export preview · {selected_count} row(s) · {export_warning}");
            }
            Err(error) => self.audit.audit_error = Some(error),
        }
    }

    fn request_pg_settings(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.pg_settings.pg_settings_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        let driver = self.active_driver().to_ascii_lowercase();
        if !(driver.contains("postgres")) {
            self.pg_settings.pg_settings_error = Some("pg_settings is PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.pg_settings.list_command(request_id, connection_id));
    }

    fn set_pg_setting_session(&mut self, name: &str, value: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.pg_settings.set_session_command(
            request_id,
            connection_id,
            name.to_owned(),
            value.to_owned(),
        ));
    }

    fn reset_pg_setting_session(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            self.pg_settings
                .reset_session_command(request_id, connection_id, name.to_owned()),
        );
    }

    fn request_fdw_inventory(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.fdw.fdw_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.active_driver().to_ascii_lowercase().contains("postgres") {
            self.fdw.fdw_error = Some("FDW administration is PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.fdw.list_command(request_id, connection_id));
    }

    fn create_fdw_server_confirmed(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.fdw.create_command(request_id, connection_id));
    }

    fn drop_fdw_server_confirmed(&mut self, name: &str, cascade: bool) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            self.fdw
                .drop_command(request_id, connection_id, name.to_owned(), cascade),
        );
    }

    fn request_replication_inventory(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.replication.replication_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.active_driver().to_ascii_lowercase().contains("postgres") {
            self.replication.replication_error = Some("Logical replication administration is PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.replication.list_command(request_id, connection_id));
    }

    fn create_publication_confirmed(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.replication.create_publication_command(request_id, connection_id));
    }

    fn drop_publication_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            self.replication
                .drop_publication_command(request_id, connection_id, name.to_owned()),
        );
    }

    fn drop_subscription_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            self.replication
                .drop_subscription_command(request_id, connection_id, name.to_owned()),
        );
    }

    fn request_event_triggers(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.event_trigger.event_trigger_error = Some("Connect a PostgreSQL database first".into());
            return;
        };
        if !self.active_driver().to_ascii_lowercase().contains("postgres") {
            self.event_trigger.event_trigger_error = Some("Event triggers are PostgreSQL-only".into());
            return;
        }
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.event_trigger.list_command(request_id, connection_id));
    }

    fn create_event_trigger_confirmed(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.event_trigger.create_command(request_id, connection_id));
    }

    fn drop_event_trigger_confirmed(&mut self, name: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(
            self.event_trigger
                .drop_command(request_id, connection_id, name.to_owned()),
        );
    }

    fn alter_event_trigger_confirmed(&mut self, name: &str, mode: &str) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.event_trigger.alter_command(
            request_id,
            connection_id,
            name.to_owned(),
            mode.to_owned(),
        ));
    }

    fn request_monitoring_snapshot(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            return;
        };
        self.monitoring.monitoring_last_poll = Some(std::time::Instant::now());
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.monitoring.snapshot_command(request_id, connection_id));
    }
}
