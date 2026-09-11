use crate::DbProTheme;
use egui::{Color32, FontId, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
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
            let cross_pos = Pos2::new(rect.right() - 14.0, rect.center().y - 6.0);
            let cross_galley =
                ui.painter()
                    .layout_no_wrap("×".to_owned(), FontId::proportional(12.0), self.theme.text_tertiary);
            ui.painter().galley(cross_pos, cross_galley, Color32::PLACEHOLDER);
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
            StatusBadgeVariant::Active | StatusBadgeVariant::Success => (
                self.theme.success,
                self.theme.success,
                if self.theme.dark_mode {
                    Color32::from_rgba_premultiplied(34, 197, 94, 25)
                } else {
                    Color32::from_rgba_premultiplied(22, 163, 74, 18)
                },
            ),
            StatusBadgeVariant::Running => (
                self.theme.info,
                self.theme.info,
                if self.theme.dark_mode {
                    Color32::from_rgba_premultiplied(59, 130, 246, 25)
                } else {
                    Color32::from_rgba_premultiplied(37, 99, 235, 18)
                },
            ),
            StatusBadgeVariant::Warning => (
                self.theme.warning,
                self.theme.warning,
                if self.theme.dark_mode {
                    Color32::from_rgba_premultiplied(245, 158, 11, 25)
                } else {
                    Color32::from_rgba_premultiplied(217, 119, 6, 18)
                },
            ),
            StatusBadgeVariant::Destructive => (
                self.theme.danger,
                self.theme.danger,
                if self.theme.dark_mode {
                    Color32::from_rgba_premultiplied(239, 68, 68, 25)
                } else {
                    Color32::from_rgba_premultiplied(220, 38, 38, 18)
                },
            ),
            StatusBadgeVariant::Archived | StatusBadgeVariant::Draft => (
                self.theme.text_tertiary,
                self.theme.text_secondary,
                self.theme.surface_hover,
            ),
        };

        let font_id = FontId::proportional(11.0);
        let text_galley = ui.painter().layout_no_wrap(self.text.to_owned(), font_id, text_color);
        let dot_w = 6.0;
        let spacing = 5.0;
        let pad_h = 7.0;
        let pad_v = 3.0;
        let total_w = pad_h * 2.0 + dot_w + spacing + text_galley.size().x;
        let total_h = text_galley.size().y + pad_v * 2.0;

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

                let chevron_str = if is_expanded { "▾" } else { "▸" };
                let chev_galley = ui.painter().layout_no_wrap(
                    chevron_str.to_owned(),
                    FontId::proportional(12.0),
                    self.theme.text_secondary,
                );
                ui.painter().galley(
                    Pos2::new(header_rect.left() + 10.0, header_rect.center().y - 6.0),
                    chev_galley,
                    Color32::PLACEHOLDER,
                );

                // Tool Icon & Name
                let icon_font = FontId::new(13.0, egui::FontFamily::Name("lucide".into()));
                let icon_galley = ui.painter().layout_no_wrap(
                    char::from(Icon::Terminal).to_string(),
                    icon_font,
                    self.theme.text_secondary,
                );
                ui.painter().galley(
                    Pos2::new(header_rect.left() + 24.0, header_rect.center().y - 6.0),
                    icon_galley,
                    Color32::PLACEHOLDER,
                );

                let name_galley = ui.painter().layout_no_wrap(
                    self.tool_name.to_owned(),
                    FontId::monospace(12.0),
                    self.theme.text_primary,
                );
                ui.painter().galley(
                    Pos2::new(header_rect.left() + 42.0, header_rect.center().y - 6.0),
                    name_galley,
                    Color32::PLACEHOLDER,
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
