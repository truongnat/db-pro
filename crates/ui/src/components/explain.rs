//! Query Explain Plan and Performance diagnostics components.
//!
//! Implements ExplainPlanTree, PlanNode, PlanCost, BottleneckBadge,
//! and IndexRecommendation per `open-ai-refer.md`.

use crate::tokens::*;
use crate::DbProTheme;
use egui::{Color32, Pos2, Rect, Response, RichText, Rounding, Stroke, Ui, Vec2};
use lucide_icons::Icon;

#[derive(Debug, Clone)]
pub struct PlanNode {
    pub node_type: String,
    pub relation: Option<String>,
    pub index_name: Option<String>,
    pub cost_estimate: f32,
    pub startup_cost: Option<f32>,
    pub total_cost: Option<f32>,
    pub actual_time_ms: f32,
    pub actual_startup_ms: Option<f32>,
    pub rows_actual: usize,
    pub rows_planned: Option<usize>,
    pub actual_loops: Option<usize>,
    pub shared_hit_blocks: Option<usize>,
    pub shared_read_blocks: Option<usize>,
    pub is_bottleneck: bool,
    pub findings: Vec<String>,
    pub children: Vec<PlanNode>,
}

impl PlanNode {
    pub fn new(node_type: impl Into<String>, cost: f32, time_ms: f32, rows: usize) -> Self {
        Self {
            node_type: node_type.into(),
            relation: None,
            index_name: None,
            cost_estimate: cost,
            startup_cost: None,
            total_cost: Some(cost),
            actual_time_ms: time_ms,
            actual_startup_ms: None,
            rows_actual: rows,
            rows_planned: None,
            actual_loops: None,
            shared_hit_blocks: None,
            shared_read_blocks: None,
            is_bottleneck: false,
            findings: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn from_query_plan(node: &db_pro_core::domain::explain_plan::QueryPlanNode) -> Self {
        let is_bottleneck = node.findings.iter().any(|f| {
            matches!(
                f.severity,
                db_pro_core::domain::explain_plan::PlanFindingSeverity::Hotspot
            )
        });
        let findings = node.findings.iter().map(|f| f.message.clone()).collect();
        let mut ui_node = Self {
            node_type: node.node_type.clone(),
            relation: node.relation.clone(),
            index_name: node.index_name.clone(),
            cost_estimate: node.total_cost.unwrap_or(0.0) as f32,
            startup_cost: node.startup_cost.map(|c| c as f32),
            total_cost: node.total_cost.map(|c| c as f32),
            actual_time_ms: node.actual_total_ms.unwrap_or(0.0) as f32,
            actual_startup_ms: node.actual_startup_ms.map(|t| t as f32),
            rows_actual: node.actual_rows.or(node.plan_rows).unwrap_or(0.0).max(0.0) as usize,
            rows_planned: node.plan_rows.map(|r| r.max(0.0) as usize),
            actual_loops: node.actual_loops.map(|l| l.max(0.0) as usize),
            shared_hit_blocks: node.shared_hit_blocks.map(|b| b.max(0.0) as usize),
            shared_read_blocks: node.shared_read_blocks.map(|b| b.max(0.0) as usize),
            is_bottleneck,
            findings,
            children: node.children.iter().map(Self::from_query_plan).collect(),
        };
        if ui_node.actual_time_ms == 0.0 {
            ui_node.actual_time_ms = ui_node.cost_estimate;
        }
        ui_node
    }

    pub fn relation(mut self, rel: impl Into<String>) -> Self {
        self.relation = Some(rel.into());
        self
    }

    pub fn index_name(mut self, idx: impl Into<String>) -> Self {
        self.index_name = Some(idx.into());
        self
    }

    pub fn bottleneck(mut self, is_bottleneck: bool) -> Self {
        self.is_bottleneck = is_bottleneck;
        self
    }

    pub fn with_child(mut self, child: PlanNode) -> Self {
        self.children.push(child);
        self
    }
}

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
                            Pos2::new(ui.cursor().min.x, ui.cursor().min.y + 2.0),
                            Vec2::new(badge_galley.size().x + 8.0, 16.0),
                        );
                        ui.painter().rect_filled(
                            badge_rect,
                            Rounding::same(RADIUS_XS),
                            self.theme.warning_soft(),
                        );
                        ui.painter().galley(
                            Pos2::new(badge_rect.left() + 4.0, badge_rect.top() + 1.0),
                            badge_galley,
                            Color32::PLACEHOLDER,
                        );
                        ui.add_space(badge_rect.width() + 4.0);
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

                self.render_node(ui, self.root, 0);
            })
            .response
    }

    fn render_node(&self, ui: &mut Ui, node: &PlanNode, depth: usize) {
        let indent = depth as f32 * 18.0;

        // Calculate cost / time percentage for Flame Tree bar
        let node_val = if self.has_runtime_stats {
            node.actual_time_ms
        } else {
            node.cost_estimate
        };
        let pct = if self.total_time_ms > 0.0 {
            (node_val / self.total_time_ms).clamp(0.0, 1.0)
        } else {
            0.0
        };

        ui.horizontal(|ui| {
            if indent > 0.0 {
                ui.add_space(indent);
            }

            // Node Icon
            let icon = if node.is_bottleneck {
                Icon::Flame
            } else if node.relation.is_some() {
                Icon::Table
            } else if node.index_name.is_some() {
                Icon::KeyRound
            } else {
                Icon::GitCommitVertical
            };

            let icon_color = if node.is_bottleneck {
                self.theme.danger
            } else if node.index_name.is_some() {
                self.theme.accent
            } else {
                self.theme.text_secondary
            };

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
            let bar_width = 44.0;
            let bar_height = 5.0;
            let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, bar_height), egui::Sense::hover());
            ui.painter().rect_filled(
                bar_rect,
                Rounding::same(2.0),
                self.theme.surface_active,
            );
            if pct > 0.01 {
                let fill_rect = Rect::from_min_size(
                    bar_rect.min,
                    Vec2::new(bar_width * pct, bar_height),
                );
                let bar_color = if pct > 0.45 || node.is_bottleneck {
                    self.theme.danger
                } else if pct > 0.20 {
                    self.theme.warning
                } else {
                    self.theme.success
                };
                ui.painter().rect_filled(fill_rect, Rounding::same(2.0), bar_color);
            }

            // Bottleneck badge
            if node.is_bottleneck {
                ui.add_space(SPACE_XXS);
                let badge_galley =
                    ui.painter()
                        .layout_no_wrap("Hotspot".to_owned(), font_caption(), self.theme.danger);
                let badge_rect = Rect::from_min_size(
                    Pos2::new(ui.cursor().min.x, ui.cursor().min.y + 2.0),
                    Vec2::new(badge_galley.size().x + 8.0, 16.0),
                );
                let fill = self.theme.danger_soft();
                ui.painter().rect_filled(badge_rect, Rounding::same(RADIUS_XS), fill);
                ui.painter().galley(
                    Pos2::new(badge_rect.left() + 4.0, badge_rect.top() + 1.0),
                    badge_galley,
                    Color32::PLACEHOLDER,
                );
                ui.add_space(badge_rect.width() + 4.0);
            }

            // Estimate skew warning badge (Actual rows vs Planned rows mismatch)
            if let (Some(planned), actual) = (node.rows_planned, node.rows_actual) {
                if planned > 0 && actual > 0 {
                    let ratio = (actual as f64) / (planned as f64);
                    if !(0.1..=10.0).contains(&ratio) {
                        let skew_text = if ratio > 1.0 {
                            format!("{:.0}x rows skew", ratio)
                        } else {
                            format!("{:.1}x rows skew", ratio)
                        };
                        let badge_galley = ui.painter().layout_no_wrap(skew_text, font_caption(), self.theme.warning);
                        let badge_rect = Rect::from_min_size(
                            Pos2::new(ui.cursor().min.x, ui.cursor().min.y + 2.0),
                            Vec2::new(badge_galley.size().x + 6.0, 16.0),
                        );
                        ui.painter().rect_filled(badge_rect, Rounding::same(RADIUS_XS), self.theme.warning_soft());
                        ui.painter().galley(
                            Pos2::new(badge_rect.left() + 3.0, badge_rect.top() + 1.0),
                            badge_galley,
                            Color32::PLACEHOLDER,
                        );
                        ui.add_space(badge_rect.width() + 4.0);
                    }
                }
            }

            // Time & rows on right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let stat_str = if self.has_runtime_stats {
                    format!("{:.2}ms ({:.0}%) · {} rows", node.actual_time_ms, pct * 100.0, node.rows_actual)
                } else {
                    format!("cost {:.1} ({:.0}%) · {} rows", node.cost_estimate, pct * 100.0, node.rows_actual)
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
                ui.add_space(indent + 24.0);
                ui.label(
                    RichText::new(format!("↳ {finding}"))
                        .size(FONT_SIZE_CAPTION)
                        .color(self.theme.warning),
                );
            });
        }

        ui.add_space(SPACE_XXS);

        for child in &node.children {
            self.render_node(ui, child, depth + 1);
        }
    }
}
