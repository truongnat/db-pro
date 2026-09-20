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
                            context.theme.warning
                        } else {
                            context.theme.text_muted
                        }),
                );
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
                egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                    ExplainPlanTree::new(&tree, plan.display_total_ms() as f32, context.theme).show(ui);
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
                "Run Explain for an estimate, or Explain ANALYZE for measured runtime (executes the query).",
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
            ui.label(RichText::new("Search").small().color(context.theme.text_muted));
            ui.add_sized(
                [220.0, 24.0],
                egui::TextEdit::singleline(&mut context.editor.query_history_search)
                    .hint_text("SQL, connection, schema"),
            );
            if compact_button(ui, "Clear History", context.theme).clicked() {
                context.editor.query_history_entries.clear();
                context.feedback.runtime_message = "Query history cleared".to_owned();
            }
        });
        if context.editor.query_history_entries.is_empty() {
            empty_state(
                ui,
                Icon::History,
                "No query history yet",
                "Executed queries will appear here.",
                context.theme,
            );
        } else {
            let search = context.editor.query_history_search.trim().to_lowercase();
            let entries = context
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
                    ui.label(RichText::new(status).small().color(context.theme.text_muted));
                    ui.label(
                        RichText::new(format!("{} ms", entry.duration_ms))
                            .small()
                            .color(context.theme.text_muted),
                    );
                    ui.label(
                        RichText::new(entry.sql.lines().next().unwrap_or("query"))
                            .monospace()
                            .small()
                            .color(context.theme.text_secondary),
                    );
                    if compact_button(ui, "Open", context.theme).clicked() {
                        action = Some(QueryOutputAction::OpenHistory(Box::new(entry.clone()), false));
                    }
                    if compact_button(ui, "Copy SQL", context.theme).clicked() {
                        ui.output_mut(|output| output.copied_text = entry.sql.clone());
                    }
                    if compact_button(ui, "Run Again", context.theme).clicked() {
                        action = Some(QueryOutputAction::OpenHistory(Box::new(entry.clone()), true));
                    }
                });
            }
        }
    });
    action
}
