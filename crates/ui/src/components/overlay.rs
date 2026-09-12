use crate::components::animation::{fade_alpha, overlay_t, small_translate};
use crate::components::feedback::kbd_badge;
use crate::DbProTheme;
use egui::{
    Align2, Area, FontFamily, FontId, Frame, Margin, Order, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke,
    Ui, Vec2,
};
use lucide_icons::Icon;

const DROPDOWN_RADIUS: f32 = 12.0;
const DROPDOWN_ITEM_HEIGHT: f32 = 34.0;
const POPOVER_RADIUS: f32 = 12.0;
const TOAST_RADIUS: f32 = 10.0;
const OPEN_TRANSLATE_PX: f32 = 4.0;

pub fn floating_surface(theme: DbProTheme, rounding: f32, margin: Margin) -> Frame {
    Frame {
        fill: theme.surface_floating,
        stroke: Stroke::new(1.0, theme.border_subtle),
        inner_margin: margin,
        rounding: Rounding::same(rounding),
        shadow: theme.floating_shadow(),
        ..Default::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TooltipPosition {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

pub struct Tooltip<'a> {
    text: &'a str,
    position: TooltipPosition,
    shortcut: Option<&'a str>,
    theme: DbProTheme,
}

impl<'a> Tooltip<'a> {
    pub fn new(text: &'a str, theme: DbProTheme) -> Self {
        Self {
            text,
            position: TooltipPosition::Top,
            shortcut: None,
            theme,
        }
    }

    pub fn position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }

    pub fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn show(self, response: &Response) -> Response {
        let is_hovered = response.hovered();
        let id = response.id.with("snappy_tooltip");
        let progress = response
            .ctx
            .animate_bool_with_time(id.with("motion"), is_hovered, 0.120);

        if progress > 0.0 && progress < 1.0 {
            response.ctx.request_repaint();
        }

        if progress <= 0.0 {
            return response.clone();
        }

        let ctx = &response.ctx;
        let screen = ctx.screen_rect();
        let target_rect = response.rect;

        let font_id = egui::FontId::proportional(12.0);
        let galley =
            ctx.fonts(|fonts| fonts.layout(self.text.to_owned(), font_id.clone(), self.theme.text_primary, 260.0));

        let mut tooltip_w = galley.size().x + 18.0;
        if self.shortcut.is_some() {
            tooltip_w += 38.0;
        }
        let tooltip_h = (galley.size().y + 12.0).max(24.0);

        let motion_offset = (1.0 - progress) * 4.0;
        let (raw_x, raw_y) = match self.position {
            TooltipPosition::Top => (
                target_rect.center().x - tooltip_w * 0.5,
                target_rect.top() - tooltip_h - 6.0 + motion_offset,
            ),
            TooltipPosition::Bottom => (
                target_rect.center().x - tooltip_w * 0.5,
                target_rect.bottom() + 6.0 - motion_offset,
            ),
            TooltipPosition::Left => (
                target_rect.left() - tooltip_w - 6.0 + motion_offset,
                target_rect.center().y - tooltip_h * 0.5,
            ),
            TooltipPosition::Right => (
                target_rect.right() + 6.0 - motion_offset,
                target_rect.center().y - tooltip_h * 0.5,
            ),
        };

        let x = raw_x.clamp(screen.left() + 8.0, screen.right() - tooltip_w - 8.0);
        let y = raw_y.clamp(screen.top() + 8.0, screen.bottom() - tooltip_h - 8.0);
        let pos = Pos2::new(x, y);

        let theme = self.theme;
        let text = self.text.to_owned();
        let shortcut = self.shortcut;

        Area::new(id)
            .order(Order::Tooltip)
            .fixed_pos(pos)
            .interactable(false)
            .show(ctx, |ui| {
                ui.set_opacity(fade_alpha(progress));
                floating_surface(theme, 6.0, Margin::symmetric(8.0, 5.0)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);
                        ui.label(RichText::new(text).size(12.0).color(theme.text_primary));
                        if let Some(sc) = shortcut {
                            kbd_badge(ui, sc, theme);
                        }
                    });
                });
            });

        response.clone()
    }
}

pub struct Popover<'a> {
    open: &'a mut bool,
    theme: DbProTheme,
}

impl<'a> Popover<'a> {
    pub fn new(open: &'a mut bool, theme: DbProTheme) -> Self {
        Self { open, theme }
    }

    pub fn show<R>(self, ui: &mut Ui, trigger: &Response, add_contents: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        if trigger.clicked() {
            *self.open = !*self.open;
        }

        let id = trigger.id.with("popover");
        let progress = overlay_t(ui.ctx(), id.with("motion"), *self.open);
        if progress <= 0.0 {
            return None;
        }

        let pos = trigger.rect.left_bottom() + Vec2::new(0.0, 4.0 + small_translate(progress, OPEN_TRANSLATE_PX));
        let popup = Area::new(id)
            .order(Order::Foreground)
            .fixed_pos(pos)
            .show(ui.ctx(), |ui| {
                ui.set_opacity(fade_alpha(progress));
                floating_surface(self.theme, POPOVER_RADIUS, Margin::same(10.0))
                    .show(ui, add_contents)
                    .inner
            });

        close_if_clicked_outside(ui, *self.open, popup.response.rect, trigger.rect, self.open);
        Some(popup.inner)
    }
}

#[derive(Clone, Copy)]
pub struct DropdownItem<'a> {
    pub label: &'a str,
    pub icon: Option<Icon>,
    pub shortcut: Option<&'a str>,
    pub enabled: bool,
    pub selected: bool,
    pub danger: bool,
}

impl<'a> DropdownItem<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: None,
            shortcut: None,
            enabled: true,
            selected: false,
            danger: false,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn danger(mut self, danger: bool) -> Self {
        self.danger = danger;
        self
    }
}

pub struct DropdownMenu<'a> {
    open: &'a mut bool,
    items: &'a [DropdownItem<'a>],
    theme: DbProTheme,
}

impl<'a> DropdownMenu<'a> {
    pub fn new(open: &'a mut bool, items: &'a [DropdownItem<'a>], theme: DbProTheme) -> Self {
        Self { open, items, theme }
    }

    pub fn show(self, ui: &mut Ui, trigger: &Response) -> Option<usize> {
        if trigger.clicked() {
            *self.open = !*self.open;
        }

        let id = trigger.id.with("dropdown_menu");
        let progress = overlay_t(ui.ctx(), id.with("motion"), *self.open);
        if progress <= 0.0 {
            return None;
        }

        let pos = trigger.rect.left_bottom() + Vec2::new(0.0, 4.0 + small_translate(progress, OPEN_TRANSLATE_PX));
        let mut chosen = None;
        let popup = Area::new(id)
            .order(Order::Foreground)
            .fixed_pos(pos)
            .show(ui.ctx(), |ui| {
                ui.set_opacity(fade_alpha(progress));
                ui.set_min_width(trigger.rect.width().max(180.0));
                floating_surface(self.theme, DROPDOWN_RADIUS, Margin::symmetric(4.0, 6.0))
                    .show(ui, |ui| {
                        for (index, item) in self.items.iter().enumerate() {
                            let response = draw_dropdown_item(ui, item, self.theme);
                            if item.enabled && response.clicked() {
                                chosen = Some(index);
                            }
                        }
                    })
                    .inner
            });

        if let Some(index) = chosen {
            *self.open = false;
            return Some(index);
        }

        close_if_clicked_outside(ui, *self.open, popup.response.rect, trigger.rect, self.open);
        None
    }
}

fn draw_dropdown_item(ui: &mut Ui, item: &DropdownItem<'_>, theme: DbProTheme) -> Response {
    let sense = if item.enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), DROPDOWN_ITEM_HEIGHT), sense);
    let hovered = response.hovered() && item.enabled;
    if hovered {
        ui.painter().rect_filled(rect, Rounding::same(8.0), theme.surface_hover);
    } else if item.selected {
        ui.painter()
            .rect_filled(rect, Rounding::same(8.0), theme.surface_active);
    }

    if response.has_focus() {
        ui.painter()
            .rect_stroke(rect.expand(1.0), Rounding::same(8.0), Stroke::new(1.5, theme.accent));
    }

    let text_color = if !item.enabled {
        theme.text_disabled
    } else if item.danger {
        theme.danger
    } else {
        theme.text_primary
    };

    let mut cursor = rect.left() + 8.0;
    if let Some(icon) = item.icon {
        ui.painter().text(
            Pos2::new(cursor, rect.center().y),
            Align2::LEFT_CENTER,
            char::from(icon).to_string(),
            FontId::new(14.0, FontFamily::Name("lucide".into())),
            text_color,
        );
        cursor += 22.0;
    }

    ui.painter().text(
        Pos2::new(cursor, rect.center().y),
        Align2::LEFT_CENTER,
        item.label,
        FontId::proportional(13.0),
        text_color,
    );

    if let Some(shortcut) = item.shortcut {
        ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(Rect::from_min_max(
                Pos2::new(rect.right() - 64.0, rect.top()),
                rect.right_bottom(),
            )),
            |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    kbd_badge(ui, shortcut, theme);
                    ui.add_space(6.0);
                });
            },
        );
    }

    if !item.enabled {
        response.on_disabled_hover_text("Unavailable")
    } else {
        response.on_hover_cursor(egui::CursorIcon::PointingHand)
    }
}

fn close_if_clicked_outside(ui: &Ui, open: bool, popup: Rect, trigger: Rect, open_flag: &mut bool) {
    if !open {
        return;
    }
    let clicked_outside = ui.input(|input| {
        input.pointer.any_click()
            && input
                .pointer
                .interact_pos()
                .is_some_and(|pos| !popup.contains(pos) && !trigger.contains(pos))
    });
    if clicked_outside {
        *open_flag = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastVariant {
    Default,
    Success,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    #[default]
    BottomRight,
}

impl ToastPosition {
    pub fn alignment_and_pos(&self, screen: Rect) -> (Pos2, egui::Align2) {
        let margin_x = 24.0;
        let margin_y = 24.0;
        match self {
            ToastPosition::TopLeft => (
                Pos2::new(screen.left() + margin_x, screen.top() + margin_y),
                egui::Align2::LEFT_TOP,
            ),
            ToastPosition::TopCenter => (
                Pos2::new(screen.center().x, screen.top() + margin_y),
                egui::Align2::CENTER_TOP,
            ),
            ToastPosition::TopRight => (
                Pos2::new(screen.right() - margin_x, screen.top() + margin_y),
                egui::Align2::RIGHT_TOP,
            ),
            ToastPosition::BottomLeft => (
                Pos2::new(screen.left() + margin_x, screen.bottom() - margin_y),
                egui::Align2::LEFT_BOTTOM,
            ),
            ToastPosition::BottomCenter => (
                Pos2::new(screen.center().x, screen.bottom() - margin_y),
                egui::Align2::CENTER_BOTTOM,
            ),
            ToastPosition::BottomRight => (
                Pos2::new(screen.right() - margin_x, screen.bottom() - margin_y),
                egui::Align2::RIGHT_BOTTOM,
            ),
        }
    }
}

pub struct Toast<'a> {
    message: &'a str,
    variant: ToastVariant,
    position: ToastPosition,
    action_label: Option<&'a str>,
    closable: bool,
    theme: DbProTheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToastResponse {
    pub action_clicked: bool,
    pub dismiss_clicked: bool,
}

impl<'a> Toast<'a> {
    pub fn new(message: &'a str, theme: DbProTheme) -> Self {
        Self {
            message,
            variant: ToastVariant::Default,
            position: ToastPosition::BottomRight,
            action_label: None,
            closable: true,
            theme,
        }
    }

    pub fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    pub fn top_left(mut self) -> Self {
        self.position = ToastPosition::TopLeft;
        self
    }

    pub fn top_center(mut self) -> Self {
        self.position = ToastPosition::TopCenter;
        self
    }

    pub fn top_right(mut self) -> Self {
        self.position = ToastPosition::TopRight;
        self
    }

    pub fn bottom_left(mut self) -> Self {
        self.position = ToastPosition::BottomLeft;
        self
    }

    pub fn bottom_center(mut self) -> Self {
        self.position = ToastPosition::BottomCenter;
        self
    }

    pub fn bottom_right(mut self) -> Self {
        self.position = ToastPosition::BottomRight;
        self
    }

    pub fn action(mut self, label: &'a str) -> Self {
        self.action_label = Some(label);
        self
    }

    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    pub fn show_floating(self, ui: &Ui, salt: impl std::hash::Hash) -> ToastResponse {
        let screen = screen_rect(ui);
        let (pos, pivot) = self.position.alignment_and_pos(screen);
        let mut response = ToastResponse::default();
        Area::new(egui::Id::new(("toast", salt)))
            .order(Order::Tooltip)
            .fixed_pos(pos)
            .pivot(pivot)
            .show(ui.ctx(), |ui| {
                response = self.show(ui);
            });
        response
    }

    pub fn show(self, ui: &mut Ui) -> ToastResponse {
        let (icon, icon_color, accent_color) = match self.variant {
            ToastVariant::Default => (Icon::Info, self.theme.text_secondary, self.theme.accent),
            ToastVariant::Success => (Icon::CheckCircle2, self.theme.success, self.theme.success),
            ToastVariant::Danger => (Icon::AlertCircle, self.theme.danger, self.theme.danger),
        };

        let mut toast_resp = ToastResponse::default();
        let frame_resp = floating_surface(self.theme, TOAST_RADIUS, Margin::symmetric(14.0, 10.0)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(char::from(icon).to_string())
                        .font(FontId::new(14.0, FontFamily::Name("lucide".into())))
                        .color(icon_color),
                );
                ui.add_space(8.0);
                ui.label(RichText::new(self.message).size(13.0).color(self.theme.text_primary));

                if let Some(label) = self.action_label {
                    ui.add_space(10.0);
                    let action_btn =
                        egui::Button::new(RichText::new(label).size(12.0).strong().color(self.theme.accent))
                            .fill(self.theme.surface_hover)
                            .stroke(Stroke::new(1.0, self.theme.border_subtle))
                            .rounding(Rounding::same(5.0));

                    if ui.add(action_btn).clicked() {
                        toast_resp.action_clicked = true;
                    }
                }

                if self.closable {
                    ui.add_space(6.0);
                    let close_btn = egui::Button::new(
                        RichText::new(char::from(Icon::X).to_string())
                            .font(FontId::new(11.0, FontFamily::Name("lucide".into())))
                            .color(self.theme.text_muted),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .frame(false);

                    let close_resp = ui.add(close_btn);
                    if close_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if close_resp.clicked() {
                        toast_resp.dismiss_clicked = true;
                    }
                }
            });
        });

        // Paint accent bar on the left edge
        let frame_rect = frame_resp.response.rect;
        let accent_rect = Rect::from_min_size(frame_rect.left_top(), Vec2::new(3.0, frame_rect.height()));
        ui.painter().rect_filled(
            accent_rect,
            Rounding {
                nw: TOAST_RADIUS,
                ne: 0.0,
                sw: TOAST_RADIUS,
                se: 0.0,
            },
            accent_color,
        );

        toast_resp
    }
}

/// Item in the ToastManager queue.
#[derive(Debug, Clone)]
pub struct ToastItem {
    pub id: u64,
    pub message: String,
    pub variant: ToastVariant,
    pub position: ToastPosition,
    pub action_label: Option<String>,
    pub duration_secs: f32,
    pub elapsed_secs: f32,
}

/// Multi-toast manager supporting simultaneous queued/stacked toasts across all positions.
#[derive(Debug, Clone, Default)]
pub struct ToastManager {
    toasts: Vec<ToastItem>,
    next_id: u64,
}

impl ToastManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, message: impl Into<String>, variant: ToastVariant, position: ToastPosition) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.toasts.push(ToastItem {
            id,
            message: message.into(),
            variant,
            position,
            action_label: None,
            duration_secs: 5.0,
            elapsed_secs: 0.0,
        });
        id
    }

    pub fn show_with_action(
        &mut self,
        message: impl Into<String>,
        variant: ToastVariant,
        position: ToastPosition,
        action: impl Into<String>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.toasts.push(ToastItem {
            id,
            message: message.into(),
            variant,
            position,
            action_label: Some(action.into()),
            duration_secs: 8.0,
            elapsed_secs: 0.0,
        });
        id
    }

    pub fn success(&mut self, message: impl Into<String>, position: ToastPosition) -> u64 {
        self.show(message, ToastVariant::Success, position)
    }

    pub fn error(&mut self, message: impl Into<String>, position: ToastPosition) -> u64 {
        self.show(message, ToastVariant::Danger, position)
    }

    pub fn info(&mut self, message: impl Into<String>, position: ToastPosition) -> u64 {
        self.show(message, ToastVariant::Default, position)
    }

    pub fn dismiss(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }

    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }

    pub fn len(&self) -> usize {
        self.toasts.len()
    }

    /// Renders all stacked toasts grouped by their position, handling auto-dismiss and clicks.
    pub fn render(&mut self, ui: &Ui, theme: DbProTheme) -> Vec<(u64, ToastResponse)> {
        let dt = ui.input(|i| i.stable_dt).min(0.1);
        for item in &mut self.toasts {
            item.elapsed_secs += dt;
        }
        // Auto-dismiss expired toasts
        self.toasts.retain(|item| item.elapsed_secs < item.duration_secs);

        if self.toasts.is_empty() {
            return Vec::new();
        }

        let screen = screen_rect(ui);
        let mut results = Vec::new();
        let mut to_dismiss = Vec::new();

        let positions = [
            ToastPosition::TopLeft,
            ToastPosition::TopCenter,
            ToastPosition::TopRight,
            ToastPosition::BottomLeft,
            ToastPosition::BottomCenter,
            ToastPosition::BottomRight,
        ];

        for pos in positions {
            let items: Vec<(usize, ToastItem)> = self
                .toasts
                .iter()
                .enumerate()
                .filter(|(_, t)| t.position == pos)
                .map(|(i, t)| (i, t.clone()))
                .collect();

            if items.is_empty() {
                continue;
            }

            let (base_pos, pivot) = pos.alignment_and_pos(screen);
            let is_top = matches!(
                pos,
                ToastPosition::TopLeft | ToastPosition::TopCenter | ToastPosition::TopRight
            );

            for (stack_idx, (_idx, item)) in items.into_iter().enumerate() {
                let offset_y = stack_idx as f32 * 54.0;
                let actual_y = if is_top {
                    base_pos.y + offset_y
                } else {
                    base_pos.y - offset_y
                };
                let item_pos = Pos2::new(base_pos.x, actual_y);

                let mut toast_widget = Toast::new(&item.message, theme)
                    .variant(item.variant)
                    .position(item.position)
                    .closable(true);

                if let Some(ref action) = item.action_label {
                    toast_widget = toast_widget.action(action);
                }

                let mut resp = ToastResponse::default();
                Area::new(egui::Id::new(("toast_manager", item.id)))
                    .order(Order::Tooltip)
                    .fixed_pos(item_pos)
                    .pivot(pivot)
                    .show(ui.ctx(), |ui| {
                        resp = toast_widget.show(ui);
                    });

                if resp.dismiss_clicked {
                    to_dismiss.push(item.id);
                }
                if resp.action_clicked {
                    results.push((item.id, resp));
                }
            }
        }

        for id in to_dismiss {
            self.dismiss(id);
        }

        results
    }
}

pub(crate) fn screen_rect(ui: &Ui) -> Rect {
    let screen = ui.ctx().screen_rect();
    if screen.width() > 1.0 && screen.height() > 1.0 {
        screen
    } else {
        ui.max_rect()
    }
}
