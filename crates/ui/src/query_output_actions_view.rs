//! Query explain/history panes and their explicit actions.
use super::*;
use egui::RichText;
use lucide_icons::Icon;

pub(super) struct QueryOutputActionsContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) session: &'a QuerySessionState,
    pub(super) editor: &'a mut QueryEditorState,
    pub(super) execution: &'a mut QueryExecutionPolicyState,
    pub(super) feedback: &'a mut FeedbackState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum QueryOutputAction {
    Explain,
    ExplainAnalyze,
    OpenHistory(Box<UiQueryHistoryEntry>, bool),
}

pub(super) fn draw_explain_pane(
    context: &mut QueryOutputActionsContext<'_>,
    ui: &mut egui::Ui,
) -> Option<QueryOutputAction> {
    let mut action = None;
    let output_width = ui.available_width();
    card_frame(context.theme).show(ui, |ui| {
        ui.set_min_width(output_width.max(0.0));
        ui.horizontal(|ui| {
            if compact_button(ui, "Explain", context.theme).clicked() {
                action = Some(QueryOutputAction::Explain);
            }
            if compact_button(ui, "Explain ANALYZE…", context.theme).clicked() {
                context.execution.explain_analyze_confirmed = false;
                action = Some(QueryOutputAction::ExplainAnalyze);
            }
            ui.checkbox(&mut context.execution.explain_show_raw_json, "Raw JSON");
            if let Some(plan) = context.session.active_explain_plan() {
                if compact_button(ui, "Copy plan", context.theme).clicked() {
                    ui.output_mut(|output| output.copied_text = plan.to_owned());
                    context.feedback.runtime_message = "Query plan copied".to_owned();
                }
            }
        });
        if context.execution.pending_explain_analyze {
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    "WARNING: EXPLAIN ANALYZE executes the statement (including writes). Confirm only when you intend to run it.",
                )
                .color(context.theme.warning),
            );
            ui.checkbox(
                &mut context.execution.explain_analyze_confirmed,
                "I understand this will execute the query",
            );
            if Button::new(context.theme)
                .text("Run EXPLAIN ANALYZE")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .enabled(context.execution.explain_analyze_confirmed)
                .show(ui)
                .clicked()
            {
                action = Some(QueryOutputAction::ExplainAnalyze);
            }
        }
        ui.add_space(8.0);
        if let Some(plan_json) = context.session.active_explain_plan() {
            if context.execution.explain_show_raw_json {
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    ui.label(RichText::new(plan_json).monospace().color(context.theme.text_secondary));
                });
            } else if let Some(plan) = db_pro_core::domain::explain_plan::parse_postgres_explain_str(plan_json) {
                if !plan.findings.is_empty() {
                    ui.add_space(4.0);
                    for finding in plan.findings.iter().take(8) {
                        let color = match finding.severity {
                            db_pro_core::domain::explain_plan::PlanFindingSeverity::Hotspot => context.theme.danger,
                            db_pro_core::domain::explain_plan::PlanFindingSeverity::Warning => context.theme.warning,
                            db_pro_core::domain::explain_plan::PlanFindingSeverity::Info => context.theme.text_muted,
                        };
                        ui.label(RichText::new(format!("• {}", finding.message)).small().color(color));
                    }
                }
                ui.add_space(6.0);
                let tree = crate::components::explain::PlanNode::from_query_plan(&plan.root);
                egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                    ExplainPlanTree::new(&tree, plan.display_total_ms() as f32, context.theme)
                        .has_runtime_stats(plan.has_runtime_stats)
                        .planning_time(plan.planning_time_ms.map(|t| t as f32))
                        .show(ui);
                });
            } else {
                ui.label(
                    RichText::new("Could not parse plan tree — showing raw output")
                        .small()
                        .color(context.theme.text_muted),
                );
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    ui.label(RichText::new(plan_json).monospace().color(context.theme.text_secondary));
                });
            }
        } else {
            empty_state(
                ui,
                Icon::ChartNoAxesCombined,
                "No query plan yet",
                "Execute EXPLAIN or EXPLAIN ANALYZE from the toolbar or query actions to see costs, plan nodes, and advisor findings.",
                context.theme,
            );
        }
    });
    action
}

pub(super) fn draw_history_pane(
    context: &mut QueryOutputActionsContext<'_>,
    ui: &mut egui::Ui,
) -> Option<QueryOutputAction> {
    let mut action = None;
    let output_width = ui.available_width();
    card_frame(context.theme).show(ui, |ui| {
        ui.set_min_width(output_width.max(0.0));
        ui.horizontal(|ui| {
            ui.label(RichText::new("Recent Executions").font(font_subheading()).strong());
            ui.add_space(SPACE_MD);
            let search_edit = egui::TextEdit::singleline(&mut context.editor.query_history_search)
                .hint_text("Filter history…")
                .desired_width(180.0);
            ui.add(search_edit);
            if !context.editor.query_history_search.is_empty()
                && compact_button(ui, "Clear", context.theme).clicked()
            {
                context.editor.query_history_search.clear();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{} total", context.editor.query_history_entries.len()))
                        .small()
                        .color(context.theme.text_muted),
                );
            });
        });
        ui.add_space(SPACE_SM);
        let filter_lower = context.editor.query_history_search.trim().to_lowercase();
        let filtered_entries: Vec<_> = context
            .editor
            .query_history_entries
            .iter()
            .rev()
            .filter(|e| filter_lower.is_empty() || e.sql.to_lowercase().contains(&filter_lower))
            .take(50)
            .collect();

        if context.editor.query_history_entries.is_empty() {
            empty_state(
                ui,
                Icon::History,
                "No query history",
                "Executed queries will appear here with timing, status, and one-click replay into editor.",
                context.theme,
            );
        } else if filtered_entries.is_empty() {
            empty_state(
                ui,
                Icon::Search,
                "No matching queries",
                "Try a different search keyword to find past query executions.",
                context.theme,
            );
        } else {
            egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                for entry in filtered_entries {
                    egui::Frame::none()
                        .fill(context.theme.surface_panel)
                        .stroke(egui::Stroke::new(1.0, context.theme.border_subtle))
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let status_color = if entry.status == UiQueryHistoryStatus::Failed {
                                    context.theme.danger
                                } else {
                                    context.theme.success
                                };
                                ui.label(
                                    RichText::new(if entry.status == UiQueryHistoryStatus::Failed { "✕ FAIL" } else { "✓ OK" })
                                        .small()
                                        .strong()
                                        .color(status_color),
                                );
                                ui.label(
                                    RichText::new(format!("{}ms", entry.duration_ms))
                                        .small()
                                        .monospace()
                                        .color(context.theme.text_secondary),
                                );
                                let rows_display = entry.row_count.or(entry.affected_rows);
                                if let Some(rows) = rows_display {
                                    ui.label(
                                        RichText::new(format!("{rows} rows"))
                                            .small()
                                            .color(context.theme.text_muted),
                                    );
                                }
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if Button::new(context.theme)
                                        .text("Replay")
                                        .size(ButtonSize::Sm)
                                        .variant(ButtonVariant::Ghost)
                                        .icon(Icon::RotateCw)
                                        .show(ui)
                                        .clicked()
                                    {
                                        action = Some(QueryOutputAction::OpenHistory(
                                            Box::new(entry.clone()),
                                            true,
                                        ));
                                    }
                                });
                            });
                            ui.add_space(2.0);
                            let preview: String = entry.sql.lines().take(2).collect::<Vec<_>>().join(" ");
                            let truncated = if preview.len() > 120 {
                                format!("{}…", &preview[..117])
                            } else {
                                preview
                            };
                            ui.label(RichText::new(truncated).monospace().small().color(context.theme.text_primary));
                        });
                    ui.add_space(4.0);
                }
            });
        }
    });
    action
}
