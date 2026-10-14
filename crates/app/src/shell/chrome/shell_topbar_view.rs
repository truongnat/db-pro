//! Native shell topbar rendering and intent collection.
use super::*;
use egui::Sense;

pub(super) fn draw_window_resize_handles(ctx: &egui::Context) {
    let fixed_size = ctx.input(|input| {
        input.viewport().maximized.unwrap_or(false) || input.viewport().fullscreen.unwrap_or(false)
    });
    if fixed_size {
        return;
    }

    let screen_rect = ctx.input(|input| input.content_rect());
    draw_corner_resize_handles(ctx, screen_rect);
    draw_edge_resize_handles(ctx, screen_rect);
}

fn draw_corner_resize_handles(ctx: &egui::Context, rect: egui::Rect) {
    let corner = 12.0;
    let handles = [
        (rect.min, egui::ResizeDirection::NorthWest, egui::CursorIcon::ResizeNwSe),
        (
            egui::pos2(rect.right() - corner, rect.top()),
            egui::ResizeDirection::NorthEast,
            egui::CursorIcon::ResizeNeSw,
        ),
        (
            egui::pos2(rect.left(), rect.bottom() - corner),
            egui::ResizeDirection::SouthWest,
            egui::CursorIcon::ResizeNeSw,
        ),
        (
            rect.right_bottom() - egui::vec2(corner, corner),
            egui::ResizeDirection::SouthEast,
            egui::CursorIcon::ResizeNwSe,
        ),
    ];
    for (index, (position, direction, cursor)) in handles.into_iter().enumerate() {
        let handle = egui::Rect::from_min_size(position, egui::vec2(corner, corner));
        interact_resize_handle(ctx, handle, direction, cursor, index);
    }
}

fn draw_edge_resize_handles(ctx: &egui::Context, rect: egui::Rect) {
    let corner = 12.0;
    let edge = 5.0;
    let handles = [
        (
            egui::Rect::from_min_max(
                egui::pos2(rect.left() + corner, rect.top()),
                egui::pos2(rect.right() - corner, rect.top() + edge),
            ),
            egui::ResizeDirection::North,
            egui::CursorIcon::ResizeNorth,
        ),
        (
            egui::Rect::from_min_max(
                egui::pos2(rect.left() + corner, rect.bottom() - edge),
                egui::pos2(rect.right() - corner, rect.bottom()),
            ),
            egui::ResizeDirection::South,
            egui::CursorIcon::ResizeSouth,
        ),
        (
            egui::Rect::from_min_max(
                egui::pos2(rect.left(), rect.top() + corner),
                egui::pos2(rect.left() + edge, rect.bottom() - corner),
            ),
            egui::ResizeDirection::West,
            egui::CursorIcon::ResizeWest,
        ),
        (
            egui::Rect::from_min_max(
                egui::pos2(rect.right() - edge, rect.top() + corner),
                egui::pos2(rect.right(), rect.bottom() - corner),
            ),
            egui::ResizeDirection::East,
            egui::CursorIcon::ResizeEast,
        ),
    ];
    for (index, (handle, direction, cursor)) in handles.into_iter().enumerate() {
        interact_resize_handle(ctx, handle, direction, cursor, index + 4);
    }
}

fn interact_resize_handle(
    ctx: &egui::Context,
    rect: egui::Rect,
    direction: egui::ResizeDirection,
    cursor: egui::CursorIcon,
    index: usize,
) {
    egui::Area::new(egui::Id::new(("frameless_window_resize", index))) // ui-rules: §5 non-widget paint — invisible OS window-resize hit region (needs Area sense/default_size), not a themed surface a component can express
        .order(egui::Order::Foreground)
        .fixed_pos(rect.min)
        .default_size(rect.size())
        .interactable(true)
        // egui 0.36: `interactable(false)` disables inner widgets entirely; hover
        // sense keeps background clicks passing through while the handle stays
        // draggable.
        .sense(Sense::hover())
        .show(ctx, |ui| {
            let response = ui.interact(rect, ui.id().with("target"), Sense::click_and_drag());
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    ui.is_enabled(),
                    format!("Resize window ({direction:?})"),
                )
            });
            if response.hovered() {
                ctx.set_cursor_icon(cursor);
            }
            if response.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(direction));
            }
        });
}

pub(super) enum ShellTopbarAction {
    ToggleSidebar,
    PreviousDocument,
    NextDocument,
    OpenCommandPalette,
    OpenComponentGallery,
    ToggleAgent,
    ToggleTheme,
    OpenQuickOpen,
}

struct WindowControl<'a> {
    theme: DbProTheme,
    icon: Icon,
    label: &'a str,
    command: egui::ViewportCommand,
    is_close: bool,
}

pub(super) struct ShellTopbarContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) sidebar_open: bool,
    pub(super) can_go_back: bool,
    pub(super) can_go_forward: bool,
    pub(super) has_connection: bool,
    pub(super) connection_name: &'a str,
    pub(super) connection_icon: Icon,
    pub(super) connection_color: egui::Color32,
    pub(super) driver: &'a str,
    pub(super) agent_open: bool,
    pub(super) dark_mode: bool,
}

impl ShellTopbarContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<ShellTopbarAction> {
        let ctx = ui.ctx().clone();
        let mut actions = Vec::new();
        let modifier = primary_modifier_label();
        egui::Panel::top("topbar")
            .exact_size(38.0)
            .frame(shell_surface_frame(
                self.theme,
                ShellSurface::App,
                egui::Margin::symmetric(SPACE_MD as i8, SPACE_XS as i8),
                true,
            ))
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                ui.horizontal_centered(|ui| {
                    self.draw_navigation(ui, &mut actions, modifier);
                    ui.add_space(SPACE_SM);
                    ui.label(RichText::new("│").font(font_caption()).color(self.theme.border_subtle));
                    ui.add_space(SPACE_SM);
                    self.draw_connection(ui, &ctx);
                    self.draw_global_actions(ui, &mut actions, modifier, &ctx);
                });
            });
        actions
    }

    fn draw_navigation(&self, ui: &mut egui::Ui, actions: &mut Vec<ShellTopbarAction>, modifier: &str) {
        let toggle_tooltip = if self.sidebar_open {
            format!("Collapse Sidebar ({}B)", modifier)
        } else {
            format!("Expand Sidebar ({}B)", modifier)
        };
        let toggle_icon = if self.sidebar_open {
            Icon::PanelLeftClose
        } else {
            Icon::PanelLeft
        };
        if Button::new(self.theme)
            .icon(toggle_icon)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .access_label(toggle_tooltip.as_str())
            .tooltip(toggle_tooltip.as_str())
            .show(ui)
            .clicked()
        {
            actions.push(ShellTopbarAction::ToggleSidebar);
        }
        ui.add_space(SPACE_XXS);

        if Button::new(self.theme)
            .icon(Icon::ArrowLeft)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .enabled(self.can_go_back)
            .access_label("Back")
            .tooltip("Back")
            .show(ui)
            .clicked()
        {
            actions.push(ShellTopbarAction::PreviousDocument);
        }

        if Button::new(self.theme)
            .icon(Icon::ArrowRight)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .enabled(self.can_go_forward)
            .access_label("Forward")
            .tooltip("Forward")
            .show(ui)
            .clicked()
        {
            actions.push(ShellTopbarAction::NextDocument);
        }
    }

    fn draw_connection(&self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.has_connection {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(char::from(self.connection_icon).to_string())
                        .font(font_icon(ICON_DEFAULT))
                        .color(self.connection_color),
                );
                let display_name = crate::components::truncate_ellipsis(self.connection_name, 22);
                let response = ui.label(
                    RichText::new(display_name)
                        .font(font_caption())
                        .strong()
                        .color(self.theme.text_primary),
                );
                Self::handle_window_drag(&response, ctx);
                if self.connection_name.len() > 22 {
                    response.on_hover_text(self.connection_name);
                }
                let driver_tag = if self.driver.to_ascii_lowercase().contains("sqlite") {
                    "SQLite"
                } else {
                    "PostgreSQL"
                };
                Badge::new(driver_tag, self.theme)
                    .variant(BadgeVariant::Secondary)
                    .compact(true)
                    .show(ui);
            });
        } else {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(char::from(Icon::Database).to_string())
                        .font(font_icon(ICON_DEFAULT))
                        .color(self.theme.accent),
                );
                let response = ui.label(
                    RichText::new("DB PRO")
                        .font(font_caption())
                        .strong()
                        .color(self.theme.text_primary),
                );
                Self::handle_window_drag(&response, ctx);
            });
        }
    }

    fn draw_global_actions(
        &self,
        ui: &mut egui::Ui,
        actions: &mut Vec<ShellTopbarAction>,
        modifier: &str,
        ctx: &egui::Context,
    ) {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if cfg!(target_os = "linux") {
                Self::draw_window_controls(ui, ctx, self.theme);
            }
            self.draw_palette_actions(ui, actions, modifier);
            self.draw_assistant_actions(ui, actions);
            self.draw_quick_open(ui, actions, modifier);
        });
    }

    fn draw_window_controls(ui: &mut egui::Ui, ctx: &egui::Context, theme: DbProTheme) -> [egui::Rect; 3] {
        let maximized = ctx.input(|input| input.viewport().maximized.unwrap_or(false));
        let mut button_rects = [egui::Rect::NOTHING; 3];
        for (index, (icon, label, command, is_close)) in [
            (Icon::X, "Close window", egui::ViewportCommand::Close, true),
            (
                if maximized { Icon::Copy } else { Icon::Square },
                if maximized { "Restore window" } else { "Maximize window" },
                egui::ViewportCommand::Maximized(!maximized),
                false,
            ),
            (
                Icon::Minus,
                "Minimize window",
                egui::ViewportCommand::Minimized(true),
                false,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            button_rects[index] = Self::draw_window_control(
                ui,
                ctx,
                WindowControl {
                    theme,
                    icon,
                    label,
                    command,
                    is_close,
                },
            );
        }
        button_rects
    }

    fn draw_window_control(ui: &mut egui::Ui, ctx: &egui::Context, control: WindowControl<'_>) -> egui::Rect {
        let WindowControl {
            theme,
            icon,
            label,
            command,
            is_close,
        } = control;
        let (rect, response) = ui.allocate_exact_size(egui::vec2(34.0, 28.0), Sense::click());
        if response.hovered() {
            let fill = if is_close { theme.danger } else { theme.surface_hover };
            ui.painter().rect_filled(rect, egui::CornerRadius::same(4.0 as u8), fill);
        }
        let icon_color = if is_close && response.hovered() {
            theme.text_on_solid(theme.danger)
        } else {
            theme.text_secondary
        };
        if response.has_focus() {
            ui.painter().rect_stroke(
                rect,
                egui::CornerRadius::same(4.0 as u8),
                egui::Stroke::new(STROKE_THIN, theme.border_focus), egui::StrokeKind::Inside);
        }
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            char::from(icon).to_string(),
            egui::FontId::new(13.0, egui::FontFamily::Name("lucide".into())),
            icon_color,
        );
        response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label));
        let clicked = response.clicked();
        response.on_hover_text(label);
        if clicked {
            ctx.send_viewport_cmd(command);
        }
        rect
    }

    fn handle_window_drag(response: &egui::Response, ctx: &egui::Context) {
        if cfg!(target_os = "linux") {
            // Frameless windows need an app-owned hit target to request window-manager dragging.
            let drag_response = response.clone().interact(Sense::click_and_drag());
            if drag_response.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            } else if drag_response.double_clicked() {
                let maximized = ctx.input(|input| input.viewport().maximized.unwrap_or(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            }
        }
    }

    fn draw_palette_actions(&self, ui: &mut egui::Ui, actions: &mut Vec<ShellTopbarAction>, modifier: &str) {
        if Button::new(self.theme)
            .icon(Icon::Command)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .access_label("Command Palette")
            .tooltip(format!("Command Palette ({}⇧P)", modifier))
            .show(ui)
            .clicked()
        {
            actions.push(ShellTopbarAction::OpenCommandPalette);
        }
        if Button::new(self.theme)
            .icon(Icon::Palette)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .access_label("Open Component Gallery")
            .tooltip("Open Component Gallery")
            .show(ui)
            .clicked()
        {
            actions.push(ShellTopbarAction::OpenComponentGallery);
        }
    }

    fn draw_assistant_actions(&self, ui: &mut egui::Ui, actions: &mut Vec<ShellTopbarAction>) {
        let agent_tooltip = if self.agent_open {
            "Close Copilot Panel"
        } else {
            "Open Copilot Assistant"
        };
        if Button::new(self.theme)
            .icon(Icon::Bot)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .access_label(agent_tooltip)
            .tooltip(agent_tooltip)
            .show(ui)
            .clicked()
        {
            actions.push(ShellTopbarAction::ToggleAgent);
        }
        let theme_icon = if self.dark_mode { Icon::Sun } else { Icon::Moon };
        let theme_tooltip = if self.dark_mode {
            "Switch to Light Theme"
        } else {
            "Switch to Dark Theme"
        };
        if Button::new(self.theme)
            .icon(theme_icon)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .access_label(theme_tooltip)
            .tooltip(theme_tooltip)
            .show(ui)
            .clicked()
        {
            actions.push(ShellTopbarAction::ToggleTheme);
        }
    }

    fn draw_quick_open(&self, ui: &mut egui::Ui, actions: &mut Vec<ShellTopbarAction>, modifier: &str) {
        ui.add_space(SPACE_SM);
        let search_width = (ui.available_width() - 32.0).clamp(180.0, 360.0);
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(search_width, crate::tokens::component::input::INPUT_HEIGHT_SM), Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::WidgetType::Button,
                ui.is_enabled(),
                format!("Search commands, tables, schemas ({modifier}P)"),
            )
        });
        let background = if response.hovered() {
            self.theme.surface_hover
        } else {
            self.theme.surface_panel
        };
        let border = if response.hovered() {
            self.theme.border_strong
        } else {
            self.theme.border_subtle
        };
        ui.painter().rect(
            rect,
            egui::CornerRadius::same(RADIUS_INPUT as u8),
            background,
            egui::Stroke::new(1.0, border), egui::StrokeKind::Inside);
        if response.has_focus() {
            crate::components::interact::paint_focus_ring(ui, rect, RADIUS_INPUT, self.theme);
        }
        let icon_font = egui::FontId::new(ICON_SM, egui::FontFamily::Name("lucide".into()));
        let label_color = self.theme.text_secondary;
        ui.painter().text(
            egui::pos2(rect.left() + SPACE_SM, rect.center().y),
            egui::Align2::LEFT_CENTER,
            char::from(Icon::Search).to_string(),
            icon_font,
            label_color,
        );
        ui.painter().text(
            egui::pos2(rect.left() + SPACE_SM + ICON_SM + SPACE_XS, rect.center().y),
            egui::Align2::LEFT_CENTER,
            "Search commands, tables, schemas...",
            font_caption(),
            label_color,
        );
        ui.painter().text(
            egui::pos2(rect.right() - SPACE_SM, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            format!("{modifier}P"),
            egui::FontId::monospace(FONT_SIZE_KBD),
            label_color,
        );
        if response.clicked() {
            actions.push(ShellTopbarAction::OpenQuickOpen);
        }
    }
}

fn primary_modifier_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl"
    }
}

#[cfg(test)]
#[path = "tests/shell_topbar_view_window_control_tests.rs"]
mod window_control_tests;

