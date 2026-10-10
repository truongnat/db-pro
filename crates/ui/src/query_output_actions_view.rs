// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
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

fn bounded_explain_metric(value: f64) -> f32 {
    if !value.is_finite() || value <= 0.0 {
        return 0.0;
    }

    // ExplainPlanTree stores metrics as f32; clamp before narrowing the core f64 value.
    value.min(f32::MAX as f64) as f32
}

pub(super) fn draw_explain_pane(
    context: &mut QueryOutputActionsContext<'_>,
    ui: &mut egui::Ui,
) -> Option<QueryOutputAction> {
    let mut action = None;
    let output_width = ui.available_width();
    Card::new(context.theme).show(ui, |ui| {
        ui.set_min_width(output_width.max(0.0));
        ui.horizontal(|ui| {
            if Button::new(context.theme)
                .text(t!("query.explain"))
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                action = Some(QueryOutputAction::Explain);
            }
            if Button::new(context.theme)
                .text(t!("query.explain_analyze"))
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                context.execution.explain_analyze_confirmed = false;
                action = Some(QueryOutputAction::ExplainAnalyze);
            }
            ui.checkbox(&mut context.execution.explain_show_raw_json, "Raw JSON");
            if let Some(plan) = context.session.active_explain_plan() {
                if Button::new(context.theme)
                    .text("Copy plan")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    ui.output_mut(|output| output.commands.push(egui::OutputCommand::CopyText(plan.to_owned())));
                    context.feedback.runtime_message = "Query plan copied".to_owned();
                }
            }
        });
        if context.execution.pending_explain_analyze {
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    // cc-scan:allow LINE_TOO_LONG — literal must not wrap
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
                    // The plan tree repeats this finding on its root row; avoid a duplicate summary label.
                    for finding in plan.findings.iter().filter(|finding| finding.code != "plan.truncated").take(8) {
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
                    ExplainPlanTree::new(&tree, bounded_explain_metric(plan.display_total_ms()), context.theme)
                        .has_runtime_stats(plan.has_runtime_stats)
                        .planning_time(plan.planning_time_ms.map(bounded_explain_metric))
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
                // cc-scan:allow LINE_TOO_LONG — literal must not wrap
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
    Card::new(context.theme).show(ui, |ui| {
        ui.set_min_width(output_width.max(0.0));
        ui.horizontal(|ui| {
            ui.label(RichText::new(t!("query.recent_executions")).font(font_subheading()).strong());
            ui.add_space(SPACE_MD);
            Input::new(
                &mut context.editor.query_history_search,
                &*t!("query.filter_history"),
                context.theme,
            )
            .width(180.0)
            .show(ui);
            if !context.editor.query_history_search.is_empty()
                && Button::new(context.theme)
                    .text(t!("common.clear"))
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
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
            EmptyState::new(
                Icon::History,
                &*t!("query.no_history"),
                &*t!("query.no_history_desc"),
                context.theme,
            )
            .show(ui);
        } else if filtered_entries.is_empty() {
            EmptyState::new(
                Icon::Search,
                &*t!("query.no_matching_queries"),
                &*t!("query.no_matching_queries_desc"),
                context.theme,
            )
            .show(ui);
        } else {
            egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                for entry in filtered_entries {
                    egui::Frame::NONE
                        .fill(context.theme.surface_panel)
                        .stroke(egui::Stroke::new(1.0, context.theme.border_subtle))
                        .corner_radius(egui::CornerRadius::same(4.0 as u8))
                        .inner_margin(egui::Margin::same(8.0 as i8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let (badge_text, badge_variant) = if entry.status == UiQueryHistoryStatus::Failed {
                                    ("✕ FAIL", BadgeVariant::Destructive)
                                } else {
                                    ("✓ OK", BadgeVariant::Success)
                                };
                                Badge::new(badge_text, context.theme)
                                    .variant(badge_variant)
                                    .compact(true)
                                    .show(ui);
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
                            if ui
                                .label(RichText::new(truncated).monospace().small().color(context.theme.text_primary))
                                .interact(egui::Sense::click())
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .on_hover_text("Open in a new editor")
                                .clicked()
                            {
                                action = Some(QueryOutputAction::OpenHistory(
                                    Box::new(entry.clone()),
                                    false,
                                ));
                            }
                        });
                    ui.add_space(4.0);
                }
            });
        }
    });
    action
}

#[cfg(test)]
mod tests {
    use super::bounded_explain_metric;

    #[test]
    fn explain_display_metrics_are_finite_and_bounded() {
        assert_eq!(bounded_explain_metric(f64::MAX), f32::MAX);
        assert_eq!(bounded_explain_metric(f64::NAN), 0.0);
        assert_eq!(bounded_explain_metric(-1.0), 0.0);
    }
}
