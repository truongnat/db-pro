// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use crate::tokens::font_icon;
use crate::DbProTheme;
use egui::{
    Align2, CornerRadius, FontId, Pos2, Rect, Response, RichText, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType,
};
use lucide_icons::Icon;

use super::{config, handler, AgentPlan, AgentSqlActionKind, AgentTaskItem};

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
        let header = handler::thinking_header(self.is_active, self.duration, self.step_count);
        let frame = egui::Frame::NONE
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(config::FRAME_BORDER_WIDTH, self.theme.border_subtle))
            .corner_radius(CornerRadius::same(config::THINKING_RADIUS as u8))
            .inner_margin(egui::Margin::symmetric(
                config::THINKING_HORIZONTAL_PADDING as i8,
                config::THINKING_VERTICAL_PADDING as i8,
            ));

        let frame_resp = frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            let (header_rect, header_resp) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), config::THINKING_HEADER_HEIGHT),
                Sense::click(),
            );
            let keyboard_toggle = if header_resp.has_focus() {
                ui.input_mut(|input| {
                    [egui::Key::Space, egui::Key::Enter]
                        .into_iter()
                        .any(|key| input.consume_key(egui::Modifiers::NONE, key))
                })
            } else {
                false
            };
            header_resp.widget_info(|| {
                WidgetInfo::selected(WidgetType::CollapsingHeader, true, *self.expanded, &header.title)
            });
            if header_resp.has_focus() {
                ui.painter().rect_stroke(
                    header_rect,
                    CornerRadius::same(config::THINKING_HEADER_RADIUS as u8),
                    Stroke::new(1.5, self.theme.border_strong),
                    egui::StrokeKind::Inside,
                );
            }
            if header_resp.hovered() {
                ui.painter().rect_filled(
                    header_rect,
                    CornerRadius::same(config::THINKING_HEADER_RADIUS as u8),
                    self.theme.surface_hover,
                );
            }

            // Egui supplies input/click signals; the handler supplied the icon and title.
            // This keeps expansion and copy/label rules independent from painting.
            ui.painter().text(
                Pos2::new(header_rect.left() + config::THINKING_CHEVRON_X, header_rect.center().y),
                Align2::CENTER_CENTER,
                char::from(if is_expanded {
                    Icon::ChevronDown
                } else {
                    Icon::ChevronRight
                })
                .to_string(),
                font_icon(config::ICON_SMALL_SIZE),
                self.theme.text_secondary,
            );
            let icon_color = if self.is_active {
                self.theme.accent
            } else {
                self.theme.text_secondary
            };
            ui.painter().text(
                Pos2::new(header_rect.left() + config::THINKING_ICON_X, header_rect.center().y),
                Align2::CENTER_CENTER,
                char::from(header.icon).to_string(),
                font_icon(config::ICON_STATUS_SIZE),
                icon_color,
            );
            ui.painter().text(
                Pos2::new(header_rect.left() + config::THINKING_TITLE_X, header_rect.center().y),
                Align2::LEFT_CENTER,
                header.title,
                FontId::proportional(config::THINKING_TEXT_SIZE),
                icon_color,
            );

            let should_toggle =
                handler::disclosure_activation(header_resp.clicked(), header_resp.has_focus(), keyboard_toggle);
            handler::toggle_expanded(self.expanded, should_toggle);
            if is_expanded {
                ui.add_space(config::THINKING_BODY_GAP);
                let body_frame = egui::Frame::NONE
                    .fill(self.theme.surface_editor)
                    .stroke(Stroke::new(config::FRAME_BORDER_WIDTH, self.theme.border_subtle))
                    .corner_radius(CornerRadius::same(config::THINKING_BODY_RADIUS as u8))
                    .inner_margin(egui::Margin::same(config::THINKING_BODY_PADDING as i8));
                body_frame.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(self.thought)
                            .size(config::TEXT_BODY_MONO_SIZE)
                            .monospace()
                            .color(self.theme.text_secondary),
                    );
                });
            }
        });

        frame_resp.response
    }
}

impl<'a> AgentPlan<'a> {
    pub fn new(title: &'a str, tasks: &'a [AgentTaskItem], theme: DbProTheme) -> Self {
        Self { title, tasks, theme }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (completed, total, progress) = handler::plan_progress(self.tasks);
        let frame = egui::Frame::NONE
            .fill(self.theme.surface_panel)
            .stroke(Stroke::new(config::FRAME_BORDER_WIDTH, self.theme.border_default))
            .corner_radius(CornerRadius::same(config::PLAN_RADIUS as u8))
            .inner_margin(egui::Margin::same(config::PLAN_PADDING as i8));

        let frame_resp = frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(self.title)
                        .size(config::PLAN_TITLE_SIZE)
                        .strong()
                        .color(self.theme.text_primary),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("{}/{} completed", completed, total))
                            .size(config::PLAN_META_SIZE)
                            .color(self.theme.text_secondary),
                    );
                });
            });
            ui.add_space(config::PLAN_PROGRESS_GAP);

            let (bar_rect, _) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), config::PLAN_PROGRESS_HEIGHT),
                Sense::hover(),
            );
            ui.painter().rect_filled(
                bar_rect,
                CornerRadius::same(config::PLAN_PROGRESS_RADIUS as u8),
                self.theme.surface_hover,
            );
            if progress > 0.0 {
                let filled_rect =
                    Rect::from_min_size(bar_rect.min, Vec2::new(bar_rect.width() * progress, bar_rect.height()));
                ui.painter().rect_filled(
                    filled_rect,
                    CornerRadius::same(config::PLAN_PROGRESS_RADIUS as u8),
                    self.theme.accent,
                );
            }
            ui.add_space(config::PLAN_TASK_LIST_TOP_SPACE);

            for (index, task) in self.tasks.iter().enumerate() {
                let visual = handler::task_visual(self.theme, task.status);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(char::from(visual.icon).to_string())
                            .font(font_icon(config::PLAN_TASK_ICON_SIZE))
                            .color(visual.icon_color),
                    );
                    ui.add_space(config::PLAN_TASK_ICON_TEXT_GAP);
                    ui.label(
                        RichText::new(format!("{}. {}", index + 1, &task.title))
                            .size(config::PLAN_TASK_TITLE_SIZE)
                            .color(visual.title_color),
                    );
                    if let Some(detail) = &task.detail {
                        ui.label(
                            RichText::new(format!("({})", detail))
                                .size(config::PLAN_TASK_DETAIL_SIZE)
                                .color(self.theme.text_tertiary),
                        );
                    }
                });
                if index + 1 < self.tasks.len() {
                    ui.add_space(config::PLAN_TASK_GAP);
                }
            }
        });

        frame_resp.response
    }
}

impl AgentSqlActionKind {
    pub fn label(&self) -> &'static str {
        handler::sql_action_label(*self)
    }

    pub fn icon(&self) -> Icon {
        handler::sql_action_icon(*self)
    }
}
