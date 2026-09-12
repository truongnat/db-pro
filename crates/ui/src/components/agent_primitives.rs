use crate::DbProTheme;
use egui::{Align2, Color32, FontFamily, FontId, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use lucide_icons::Icon;

// ── ContextChip & AgentContextBar ────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextChipKind {
    Connection,
    Database,
    Schema,
    Table,
    Editor,
    File,
}

impl ContextChipKind {
    pub fn icon(&self) -> Icon {
        match self {
            ContextChipKind::Connection => Icon::Server,
            ContextChipKind::Database => Icon::Database,
            ContextChipKind::Schema => Icon::Folder,
            ContextChipKind::Table => Icon::Table,
            ContextChipKind::Editor => Icon::Code,
            ContextChipKind::File => Icon::FileText,
        }
    }
}

pub struct ContextChip<'a> {
    kind: ContextChipKind,
    label: &'a str,
    removable: bool,
    theme: DbProTheme,
}

impl<'a> ContextChip<'a> {
    pub fn new(kind: ContextChipKind, label: &'a str, theme: DbProTheme) -> Self {
        Self {
            kind,
            label,
            removable: false,
            theme,
        }
    }

    pub fn removable(mut self, removable: bool) -> Self {
        self.removable = removable;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let text_font = FontId::proportional(11.5);
        let icon_font = FontId::new(13.0, egui::FontFamily::Name("lucide".into()));
        let icon_char = char::from(self.kind.icon()).to_string();
        let icon_galley = ui
            .painter()
            .layout_no_wrap(icon_char, icon_font, self.theme.text_secondary);
        let text_galley = ui
            .painter()
            .layout_no_wrap(self.label.to_owned(), text_font, self.theme.text_primary);

        let icon_size = icon_galley.size();
        let rem_w = if self.removable { 14.0 } else { 0.0 };
        let width = icon_size.x + 6.0 + text_galley.size().x + rem_w + 16.0;
        let height = 22.0;

        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
        let is_hovered = response.hovered();

        let fill = if is_hovered {
            self.theme.surface_hover
        } else {
            self.theme.surface_editor
        };
        let stroke = if is_hovered {
            Stroke::new(1.0, self.theme.border_strong)
        } else {
            Stroke::new(1.0, self.theme.border_default)
        };

        ui.painter().rect_filled(rect, Rounding::same(11.0), fill);
        ui.painter().rect_stroke(rect, Rounding::same(11.0), stroke);

        // Icon
        let icon_pos = Pos2::new(rect.left() + 8.0, rect.center().y - icon_size.y * 0.5);
        ui.painter().galley(icon_pos, icon_galley, Color32::PLACEHOLDER);

        // Text
        let text_pos = Pos2::new(
            rect.left() + 8.0 + icon_size.x + 6.0,
            rect.center().y - text_galley.size().y * 0.5,
        );
        ui.painter().galley(text_pos, text_galley, Color32::PLACEHOLDER);

        // Removable cross
        if self.removable {
            let cross_pos = Pos2::new(rect.right() - 11.0, rect.center().y);
            ui.painter().text(
                cross_pos,
                Align2::CENTER_CENTER,
                char::from(Icon::X).to_string(),
                FontId::new(10.5, FontFamily::Name("lucide".into())),
                self.theme.text_tertiary,
            );
        }

        response
    }
}

// ── StatusBadge ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusBadgeVariant {
    Active,
    Running,
    Success,
    Warning,
    Destructive,
    Archived,
    Draft,
}

pub struct StatusBadge<'a> {
    text: &'a str,
    variant: StatusBadgeVariant,
    theme: DbProTheme,
}

impl<'a> StatusBadge<'a> {
    pub fn new(text: &'a str, variant: StatusBadgeVariant, theme: DbProTheme) -> Self {
        Self { text, variant, theme }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (dot_color, text_color, bg_color) = match self.variant {
            StatusBadgeVariant::Active | StatusBadgeVariant::Success => {
                (self.theme.success, self.theme.success, self.theme.success_soft())
            }
            StatusBadgeVariant::Running => (self.theme.info, self.theme.info, self.theme.info_soft()),
            StatusBadgeVariant::Warning => (self.theme.warning, self.theme.warning, self.theme.warning_soft()),
            StatusBadgeVariant::Destructive => (self.theme.danger, self.theme.danger, self.theme.danger_soft()),
            StatusBadgeVariant::Archived | StatusBadgeVariant::Draft => (
                self.theme.text_tertiary,
                self.theme.text_secondary,
                self.theme.surface_hover,
            ),
        };

        let font_id = DbProTheme::ui_medium_font(12.0);
        let text_galley = ui.painter().layout_no_wrap(self.text.to_owned(), font_id, text_color);
        let dot_w = 5.0;
        let spacing = 4.0;
        let pad_h = 8.0;
        let pad_v = 2.0;
        let total_w = pad_h * 2.0 + dot_w + spacing + text_galley.size().x;
        let total_h = (text_galley.size().y + pad_v * 2.0).max(20.0);

        let (rect, response) = ui.allocate_exact_size(Vec2::new(total_w, total_h), Sense::hover());
        ui.painter().rect_filled(rect, Rounding::same(total_h * 0.5), bg_color);
        ui.painter().rect_stroke(
            rect,
            Rounding::same(total_h * 0.5),
            Stroke::new(1.0, dot_color.linear_multiply(0.35)),
        );

        let dot_center = Pos2::new(rect.left() + pad_h + dot_w * 0.5, rect.center().y);
        ui.painter().circle_filled(dot_center, 2.5, dot_color);

        let text_pos = Pos2::new(
            rect.left() + pad_h + dot_w + spacing,
            rect.center().y - text_galley.size().y * 0.5,
        );
        ui.painter().galley(text_pos, text_galley, Color32::PLACEHOLDER);

        response
    }
}

// ── ToolCall Component ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCallStatus {
    Running,
    Success,
    Failed,
}

pub struct ToolCall<'a> {
    tool_name: &'a str,
    duration: Option<&'a str>,
    status: ToolCallStatus,
    input_preview: &'a str,
    output_preview: Option<&'a str>,
    expanded: &'a mut bool,
    theme: DbProTheme,
}

impl<'a> ToolCall<'a> {
    pub fn new(
        tool_name: &'a str,
        status: ToolCallStatus,
        input_preview: &'a str,
        expanded: &'a mut bool,
        theme: DbProTheme,
    ) -> Self {
        Self {
            tool_name,
            duration: None,
            status,
            input_preview,
            output_preview: None,
            expanded,
            theme,
        }
    }

    pub fn duration(mut self, dur: &'a str) -> Self {
        self.duration = Some(dur);
        self
    }

    pub fn output_preview(mut self, out: &'a str) -> Self {
        self.output_preview = Some(out);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let is_expanded = *self.expanded;
        let frame = egui::Frame::none()
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(1.0, self.theme.border_default))
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::same(0.0));

        frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // ── Header Bar ──────────────────────────────────────────────
                let (header_rect, header_resp) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 32.0), Sense::click());
                if header_resp.hovered() {
                    ui.painter()
                        .rect_filled(header_rect, Rounding::same(8.0), self.theme.surface_hover);
                }

                let chevron_icon = if is_expanded {
                    Icon::ChevronDown
                } else {
                    Icon::ChevronRight
                };
                ui.painter().text(
                    Pos2::new(header_rect.left() + 14.0, header_rect.center().y),
                    Align2::CENTER_CENTER,
                    char::from(chevron_icon).to_string(),
                    FontId::new(10.5, FontFamily::Name("lucide".into())),
                    self.theme.text_secondary,
                );

                // Tool Icon & Name
                ui.painter().text(
                    Pos2::new(header_rect.left() + 30.0, header_rect.center().y),
                    Align2::CENTER_CENTER,
                    char::from(Icon::Terminal).to_string(),
                    FontId::new(13.0, FontFamily::Name("lucide".into())),
                    self.theme.text_secondary,
                );

                ui.painter().text(
                    Pos2::new(header_rect.left() + 44.0, header_rect.center().y),
                    Align2::LEFT_CENTER,
                    self.tool_name,
                    FontId::monospace(12.0),
                    self.theme.text_primary,
                );

                // Duration & Status on right
                let mut right_cursor = header_rect.right() - 10.0;
                let status_badge_text = match self.status {
                    ToolCallStatus::Running => "running",
                    ToolCallStatus::Success => "done",
                    ToolCallStatus::Failed => "failed",
                };
                let badge_font = FontId::proportional(10.5);
                let badge_galley =
                    ui.painter()
                        .layout_no_wrap(status_badge_text.to_owned(), badge_font, self.theme.text_secondary);
                right_cursor -= badge_galley.size().x + 12.0;

                let badge_rect = Rect::from_min_size(
                    Pos2::new(right_cursor, header_rect.center().y - 9.0),
                    Vec2::new(badge_galley.size().x + 12.0, 18.0),
                );
                let badge_fill = match self.status {
                    ToolCallStatus::Success => self.theme.success.linear_multiply(0.15),
                    ToolCallStatus::Running => self.theme.info.linear_multiply(0.15),
                    ToolCallStatus::Failed => self.theme.danger.linear_multiply(0.15),
                };
                ui.painter().rect_filled(badge_rect, Rounding::same(9.0), badge_fill);
                ui.painter().galley(
                    Pos2::new(badge_rect.left() + 6.0, badge_rect.top() + 2.0),
                    badge_galley,
                    Color32::PLACEHOLDER,
                );

                if let Some(dur) = self.duration {
                    let dur_galley =
                        ui.painter()
                            .layout_no_wrap(dur.to_owned(), FontId::monospace(11.0), self.theme.text_tertiary);
                    right_cursor -= dur_galley.size().x + 12.0;
                    ui.painter().galley(
                        Pos2::new(right_cursor, header_rect.center().y - 6.0),
                        dur_galley,
                        Color32::PLACEHOLDER,
                    );
                }

                if header_resp.clicked() {
                    *self.expanded = !*self.expanded;
                }

                // ── Collapsible Body ────────────────────────────────────────
                if is_expanded {
                    ui.painter().hline(
                        header_rect.x_range(),
                        header_rect.bottom(),
                        Stroke::new(1.0, self.theme.border_subtle),
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("INPUT PARAMETERS")
                                    .size(10.5)
                                    .strong()
                                    .color(self.theme.text_tertiary),
                            );
                            ui.add_space(2.0);
                            let code_frame = egui::Frame::none()
                                .fill(self.theme.surface_editor)
                                .rounding(Rounding::same(6.0))
                                .stroke(Stroke::new(1.0, self.theme.border_subtle))
                                .inner_margin(egui::Margin::same(8.0));
                            code_frame.show(ui, |ui| {
                                ui.set_width(ui.available_width() - 16.0);
                                ui.label(
                                    RichText::new(self.input_preview)
                                        .monospace()
                                        .size(11.5)
                                        .color(self.theme.text_primary),
                                );
                            });

                            if let Some(out) = self.output_preview {
                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new("OUTPUT RESULT")
                                        .size(10.5)
                                        .strong()
                                        .color(self.theme.text_tertiary),
                                );
                                ui.add_space(2.0);
                                let out_frame = egui::Frame::none()
                                    .fill(self.theme.surface_editor)
                                    .rounding(Rounding::same(6.0))
                                    .stroke(Stroke::new(1.0, self.theme.border_subtle))
                                    .inner_margin(egui::Margin::same(8.0));
                                out_frame.show(ui, |ui| {
                                    ui.set_width(ui.available_width() - 16.0);
                                    ui.label(
                                        RichText::new(out)
                                            .monospace()
                                            .size(11.5)
                                            .color(self.theme.text_secondary),
                                    );
                                });
                            }
                        });
                        ui.add_space(12.0);
                    });
                    ui.add_space(8.0);
                }
            })
            .response
    }
}

// ── ExecutionApproval Component ──────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Destructive,
}

pub struct ExecutionApproval<'a> {
    title: &'a str,
    impact: &'a str,
    sql_preview: &'a str,
    risk: RiskLevel,
    theme: DbProTheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionApprovalAction {
    Run,
    Preview,
    Cancel,
}

impl<'a> ExecutionApproval<'a> {
    pub fn new(title: &'a str, impact: &'a str, sql_preview: &'a str, risk: RiskLevel, theme: DbProTheme) -> Self {
        Self {
            title,
            impact,
            sql_preview,
            risk,
            theme,
        }
    }

    pub fn show(self, ui: &mut Ui) -> Option<ExecutionApprovalAction> {
        let frame = egui::Frame::none()
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(1.0, self.theme.border_default))
            .rounding(Rounding::same(10.0))
            .inner_margin(egui::Margin::same(14.0));

        let mut chosen_action = None;

        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Header with Title & Risk Badge
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(self.title)
                        .size(13.5)
                        .strong()
                        .color(self.theme.text_primary),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (risk_text, risk_var) = match self.risk {
                        RiskLevel::Low => ("Low Risk", StatusBadgeVariant::Success),
                        RiskLevel::Medium => ("Medium Risk", StatusBadgeVariant::Warning),
                        RiskLevel::High => ("High Risk", StatusBadgeVariant::Destructive),
                        RiskLevel::Destructive => ("Destructive", StatusBadgeVariant::Destructive),
                    };
                    StatusBadge::new(risk_text, risk_var, self.theme).show(ui);
                });
            });

            ui.add_space(6.0);

            // Impact Statement
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Impact:")
                        .size(12.0)
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.label(RichText::new(self.impact).size(12.0).color(self.theme.text_primary));
            });

            ui.add_space(8.0);

            // SQL Code Preview Frame
            let code_frame = egui::Frame::none()
                .fill(self.theme.surface_editor)
                .stroke(Stroke::new(1.0, self.theme.border_subtle))
                .rounding(Rounding::same(6.0))
                .inner_margin(egui::Margin::same(10.0));

            code_frame.show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(self.sql_preview)
                        .monospace()
                        .size(12.0)
                        .color(self.theme.text_primary),
                );
            });

            ui.add_space(12.0);

            // Action Buttons Bar
            ui.horizontal(|ui| {
                if crate::components::Button::new(self.theme)
                    .text("Run Changes")
                    .variant(crate::components::ButtonVariant::Default)
                    .size(crate::components::ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    chosen_action = Some(ExecutionApprovalAction::Run);
                }

                ui.add_space(4.0);

                if crate::components::Button::new(self.theme)
                    .text("Preview SQL")
                    .variant(crate::components::ButtonVariant::Outline)
                    .size(crate::components::ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    chosen_action = Some(ExecutionApprovalAction::Preview);
                }

                ui.add_space(4.0);

                if crate::components::Button::new(self.theme)
                    .text("Cancel")
                    .variant(crate::components::ButtonVariant::Ghost)
                    .size(crate::components::ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    chosen_action = Some(ExecutionApprovalAction::Cancel);
                }
            });
        });

        chosen_action
    }
}

// ── AgentThinking Component ──────────────────────────────────────────────────

pub struct AgentThinking<'a> {
    thought: &'a str,
    duration: Option<&'a str>,
    step_count: Option<usize>,
    is_active: bool,
    expanded: &'a mut bool,
    theme: DbProTheme,
}

impl<'a> AgentThinking<'a> {
    pub fn new(thought: &'a str, expanded: &'a mut bool, theme: DbProTheme) -> Self {
        Self {
            thought,
            duration: None,
            step_count: None,
            is_active: false,
            expanded,
            theme,
        }
    }

    pub fn duration(mut self, duration: &'a str) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn step_count(mut self, count: usize) -> Self {
        self.step_count = Some(count);
        self
    }

    pub fn is_active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let is_expanded = *self.expanded;
        let is_active = self.is_active;

        let frame = egui::Frame::none()
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(1.0, self.theme.border_subtle))
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::symmetric(10.0, 6.0));

        let frame_resp = frame.show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Header line
            let (header_rect, header_resp) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 24.0), Sense::click());

            if header_resp.hovered() {
                ui.painter()
                    .rect_filled(header_rect, Rounding::same(6.0), self.theme.surface_hover);
            }

            // Chevron
            let chevron_icon = if is_expanded {
                Icon::ChevronDown
            } else {
                Icon::ChevronRight
            };
            ui.painter().text(
                Pos2::new(header_rect.left() + 8.0, header_rect.center().y),
                Align2::CENTER_CENTER,
                char::from(chevron_icon).to_string(),
                FontId::new(10.0, FontFamily::Name("lucide".into())),
                self.theme.text_secondary,
            );

            // Icon: Sparkles or Spinner
            let icon_x = header_rect.left() + 22.0;
            if is_active {
                let time = ui.input(|i| i.time);
                let spin_angle = (time * 6.0) as f32;
                ui.ctx().request_repaint();
                ui.painter().text(
                    Pos2::new(icon_x, header_rect.center().y),
                    Align2::CENTER_CENTER,
                    char::from(Icon::LoaderCircle).to_string(),
                    FontId::new(12.0, FontFamily::Name("lucide".into())),
                    self.theme.accent,
                );
                let _ = spin_angle;
            } else {
                ui.painter().text(
                    Pos2::new(icon_x, header_rect.center().y),
                    Align2::CENTER_CENTER,
                    char::from(Icon::Brain).to_string(),
                    FontId::new(12.0, FontFamily::Name("lucide".into())),
                    self.theme.text_secondary,
                );
            }

            // Label
            let title_text = if is_active {
                "Thinking...".to_owned()
            } else {
                let mut s = "Thought".to_owned();
                if let Some(dur) = self.duration {
                    s.push_str(&format!(" for {}", dur));
                }
                if let Some(steps) = self.step_count {
                    s.push_str(&format!(" ({} step{})", steps, if steps > 1 { "s" } else { "" }));
                }
                s
            };

            ui.painter().text(
                Pos2::new(header_rect.left() + 36.0, header_rect.center().y),
                Align2::LEFT_CENTER,
                title_text,
                FontId::proportional(11.5),
                if is_active {
                    self.theme.accent
                } else {
                    self.theme.text_secondary
                },
            );

            if header_resp.clicked() {
                *self.expanded = !*self.expanded;
            }

            // Expanded thought body
            if is_expanded {
                ui.add_space(4.0);
                let body_frame = egui::Frame::none()
                    .fill(self.theme.surface_editor)
                    .stroke(Stroke::new(1.0, self.theme.border_subtle))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(egui::Margin::same(8.0));

                body_frame.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(self.thought)
                            .size(11.5)
                            .monospace()
                            .color(self.theme.text_secondary),
                    );
                });
            }
        });

        frame_resp.response
    }
}

// ── AgentPlan & Task Checklist ───────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct AgentTaskItem {
    pub title: String,
    pub status: AgentTaskStatus,
    pub detail: Option<String>,
}

impl AgentTaskItem {
    pub fn new(title: impl Into<String>, status: AgentTaskStatus) -> Self {
        Self {
            title: title.into(),
            status,
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

pub struct AgentPlan<'a> {
    title: &'a str,
    tasks: &'a [AgentTaskItem],
    theme: DbProTheme,
}

impl<'a> AgentPlan<'a> {
    pub fn new(title: &'a str, tasks: &'a [AgentTaskItem], theme: DbProTheme) -> Self {
        Self { title, tasks, theme }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let total = self.tasks.len();
        let completed = self
            .tasks
            .iter()
            .filter(|t| t.status == AgentTaskStatus::Completed)
            .count();
        let progress = if total > 0 {
            completed as f32 / total as f32
        } else {
            0.0
        };

        let frame = egui::Frame::none()
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(1.0, self.theme.border_default))
            .rounding(Rounding::same(8.0))
            .inner_margin(egui::Margin::same(12.0));

        let frame_resp = frame.show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Header: Title & completion badge
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(self.title)
                        .size(13.0)
                        .strong()
                        .color(self.theme.text_primary),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{}/{} completed", completed, total))
                            .size(11.5)
                            .color(self.theme.text_secondary),
                    );
                });
            });

            ui.add_space(6.0);

            // Progress bar
            let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 4.0), Sense::hover());
            ui.painter()
                .rect_filled(bar_rect, Rounding::same(2.0), self.theme.surface_hover);
            if progress > 0.0 {
                let filled_rect =
                    Rect::from_min_size(bar_rect.min, Vec2::new(bar_rect.width() * progress, bar_rect.height()));
                ui.painter()
                    .rect_filled(filled_rect, Rounding::same(2.0), self.theme.accent);
            }

            ui.add_space(10.0);

            // Task items list
            for (idx, task) in self.tasks.iter().enumerate() {
                ui.horizontal(|ui| {
                    let (icon, icon_color) = match task.status {
                        AgentTaskStatus::Completed => (Icon::CheckCircle2, self.theme.success),
                        AgentTaskStatus::Running => {
                            ui.ctx().request_repaint();
                            (Icon::LoaderCircle, self.theme.info)
                        }
                        AgentTaskStatus::Pending => (Icon::Circle, self.theme.text_tertiary),
                        AgentTaskStatus::Failed => (Icon::AlertCircle, self.theme.danger),
                        AgentTaskStatus::Skipped => (Icon::MinusCircle, self.theme.text_muted),
                    };

                    ui.label(
                        RichText::new(char::from(icon).to_string())
                            .font(FontId::new(13.0, FontFamily::Name("lucide".into())))
                            .color(icon_color),
                    );

                    ui.add_space(2.0);

                    let title_color = match task.status {
                        AgentTaskStatus::Completed => self.theme.text_primary,
                        AgentTaskStatus::Running => self.theme.text_primary,
                        AgentTaskStatus::Pending => self.theme.text_secondary,
                        AgentTaskStatus::Failed => self.theme.danger,
                        AgentTaskStatus::Skipped => self.theme.text_disabled,
                    };

                    ui.label(
                        RichText::new(format!("{}. {}", idx + 1, &task.title))
                            .size(12.0)
                            .color(title_color),
                    );

                    if let Some(ref detail) = task.detail {
                        ui.label(
                            RichText::new(format!("({})", detail))
                                .size(11.0)
                                .color(self.theme.text_tertiary),
                        );
                    }
                });

                if idx + 1 < self.tasks.len() {
                    ui.add_space(6.0);
                }
            }
        });

        frame_resp.response
    }
}

// ── AgentSqlAction Chips ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentSqlActionKind {
    Explain,
    Optimize,
    FixError,
    GenerateMigration,
    DescribeSchema,
    ConvertDialect,
}

impl AgentSqlActionKind {
    pub fn label(&self) -> &'static str {
        match self {
            AgentSqlActionKind::Explain => "Explain Query",
            AgentSqlActionKind::Optimize => "Optimize Query",
            AgentSqlActionKind::FixError => "Fix Error",
            AgentSqlActionKind::GenerateMigration => "Generate Migration",
            AgentSqlActionKind::DescribeSchema => "Describe Schema",
            AgentSqlActionKind::ConvertDialect => "Convert Dialect",
        }
    }

    pub fn icon(&self) -> Icon {
        match self {
            AgentSqlActionKind::Explain => Icon::ChartNoAxesCombined,
            AgentSqlActionKind::Optimize => Icon::Gauge,
            AgentSqlActionKind::FixError => Icon::Wrench,
            AgentSqlActionKind::GenerateMigration => Icon::GitFork,
            AgentSqlActionKind::DescribeSchema => Icon::FileSpreadsheet,
            AgentSqlActionKind::ConvertDialect => Icon::RefreshCw,
        }
    }
}
