use crate::components::animation::{fade_alpha, overlay_t, small_translate};
use crate::components::feedback::kbd_badge;
use crate::DbProTheme;
use egui::{
    Align2, Area, Color32, FontFamily, FontId, Frame, Margin, Order, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui,
    Vec2,
};
use lucide_icons::Icon;

use super::{config, handler};

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

        let pos =
            trigger.rect.left_bottom() + Vec2::new(0.0, 4.0 + small_translate(progress, config::OPEN_TRANSLATE_PX));
        let popup = Area::new(id)
            .order(Order::Foreground)
            .fixed_pos(pos)
            .constrain_to(screen_rect(ui).shrink(config::CONTEXT_MENU_SCREEN_INSET))
            .show(ui.ctx(), |ui| {
                ui.set_opacity(fade_alpha(progress));
                floating_surface(self.theme, config::POPOVER_RADIUS, Margin::same(10.0))
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

        let pos =
            trigger.rect.left_bottom() + Vec2::new(0.0, 4.0 + small_translate(progress, config::OPEN_TRANSLATE_PX));
        let mut chosen = None;
        let popup = Area::new(id)
            .order(Order::Foreground)
            .fixed_pos(pos)
            .show(ui.ctx(), |ui| {
                ui.set_opacity(fade_alpha(progress));
                ui.set_min_width(trigger.rect.width().max(180.0));
                floating_surface(self.theme, config::DROPDOWN_RADIUS, Margin::symmetric(4.0, 6.0))
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
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), config::DROPDOWN_ITEM_HEIGHT), sense);
    let hovered = response.hovered() && item.enabled;
    if hovered {
        ui.painter()
            .rect_filled(rect, Rounding::same(config::DROPDOWN_ITEM_RADIUS), theme.surface_hover);
    } else if item.selected {
        ui.painter()
            .rect_filled(rect, Rounding::same(config::DROPDOWN_ITEM_RADIUS), theme.surface_active);
    }

    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(1.0),
            Rounding::same(config::DROPDOWN_ITEM_RADIUS),
            Stroke::new(1.5, theme.accent),
        );
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
            FontId::new(config::DROPDOWN_ICON_SIZE, FontFamily::Name("lucide".into())),
            text_color,
        );
        cursor += 22.0;
    }

    ui.painter().text(
        Pos2::new(cursor, rect.center().y),
        Align2::LEFT_CENTER,
        item.label,
        FontId::proportional(config::DROPDOWN_LABEL_SIZE),
        text_color,
    );

    if let Some(shortcut) = item.shortcut {
        let badge_rect = Rect::from_min_max(Pos2::new(rect.right() - 64.0, rect.top()), rect.right_bottom());
        let mut badge_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(badge_rect)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
        );
        badge_ui.set_clip_rect(badge_ui.clip_rect().intersect(badge_rect));
        kbd_badge(&mut badge_ui, shortcut, theme);
        badge_ui.add_space(6.0);
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
    if handler::should_close_on_outside_click(open, clicked_outside, clicked_outside) {
        *open_flag = false;
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

/// Capture-harness hook: `DB_PRO_DEBUG_CTX_AT=x,y` force-opens the menu whose
/// widget rect contains that screen point. `OnceLock` keeps the env lookup to
/// one read per process; `None` makes the check a single branch per widget.
fn debug_force_trigger(response: &egui::Response) -> bool {
    static POS: std::sync::OnceLock<Option<Pos2>> = std::sync::OnceLock::new();
    let pos = POS.get_or_init(|| {
        let raw = std::env::var("DB_PRO_DEBUG_CTX_AT").ok()?;
        let (x, y) = raw.split_once([',', 'x'])?;
        Some(Pos2::new(x.trim().parse().ok()?, y.trim().parse().ok()?))
    });
    matches!(pos, Some(p) if response.rect.contains(*p))
}

/// Determines if a context-menu trigger occurred on the widget response.
/// Robustly handles:
/// - Secondary click (Right-click) with or without Ctrl/Cmd
/// - macOS Ctrl+Click or Cmd+Click on primary button
/// - Keyboard Shift+F10 on the *focused* widget, not only the hovered one
pub fn is_context_menu_triggered(response: &egui::Response, ui: &egui::Ui) -> bool {
    let pointer_in_rect = ui
        .input(|i| i.pointer.latest_pos().or_else(|| i.pointer.interact_pos()))
        .is_some_and(|pos| response.rect.contains(pos));
    // `Sense::click()` is focusable in egui, so a widget reached with Tab can hold focus
    // with the pointer somewhere else entirely; gating on hover alone made Shift+F10
    // unreachable for keyboard-only users.
    let is_target = response.hovered() || pointer_in_rect || response.has_focus();

    if !is_target {
        return false;
    }

    ui.input(|i| {
        // 1. Right click (Secondary button) - with or without Ctrl/Cmd
        let sec_click = i.pointer.button_clicked(egui::PointerButton::Secondary);
        let sec_down = i.pointer.button_down(egui::PointerButton::Secondary);

        // 2. Primary button (Left click) with Ctrl (macOS standard) or Cmd
        let prim_click = i.pointer.button_clicked(egui::PointerButton::Primary);
        let ctrl_click = prim_click && (i.modifiers.ctrl || i.modifiers.command || i.modifiers.mac_cmd);

        // 3. Shift + F10
        let key_menu = i.modifiers.shift && i.key_pressed(egui::Key::F10);

        sec_click || (sec_down && prim_click) || ctrl_click || key_menu
    })
}

/// Displays a floating context menu at the click position using Foreground Area,
/// guaranteeing proper z-index and avoiding clipping in nested panels or scroll areas.
pub fn context_action_menu(
    ui: &mut egui::Ui,
    response: &egui::Response,
    theme: DbProTheme,
    add_contents: impl FnOnce(&mut egui::Ui, &mut bool),
) {
    let popup_id = response.id.with("floating_ctx_menu");
    let is_open_id = popup_id.with("is_open");
    let pos_id = popup_id.with("pos");

    let triggered = is_context_menu_triggered(response, ui) || debug_force_trigger(response);
    if triggered {
        let click_pos = ui
            .input(|i| i.pointer.latest_pos().or_else(|| i.pointer.interact_pos()))
            .unwrap_or_else(|| response.rect.left_bottom());
        ui.ctx().data_mut(|d| {
            d.insert_temp(is_open_id, true);
            d.insert_temp(pos_id, click_pos);
        });
    }

    let is_open = ui.ctx().data(|d| d.get_temp::<bool>(is_open_id)).unwrap_or(false);
    if !is_open {
        return;
    }

    let mut menu_pos = ui
        .ctx()
        .data(|d| d.get_temp::<Pos2>(pos_id))
        .unwrap_or_else(|| response.rect.left_bottom());
    let screen = screen_rect(ui);

    // Keep menu inside screen boundaries
    menu_pos.x = menu_pos.x.clamp(
        screen.left() + config::CONTEXT_MENU_SCREEN_INSET,
        (screen.right() - config::CONTEXT_MENU_MAX_WIDTH).max(screen.left() + config::CONTEXT_MENU_SCREEN_INSET),
    );
    menu_pos.y = menu_pos.y.clamp(
        screen.top() + config::CONTEXT_MENU_SCREEN_INSET,
        (screen.bottom() - config::CONTEXT_MENU_MAX_HEIGHT).max(screen.top() + config::CONTEXT_MENU_SCREEN_INSET),
    );

    let mut close_menu = false;

    let area_resp = Area::new(popup_id)
        .order(Order::Foreground)
        .fixed_pos(menu_pos)
        .show(ui.ctx(), |ui| {
            floating_surface(
                theme,
                config::CONTEXT_MENU_RADIUS,
                Margin::symmetric(
                    config::CONTEXT_MENU_HORIZONTAL_PADDING,
                    config::CONTEXT_MENU_VERTICAL_PADDING,
                ),
            )
            .show(ui, |ui| {
                ui.set_min_width(config::CONTEXT_MENU_MIN_WIDTH);
                ui.vertical(|ui| {
                    add_contents(ui, &mut close_menu);
                });
            });
        });

    if !triggered && ui.input(|i| i.pointer.any_click()) {
        if let Some(click_pos) = ui.input(|i| i.pointer.interact_pos()) {
            if !area_resp.response.rect.contains(click_pos) {
                close_menu = true;
            }
        }
    }

    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        close_menu = true;
    }

    if close_menu {
        ui.ctx().data_mut(|d| d.insert_temp(is_open_id, false));
    }
}

/// Helper button for clean, pixel-perfect DBeaver / Codex style context menu rows with icons and shortcut badges.
pub fn ctx_menu_item(
    ui: &mut Ui,
    icon: Option<Icon>,
    label: &str,
    shortcut: Option<&str>,
    color: Color32,
    theme: DbProTheme,
) -> egui::Response {
    let width = ui.available_width().max(200.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, config::CONTEXT_MENU_ITEM_HEIGHT), Sense::click());
    let hovered = response.hovered();
    if hovered {
        ui.painter().rect_filled(
            rect,
            Rounding::same(config::CONTEXT_MENU_ITEM_RADIUS),
            theme.surface_hover,
        );
    }
    let mut cursor = rect.left() + config::CONTEXT_MENU_ITEM_HORIZONTAL_PADDING;
    if let Some(ic) = icon {
        ui.painter().text(
            Pos2::new(cursor, rect.center().y),
            Align2::LEFT_CENTER,
            char::from(ic).to_string(),
            FontId::new(config::CONTEXT_MENU_ICON_SIZE, FontFamily::Name("lucide".into())),
            color,
        );
        cursor += config::CONTEXT_MENU_ICON_OFFSET;
    }
    ui.painter().text(
        Pos2::new(cursor, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(config::CONTEXT_MENU_LABEL_SIZE),
        color,
    );
    if let Some(sc) = shortcut {
        ui.painter().text(
            Pos2::new(rect.right() - 8.0, rect.center().y),
            Align2::RIGHT_CENTER,
            sc,
            FontId::monospace(config::CONTEXT_MENU_SHORTCUT_SIZE),
            theme.text_muted,
        );
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
