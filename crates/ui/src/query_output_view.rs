//! Query output tabs and result panes.
use super::*;
use egui::RichText;
use lucide_icons::Icon;

impl DbProApp {
    /// Body of the selected output tab.
    pub(super) fn draw_output_pane(&mut self, ui: &mut egui::Ui, result: Option<&UiQueryResult>) {
        match self.query.output.active_tab_for_document(
            self.query
                .session
                .active_document()
                .map(|document| document.id.as_str()),
        ) {
            OutputTab::Results => self.draw_results_pane(ui, result),
            OutputTab::Chart => self.draw_chart_pane(ui, result),
            OutputTab::Messages => self.draw_messages_pane(ui),
            OutputTab::Explain => self.draw_explain_pane(ui),
            OutputTab::History => self.draw_history_pane(ui),
        }
    }

    /// Results grid plus its row-count / export header.
    pub(super) fn draw_results_pane(&mut self, ui: &mut egui::Ui, result: Option<&UiQueryResult>) {
        let results_width = ui.max_rect().width();
        grid_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(results_width.max(0.0));
            let result_count = self.query.session.active_result_count();
            if result_count > 1 {
                ui.horizontal(|ui| {
                    let active_index = self
                        .query
                        .session
                        .documents
                        .get(self.query.session.active_document_index)
                        .map_or(0, |doc| doc.active_result_index);
                    for index in 0..result_count {
                        if ui
                            .selectable_label(active_index == index, format!("Result {}", index + 1))
                            .clicked()
                        {
                            self.set_active_query_result(index);
                        }
                    }
                });
            }
            ui.horizontal(|ui| {
                if let Some(value) = result {
                    ui.label(
                        RichText::new(format!("{} rows · {} ms", value.row_count, value.duration_ms))
                            .small()
                            .color(self.theme.text_muted),
                    );
                    if compact_button(ui, "Export", self.theme).clicked() {
                        self.overlay.export_open = true;
                    }
                }
            });
            if let Some(result) = result {
                self.draw_result_grid(ui, result);
            } else {
                ui.centered_and_justified(|ui| {
                    empty_state(
                        ui,
                        Icon::Table2,
                        "No results yet",
                        "Run a query to populate this result grid.",
                        self.theme,
                    );
                });
            }
        });
        self.draw_export_dialog(ui, result);
        self.draw_destructive_run_dialog(ui);
    }

    /// Query notice log.
    pub(super) fn draw_messages_pane(&mut self, ui: &mut egui::Ui) {
        let mut context = query_output_panes_view::QueryOutputPanesContext {
            theme: self.theme,
            session: &mut self.query.session,
        };
        query_output_panes_view::draw_messages_pane(&mut context, ui);
    }

    pub(super) fn draw_chart_pane(&mut self, ui: &mut egui::Ui, result: Option<&UiQueryResult>) {
        let mut context = query_output_panes_view::QueryOutputPanesContext {
            theme: self.theme,
            session: &mut self.query.session,
        };
        query_output_panes_view::draw_chart_pane(&mut context, ui, result);
    }

    /// EXPLAIN output for the last explain request.
    pub(super) fn draw_explain_pane(&mut self, ui: &mut egui::Ui) {
        let output_width = ui.available_width();
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(output_width.max(0.0));
            ui.horizontal(|ui| {
                if compact_button(ui, "Explain", self.theme).clicked() {
                    self.explain_query();
                }
                if compact_button(ui, "Explain ANALYZE…", self.theme).clicked() {
                    self.query.execution.explain_analyze_confirmed = false;
                    self.explain_query_analyze();
                }
                ui.checkbox(&mut self.query.execution.explain_show_raw_json, "Raw JSON");
                if let Some(plan) = self.query.session.active_explain_plan() {
                    if compact_button(ui, "Copy plan", self.theme).clicked() {
                        ui.output_mut(|o| o.copied_text = plan.to_owned());
                        self.feedback.runtime_message = "Query plan copied".to_owned();
                    }
                }
            });
            if self.query.execution.pending_explain_analyze {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(
                        "WARNING: EXPLAIN ANALYZE executes the statement (including writes). Confirm only when you intend to run it.",
                    )
                    .color(self.theme.warning),
                );
                ui.checkbox(
                    &mut self.query.execution.explain_analyze_confirmed,
                    "I understand this will execute the query",
                );
                if Button::new(self.theme)
                    .text("Run EXPLAIN ANALYZE")
                    .variant(ButtonVariant::Default)
                    .size(ButtonSize::Sm)
                    .enabled(self.query.execution.explain_analyze_confirmed)
                    .show(ui)
                    .clicked()
                {
                    self.explain_query_analyze();
                }
            }
            ui.add_space(8.0);
            if let Some(plan_json) = self.query.session.active_explain_plan() {
                if self.query.execution.explain_show_raw_json {
                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        ui.label(RichText::new(plan_json).monospace().color(self.theme.text_secondary));
                    });
                } else if let Some(plan) =
                    db_pro_core::domain::explain_plan::parse_postgres_explain_str(plan_json)
                {
                    let mode = if plan.has_runtime_stats {
                        "Runtime (EXPLAIN ANALYZE)"
                    } else {
                        "Estimate-only (EXPLAIN)"
                    };
                    ui.label(
                        RichText::new(mode)
                            .small()
                            .strong()
                            .color(if plan.has_runtime_stats {
                                self.theme.warning
                            } else {
                                self.theme.text_muted
                            }),
                    );
                    if !plan.findings.is_empty() {
                        ui.add_space(4.0);
                        for finding in plan.findings.iter().take(8) {
                            let color = match finding.severity {
                                db_pro_core::domain::explain_plan::PlanFindingSeverity::Hotspot => {
                                    self.theme.danger
                                }
                                db_pro_core::domain::explain_plan::PlanFindingSeverity::Warning => {
                                    self.theme.warning
                                }
                                db_pro_core::domain::explain_plan::PlanFindingSeverity::Info => {
                                    self.theme.text_muted
                                }
                            };
                            ui.label(RichText::new(format!("• {}", finding.message)).small().color(color));
                        }
                    }
                    ui.add_space(6.0);
                    let tree = crate::components::explain::PlanNode::from_query_plan(&plan.root);
                    egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                        ExplainPlanTree::new(&tree, plan.display_total_ms() as f32, self.theme).show(ui);
                    });
                } else {
                    ui.label(
                        RichText::new("Could not parse plan tree — showing raw output")
                            .small()
                            .color(self.theme.text_muted),
                    );
                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        ui.label(RichText::new(plan_json).monospace().color(self.theme.text_secondary));
                    });
                }
            } else {
                empty_state(
                    ui,
                    Icon::ChartNoAxesCombined,
                    "No query plan yet",
                    "Run Explain for an estimate, or Explain ANALYZE for measured runtime (executes the query).",
                    self.theme,
                );
            }
        });
    }

    /// Recently executed queries.
    pub(super) fn draw_history_pane(&mut self, ui: &mut egui::Ui) {
        let output_width = ui.available_width();
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(output_width.max(0.0));
            ui.horizontal(|ui| {
                ui.label(RichText::new("Search").small().color(self.theme.text_muted));
                ui.add_sized(
                    [220.0, 24.0],
                    egui::TextEdit::singleline(&mut self.query.editor.query_history_search)
                        .hint_text("SQL, connection, schema"),
                );
                if compact_button(ui, "Clear History", self.theme).clicked() {
                    self.query.editor.query_history_entries.clear();
                    self.feedback.runtime_message = "Query history cleared".to_owned();
                }
            });
            if self.query.editor.query_history_entries.is_empty() {
                empty_state(
                    ui,
                    Icon::History,
                    "No query history yet",
                    "Executed queries will appear here.",
                    self.theme,
                );
            } else {
                let search = self.query.editor.query_history_search.trim().to_lowercase();
                let entries = self
                    .query
                    .editor
                    .query_history_entries
                    .iter()
                    .rev()
                    .filter(|entry| {
                        search.is_empty()
                            || entry.sql.to_lowercase().contains(&search)
                            || entry
                                .connection_id
                                .as_deref()
                                .is_some_and(|connection| connection.to_lowercase().contains(&search))
                            || entry
                                .schema
                                .as_deref()
                                .is_some_and(|schema| schema.to_lowercase().contains(&search))
                    })
                    .take(20)
                    .cloned()
                    .collect::<Vec<_>>();
                for entry in entries {
                    ui.horizontal_wrapped(|ui| {
                        let status = match entry.status {
                            UiQueryHistoryStatus::Success => "OK",
                            UiQueryHistoryStatus::Failed => "Failed",
                            UiQueryHistoryStatus::Cancelled => "Cancelled",
                        };
                        ui.label(RichText::new(status).small().color(self.theme.text_muted));
                        ui.label(
                            RichText::new(format!("{} ms", entry.duration_ms))
                                .small()
                                .color(self.theme.text_muted),
                        );
                        ui.label(
                            RichText::new(entry.sql.lines().next().unwrap_or("query"))
                                .monospace()
                                .small()
                                .color(self.theme.text_secondary),
                        );
                        if compact_button(ui, "Open", self.theme).clicked() {
                            self.open_history_entry(&entry, false);
                        }
                        if compact_button(ui, "Copy SQL", self.theme).clicked() {
                            ui.output_mut(|output| output.copied_text = entry.sql.clone());
                        }
                        if compact_button(ui, "Run Again", self.theme).clicked() {
                            self.open_history_entry(&entry, true);
                        }
                    });
                }
            }
        });
    }
}
