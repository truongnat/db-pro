use super::*;
use egui::{RichText, Ui};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GalleryCategory {
    #[default]
    All,
    Buttons,
    Badges,
    Inputs,
    Selection,
    Cards,
    Alerts,
    Feedback,
    Overlays,
    Navigation,
    Tables,
    DevTools,
    DatabaseShell,
    AgentUi,
}

#[derive(Debug, Clone)]
pub struct ComponentGalleryState {
    pub category: GalleryCategory,
    pub input_text: String,
    pub input_error_text: String,
    pub search_text: String,
    pub password_text: String,
    pub show_password: bool,
    pub textarea_text: String,
    pub checkbox_1: bool,
    pub checkbox_2: bool,
    pub switch_1: bool,
    pub switch_2: bool,
    pub radio_selected: usize,
    pub slider_val: f32,
    pub select_options: Vec<String>,
    pub select_idx: usize,
    pub segmented_tab_idx: usize,
    pub underline_tab_idx: usize,
    pub btn_loading: bool,
    pub progress_val: f32,
    pub table_search: String,
    pub table_selected_rows: std::collections::HashSet<usize>,
    pub table_sort_col: Option<usize>,
    pub table_sort_desc: bool,
    pub tree_server_expanded: bool,
    pub tree_db_expanded: bool,
    pub tree_schema_expanded: bool,
    pub tree_table_expanded: bool,
    pub tool_call_expanded: bool,
    pub approval_status: Option<String>,
    pub form_field_text: String,
    pub dialog_open: bool,
    pub shortcuts_dialog_open: bool,
    pub sheet_open: bool,
    pub popover_open: bool,
    pub dropdown_open: bool,
    pub pagination_page: usize,
    pub form_name: String,
    pub form_host: String,
    pub form_port: String,
    pub form_database: String,
    pub form_ssl: bool,
    pub form_engine: usize,
    pub form_notes: String,
    pub form_error: Option<String>,
    pub select_loaded: usize,
    pub alert_warning_open: bool,
    pub alert_error_open: bool,
    pub toasts: ToastManager,
    pub form_state: FormState,
    pub thinking_expanded: bool,
    pub destructive_dialog_open: bool,
    pub destructive_keyword: String,
    pub tx_in_transaction: bool,
    pub tx_pending_mutations: usize,
    pub tx_auto_commit: bool,
    pub tx_status_message: Option<String>,
    pub agent_action_output: Option<String>,
    pub composer_prompt: String,
    pub composer_is_generating: bool,
    pub activity_bar_selected: ActivityBarItemKind,
    pub db_card_status: ConnectionStatus,
    pub sql_editor_status: Option<String>,
}

impl Default for ComponentGalleryState {
    fn default() -> Self {
        let mut form_state = FormState::new();
        form_state.register(
            "form_name",
            vec![
                FieldRule::required("Display name is required"),
                FieldRule::min_length(3, "Name must be at least 3 characters"),
            ],
        );
        form_state.register(
            "form_host",
            vec![
                FieldRule::required("Host is required"),
                FieldRule::hostname("Must be a valid hostname or IP address"),
            ],
        );
        form_state.register(
            "form_port",
            vec![
                FieldRule::required("Port is required"),
                FieldRule::port("Port must be between 1 and 65535"),
            ],
        );
        form_state.register(
            "form_database",
            vec![
                FieldRule::required("Database name is required"),
                FieldRule::min_length(2, "Database name must be at least 2 characters"),
            ],
        );
        form_state.register(
            "form_password",
            vec![
                FieldRule::required("Password is required"),
                FieldRule::min_length(6, "Password must be at least 6 characters"),
            ],
        );

        Self {
            category: GalleryCategory::All,
            input_text: "postgres_prod_replica".to_owned(),
            input_error_text: "invalid_connection_string".to_owned(),
            search_text: "".to_owned(),
            password_text: "secret_db_pass_123".to_owned(),
            show_password: false,
            textarea_text: "SELECT users.id, users.email, COUNT(orders.id) AS total_orders\nFROM users\nLEFT JOIN orders ON orders.user_id = users.id\nGROUP BY users.id;".to_owned(),
            checkbox_1: true,
            checkbox_2: false,
            switch_1: true,
            switch_2: false,
            radio_selected: 0,
            slider_val: 65.0,
            select_options: (1..=36)
                .map(|n| format!("db-cluster-{n:02}  ·  postgres://host-{n}.internal:5432"))
                .collect(),
            select_idx: 0,
            segmented_tab_idx: 0,
            underline_tab_idx: 0,
            btn_loading: false,
            progress_val: 0.65,
            table_search: String::new(),
            table_selected_rows: [0, 2].into_iter().collect(),
            table_sort_col: Some(0),
            table_sort_desc: false,
            tree_server_expanded: true,
            tree_db_expanded: true,
            tree_schema_expanded: true,
            tree_table_expanded: true,
            tool_call_expanded: true,
            approval_status: None,
            form_field_text: "db-pro-prod".to_owned(),
            dialog_open: false,
            shortcuts_dialog_open: false,
            sheet_open: false,
            popover_open: false,
            dropdown_open: false,
            pagination_page: 1,
            form_name: "Production replica".to_owned(),
            form_host: "db.internal".to_owned(),
            form_port: "5432".to_owned(),
            form_database: "app_prod".to_owned(),
            form_ssl: true,
            form_engine: 0,
            form_notes: String::new(),
            form_error: None,
            select_loaded: 12,
            alert_warning_open: true,
            alert_error_open: true,
            toasts: ToastManager::default(),
            form_state,
            thinking_expanded: true,
            destructive_dialog_open: false,
            destructive_keyword: String::new(),
            tx_in_transaction: true,
            tx_pending_mutations: 3,
            tx_auto_commit: false,
            tx_status_message: None,
            agent_action_output: None,
            composer_prompt: String::new(),
            composer_is_generating: false,
            activity_bar_selected: ActivityBarItemKind::Explorer,
            db_card_status: ConnectionStatus::Connected,
            sql_editor_status: None,
        }
    }
}

impl DbProApp {
    pub(super) fn draw_component_gallery(&mut self, ui: &mut Ui) {
        let theme = self.theme;

        egui::ScrollArea::vertical()
            .id_salt("component_gallery_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(SPACE_SM);

                // ── Toolbar Header ──────────────────────────────────────────
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("✨").size(20.0));
                            ui.label(
                                RichText::new("Common UI Design System")
                                    .size(20.0)
                                    .strong()
                                    .color(theme.text_primary),
                            );
                            Badge::new("Design System", theme)
                                .variant(BadgeVariant::Default)
                                .dot(true)
                                .show(ui);
                        });
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new(
                                "Interactive living style guide and reusable component primitives for DB Pro Native.",
                            )
                            .size(13.0)
                            .color(theme.text_secondary),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Reset button
                        if Button::new(theme)
                            .text("Reset Demo")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Sm)
                            .icon(Icon::RotateCcw)
                            .show(ui)
                            .clicked()
                        {
                            self.gallery_state = ComponentGalleryState::default();
                        }

                        ui.add_space(8.0);

                        // Live Light / Dark theme toggle
                        let is_dark = self.dark_mode;
                        let theme_icon = if is_dark { Icon::Sun } else { Icon::Moon };
                        let theme_label = if is_dark { "Light Mode" } else { "Dark Mode" };
                        if Button::new(theme)
                            .text(theme_label)
                            .variant(ButtonVariant::Secondary)
                            .size(ButtonSize::Sm)
                            .icon(theme_icon)
                            .show(ui)
                            .clicked()
                        {
                            self.dark_mode = !self.dark_mode;
                        }
                    });
                });

                ui.add_space(16.0);

                // ── Category Filter Bar ─────────────────────────────────────
                let categories = [
                    (GalleryCategory::All, "All Components"),
                    (GalleryCategory::Buttons, "Buttons"),
                    (GalleryCategory::Badges, "Badges"),
                    (GalleryCategory::Inputs, "Forms & Inputs"),
                    (GalleryCategory::Selection, "Selection"),
                    (GalleryCategory::Cards, "Cards"),
                    (GalleryCategory::Alerts, "Alerts"),
                    (GalleryCategory::Feedback, "Feedback"),
                    (GalleryCategory::Overlays, "Overlays"),
                    (GalleryCategory::Navigation, "Navigation"),
                    (GalleryCategory::Tables, "Data Tables"),
                    (GalleryCategory::DevTools, "Developer Tools"),
                    (GalleryCategory::DatabaseShell, "Database & Shell"),
                    (GalleryCategory::AgentUi, "AI Agent UI"),
                ];

                let mut current_cat_idx = categories
                    .iter()
                    .position(|(c, _)| *c == self.gallery_state.category)
                    .unwrap_or(0);
                let cat_labels: Vec<&str> = categories.iter().map(|(_, l)| *l).collect();

                SegmentedTabs::new(&mut current_cat_idx, &cat_labels, theme).show(ui);
                self.gallery_state.category = categories[current_cat_idx].0;

                ui.add_space(20.0);

                let cat = self.gallery_state.category;

                // ── 1. BUTTONS ──────────────────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::Buttons {
                    self.draw_gallery_buttons_section(ui);
                    ui.add_space(24.0);
                }

                // ── 2. BADGES ───────────────────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::Badges {
                    self.draw_gallery_badges_section(ui);
                    ui.add_space(24.0);
                }

                // ── 3. FORMS & INPUTS ───────────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::Inputs {
                    self.draw_gallery_inputs_section(ui);
                    ui.add_space(24.0);
                }

                // ── 4. SELECTION CONTROLS ───────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::Selection {
                    self.draw_gallery_selection_section(ui);
                    ui.add_space(24.0);
                }

                // ── 5. CARDS & CONTAINERS ───────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::Cards {
                    self.draw_gallery_cards_section(ui);
                    ui.add_space(24.0);
                }

                // ── 6. ALERTS & NOTICES ─────────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::Alerts {
                    self.draw_gallery_alerts_section(ui);
                    ui.add_space(24.0);
                }

                // ── 7. FEEDBACK & PROGRESS ──────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::Feedback {
                    self.draw_gallery_feedback_section(ui);
                    ui.add_space(24.0);
                }

                // ── 8. OVERLAYS ─────────────────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::Overlays {
                    self.draw_gallery_overlays_section(ui);
                    ui.add_space(24.0);
                }

                // ── 9. NAVIGATION & TABS ────────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::Navigation {
                    self.draw_gallery_navigation_section(ui);
                    ui.add_space(24.0);
                }

                // ── 9. DATA TABLES ──────────────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::Tables {
                    self.draw_gallery_tables_section(ui);
                    ui.add_space(24.0);
                }

                // ── 10. DEVELOPER TOOLS ─────────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::DevTools {
                    self.draw_gallery_devtools_section(ui);
                    ui.add_space(24.0);
                }

                // ── 11. DATABASE & SHELL ─────────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::DatabaseShell {
                    self.draw_gallery_database_shell_section(ui);
                    ui.add_space(24.0);
                }

                // ── 12. AI AGENT UI ─────────────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::AgentUi {
                    self.draw_gallery_agent_ui_section(ui);
                    ui.add_space(32.0);
                }
            });

        let _events = self.gallery_state.toasts.render(ui, theme);
    }

    pub(super) fn draw_section_heading(&self, ui: &mut Ui, title: &str, subtitle: &str) {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(16.0).strong().color(self.theme.text_primary));
            ui.add_space(2.0);
            ui.label(RichText::new(subtitle).size(12.0).color(self.theme.text_muted));
        });
        ui.add_space(10.0);
    }
}
