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
    pub cost_estimate: f32,
    pub actual_time_ms: f32,
    pub rows_actual: usize,
    pub is_bottleneck: bool,
    pub children: Vec<PlanNode>,
}

impl PlanNode {
    pub fn new(node_type: impl Into<String>, cost: f32, time_ms: f32, rows: usize) -> Self {
        Self {
            node_type: node_type.into(),
            relation: None,
            cost_estimate: cost,
            actual_time_ms: time_ms,
            rows_actual: rows,
            is_bottleneck: false,
            children: Vec::new(),
        }
    }

    pub fn relation(mut self, rel: impl Into<String>) -> Self {
        self.relation = Some(rel.into());
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
}

impl<'a> ExplainPlanTree<'a> {
    pub fn new(root: &'a PlanNode, total_time_ms: f32, theme: DbProTheme) -> Self {
        Self {
            root,
            total_time_ms,
            theme,
        }
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

                // Header
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Execution Plan Analysis")
                            .size(FONT_SIZE_UI_LABEL)
                            .strong()
                            .color(self.theme.text_primary),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("Total: {:.2}ms", self.total_time_ms))
                                .size(FONT_SIZE_CAPTION)
                                .monospace()
                                .color(self.theme.text_secondary),
                        );
                    });
                });

                ui.add_space(SPACE_SM);

                self.render_node(ui, self.root, 0);
            })
            .response
    }

    fn render_node(&self, ui: &mut Ui, node: &PlanNode, depth: usize) {
        let indent = depth as f32 * 20.0;

        ui.horizontal(|ui| {
            ui.add_space(indent);

            // Node Icon
            let icon = if node.is_bottleneck {
                Icon::Flame
            } else if node.relation.is_some() {
                Icon::Table
            } else {
                Icon::GitCommitVertical
            };

            let icon_color = if node.is_bottleneck {
                self.theme.danger
            } else {
                self.theme.text_secondary
            };

            ui.label(
                RichText::new(char::from(icon).to_string())
                    .font(font_icon(ICON_SM))
                    .color(icon_color),
            );

            // Node title & relation
            ui.label(
                RichText::new(&node.node_type)
                    .size(FONT_SIZE_BODY_SM)
                    .strong()
                    .color(self.theme.text_primary),
            );

            if let Some(ref rel) = node.relation {
                ui.label(
                    RichText::new(format!("on {}", rel))
                        .size(FONT_SIZE_CAPTION)
                        .color(self.theme.text_secondary),
                );
            }

            // Bottleneck badge
            if node.is_bottleneck {
                ui.add_space(SPACE_XS);
                let badge_galley =
                    ui.painter()
                        .layout_no_wrap("Bottleneck".to_owned(), font_caption(), self.theme.danger);
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

            // Time & rows on right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{:.2}ms · {} rows", node.actual_time_ms, node.rows_actual))
                        .size(FONT_SIZE_CAPTION)
                        .monospace()
                        .color(self.theme.text_tertiary),
                );
            });
        });

        ui.add_space(SPACE_XXS);

        for child in &node.children {
            self.render_node(ui, child, depth + 1);
        }
    }
}
