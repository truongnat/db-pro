//! Native shell topbar rendering and intent collection.
use super::*;
use egui::Sense;

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

pub(super) struct ShellTopbarContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) sidebar_open: bool,
    pub(super) active_document_index: usize,
    pub(super) document_count: usize,
    pub(super) has_connection: bool,
    pub(super) connection_name: &'a str,
    pub(super) connection_icon: Icon,
    pub(super) connection_color: egui::Color32,
    pub(super) driver: &'a str,
    pub(super) agent_open: bool,
    pub(super) dark_mode: bool,
}

impl ShellTopbarContext<'_> {
    pub(super) fn draw(&self, ctx: &egui::Context) -> Vec<ShellTopbarAction> {
        let mut actions = Vec::new();
        let modifier = primary_modifier_label();
        TopBottomPanel::top("topbar")
            .exact_height(38.0)
            .frame(egui::Frame {
                fill: self.theme.surface_app,
                inner_margin: egui::Margin::symmetric(SPACE_MD, 4.0),
                stroke: egui::Stroke::new(STROKE_THIN, self.theme.border_subtle),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                ui.horizontal_centered(|ui| {
                    self.draw_navigation(ui, &mut actions, modifier);
                    ui.add_space(SPACE_SM);
                    ui.label(RichText::new("│").font(font_caption()).color(self.theme.border_subtle));
                    ui.add_space(SPACE_SM);
                    self.draw_connection(ui);
                    self.draw_global_actions(ui, &mut actions, modifier);
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
            .tooltip(toggle_tooltip)
            .show(ui)
            .clicked()
        {
            actions.push(ShellTopbarAction::ToggleSidebar);
        }
        ui.add_space(2.0);

        let can_go_back = self.active_document_index > 0;
        if Button::new(self.theme)
            .icon(Icon::ArrowLeft)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .enabled(can_go_back)
            .tooltip("Previous Document")
            .show(ui)
            .clicked()
        {
            actions.push(ShellTopbarAction::PreviousDocument);
        }

        let can_go_forward = self.active_document_index + 1 < self.document_count;
        if Button::new(self.theme)
            .icon(Icon::ArrowRight)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .enabled(can_go_forward)
            .tooltip("Next Document")
            .show(ui)
            .clicked()
        {
            actions.push(ShellTopbarAction::NextDocument);
        }
    }

    fn draw_connection(&self, ui: &mut egui::Ui) {
        if self.has_connection {
            ui.horizontal(|ui| {
                ui.label(icon_text(self.connection_icon, "", self.connection_color));
                let display_name = crate::components::truncate_ellipsis(self.connection_name, 22);
                let response = ui.label(
                    RichText::new(display_name)
                        .font(font_caption())
                        .strong()
                        .color(self.theme.text_primary),
                );
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
                ui.label(icon_text(Icon::Database, "", self.theme.accent));
                ui.label(
                    RichText::new("DB PRO")
                        .font(font_caption())
                        .strong()
                        .color(self.theme.text_primary),
                );
            });
        }
    }

    fn draw_global_actions(&self, ui: &mut egui::Ui, actions: &mut Vec<ShellTopbarAction>, modifier: &str) {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            self.draw_palette_actions(ui, actions, modifier);
            self.draw_assistant_actions(ui, actions);
            self.draw_quick_open(ui, actions, modifier);
        });
    }

    fn draw_palette_actions(&self, ui: &mut egui::Ui, actions: &mut Vec<ShellTopbarAction>, modifier: &str) {
        if Button::new(self.theme)
            .icon(Icon::Command)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
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
        let (rect, response) = ui.allocate_exact_size(egui::vec2(search_width, 26.0), Sense::click());
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
            egui::Rounding::same(RADIUS_MD),
            background,
            egui::Stroke::new(1.0, border),
        );
        let icon_font = egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into()));
        ui.painter().text(
            egui::pos2(rect.left() + 8.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            char::from(Icon::Search).to_string(),
            icon_font,
            self.theme.text_muted,
        );
        ui.painter().text(
            egui::pos2(rect.left() + 26.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            "Search commands, tables, schemas...",
            egui::FontId::proportional(11.5),
            self.theme.text_muted,
        );
        ui.painter().text(
            egui::pos2(rect.right() - 8.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            format!("{modifier}P"),
            egui::FontId::monospace(10.0),
            self.theme.text_muted,
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
