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
                ui.add_space(16.0);

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

    fn draw_section_heading(&self, ui: &mut Ui, title: &str, subtitle: &str) {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(16.0).strong().color(self.theme.text_primary));
            ui.add_space(2.0);
            ui.label(RichText::new(subtitle).size(12.0).color(self.theme.text_muted));
        });
        ui.add_space(10.0);
    }

    fn draw_gallery_buttons_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Buttons",
            "Displays a button or a component that looks like a button with various variants and sizes.",
        );

        Card::new(theme).show(ui, |ui| {
            ui.label(
                RichText::new("Variants")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                Button::new(theme)
                    .text("Primary (Default)")
                    .variant(ButtonVariant::Default)
                    .show(ui);
                ui.add_space(6.0);
                Button::new(theme)
                    .text("Secondary")
                    .variant(ButtonVariant::Secondary)
                    .show(ui);
                ui.add_space(6.0);
                Button::new(theme)
                    .text("Outline")
                    .variant(ButtonVariant::Outline)
                    .show(ui);
                ui.add_space(6.0);
                Button::new(theme).text("Ghost").variant(ButtonVariant::Ghost).show(ui);
                ui.add_space(6.0);
                Button::new(theme)
                    .text("Destructive")
                    .variant(ButtonVariant::Destructive)
                    .icon(Icon::Trash2)
                    .show(ui);
                ui.add_space(6.0);
                Button::new(theme)
                    .text("Link Action")
                    .variant(ButtonVariant::Link)
                    .show(ui);
            });

            ui.add_space(16.0);
            ui.label(RichText::new("Sizes").size(13.0).strong().color(theme.text_secondary));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                Button::new(theme).text("Small (Sm)").size(ButtonSize::Sm).show(ui);
                ui.add_space(6.0);
                Button::new(theme)
                    .text("Default (Md)")
                    .size(ButtonSize::Default)
                    .show(ui);
                ui.add_space(6.0);
                Button::new(theme).text("Large (Lg)").size(ButtonSize::Lg).show(ui);
                ui.add_space(10.0);
                Button::new(theme)
                    .icon(Icon::Plus)
                    .size(ButtonSize::Icon)
                    .variant(ButtonVariant::Outline)
                    .access_label("Add")
                    .show(ui);
                ui.add_space(6.0);
                Button::new(theme)
                    .icon(Icon::Copy)
                    .size(ButtonSize::IconSm)
                    .variant(ButtonVariant::Secondary)
                    .access_label("Copy")
                    .show(ui);
            });

            ui.add_space(16.0);
            ui.label(
                RichText::new("Interactive States & Loading")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                let loading = self.gallery_state.btn_loading;
                if Button::new(theme)
                    .text(if loading { "Processing..." } else { "Click to Load" })
                    .icon(Icon::Play)
                    .loading(loading)
                    .show(ui)
                    .clicked()
                {
                    self.gallery_state.btn_loading = true;
                }

                if loading {
                    ui.add_space(8.0);
                    if Button::new(theme)
                        .text("Stop")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.gallery_state.btn_loading = false;
                    }
                }

                ui.add_space(12.0);
                Button::new(theme)
                    .text("Always Loading (Primary)")
                    .loading(true)
                    .show(ui);

                ui.add_space(12.0);
                Button::new(theme)
                    .text("Always Loading (Outline)")
                    .variant(ButtonVariant::Outline)
                    .loading(true)
                    .show(ui);

                ui.add_space(12.0);
                Button::new(theme).text("Disabled Button").enabled(false).show(ui);
            });
        });
    }

    fn draw_gallery_badges_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Badges & Status Pills",
            "Status indicators, categorization tags, and notification counters.",
        );

        Card::new(theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                Badge::new("Default Accent", theme)
                    .variant(BadgeVariant::Default)
                    .show(ui);
                ui.add_space(8.0);
                Badge::new("Secondary Pill", theme)
                    .variant(BadgeVariant::Secondary)
                    .show(ui);
                ui.add_space(8.0);
                Badge::new("Outline Tag", theme).variant(BadgeVariant::Outline).show(ui);
                ui.add_space(8.0);
                Badge::new("Active Node", theme)
                    .variant(BadgeVariant::Success)
                    .dot(true)
                    .show(ui);
                ui.add_space(8.0);
                Badge::new("Warning Alert", theme)
                    .variant(BadgeVariant::Warning)
                    .icon(Icon::AlertTriangle)
                    .show(ui);
                ui.add_space(8.0);
                Badge::new("Connection Error", theme)
                    .variant(BadgeVariant::Destructive)
                    .dot(true)
                    .show(ui);
                ui.add_space(8.0);
                Badge::new("PostgreSQL 16.2", theme)
                    .variant(BadgeVariant::Info)
                    .icon(Icon::Database)
                    .show(ui);
            });

            ui.add_space(16.0);
            ui.label(RichText::new("Avatars").size(13.0).strong().color(theme.text_secondary));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                Avatar::new(theme).initials("TD").size(AvatarSize::Sm).show(ui);
                ui.add_space(8.0);
                Avatar::new(theme).initials("QP").size(AvatarSize::Md).show(ui);
                ui.add_space(8.0);
                Avatar::new(theme).icon(Icon::Bot).size(AvatarSize::Lg).show(ui);
            });
        });
    }

    fn draw_gallery_inputs_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Form Handling & Validation",
            "Production form with React Hook Form semantics: schema rules, touched/dirty tracking, real-time validation, and inline error states.",
        );

        Card::new(theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("New connection")
                            .size(15.0)
                            .strong()
                            .color(theme.text_primary),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("Saved connections appear in the explorer after a successful test.")
                            .size(12.0)
                            .color(theme.text_secondary),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut mode_idx = match self.gallery_state.form_state.mode {
                        ValidationMode::OnTouched => 0,
                        ValidationMode::OnChange => 1,
                        ValidationMode::OnBlur => 2,
                        ValidationMode::OnSubmit => 3,
                    };
                    let modes = ["onTouched", "onChange", "onBlur", "onSubmit"];
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Mode:")
                                .font(DbProTheme::ui_medium_font(11.5))
                                .color(theme.text_muted),
                        );
                        SegmentedTabs::new(&mut mode_idx, &modes, theme).show(ui);
                    });
                    self.gallery_state.form_state.mode = match mode_idx {
                        0 => ValidationMode::OnTouched,
                        1 => ValidationMode::OnChange,
                        2 => ValidationMode::OnBlur,
                        _ => ValidationMode::OnSubmit,
                    };
                });
            });
            ui.add_space(16.0);

            if let Some(err) = self.gallery_state.form_error.clone() {
                if let Some(dismiss) = Alert::new("Form validation error", &err, theme)
                    .variant(AlertVariant::Destructive)
                    .dismissable(true)
                    .show(ui)
                {
                    if dismiss.clicked() {
                        self.gallery_state.form_error = None;
                    }
                }
                ui.add_space(12.0);
            }

            ui.columns(2, |columns| {
                // Column 1
                let ui = &mut columns[0];

                // Field 1: Display Name
                let name_err = self.gallery_state.form_state.get_error("form_name");
                let show_name_err = self.gallery_state.form_state.should_show_error("form_name");
                let mut name_field = FormField::new(
                    "Display name",
                    &mut self.gallery_state.form_name,
                    "Production replica",
                    theme,
                )
                .required(self.gallery_state.form_state.is_required("form_name"))
                .helper_text("Shown in the sidebar and command palette.");
                if show_name_err {
                    if let Some(err) = name_err {
                        name_field = name_field.error_text(err);
                    }
                }
                let resp = name_field.show(ui);
                if resp.changed() {
                    self.gallery_state.form_state.set_dirty("form_name");
                    self.gallery_state
                        .form_state
                        .validate_field("form_name", &self.gallery_state.form_name);
                }
                if resp.lost_focus() {
                    self.gallery_state.form_state.touch("form_name");
                    self.gallery_state
                        .form_state
                        .validate_field("form_name", &self.gallery_state.form_name);
                }

                ui.add_space(12.0);

                // Field 2: Host
                let host_err = self.gallery_state.form_state.get_error("form_host");
                let show_host_err = self.gallery_state.form_state.should_show_error("form_host");
                let mut host_field = FormField::new("Host", &mut self.gallery_state.form_host, "db.internal", theme)
                    .required(self.gallery_state.form_state.is_required("form_host"))
                    .helper_text("Domain name or IPv4/IPv6 address.");
                if show_host_err {
                    if let Some(err) = host_err {
                        host_field = host_field.error_text(err);
                    }
                }
                let resp = host_field.show(ui);
                if resp.changed() {
                    self.gallery_state.form_state.set_dirty("form_host");
                    self.gallery_state
                        .form_state
                        .validate_field("form_host", &self.gallery_state.form_host);
                }
                if resp.lost_focus() {
                    self.gallery_state.form_state.touch("form_host");
                    self.gallery_state
                        .form_state
                        .validate_field("form_host", &self.gallery_state.form_host);
                }

                ui.add_space(12.0);

                // Field 3: Port
                let port_err = self.gallery_state.form_state.get_error("form_port");
                let show_port_err = self.gallery_state.form_state.should_show_error("form_port");
                let mut port_field = FormField::new("Port", &mut self.gallery_state.form_port, "5432", theme)
                    .required(self.gallery_state.form_state.is_required("form_port"))
                    .helper_text("TCP port 1–65535.");
                if show_port_err {
                    if let Some(err) = port_err {
                        port_field = port_field.error_text(err);
                    }
                }
                let resp = port_field.show(ui);
                if resp.changed() {
                    self.gallery_state.form_state.set_dirty("form_port");
                    self.gallery_state
                        .form_state
                        .validate_field("form_port", &self.gallery_state.form_port);
                }
                if resp.lost_focus() {
                    self.gallery_state.form_state.touch("form_port");
                    self.gallery_state
                        .form_state
                        .validate_field("form_port", &self.gallery_state.form_port);
                }

                // Column 2
                let ui = &mut columns[1];

                // Field 4: Database
                let db_err = self.gallery_state.form_state.get_error("form_database");
                let show_db_err = self.gallery_state.form_state.should_show_error("form_database");
                let mut db_field = FormField::new("Database", &mut self.gallery_state.form_database, "app_prod", theme)
                    .required(self.gallery_state.form_state.is_required("form_database"))
                    .helper_text("Default catalog database.");
                if show_db_err {
                    if let Some(err) = db_err {
                        db_field = db_field.error_text(err);
                    }
                }
                let resp = db_field.show(ui);
                if resp.changed() {
                    self.gallery_state.form_state.set_dirty("form_database");
                    self.gallery_state
                        .form_state
                        .validate_field("form_database", &self.gallery_state.form_database);
                }
                if resp.lost_focus() {
                    self.gallery_state.form_state.touch("form_database");
                    self.gallery_state
                        .form_state
                        .validate_field("form_database", &self.gallery_state.form_database);
                }

                ui.add_space(12.0);

                // Field 5: Password
                let pass_err = self.gallery_state.form_state.get_error("form_password");
                let show_pass_err = self.gallery_state.form_state.should_show_error("form_password");
                let mut pass_field = PasswordInput::new(
                    &mut self.gallery_state.password_text,
                    "Enter password…",
                    &mut self.gallery_state.show_password,
                    theme,
                )
                .label("Password")
                .required(self.gallery_state.form_state.is_required("form_password"))
                .helper_text("Must be at least 6 characters.");
                if show_pass_err {
                    if let Some(err) = pass_err {
                        pass_field = pass_field.error_text(err);
                    }
                }
                let resp = pass_field.show(ui);
                if resp.changed() {
                    self.gallery_state.form_state.set_dirty("form_password");
                    self.gallery_state
                        .form_state
                        .validate_field("form_password", &self.gallery_state.password_text);
                }
                if resp.lost_focus() {
                    self.gallery_state.form_state.touch("form_password");
                    self.gallery_state
                        .form_state
                        .validate_field("form_password", &self.gallery_state.password_text);
                }

                ui.add_space(12.0);
                Switch::new(&mut self.gallery_state.form_ssl, theme)
                    .label("Require SSL / TLS")
                    .description("Refuse unencrypted plaintext connections.")
                    .show(ui);
            });

            ui.add_space(12.0);
            let loaded = self
                .gallery_state
                .select_loaded
                .min(self.gallery_state.select_options.len());
            let mut load_more = false;
            let has_more = loaded < self.gallery_state.select_options.len();
            Select::new(
                "gallery_form_engine",
                &mut self.gallery_state.form_engine,
                &self.gallery_state.select_options[..loaded],
                theme,
            )
            .label("Cluster Pool")
            .has_more(has_more)
            .load_more(&mut load_more)
            .show(ui);
            if load_more {
                self.gallery_state.select_loaded = (loaded + 12).min(self.gallery_state.select_options.len());
            }

            ui.add_space(12.0);
            Textarea::new(&mut self.gallery_state.form_notes, "Optional notes…", theme)
                .label("Connection Notes")
                .min_rows(3)
                .show(ui);

            ui.add_space(16.0);
            let is_form_valid = self.gallery_state.form_state.check_validity(&[
                ("form_name", &self.gallery_state.form_name),
                ("form_host", &self.gallery_state.form_host),
                ("form_port", &self.gallery_state.form_port),
                ("form_database", &self.gallery_state.form_database),
                ("form_password", &self.gallery_state.password_text),
            ]) && self.gallery_state.form_state.is_valid();

            ui.horizontal(|ui| {
                let mut submit_btn = Button::new(theme)
                    .text("Save connection")
                    .enabled(is_form_valid)
                    .show(ui);

                if !is_form_valid {
                    submit_btn =
                        submit_btn.on_hover_text("Form contains invalid fields. Please resolve errors to save.");
                }

                if is_form_valid && submit_btn.clicked() {
                    let fields = [
                        ("form_name", self.gallery_state.form_name.as_str()),
                        ("form_host", self.gallery_state.form_host.as_str()),
                        ("form_port", self.gallery_state.form_port.as_str()),
                        ("form_database", self.gallery_state.form_database.as_str()),
                        ("form_password", self.gallery_state.password_text.as_str()),
                    ];
                    let is_valid = self.gallery_state.form_state.handle_submit(&fields, || {
                        self.gallery_state.form_error = None;
                        self.gallery_state
                            .toasts
                            .success("Connection validated & saved successfully", ToastPosition::BottomRight);
                    });
                    if !is_valid {
                        self.gallery_state.form_error =
                            Some("Please correct the highlighted validation errors above.".to_owned());
                    }
                }
                if Button::new(theme)
                    .text("Test connection")
                    .variant(ButtonVariant::Outline)
                    .show(ui)
                    .clicked()
                {
                    self.gallery_state.toasts.show_with_action(
                        "Ping: reached host in 38ms (SSL verified)",
                        ToastVariant::Default,
                        ToastPosition::BottomRight,
                        "Details",
                    );
                }
                if Button::new(theme)
                    .text("Clear form")
                    .variant(ButtonVariant::Ghost)
                    .show(ui)
                    .clicked()
                {
                    self.gallery_state.form_name.clear();
                    self.gallery_state.form_host.clear();
                    self.gallery_state.form_port.clear();
                    self.gallery_state.form_database.clear();
                    self.gallery_state.password_text.clear();
                    self.gallery_state.form_notes.clear();
                    self.gallery_state.form_error = None;
                    self.gallery_state.form_state.reset();
                }
            });
        });
    }

    fn draw_gallery_selection_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Selection & Sliders",
            "Checkboxes, animated toggle switches, radio groups, dropdown selects, and sliders.",
        );

        Card::new(theme).show(ui, |ui| {
            ui.columns(3, |columns| {
                // Column 1: Checkboxes
                let ui = &mut columns[0];
                ui.label(
                    RichText::new("Checkboxes")
                        .size(13.0)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(8.0);
                Checkbox::new(&mut self.gallery_state.checkbox_1, "Auto-commit queries", theme)
                    .description("Execute each SQL statement immediately in its own transaction.")
                    .show(ui);
                ui.add_space(8.0);
                Checkbox::new(&mut self.gallery_state.checkbox_2, "Format SQL on save", theme)
                    .description("Automatically aligns keywords and clauses.")
                    .show(ui);
                ui.add_space(8.0);
                let mut disabled_check = true;
                Checkbox::new(&mut disabled_check, "Enforce SSL encryption", theme)
                    .description("Required by your organization policy.")
                    .enabled(false)
                    .show(ui);

                // Column 2: Switches & Radios
                let ui = &mut columns[1];
                ui.label(
                    RichText::new("Switches & Radios")
                        .size(13.0)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(8.0);
                Switch::new(&mut self.gallery_state.switch_1, theme)
                    .label("Copilot Assistant")
                    .description("AI schema suggestions in real time.")
                    .show(ui);
                ui.add_space(10.0);
                Switch::new(&mut self.gallery_state.switch_2, theme)
                    .label("Query Execution Safety Guard")
                    .description("Confirm before destructive statements.")
                    .show(ui);

                ui.add_space(12.0);
                for (idx, label) in ["Read Committed", "Repeatable Read", "Serializable"].iter().enumerate() {
                    let is_sel = idx == self.gallery_state.radio_selected;
                    if Radio::new(is_sel, label, theme).show(ui).clicked() {
                        self.gallery_state.radio_selected = idx;
                    }
                    ui.add_space(4.0);
                }

                // Column 3: Select & Slider — long list, scroll cap, load more, flip-up
                let ui = &mut columns[2];
                ui.label(
                    RichText::new("Dropdown Select & Slider")
                        .font(DbProTheme::ui_medium_font(13.0))
                        .color(theme.text_secondary),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("36 clusters, 8-row max height, Load more, opens upward if clipped.")
                        .size(11.5)
                        .color(theme.text_muted),
                );
                ui.add_space(8.0);
                let loaded = self
                    .gallery_state
                    .select_loaded
                    .min(self.gallery_state.select_options.len());
                let mut load_more = false;
                let has_more = loaded < self.gallery_state.select_options.len();
                Select::new(
                    "gallery_db_select",
                    &mut self.gallery_state.select_idx,
                    &self.gallery_state.select_options[..loaded],
                    theme,
                )
                .label("Target connection profile")
                .has_more(has_more)
                .load_more(&mut load_more)
                .show(ui);
                if load_more {
                    self.gallery_state.select_loaded = (loaded + 12).min(self.gallery_state.select_options.len());
                }

                ui.add_space(16.0);
                Slider::new(&mut self.gallery_state.slider_val, 10.0..=500.0, theme)
                    .label("Query result page size (rows)")
                    .show(ui);
            });
        });
    }

    fn draw_gallery_cards_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Cards & Metric Displays",
            "Composite containers and statistical overview cards.",
        );

        ui.columns(2, |columns| {
            MetricCard::new("Active connections", "8 / 10", theme)
                .change("+2 this session", true)
                .icon(Icon::Database)
                .show(&mut columns[0]);
            MetricCard::new("Queries today", "14,289", theme)
                .change("+18.4% vs yesterday", true)
                .icon(Icon::Terminal)
                .show(&mut columns[1]);
        });
        ui.add_space(12.0);
        ui.columns(2, |columns| {
            MetricCard::new("Slow query rate", "0.42%", theme)
                .change("−0.15% vs last week", true)
                .icon(Icon::Gauge)
                .show(&mut columns[0]);
            MetricCard::new("Staged mutations", "3", theme)
                .change("Needs review", false)
                .icon(Icon::FileEdit)
                .show(&mut columns[1]);
        });
    }

    fn draw_gallery_alerts_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Alerts & Notifications",
            "Inline callouts for system alerts, warnings, and confirmations.",
        );

        ui.columns(2, |columns| {
            let ui = &mut columns[0];
            Alert::new(
                "Connection Established",
                "Successfully connected to PostgreSQL cluster xe_lac_hong_prod via localhost:5432.",
                theme,
            )
            .variant(AlertVariant::Success)
            .show(ui);

            ui.add_space(10.0);

            if self.gallery_state.alert_warning_open {
                if let Some(close) = Alert::new(
                    "Uncommitted transaction",
                    "You have active mutations in session 42. Closing this tab will roll them back.",
                    theme,
                )
                .variant(AlertVariant::Warning)
                .dismissable(true)
                .show(ui)
                {
                    if close.clicked() {
                        self.gallery_state.alert_warning_open = false;
                        self.gallery_state
                            .toasts
                            .info("Warning dismissed", ToastPosition::BottomRight);
                    }
                }
            } else {
                ui.label(
                    RichText::new("Warning dismissed — reset the gallery to see it again.")
                        .size(12.0)
                        .color(theme.text_muted),
                );
            }

            let ui = &mut columns[1];
            if self.gallery_state.alert_error_open {
                if let Some(close) = Alert::new(
                    "Execution interrupted",
                    "Query cancelled: statement timeout exceeded (30,000 ms).",
                    theme,
                )
                .variant(AlertVariant::Destructive)
                .dismissable(true)
                .show(ui)
                {
                    if close.clicked() {
                        self.gallery_state.alert_error_open = false;
                        self.gallery_state
                            .toasts
                            .error("Error dismissed", ToastPosition::BottomRight);
                    }
                }
            } else {
                ui.label(
                    RichText::new("Error dismissed — reset the gallery to see it again.")
                        .size(12.0)
                        .color(theme.text_muted),
                );
            }

            ui.add_space(10.0);

            Alert::new(
                "Database Migration Ready",
                "Schema comparison generated 4 migration scripts. Review DDL impact before applying to production.",
                theme,
            )
            .variant(AlertVariant::Info)
            .show(ui);
        });
    }

    fn draw_gallery_feedback_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Feedback, Progress & Shortcuts",
            "Progress bars, animated spinners, and keyboard shortcut badges.",
        );

        Card::new(theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                // Determinate Animated Progress
                ui.vertical(|ui| {
                    ui.set_width(320.0);
                    let pct = (self.gallery_state.progress_val * 100.0).round() as i32;
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("Smooth Progress ({}%)", pct))
                                .size(12.5)
                                .strong()
                                .color(theme.text_secondary),
                        );
                        ui.add_space(8.0);
                        if Button::new(theme)
                            .text("-10%")
                            .size(ButtonSize::Sm)
                            .variant(ButtonVariant::Ghost)
                            .show(ui)
                            .clicked()
                        {
                            self.gallery_state.progress_val = (self.gallery_state.progress_val - 0.1).max(0.0);
                        }
                        if Button::new(theme)
                            .text("+10%")
                            .size(ButtonSize::Sm)
                            .variant(ButtonVariant::Ghost)
                            .show(ui)
                            .clicked()
                        {
                            self.gallery_state.progress_val = (self.gallery_state.progress_val + 0.1).min(1.0);
                        }
                    });
                    ui.add_space(6.0);
                    Progress::new(self.gallery_state.progress_val, theme)
                        .height(8.0)
                        .animated(true)
                        .show(ui);
                });

                ui.add_space(32.0);

                // Indeterminate Animated Progress Beam
                ui.vertical(|ui| {
                    ui.set_width(280.0);
                    ui.label(
                        RichText::new("Indeterminate Animated Beam")
                            .size(12.5)
                            .strong()
                            .color(theme.text_secondary),
                    );
                    ui.add_space(6.0);
                    Progress::indeterminate(theme).height(8.0).show(ui);
                });

                ui.add_space(32.0);

                // Vector Spinner & Pulse
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Animated Vector Spinner")
                            .size(12.5)
                            .strong()
                            .color(theme.text_secondary),
                    );
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        Spinner::new(theme).size(20.0).show(ui);
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Syncing schema metadata...")
                                .size(12.0)
                                .color(theme.text_muted),
                        );
                    });
                });
            });

            ui.add_space(16.0);
            separator_with_text(ui, "KEYBOARD SHORTCUTS & BADGES", theme);
            ui.add_space(12.0);

            ui.horizontal_wrapped(|ui| {
                if Button::new(theme)
                    .text("Open Shortcuts Cheatsheet (Dialog)")
                    .icon(Icon::Keyboard)
                    .variant(ButtonVariant::Secondary)
                    .show(ui)
                    .clicked()
                {
                    self.gallery_state.shortcuts_dialog_open = true;
                }

                ui.add_space(16.0);
                ui.label(RichText::new("Quick Open:").size(12.0).color(theme.text_secondary));
                kbd_combo(ui, &["Cmd", "P"], theme);
                ui.add_space(12.0);

                ui.label(RichText::new("Command Palette:").size(12.0).color(theme.text_secondary));
                kbd_combo(ui, &["Shift", "Cmd", "P"], theme);
                ui.add_space(12.0);

                ui.label(RichText::new("Run Query:").size(12.0).color(theme.text_secondary));
                kbd_combo(ui, &["Cmd", "Enter"], theme);
                ui.add_space(12.0);

                ui.label(RichText::new("Format Code:").size(12.0).color(theme.text_secondary));
                kbd_combo(ui, &["Opt", "Shift", "F"], theme);
            });

            // Keyboard Shortcuts Dialog
            let mut shortcuts_open = self.gallery_state.shortcuts_dialog_open;
            Dialog::new(&mut shortcuts_open, "Keyboard Shortcuts Cheatsheet", theme)
                .id_salt("gallery_shortcuts_cheatsheet_dialog")
                .description("System-wide hotkeys and shortcuts for rapid database workflows.")
                .width(540.0)
                .show(ui, |ui| {
                    type ShortcutEntry = (&'static str, &'static [&'static str]);
                    type ShortcutGroup = (&'static str, &'static [ShortcutEntry]);
                    let shortcut_groups: [ShortcutGroup; 4] = [
                        (
                            "Navigation & Workspace",
                            &[
                                ("Quick Open Workspaces", &["Cmd", "P"]),
                                ("Command Palette", &["Shift", "Cmd", "P"]),
                                ("Toggle Primary Sidebar", &["Cmd", "B"]),
                                ("Close Active Tab", &["Cmd", "W"]),
                                ("New Query Document", &["Cmd", "T"]),
                            ],
                        ),
                        (
                            "Query & SQL Editor",
                            &[
                                ("Execute Full Query", &["Cmd", "Enter"]),
                                ("Execute Current Statement", &["Shift", "Cmd", "Enter"]),
                                ("Explain Query Plan", &["Opt", "Cmd", "E"]),
                                ("Format SQL Code", &["Opt", "Shift", "F"]),
                                ("Find / Replace", &["Cmd", "F"]),
                            ],
                        ),
                        (
                            "Data Grid & Explorer",
                            &[
                                ("Refresh Table Data", &["Cmd", "R"]),
                                ("Filter Rows", &["Shift", "Cmd", "F"]),
                                ("Export Results to CSV", &["Shift", "Cmd", "E"]),
                                ("Stage New Row", &["Cmd", "N"]),
                                ("Revert Unsaved Changes", &["Cmd", "Z"]),
                            ],
                        ),
                        (
                            "AI Agent & Copilot",
                            &[
                                ("Open AI Agent Panel", &["Cmd", "J"]),
                                ("Generate SQL from Prompt", &["Cmd", "K"]),
                                ("Approve AI Migration", &["Cmd", "Y"]),
                                ("Reject AI Proposal", &["Cmd", "N"]),
                            ],
                        ),
                    ];

                    for (group_title, items) in shortcut_groups {
                        ui.add_space(10.0);
                        ui.label(RichText::new(group_title).size(12.5).strong().color(theme.text_primary));
                        ui.add_space(6.0);

                        for (action_name, keys) in items {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(*action_name).size(12.0).color(theme.text_secondary));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    kbd_combo(ui, keys, theme);
                                });
                            });
                            ui.add_space(3.0);
                        }
                    }

                    ui.add_space(16.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if Button::new(theme)
                            .text("Done")
                            .variant(ButtonVariant::Default)
                            .show(ui)
                            .clicked()
                        {
                            self.gallery_state.shortcuts_dialog_open = false;
                        }
                    });
                });
            self.gallery_state.shortcuts_dialog_open = shortcuts_open;

            ui.add_space(12.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Floating Toasts (Multi-Stack & Positions):")
                            .size(12.0)
                            .strong()
                            .color(theme.text_secondary),
                    );
                    if !self.gallery_state.toasts.is_empty() {
                        ui.add_space(8.0);
                        if Button::new(theme)
                            .text(format!("Clear all ({})", self.gallery_state.toasts.len()))
                            .size(ButtonSize::Sm)
                            .variant(ButtonVariant::Ghost)
                            .show(ui)
                            .clicked()
                        {
                            self.gallery_state.toasts.clear();
                        }
                    }
                });
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    if Button::new(theme)
                        .text("Top Left")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Secondary)
                        .show(ui)
                        .clicked()
                    {
                        self.gallery_state
                            .toasts
                            .info("Top Left notification", ToastPosition::TopLeft);
                    }
                    if Button::new(theme)
                        .text("Top Center")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Secondary)
                        .show(ui)
                        .clicked()
                    {
                        self.gallery_state.toasts.success(
                            "Top Center: Query executed successfully (12ms)",
                            ToastPosition::TopCenter,
                        );
                    }
                    if Button::new(theme)
                        .text("Top Right")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Secondary)
                        .show(ui)
                        .clicked()
                    {
                        self.gallery_state.toasts.show_with_action(
                            "Top Right notification",
                            ToastVariant::Default,
                            ToastPosition::TopRight,
                            "View",
                        );
                    }
                    if Button::new(theme)
                        .text("Bottom Left")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Secondary)
                        .show(ui)
                        .clicked()
                    {
                        self.gallery_state
                            .toasts
                            .info("Bottom Left notification", ToastPosition::BottomLeft);
                    }
                    if Button::new(theme)
                        .text("Bottom Center")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Secondary)
                        .show(ui)
                        .clicked()
                    {
                        self.gallery_state.toasts.show_with_action(
                            "Bottom Center: Connection timeout",
                            ToastVariant::Danger,
                            ToastPosition::BottomCenter,
                            "Retry",
                        );
                    }
                    if Button::new(theme)
                        .text("Bottom Right (Default)")
                        .size(ButtonSize::Sm)
                        .variant(ButtonVariant::Secondary)
                        .show(ui)
                        .clicked()
                    {
                        self.gallery_state
                            .toasts
                            .success("Bottom Right: Changes saved", ToastPosition::BottomRight);
                    }

                    ui.add_space(8.0);
                    if Button::new(theme)
                        .text("🔥 Fire 3 Stacked Toasts")
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.gallery_state
                            .toasts
                            .success("1. Schema synchronized with remote", ToastPosition::BottomRight);
                        self.gallery_state.toasts.show_with_action(
                            "2. Migration 0042 staged for review",
                            ToastVariant::Default,
                            ToastPosition::BottomRight,
                            "Review",
                        );
                        self.gallery_state.toasts.show_with_action(
                            "3. Connection pool saturated (95%)",
                            ToastVariant::Danger,
                            ToastPosition::BottomRight,
                            "Expand",
                        );
                    }
                });
            });

            ui.add_space(16.0);
            separator_with_text(ui, "SKELETON, EMPTY STATE, TOAST", theme);
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(180.0);
                    Skeleton::new(theme).size(160.0, 12.0).show(ui);
                    ui.add_space(8.0);
                    Skeleton::new(theme).size(120.0, 12.0).show(ui);
                    ui.add_space(8.0);
                    Skeleton::new(theme).size(180.0, 28.0).rounding(8.0).show(ui);
                });
                ui.add_space(24.0);
                ui.vertical(|ui| {
                    ui.set_width(260.0);
                    EmptyState::new(
                        Icon::FolderOpen,
                        "No saved queries",
                        "Create your first query to get started.",
                        theme,
                    )
                    .action("New query")
                    .show(ui);
                });
                ui.add_space(24.0);
                ui.vertical(|ui| {
                    ui.set_width(280.0);
                    Toast::new("Changes saved", theme)
                        .variant(ToastVariant::Success)
                        .show(ui);
                    ui.add_space(8.0);
                    Toast::new("Failed to save changes", theme)
                        .variant(ToastVariant::Danger)
                        .action("Retry")
                        .show(ui);
                });
            });
        });
    }

    fn draw_gallery_navigation_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(ui, "Navigation & Tabs", "Segmented controls and underline tabs.");

        Card::new(theme).show(ui, |ui| {
            ui.label(
                RichText::new("Segmented Pill Tabs")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(8.0);
            SegmentedTabs::new(
                &mut self.gallery_state.segmented_tab_idx,
                &["Overview", "Query Analytics", "Locks & Activity", "Storage Metrics"],
                theme,
            )
            .show(ui);

            ui.add_space(20.0);

            ui.label(
                RichText::new("Underline Tabs")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(8.0);
            UnderlineTabs::new(
                &mut self.gallery_state.underline_tab_idx,
                &["Columns & Types", "Indexes (4)", "Foreign Keys (2)", "DDL Script"],
                theme,
            )
            .show(ui);
        });

        ui.add_space(16.0);
        Card::new(theme).show(ui, |ui| {
            PageHeader::new("Schema explorer", theme)
                .description("Browse databases, schemas, and objects.")
                .show(ui);
            ui.add_space(12.0);
            let crumbs = [
                BreadcrumbItem::new("localhost"),
                BreadcrumbItem::new("postgres"),
                BreadcrumbItem::new("public").current(true),
            ];
            Breadcrumb::new(&crumbs, theme).show(ui);
            ui.add_space(12.0);
            SectionHeader::new("Tables", theme)
                .description("Base tables in the selected schema")
                .show(ui);
            ui.add_space(12.0);
            Toolbar::new(theme).show(ui, |ui| {
                toolbar_button(ui, "Run", Icon::Play, theme);
                toolbar_button(ui, "Explain", Icon::Search, theme);
                Button::new(theme)
                    .icon(Icon::RefreshCw)
                    .size(ButtonSize::IconSm)
                    .variant(ButtonVariant::Ghost)
                    .show(ui);
            });
            ui.add_space(12.0);
            Pagination::new(&mut self.gallery_state.pagination_page, 8, theme).show(ui);
        });
    }

    fn draw_gallery_overlays_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Overlays",
            "Tooltip, popover, dropdown, dialog, and sheet using the shared floating surface.",
        );

        Card::new(theme).show(ui, |ui| {
            ui.label(
                RichText::new("Tooltips (Positions & Shortcuts):")
                    .size(12.5)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                let btn_top = Button::new(theme)
                    .text("Top Tooltip")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Outline)
                    .show(ui);
                Tooltip::new("Tooltip positioned on top", theme)
                    .position(TooltipPosition::Top)
                    .show(&btn_top);

                ui.add_space(8.0);
                let btn_bottom = Button::new(theme)
                    .text("Bottom Tooltip")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Outline)
                    .show(ui);
                Tooltip::new("Tooltip positioned on bottom", theme)
                    .position(TooltipPosition::Bottom)
                    .show(&btn_bottom);

                ui.add_space(8.0);
                let btn_left = Button::new(theme)
                    .text("Left Tooltip")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Outline)
                    .show(ui);
                Tooltip::new("Tooltip positioned on left", theme)
                    .position(TooltipPosition::Left)
                    .show(&btn_left);

                ui.add_space(8.0);
                let btn_right = Button::new(theme)
                    .text("Right Tooltip")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Outline)
                    .show(ui);
                Tooltip::new("Tooltip positioned on right", theme)
                    .position(TooltipPosition::Right)
                    .show(&btn_right);

                ui.add_space(8.0);
                let btn_shortcut = Button::new(theme)
                    .text("With Hotkey")
                    .icon(Icon::Copy)
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Secondary)
                    .show(ui);
                Tooltip::new("Copy active SQL query", theme)
                    .shortcut("⌘C")
                    .position(TooltipPosition::Top)
                    .show(&btn_shortcut);
            });

            ui.add_space(16.0);
            ui.label(
                RichText::new("Popovers, Menus & Overlays:")
                    .size(12.5)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                let popover_trigger = Button::new(theme)
                    .text("Open popover")
                    .variant(ButtonVariant::Secondary)
                    .show(ui);
                Popover::new(&mut self.gallery_state.popover_open, theme).show(ui, &popover_trigger, |ui| {
                    ui.set_min_width(220.0);
                    ui.label(
                        RichText::new("Pinned Schema Context")
                            .size(13.0)
                            .strong()
                            .color(theme.text_primary),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("public.users · 24 columns · 1.8M rows")
                            .size(12.0)
                            .color(theme.text_secondary),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Click outside anywhere to dismiss.")
                            .size(11.0)
                            .color(theme.text_muted),
                    );
                });

                ui.add_space(8.0);
                let menu_trigger = Button::new(theme).text("Actions").icon(Icon::ChevronDown).show(ui);
                let items = [
                    DropdownItem::new("Run query").icon(Icon::Play).shortcut("⌘↵"),
                    DropdownItem::new("Explain").icon(Icon::Search),
                    DropdownItem::new("Delete connection").icon(Icon::Trash2).danger(true),
                ];
                DropdownMenu::new(&mut self.gallery_state.dropdown_open, &items, theme).show(ui, &menu_trigger);

                ui.add_space(8.0);
                if Button::new(theme)
                    .text("Open dialog")
                    .variant(ButtonVariant::Outline)
                    .show(ui)
                    .clicked()
                {
                    self.gallery_state.dialog_open = true;
                }

                ui.add_space(8.0);
                if Button::new(theme)
                    .text("Open sheet")
                    .variant(ButtonVariant::Ghost)
                    .show(ui)
                    .clicked()
                {
                    self.gallery_state.sheet_open = true;
                }
            });
        });

        let mut dialog_open = self.gallery_state.dialog_open;
        let mut cancel = false;
        let mut confirm = false;
        Dialog::new(&mut dialog_open, "Drop table", theme)
            .id_salt("gallery_drop_table_dialog")
            .description("This cannot be undone. Dependent views will fail until recreated.")
            .show(ui, |ui| {
                ui.label(
                    RichText::new("public.users will be removed from the catalog.")
                        .size(13.0)
                        .color(theme.text_secondary),
                );
                ui.add_space(16.0);
                let actions = dialog_actions(ui, theme, "Cancel", "Drop table");
                cancel = actions.0;
                confirm = actions.1;
            });
        if cancel || confirm {
            dialog_open = false;
        }
        self.gallery_state.dialog_open = dialog_open;

        let mut sheet_open = self.gallery_state.sheet_open;
        Sheet::new(&mut sheet_open, "Object inspector", theme)
            .id_salt("gallery_object_inspector_sheet")
            .show(ui, |ui| {
                ui.label(RichText::new("users").size(14.0).strong().color(theme.text_primary));
                ui.add_space(6.0);
                ui.label(
                    RichText::new("BASE TABLE · 128,490 rows")
                        .size(12.0)
                        .color(theme.text_secondary),
                );
                ui.add_space(12.0);
                SectionHeader::new("Columns", theme).show(ui);
                ui.add_space(6.0);
                ui.label(RichText::new("id · uuid").size(12.5).color(theme.text_primary));
                ui.label(RichText::new("email · text").size(12.5).color(theme.text_primary));
            });
        self.gallery_state.sheet_open = sheet_open;
    }

    fn draw_gallery_tables_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Data Display Table",
            "Modern data table with sortable columns, row selection, live filtering, and pagination.",
        );

        let all_rows = [
            (
                "users",
                "BASE TABLE",
                "128,490",
                128490,
                "14.2 MB",
                14.2,
                BadgeVariant::Success,
                "Active",
                Icon::Table,
            ),
            (
                "trips",
                "BASE TABLE",
                "1,842,109",
                1842109,
                "184.6 MB",
                184.6,
                BadgeVariant::Success,
                "Active",
                Icon::Table,
            ),
            (
                "trip_legs",
                "BASE TABLE",
                "4,291,012",
                4291012,
                "412.0 MB",
                412.0,
                BadgeVariant::Success,
                "Active",
                Icon::Table,
            ),
            (
                "vehicles",
                "BASE TABLE",
                "3,450",
                3450,
                "512 KB",
                0.5,
                BadgeVariant::Success,
                "Active",
                Icon::Table,
            ),
            (
                "v_active_bookings",
                "VIEW",
                "—",
                0,
                "—",
                0.0,
                BadgeVariant::Info,
                "View",
                Icon::Eye,
            ),
            (
                "audit_logs_archive",
                "PARTITION",
                "8,920,111",
                8920111,
                "980.2 MB",
                980.2,
                BadgeVariant::Secondary,
                "Archived",
                Icon::Archive,
            ),
        ];

        // ── 1. Table Toolbar ──────────────────────────────────────────
        ui.horizontal(|ui| {
            let _ = ui.allocate_ui_with_layout(
                egui::Vec2::new(260.0, 32.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| SearchInput::new(&mut self.gallery_state.table_search, "Filter objects...", theme).show(ui),
            );

            let filter_query = self.gallery_state.table_search.trim().to_lowercase();
            let matching_count = all_rows
                .iter()
                .filter(|r| {
                    filter_query.is_empty()
                        || r.0.to_lowercase().contains(&filter_query)
                        || r.1.to_lowercase().contains(&filter_query)
                })
                .count();

            ui.add_space(8.0);
            let badge_text = format!("{matching_count} objects");
            Badge::new(&badge_text, theme).variant(BadgeVariant::Secondary).show(ui);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if Button::new(theme)
                    .text("Export CSV")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .icon(Icon::Download)
                    .show(ui)
                    .clicked()
                {
                    // Noop demonstration
                }

                if Button::new(theme)
                    .text("Refresh")
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .icon(Icon::RotateCcw)
                    .show(ui)
                    .clicked()
                {
                    self.gallery_state.table_search.clear();
                }
            });
        });

        ui.add_space(10.0);

        // Filter and sort rows
        let filter_query = self.gallery_state.table_search.trim().to_lowercase();
        let mut row_indices: Vec<usize> = (0..all_rows.len())
            .filter(|&idx| {
                let r = &all_rows[idx];
                filter_query.is_empty()
                    || r.0.to_lowercase().contains(&filter_query)
                    || r.1.to_lowercase().contains(&filter_query)
            })
            .collect();

        if let Some(sort_col) = self.gallery_state.table_sort_col {
            let desc = self.gallery_state.table_sort_desc;
            row_indices.sort_by(|&a, &b| {
                let ra = &all_rows[a];
                let rb = &all_rows[b];
                let ord = match sort_col {
                    0 => ra.0.cmp(rb.0),
                    1 => ra.1.cmp(rb.1),
                    2 => ra.3.cmp(&rb.3),
                    3 => ra.5.partial_cmp(&rb.5).unwrap_or(std::cmp::Ordering::Equal),
                    4 => ra.7.cmp(rb.7),
                    _ => std::cmp::Ordering::Equal,
                };
                if desc {
                    ord.reverse()
                } else {
                    ord
                }
            });
        }

        let columns = [
            TableColumn::new("Table Name").sortable(true),
            TableColumn::new("Type").width(120.0).sortable(true),
            TableColumn::new("Row Count")
                .width(130.0)
                .align(TableColumnAlign::Right)
                .sortable(true),
            TableColumn::new("Size")
                .width(110.0)
                .align(TableColumnAlign::Right)
                .sortable(true),
            TableColumn::new("Status")
                .width(120.0)
                .align(TableColumnAlign::Center)
                .sortable(true),
            TableColumn::new("Actions").width(100.0).align(TableColumnAlign::Right),
        ];

        let visible_count = row_indices.len();
        let selected_count = row_indices
            .iter()
            .filter(|idx| self.gallery_state.table_selected_rows.contains(idx))
            .count();
        let all_selected = visible_count > 0 && selected_count == visible_count;
        let is_indeterminate = selected_count > 0 && !all_selected;

        let sort_col = self.gallery_state.table_sort_col;
        let sort_desc = self.gallery_state.table_sort_desc;

        let mut toggle_all_target = None;
        let mut toggle_row_target = None;
        let mut toggle_sort_col = None;

        Table::new(&columns, theme)
            .selectable(true, all_selected)
            .indeterminate(is_indeterminate)
            .sort(sort_col, sort_desc)
            .row_height(44.0)
            .vertical_grid(false)
            .show(
                ui,
                visible_count,
                |v_idx| {
                    let real_idx = row_indices[v_idx];
                    self.gallery_state.table_selected_rows.contains(&real_idx)
                },
                |new_all| {
                    toggle_all_target = Some(new_all);
                },
                |v_idx| {
                    let real_idx = row_indices[v_idx];
                    toggle_row_target = Some(real_idx);
                },
                |clicked_col| {
                    toggle_sort_col = Some(clicked_col);
                },
                |ui, v_idx, col_idx| {
                    let real_idx = row_indices[v_idx];
                    let row = &all_rows[real_idx];

                    match col_idx {
                        0 => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(char::from(row.8).to_string())
                                        .font(egui::FontId::new(13.0, egui::FontFamily::Name("lucide".into())))
                                        .color(theme.accent),
                                );
                                ui.add_space(6.0);
                                ui.label(RichText::new(row.0).strong().size(12.5).color(theme.text_primary));
                            });
                        }
                        1 => {
                            Badge::new(row.1, theme).variant(BadgeVariant::Outline).show(ui);
                        }
                        2 => {
                            ui.label(RichText::new(row.2).monospace().size(12.0).color(theme.text_primary));
                        }
                        3 => {
                            ui.label(RichText::new(row.4).monospace().size(12.0).color(theme.text_secondary));
                        }
                        4 => {
                            Badge::new(row.7, theme).variant(row.6).dot(true).compact(true).show(ui);
                        }
                        5 => {
                            ui.horizontal(|ui| {
                                Button::new(theme)
                                    .icon(Icon::Pencil)
                                    .size(ButtonSize::IconSm)
                                    .variant(ButtonVariant::Ghost)
                                    .access_label("Edit")
                                    .show(ui);
                                Button::new(theme)
                                    .icon(Icon::MoreHorizontal)
                                    .size(ButtonSize::IconSm)
                                    .variant(ButtonVariant::Ghost)
                                    .access_label("More actions")
                                    .show(ui);
                            });
                        }
                        _ => {}
                    }
                },
            );

        // Apply state updates
        if let Some(new_all) = toggle_all_target {
            if new_all {
                for &idx in &row_indices {
                    self.gallery_state.table_selected_rows.insert(idx);
                }
            } else {
                for &idx in &row_indices {
                    self.gallery_state.table_selected_rows.remove(&idx);
                }
            }
        }

        if let Some(target) = toggle_row_target {
            if self.gallery_state.table_selected_rows.contains(&target) {
                self.gallery_state.table_selected_rows.remove(&target);
            } else {
                self.gallery_state.table_selected_rows.insert(target);
            }
        }

        if let Some(clicked_col) = toggle_sort_col {
            if self.gallery_state.table_sort_col == Some(clicked_col) {
                if !self.gallery_state.table_sort_desc {
                    self.gallery_state.table_sort_desc = true;
                } else {
                    self.gallery_state.table_sort_col = None;
                    self.gallery_state.table_sort_desc = false;
                }
            } else {
                self.gallery_state.table_sort_col = Some(clicked_col);
                self.gallery_state.table_sort_desc = false;
            }
        }

        // ── 3. Table Pagination & Selection Footer ────────────────────
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let selected_count = self.gallery_state.table_selected_rows.len();
            ui.label(
                RichText::new(format!("{selected_count} of {} row(s) selected", all_rows.len()))
                    .size(12.0)
                    .color(theme.text_secondary),
            );

            if selected_count > 0 {
                ui.add_space(4.0);
                if Button::new(theme)
                    .text("Clear selection")
                    .variant(ButtonVariant::Link)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.gallery_state.table_selected_rows.clear();
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                Button::new(theme)
                    .text("Next")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .icon(Icon::ChevronRight)
                    .enabled(false)
                    .show(ui);

                ui.add_space(4.0);
                Button::new(theme)
                    .text("Previous")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .icon(Icon::ChevronLeft)
                    .enabled(false)
                    .show(ui);

                ui.add_space(8.0);
                ui.label(RichText::new("Page 1 of 1").size(12.0).color(theme.text_muted));
            });
        });
    }

    fn draw_gallery_devtools_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Developer Tools & Code Primitives",
            "Specialized primitives for developer environments: InlineCode, CodeBlock with copy, DiffViewer, and Schema Tree.",
        );

        Card::new(theme).show(ui, |ui| {
            // Row 1: InlineCode & CodeBlock
            ui.label(RichText::new("Inline Code & CodeBlock").size(13.0).strong().color(theme.text_secondary));
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Execute query with safe limit:").size(13.0).color(theme.text_primary));
                InlineCode::new("SELECT * FROM users WHERE status = 'active' LIMIT 50;", theme).show(ui);
                ui.label(RichText::new("or connect via").size(13.0).color(theme.text_primary));
                InlineCode::new("postgresql://localhost:5432/main", theme).show(ui);
            });

            ui.add_space(14.0);

            let sample_sql = "-- Optimize query: create composite index for fast join\nCREATE INDEX CONCURRENTLY idx_users_organization_created\nON users (organization_id, created_at DESC)\nWHERE deleted_at IS NULL;\n\nSELECT u.id, u.email, o.name AS organization\nFROM users u\nJOIN organizations o ON o.id = u.organization_id\nWHERE u.status = 'active'\nORDER BY u.created_at DESC\nLIMIT 25;";

            CodeBlock::new(sample_sql, theme)
                .language("sql")
                .show_line_numbers(true)
                .show(ui);

            ui.add_space(20.0);

            // Row 2: DiffViewer & Schema Tree
            ui.columns(2, |cols| {
                // Col 1: DiffViewer
                let ui = &mut cols[0];
                ui.label(RichText::new("Schema & SQL Diff Proposal").size(13.0).strong().color(theme.text_secondary));
                ui.add_space(6.0);

                let diff_lines = [
                    DiffLine::context(10, 10, "-- Schema migration for public.users"),
                    DiffLine::context(11, 11, "ALTER TABLE users ADD COLUMN is_verified BOOLEAN DEFAULT false;"),
                    DiffLine::removed(12, "CREATE INDEX idx_users_temp ON users(email);"),
                    DiffLine::added(12, "CREATE INDEX CONCURRENTLY idx_users_email_verified"),
                    DiffLine::added(13, "ON users (email, is_verified) WHERE is_verified = true;"),
                    DiffLine::context(13, 14, "COMMENT ON COLUMN users.is_verified IS 'Email verified status';"),
                ];

                DiffViewer::new("migration_0042_user_verification.sql", &diff_lines, theme).show(ui);

                // Col 2: Schema Tree
                let ui = &mut cols[1];
                ui.label(RichText::new("Hierarchical Database Tree").size(13.0).strong().color(theme.text_secondary));
                ui.add_space(6.0);

                let tree_frame = egui::Frame::none()
                    .fill(theme.surface_editor)
                    .stroke(egui::Stroke::new(1.0, theme.border_default))
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(6.0));

                tree_frame.show(ui, |ui| {
                    DatabaseTreeNode::new("localhost:5432", TreeNodeKind::Server, 0, theme)
                        .detail("PostgreSQL 16.2")
                        .expanded(&mut self.gallery_state.tree_server_expanded)
                        .show(ui);

                    reveal_children(ui, ui.id().with("tree_server"), self.gallery_state.tree_server_expanded, |ui| {
                        DatabaseTreeNode::new("production_db", TreeNodeKind::Database, 1, theme)
                            .detail("UTF-8")
                            .expanded(&mut self.gallery_state.tree_db_expanded)
                            .show(ui);

                        reveal_children(ui, ui.id().with("tree_db"), self.gallery_state.tree_db_expanded, |ui| {
                            DatabaseTreeNode::new("public", TreeNodeKind::Schema, 2, theme)
                                .detail("12 tables")
                                .expanded(&mut self.gallery_state.tree_schema_expanded)
                                .show(ui);

                            reveal_children(
                                ui,
                                ui.id().with("tree_schema"),
                                self.gallery_state.tree_schema_expanded,
                                |ui| {
                                    DatabaseTreeNode::new("users", TreeNodeKind::Table, 3, theme)
                                        .detail("1,842,109 rows")
                                        .selected(true)
                                        .expanded(&mut self.gallery_state.tree_table_expanded)
                                        .show(ui);

                                    reveal_children(
                                        ui,
                                        ui.id().with("tree_table"),
                                        self.gallery_state.tree_table_expanded,
                                        |ui| {
                                            DatabaseTreeNode::new("id", TreeNodeKind::PrimaryKey, 4, theme)
                                                .detail("bigint PK")
                                                .show(ui);
                                            DatabaseTreeNode::new("email", TreeNodeKind::Column, 4, theme)
                                                .detail("varchar(255)")
                                                .show(ui);
                                            DatabaseTreeNode::new("organization_id", TreeNodeKind::ForeignKey, 4, theme)
                                                .detail("bigint -> orgs.id")
                                                .show(ui);
                                            DatabaseTreeNode::new("idx_users_email", TreeNodeKind::Index, 4, theme)
                                                .detail("btree (email)")
                                                .show(ui);
                                        },
                                    );

                                    DatabaseTreeNode::new("orders", TreeNodeKind::Table, 3, theme)
                                        .detail("4,291,012 rows")
                                        .show(ui);

                                    DatabaseTreeNode::new("v_active_bookings", TreeNodeKind::View, 3, theme)
                                        .detail("view")
                                        .show(ui);
                                },
                            );
                        });
                    });
                });
            });

            ui.add_space(20.0);

            // Row 3: SQL Editor Toolbar & Action Bar
            ui.label(RichText::new("SQL Editor Toolbar & Diagnostics").size(13.0).strong().color(theme.text_secondary));
            ui.add_space(6.0);

            let sql_action = SqlEditorToolbar::new(false, true, theme).show(ui);
            match sql_action {
                Some(SqlEditorAction::RunQuery) => {
                    self.gallery_state.sql_editor_status = Some("Executing full SQL buffer...".to_owned());
                }
                Some(SqlEditorAction::RunSelection) => {
                    self.gallery_state.sql_editor_status = Some("Executing selected SQL range (Ln 5-8)...".to_owned());
                }
                Some(SqlEditorAction::ExplainQuery) => {
                    self.gallery_state.sql_editor_status = Some("Running EXPLAIN (ANALYZE, BUFFERS)...".to_owned());
                }
                Some(SqlEditorAction::FormatSql) => {
                    self.gallery_state.sql_editor_status = Some("SQL formatted with canonical keyword casing.".to_owned());
                }
                Some(SqlEditorAction::AskAi) => {
                    self.gallery_state.sql_editor_status = Some("Opening Agent Copilot with active SQL context...".to_owned());
                }
                _ => {}
            }

            if let Some(ref status) = self.gallery_state.sql_editor_status {
                ui.add_space(6.0);
                Alert::new("Editor Event", status, theme)
                    .variant(AlertVariant::Default)
                    .show(ui);
            }

            ui.add_space(20.0);

            // Row 4: Explain Plan Tree & Performance Bottlenecks
            ui.columns(2, |cols| {
                // Col 1: ExplainPlanTree
                let ui = &mut cols[0];
                let plan_root = PlanNode::new("Hash Join", 1420.5, 38.4, 18420)
                    .with_child(
                        PlanNode::new("Seq Scan", 940.0, 31.2, 142000)
                            .relation("users")
                            .bottleneck(true),
                    )
                    .with_child(
                        PlanNode::new("Index Scan", 48.0, 1.8, 18420)
                            .relation("organizations")
                            .bottleneck(false),
                    );

                ExplainPlanTree::new(&plan_root, 38.4, theme).show(ui);

                // Col 2: Execution Log Viewer
                let ui = &mut cols[1];
                ui.label(RichText::new("Query Execution Log").size(13.0).strong().color(theme.text_secondary));
                ui.add_space(6.0);

                let log_entries = [
                    LogEntry::new("12:04:01.102", LogLevel::Info, "Connected to PostgreSQL 16.2 on port 5432"),
                    LogEntry::new("12:04:01.350", LogLevel::Notice, "Temporary table pg_temp_04 created"),
                    LogEntry::new("12:04:02.810", LogLevel::Warning, "Query scan cost exceeds budget (1420 > 1000)"),
                    LogEntry::new("12:04:03.119", LogLevel::Info, "Query returned 25 rows in 38.4ms"),
                ];

                LogViewer::new(&log_entries, theme).show(ui);
            });

            ui.add_space(20.0);

            // Row 5: Terminal Output & Progress Ring
            ui.columns(2, |cols| {
                // Col 1: TerminalBlock
                let ui = &mut cols[0];
                ui.label(RichText::new("Terminal / CLI Output").size(13.0).strong().color(theme.text_secondary));
                ui.add_space(6.0);
                TerminalBlock::new("psql -h localhost -U postgres -d production_db\npsql (16.2)\nType \"help\" for help.\n\nproduction_db=# VACUUM ANALYZE users;\nVACUUM", theme)
                    .title("psql session: production_db")
                    .show(ui);

                // Col 2: ProgressRing
                let ui = &mut cols[1];
                ui.label(RichText::new("Progress Ring Indicator").size(13.0).strong().color(theme.text_secondary));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ProgressRing::new(0.75, 20.0, theme).show(ui);
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new("75% Schema Indexing").size(12.5).strong().color(theme.text_primary));
                        ui.label(RichText::new("Building idx_users_email concurrently...").size(11.5).color(theme.text_secondary));
                    });
                });
            });
        });
    }

    fn draw_gallery_database_shell_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Database Connection & Application Shell",
            "Connection cards, health monitors, activity bar, status bar, and driver badges.",
        );

        Card::new(theme).show(ui, |ui| {
            // Row 1: Connection Cards & Badges
            ui.label(
                RichText::new("Connection Cards & Database Driver Badges")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(6.0);

            ui.horizontal_wrapped(|ui| {
                DatabaseTypeBadge::new(DatabaseDriver::PostgreSql, theme).show(ui);
                ui.add_space(6.0);
                DatabaseTypeBadge::new(DatabaseDriver::Sqlite, theme).show(ui);
                ui.add_space(6.0);
                DatabaseTypeBadge::new(DatabaseDriver::MySql, theme).show(ui);
                ui.add_space(6.0);
                DatabaseTypeBadge::new(DatabaseDriver::SqlServer, theme).show(ui);
            });

            ui.add_space(14.0);

            ui.columns(2, |cols| {
                // Col 1: ConnectionCard
                let ui = &mut cols[0];
                let card_action = ConnectionCard::new(
                    "Production PostgreSQL Cluster",
                    DatabaseDriver::PostgreSql,
                    "db.internal.company.com:5432",
                    "production_app",
                    self.gallery_state.db_card_status,
                    theme,
                )
                .ssl(true)
                .show(ui);

                match card_action {
                    Some(ConnectionCardAction::Connect) => {
                        self.gallery_state.db_card_status = ConnectionStatus::Connected;
                        self.gallery_state.toasts.show(
                            "Connected to Production Cluster.",
                            ToastVariant::Success,
                            ToastPosition::BottomRight,
                        );
                    }
                    Some(ConnectionCardAction::Disconnect) => {
                        self.gallery_state.db_card_status = ConnectionStatus::Disconnected;
                        self.gallery_state.toasts.show(
                            "Disconnected from database.",
                            ToastVariant::Default,
                            ToastPosition::BottomRight,
                        );
                    }
                    Some(ConnectionCardAction::Edit) => {
                        self.gallery_state.dialog_open = true;
                    }
                    _ => {}
                }

                // Col 2: Connection Health Indicators
                let ui = &mut cols[1];
                ui.label(
                    RichText::new("Connection Health & Latency Monitors")
                        .size(12.5)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(6.0);

                ConnectionIndicator::new("Xe Lạc Hồng (PostgreSQL)", "pg 16.2", ConnectionHealth::Healthy, theme)
                    .latency(42)
                    .show(ui);

                ui.add_space(8.0);

                ConnectionIndicator::new("Analytics SQLite", "sqlite 3.45", ConnectionHealth::Degraded, theme)
                    .latency(280)
                    .show(ui);

                ui.add_space(8.0);

                ConnectionIndicator::new(
                    "Legacy Reporting DB",
                    "mysql 8.0",
                    ConnectionHealth::Disconnected,
                    theme,
                )
                .show(ui);
            });

            ui.add_space(20.0);

            // Row 2: Status Bar Demo
            ui.label(
                RichText::new("Application Status Bar (Editor & Database Meta)")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(6.0);

            let left_status = [
                StatusBarItem::new("Xe Lạc Hồng (PostgreSQL)")
                    .icon(Icon::Database)
                    .accent(true),
                StatusBarItem::new("public.users").icon(Icon::Table),
                StatusBarItem::new("UTF-8 · READ COMMITTED"),
            ];
            let right_status = [
                StatusBarItem::new("Ln 42, Col 18"),
                StatusBarItem::new("1,842,109 rows"),
                StatusBarItem::new("42ms latency").icon(Icon::Zap),
            ];

            StatusBar::new(&left_status, &right_status, theme).show(ui);
        });
    }

    fn draw_gallery_agent_ui_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "AI Agent Workspace & Execution Components",
            "Context awareness, agent reasoning trace, plan checklist, tool approvals, transaction bar, and safe execution boundaries.",
        );

        Card::new(theme).show(ui, |ui| {
            // 0. Interactive Agent Composer
            ui.label(
                RichText::new("Agent Prompt Composer & Model Selector")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(6.0);

            let composer_action = AgentComposer::new(
                &mut self.gallery_state.composer_prompt,
                "Claude 3.5 Sonnet",
                AgentMode::Code,
                theme,
            )
            .is_generating(self.gallery_state.composer_is_generating)
            .token_usage(142)
            .show(ui);

            match composer_action {
                Some(AgentComposerAction::Submit) => {
                    let prompt_text = self.gallery_state.composer_prompt.clone();
                    self.gallery_state.composer_prompt.clear();
                    self.gallery_state.toasts.show(
                        format!("Agent request submitted: {}", prompt_text),
                        ToastVariant::Default,
                        ToastPosition::BottomRight,
                    );
                }
                Some(AgentComposerAction::Stop) => {
                    self.gallery_state.composer_is_generating = false;
                }
                _ => {}
            }

            ui.add_space(16.0);

            // 1. Agent Context Bar
            ui.label(
                RichText::new("Agent Workspace Context Bar")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("Active Context:")
                        .size(12.0)
                        .strong()
                        .color(theme.text_muted),
                );
                ui.add_space(4.0);
                ContextChip::new(ContextChipKind::Connection, "Xe Lạc Hồng (PostgreSQL)", theme).show(ui);
                ui.add_space(4.0);
                ContextChip::new(ContextChipKind::Database, "production_db", theme).show(ui);
                ui.add_space(4.0);
                ContextChip::new(ContextChipKind::Table, "public.users", theme)
                    .removable(true)
                    .show(ui);
                ui.add_space(4.0);
                ContextChip::new(ContextChipKind::Editor, "query.sql:1-18", theme)
                    .removable(true)
                    .show(ui);
                ui.add_space(4.0);
                ContextChip::new(ContextChipKind::File, "schema.sql", theme)
                    .removable(true)
                    .show(ui);
            });

            ui.add_space(16.0);

            // 2. Status Badges
            ui.label(
                RichText::new("Status Badges")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                StatusBadge::new("Connected", StatusBadgeVariant::Active, theme).show(ui);
                ui.add_space(6.0);
                StatusBadge::new("Query Running...", StatusBadgeVariant::Running, theme).show(ui);
                ui.add_space(6.0);
                StatusBadge::new("Migration Passed", StatusBadgeVariant::Success, theme).show(ui);
                ui.add_space(6.0);
                StatusBadge::new("High Latency Warning", StatusBadgeVariant::Warning, theme).show(ui);
                ui.add_space(6.0);
                StatusBadge::new("Connection Dropped", StatusBadgeVariant::Destructive, theme).show(ui);
                ui.add_space(6.0);
                StatusBadge::new("Archived Partition", StatusBadgeVariant::Archived, theme).show(ui);
                ui.add_space(6.0);
                StatusBadge::new("Draft SQL", StatusBadgeVariant::Draft, theme).show(ui);
            });

            ui.add_space(16.0);

            // 3. Agent Action Chips
            ui.label(
                RichText::new("Quick Agent Action Chips")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                for action_kind in [
                    AgentSqlActionKind::Explain,
                    AgentSqlActionKind::Optimize,
                    AgentSqlActionKind::FixError,
                    AgentSqlActionKind::GenerateMigration,
                    AgentSqlActionKind::DescribeSchema,
                    AgentSqlActionKind::ConvertDialect,
                ] {
                    if Button::new(theme)
                        .text(action_kind.label())
                        .icon(action_kind.icon())
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.gallery_state.agent_action_output =
                            Some(format!("Triggered agent action: {}", action_kind.label()));
                    }
                    ui.add_space(6.0);
                }
            });

            if let Some(ref action_text) = self.gallery_state.agent_action_output {
                ui.add_space(8.0);
                Alert::new("Agent Prompt Initiated", action_text, theme)
                    .variant(AlertVariant::Default)
                    .show(ui);
            }

            ui.add_space(20.0);

            // 4. Agent Reasoning & Multi-step Plan Checklist
            ui.columns(2, |cols| {
                // Col 1: Agent Thinking & Reasoning
                let ui = &mut cols[0];
                ui.label(
                    RichText::new("Agent Reasoning & Thought Stream")
                        .size(13.0)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(6.0);

                let mut thinking_exp = self.gallery_state.thinking_expanded;
                AgentThinking::new(
                    "1. Parsing user intent: 'Find slow queries in the past hour'.\n2. Querying pg_stat_statements for top total_exec_time.\n3. Found query #142 (avg_time = 420ms, calls = 14,200).\n4. Analyzing EXPLAIN plan: Sequential scan on table `users` filtering by `email`.\n5. Recommendation: Add B-tree index on `users(email)`.",
                    &mut thinking_exp,
                    theme,
                )
                .duration("3.8s")
                .step_count(5)
                .show(ui);
                self.gallery_state.thinking_expanded = thinking_exp;

                ui.add_space(8.0);

                let mut live_active = false;
                AgentThinking::new("Analyzing database indexes...", &mut live_active, theme)
                    .is_active(true)
                    .show(ui);

                // Col 2: Multi-step Plan Checklist
                let ui = &mut cols[1];
                ui.label(
                    RichText::new("Agent Execution Plan (Checklist)")
                        .size(13.0)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(6.0);

                let plan_tasks = vec![
                    AgentTaskItem::new("Introspect table schema & column statistics", AgentTaskStatus::Completed)
                        .with_detail("42ms"),
                    AgentTaskItem::new("Analyze sequential scans & explain plans", AgentTaskStatus::Completed)
                        .with_detail("120ms"),
                    AgentTaskItem::new("Generate concurrent index migration SQL", AgentTaskStatus::Running),
                    AgentTaskItem::new("Validate safety & prepare approval card", AgentTaskStatus::Pending),
                ];

                AgentPlan::new("Index Optimization Workflow", &plan_tasks, theme).show(ui);
            });

            ui.add_space(20.0);

            // 5. Tool Calls & Execution Approvals
            ui.columns(2, |cols| {
                // Col 1: ToolCall
                let ui = &mut cols[0];
                ui.label(
                    RichText::new("Tool Call Execution Step")
                        .size(13.0)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(6.0);

                let mut tool_exp = self.gallery_state.tool_call_expanded;
                ToolCall::new(
                    "introspect_schema_indexes",
                    ToolCallStatus::Success,
                    "{\n  \"schema\": \"public\",\n  \"table\": \"users\",\n  \"include_stats\": true\n}",
                    &mut tool_exp,
                    theme,
                )
                .duration("42ms")
                .output_preview(
                    "{\n  \"indexes_found\": 3,\n  \"missing_foreign_keys\": 0,\n  \"estimated_scan_cost\": 14820.5\n}",
                )
                .show(ui);
                self.gallery_state.tool_call_expanded = tool_exp;

                ui.add_space(8.0);

                let mut running_exp = false;
                ToolCall::new(
                    "analyze_query_bottlenecks",
                    ToolCallStatus::Running,
                    "{\n  \"query_hash\": \"0x9b4a18f\",\n  \"sample_rate\": 0.1\n}",
                    &mut running_exp,
                    theme,
                )
                .duration("120ms")
                .show(ui);

                // Col 2: Execution Approval
                let ui = &mut cols[1];
                ui.label(
                    RichText::new("Dangerous / Mutating Action Approval")
                        .size(13.0)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(6.0);

                let action = ExecutionApproval::new(
                    "Apply Database Migration: Add Concurrent Index",
                    "public.users (1,842,109 rows affected) - zero lock impact",
                    "CREATE INDEX CONCURRENTLY idx_users_email ON users(email);",
                    RiskLevel::Medium,
                    theme,
                )
                .show(ui);

                match action {
                    Some(ExecutionApprovalAction::Run) => {
                        self.gallery_state.approval_status =
                            Some("Migration executed successfully via background worker.".to_owned());
                    }
                    Some(ExecutionApprovalAction::Preview) => {
                        self.gallery_state.approval_status =
                            Some("Opening interactive SQL preview buffer...".to_owned());
                    }
                    Some(ExecutionApprovalAction::Cancel) => {
                        self.gallery_state.approval_status = Some("Proposal rejected by user.".to_owned());
                    }
                    None => {}
                }

                if let Some(ref msg) = self.gallery_state.approval_status {
                    ui.add_space(8.0);
                    Alert::new("Execution Action Triggered", msg, theme)
                        .variant(AlertVariant::Default)
                        .show(ui);
                }
            });

            ui.add_space(20.0);

            // 6. Database Transaction & Safety Management
            ui.label(
                RichText::new("Transaction Control & Destructive Operation Safety")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(6.0);

            // Transaction Bar demo
            let tx_action = TransactionBar::new(
                self.gallery_state.tx_in_transaction,
                self.gallery_state.tx_pending_mutations,
                theme,
            )
            .auto_commit(self.gallery_state.tx_auto_commit)
            .isolation_level("READ COMMITTED")
            .show(ui);

            match tx_action {
                Some(TransactionAction::Commit) => {
                    self.gallery_state.tx_pending_mutations = 0;
                    self.gallery_state.tx_in_transaction = false;
                    self.gallery_state.tx_status_message =
                        Some("Transaction committed: 3 mutations written to disk.".to_owned());
                }
                Some(TransactionAction::Rollback) => {
                    self.gallery_state.tx_pending_mutations = 0;
                    self.gallery_state.tx_in_transaction = false;
                    self.gallery_state.tx_status_message =
                        Some("Transaction rolled back: all mutations reverted.".to_owned());
                }
                Some(TransactionAction::Begin) => {
                    self.gallery_state.tx_in_transaction = true;
                    self.gallery_state.tx_pending_mutations = 1;
                    self.gallery_state.tx_status_message =
                        Some("New transaction started with READ COMMITTED isolation.".to_owned());
                }
                Some(TransactionAction::ToggleAutoCommit(val)) => {
                    self.gallery_state.tx_auto_commit = val;
                }
                None => {}
            }

            if let Some(ref msg) = self.gallery_state.tx_status_message {
                ui.add_space(6.0);
                Alert::new("Transaction Event", msg, theme)
                    .variant(AlertVariant::Default)
                    .show(ui);
            }

            ui.add_space(12.0);

            // Destructive Operation Modal Trigger
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Destructive Action Protection:")
                        .size(12.0)
                        .color(theme.text_secondary),
                );
                ui.add_space(6.0);

                if Button::new(theme)
                    .text("Trigger DROP TABLE Safety Modal")
                    .icon(Icon::Trash2)
                    .variant(ButtonVariant::Destructive)
                    .size(ButtonSize::Sm)
                    .show(ui)
                    .clicked()
                {
                    self.gallery_state.destructive_dialog_open = true;
                    self.gallery_state.destructive_keyword.clear();
                }
            });
        });

        // Render the Destructive Operation Dialog when open
        if self.gallery_state.destructive_dialog_open {
            let mut is_open = self.gallery_state.destructive_dialog_open;
            let mut keyword = self.gallery_state.destructive_keyword.clone();
            let confirmed = DestructiveOperationDialog::new(
                &mut is_open,
                "Drop Production Table",
                "This action will permanently remove public.audit_logs (14,209,102 rows) and cascade delete all associated foreign key records. This cannot be undone.",
                "public.audit_logs",
                "DROP",
                &mut keyword,
                theme,
            )
            .show(ui);

            self.gallery_state.destructive_dialog_open = is_open;
            self.gallery_state.destructive_keyword = keyword;

            if confirmed {
                self.gallery_state.toasts.show(
                    "Table public.audit_logs dropped permanently.",
                    ToastVariant::Danger,
                    ToastPosition::BottomRight,
                );
            }
        }
    }
}
