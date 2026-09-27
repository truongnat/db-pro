use super::config::*;
use super::handler::*;
use crate::tokens::*;
use crate::DbProTheme;
use egui::{Id, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType};

pub struct ExplainPlanTree<'a> {
    root: &'a PlanNode,
    total_time_ms: f32,
    theme: DbProTheme,
    planning_time_ms: Option<f32>,
    has_runtime_stats: bool,
}

impl<'a> ExplainPlanTree<'a> {
    pub fn new(root: &'a PlanNode, total_time_ms: f32, theme: DbProTheme) -> Self {
        Self {
            root,
            total_time_ms,
            theme,
            planning_time_ms: None,
            has_runtime_stats: false,
        }
    }

    pub fn planning_time(mut self, time_ms: Option<f32>) -> Self {
        self.planning_time_ms = time_ms;
        self
    }

    pub fn has_runtime_stats(mut self, runtime: bool) -> Self {
        self.has_runtime_stats = runtime;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        // Seed semantic IDs once; descendant indices keep painter-only labels unique and stable per tree.
        let tree_id = ui.auto_id_with("explain_plan_tree");
        ui.skip_ahead_auto_ids(1);
        let frame = egui::Frame::none()
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(STROKE_THIN, self.theme.border_default))
            .rounding(Rounding::same(RADIUS_CARD))
            .inner_margin(egui::Margin::same(SPACE_MD));

        frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // Header with high-tier summary stats (DataGrip/DBeaver parity)
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Execution Plan Analysis")
                            .size(FONT_SIZE_UI_LABEL)
                            .strong()
                            .color(self.theme.text_primary),
                    );

                    if self.has_runtime_stats {
                        let badge_galley = ui.painter().layout_no_wrap(
                            "EXPLAIN ANALYZE".to_owned(),
                            font_caption(),
                            self.theme.warning,
                        );
                        let badge_rect = Rect::from_min_size(
                            Pos2::new(ui.cursor().min.x, ui.cursor().min.y + EXPLAIN_BADGE_PAD_Y),
                            Vec2::new(badge_galley.size().x + EXPLAIN_BADGE_PAD_X, EXPLAIN_BADGE_HEIGHT),
                        );
                        ui.painter()
                            .rect_filled(badge_rect, Rounding::same(RADIUS_XS), self.theme.warning_soft());
                        ui.painter().galley(
                            Pos2::new(
                                badge_rect.left() + EXPLAIN_BADGE_TEXT_PAD_X,
                                badge_rect.top() + EXPLAIN_BADGE_TEXT_TOP_OFFSET,
                            ),
                            badge_galley,
                            self.theme.warning,
                        );
                        label_painted_badge(ui, badge_rect, tree_id.with("analyze_badge"), "EXPLAIN ANALYZE");
                        ui.add_space(badge_rect.width() + SPACE_XS);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let label = if self.has_runtime_stats {
                            format!("Total Exec: {:.2}ms", self.total_time_ms)
                        } else {
                            format!("Total Cost: {:.2}", self.total_time_ms)
                        };
                        ui.label(
                            RichText::new(label)
                                .size(FONT_SIZE_CAPTION)
                                .monospace()
                                .strong()
                                .color(self.theme.text_primary),
                        );
                        if let Some(plan_time) = self.planning_time_ms {
                            ui.label(
                                RichText::new(format!("Planning: {:.2}ms · ", plan_time))
                                    .size(FONT_SIZE_CAPTION)
                                    .monospace()
                                    .color(self.theme.text_secondary),
                            );
                        }
                    });
                });

                ui.add_space(SPACE_SM);

                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.render_node(ui, self.root, 0, tree_id));
            })
            .response
    }

    fn render_node(&self, ui: &mut Ui, node: &PlanNode, depth: usize, node_id: Id) {
        let indent = depth as f32 * EXPLAIN_ROW_INDENT;

        // Calculate cost / time percentage for Flame Tree bar
        let node_val = if self.has_runtime_stats {
            node.actual_time_ms
        } else {
            node.cost_estimate
        };
        let metrics = flame_bar_metrics(node_val, self.total_time_ms, node.is_bottleneck, &self.theme);

        ui.horizontal(|ui| {
            if indent > 0.0 {
                ui.add_space(indent);
            }

            // Node Icon
            let (icon, icon_color) = node_icon_and_color(
                node.is_bottleneck,
                node.relation.is_some(),
                node.index_name.is_some(),
                &self.theme,
            );

            ui.label(
                RichText::new(char::from(icon).to_string())
                    .font(font_icon(ICON_SM))
                    .color(icon_color),
            );

            // Node title & relation/index
            ui.label(
                RichText::new(&node.node_type)
                    .size(FONT_SIZE_BODY_SM)
                    .strong()
                    .color(self.theme.text_primary),
            );

            if let Some(ref rel) = node.relation {
                ui.label(
                    RichText::new(format!("on {rel}"))
                        .size(FONT_SIZE_CAPTION)
                        .color(self.theme.text_secondary),
                );
            }

            if let Some(ref idx) = node.index_name {
                ui.label(
                    RichText::new(format!("using {idx}"))
                        .size(FONT_SIZE_CAPTION)
                        .monospace()
                        .color(self.theme.accent),
                );
            }

            // Visual Cost/Time Flame Bar
            let (bar_rect, bar_response) =
                ui.allocate_exact_size(Vec2::new(EXPLAIN_BAR_WIDTH, EXPLAIN_BAR_HEIGHT), Sense::hover());
            let metric_label = if self.has_runtime_stats {
                "Actual time share"
            } else {
                "Estimated cost share"
            };
            let accessible_bar_label = format!("{metric_label}: {:.0}%", metrics.percentage * 100.0);
            bar_response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &accessible_bar_label));
            ui.painter().rect_filled(
                bar_rect,
                Rounding::same(EXPLAIN_BAR_ROUNDING),
                self.theme.surface_active,
            );
            if should_render_flame_bar(metrics.percentage) {
                let fill_rect = Rect::from_min_size(
                    bar_rect.min,
                    Vec2::new(EXPLAIN_BAR_WIDTH * metrics.percentage, EXPLAIN_BAR_HEIGHT),
                );
                ui.painter()
                    .rect_filled(fill_rect, Rounding::same(EXPLAIN_BAR_ROUNDING), metrics.color);
            }

            // Bottleneck badge
            if node.is_bottleneck {
                ui.add_space(SPACE_XXS);
                let badge_galley = ui
                    .painter()
                    .layout_no_wrap("Hotspot".to_owned(), font_caption(), self.theme.danger);
                let badge_rect = Rect::from_min_size(
                    Pos2::new(ui.cursor().min.x, ui.cursor().min.y + EXPLAIN_BADGE_PAD_Y),
                    Vec2::new(badge_galley.size().x + EXPLAIN_BADGE_PAD_X, EXPLAIN_BADGE_HEIGHT),
                );
                let fill = self.theme.danger_soft();
                ui.painter().rect_filled(badge_rect, Rounding::same(RADIUS_XS), fill);
                ui.painter().galley(
                    Pos2::new(
                        badge_rect.left() + EXPLAIN_BADGE_TEXT_PAD_X,
                        badge_rect.top() + EXPLAIN_BADGE_TEXT_TOP_OFFSET,
                    ),
                    badge_galley,
                    self.theme.danger,
                );
                label_painted_badge(ui, badge_rect, node_id.with("hotspot_badge"), "Hotspot");
                ui.add_space(badge_rect.width() + SPACE_XS);
            }

            // Estimate skew warning badge (Actual rows vs Planned rows mismatch)
            if let Some(skew_text) = calculate_row_skew(node.rows_planned, node.rows_actual) {
                let badge_galley = ui
                    .painter()
                    .layout_no_wrap(skew_text.clone(), font_caption(), self.theme.warning);
                let badge_rect = Rect::from_min_size(
                    Pos2::new(ui.cursor().min.x, ui.cursor().min.y + EXPLAIN_BADGE_PAD_Y),
                    Vec2::new(badge_galley.size().x + EXPLAIN_SKEW_BADGE_PAD_X, EXPLAIN_BADGE_HEIGHT),
                );
                ui.painter()
                    .rect_filled(badge_rect, Rounding::same(RADIUS_XS), self.theme.warning_soft());
                ui.painter().galley(
                    Pos2::new(
                        badge_rect.left() + EXPLAIN_SKEW_BADGE_TEXT_PAD_X,
                        badge_rect.top() + EXPLAIN_BADGE_TEXT_TOP_OFFSET,
                    ),
                    badge_galley,
                    self.theme.warning,
                );
                label_painted_badge(ui, badge_rect, node_id.with("skew_badge"), &skew_text);
                ui.add_space(badge_rect.width() + SPACE_XS);
            }

            // Time & rows on right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let stat_str = if self.has_runtime_stats {
                    node_runtime_stat_text(
                        node.actual_time_ms,
                        metrics.percentage,
                        node.rows_actual,
                        node.actual_loops,
                    )
                } else {
                    node_stat_text(
                        false,
                        node.actual_time_ms,
                        node.cost_estimate,
                        metrics.percentage,
                        node.rows_actual,
                    )
                };
                ui.label(
                    RichText::new(stat_str)
                        .size(FONT_SIZE_CAPTION)
                        .monospace()
                        .color(self.theme.text_tertiary),
                );
            });
        });

        // Findings / advice for this specific node
        for finding in &node.findings {
            ui.horizontal(|ui| {
                ui.add_space(indent + EXPLAIN_FINDING_INDENT_OFFSET);
                ui.label(
                    RichText::new(format!("↳ {finding}"))
                        .size(FONT_SIZE_CAPTION)
                        .color(self.theme.warning),
                );
            });
        }

        ui.add_space(SPACE_XXS);

        for (child_index, child) in node.children.iter().enumerate() {
            self.render_node(ui, child, depth + 1, node_id.with(child_index));
        }
    }
}

fn label_painted_badge(ui: &mut Ui, rect: Rect, id: Id, label: &str) {
    // Painter-only text is not exposed to accessibility, so register a hover-only label on the same bounds.
    ui.interact(rect, id, Sense::hover())
        .widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, label));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explain_tree_renders_runtime_badges_without_panicking() {
        let theme = DbProTheme::light();
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let child = PlanNode::new("Seq Scan", 30.0, 1.0, 10)
            .relation("users")
            .bottleneck(true);
        let root = PlanNode::new("Nested Loop", 40.0, 2.0, 10)
            .with_child(child.clone())
            .with_child(child);

        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let response = ExplainPlanTree::new(&root, 2.0, theme).has_runtime_stats(true).show(ui);
                assert!(response.rect.width() > 0.0);
                assert!(response.rect.height() > 0.0);
            });
        });
    }
}
