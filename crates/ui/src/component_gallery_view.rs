use super::*;
use egui::{RichText, Ui};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GalleryCategory {
    #[default]
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
    pub date_picker_val: Option<SimpleDate>,
    pub toggle_single: bool,
    pub toggle_group_val: usize,
    pub accordion_open: Option<String>,
    pub collapsible_open: bool,
    pub radio_group_val: usize,
    pub alert_dialog_open: bool,
    pub command_query: String,
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
            category: GalleryCategory::Buttons,
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
            date_picker_val: Some(SimpleDate::new(2026, 9, 23)),
            toggle_single: true,
            toggle_group_val: 1,
            accordion_open: Some("acc-1".to_string()),
            collapsible_open: true,
            radio_group_val: 1,
            alert_dialog_open: false,
            command_query: String::new(),
        }
    }
}

impl DbProApp {
    pub(super) fn draw_component_gallery(&mut self, ui: &mut Ui) {
        let available_height = ui.available_height();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            self.draw_gallery_category_navigation(ui, available_height);

            ui.separator();

            ui.vertical(|ui| {
                ui.set_min_width(ui.available_width());
                self.draw_gallery_header(ui);
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("component_gallery_detail_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.add_space(SPACE_MD);
                        ui.horizontal(|ui| {
                            ui.add_space(SPACE_LG);
                            ui.vertical(|ui| {
                                ui.set_max_width(1120.0);
                                self.draw_selected_gallery_category(ui);
                                ui.add_space(SPACE_2XL);
                            });
                            ui.add_space(SPACE_LG);
                        });
                    });
            });
        });

        let _events = self.gallery_state.toasts.render(ui, self.theme);
    }

    fn draw_gallery_header(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        egui::Frame::none()
            .fill(theme.surface_app)
            .inner_margin(egui::Margin::symmetric(SPACE_LG, SPACE_SM))
            .show(ui, |ui| {
                ui.set_min_height(TOOLBAR_HEIGHT);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(char::from(Icon::Palette).to_string())
                            .font(font_icon(ICON_DEFAULT))
                            .color(theme.accent),
                    );
                    ui.add_space(SPACE_XS);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new("Component Gallery")
                                .font(font_subheading())
                                .color(theme.text_primary),
                        );
                        ui.label(
                            RichText::new("DB Pro workstation primitives")
                                .font(font_caption())
                                .color(theme.text_tertiary),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if Button::new(theme)
                            .icon(if self.preferences.dark_mode {
                                Icon::Sun
                            } else {
                                Icon::Moon
                            })
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip(if self.preferences.dark_mode {
                                "Preview light theme"
                            } else {
                                "Preview dark theme"
                            })
                            .show(ui)
                            .clicked()
                        {
                            self.preferences.dark_mode = !self.preferences.dark_mode;
                        }

                        ui.add_space(SPACE_XS);
                        if Button::new(theme)
                            .text("Reset")
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::Sm)
                            .icon(Icon::RotateCcw)
                            .tooltip("Reset interactive component samples")
                            .show(ui)
                            .clicked()
                        {
                            self.gallery_state = ComponentGalleryState::default();
                        }
                    });
                });
            });
    }

    fn draw_gallery_category_navigation(&mut self, ui: &mut Ui, available_height: f32) {
        let theme = self.theme;
        egui::Frame::none()
            .fill(theme.surface_panel)
            .inner_margin(egui::Margin::same(SPACE_SM))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                ui.set_min_size(egui::vec2(196.0, available_height));
                ui.set_max_width(196.0);

                ui.label(
                    RichText::new("LIBRARY")
                        .font(font_caption())
                        .color(theme.text_tertiary),
                );
                ui.add_space(SPACE_SM);

                self.draw_gallery_category_group(
                    ui,
                    &[
                        (GalleryCategory::Buttons, Icon::MousePointerClick, "Buttons"),
                        (GalleryCategory::Badges, Icon::Badge, "Badges & status"),
                        (GalleryCategory::Inputs, Icon::TextCursorInput, "Forms & inputs"),
                        (GalleryCategory::Selection, Icon::SlidersHorizontal, "Selection"),
                        (GalleryCategory::Cards, Icon::PanelsTopLeft, "Surfaces"),
                    ],
                );

                ui.add_space(SPACE_MD);
                ui.label(
                    RichText::new("SYSTEM")
                        .font(font_caption())
                        .color(theme.text_tertiary),
                );
                ui.add_space(SPACE_XS);
                self.draw_gallery_category_group(
                    ui,
                    &[
                        (GalleryCategory::Alerts, Icon::TriangleAlert, "Alerts"),
                        (GalleryCategory::Feedback, Icon::Activity, "Feedback"),
                        (GalleryCategory::Overlays, Icon::PanelTopOpen, "Overlays"),
                        (GalleryCategory::Navigation, Icon::Waypoints, "Navigation"),
                    ],
                );

                ui.add_space(SPACE_MD);
                ui.label(
                    RichText::new("WORKSTATION")
                        .font(font_caption())
                        .color(theme.text_tertiary),
                );
                ui.add_space(SPACE_XS);
                self.draw_gallery_category_group(
                    ui,
                    &[
                        (GalleryCategory::Tables, Icon::Table2, "Data tables"),
                        (GalleryCategory::DevTools, Icon::Code2, "Developer tools"),
                        (GalleryCategory::DatabaseShell, Icon::Database, "Database shell"),
                        (GalleryCategory::AgentUi, Icon::Bot, "Agent UI"),
                    ],
                );
                });
            });
    }

    fn draw_gallery_category_group(
        &mut self,
        ui: &mut Ui,
        categories: &[(GalleryCategory, Icon, &'static str)],
    ) {
        for (category, icon, label) in categories {
            let selected = self.gallery_state.category == *category;
            if Button::new(self.theme)
                .text(*label)
                .icon(*icon)
                .variant(if selected {
                    ButtonVariant::Secondary
                } else {
                    ButtonVariant::Ghost
                })
                .size(ButtonSize::Sm)
                .full_width(true)
                .show(ui)
                .clicked()
            {
                self.gallery_state.category = *category;
            }
            ui.add_space(SPACE_XXS);
        }
    }

    fn draw_selected_gallery_category(&mut self, ui: &mut Ui) {
        match self.gallery_state.category {
            GalleryCategory::Buttons => self.draw_gallery_buttons_section(ui),
            GalleryCategory::Badges => self.draw_gallery_badges_section(ui),
            GalleryCategory::Inputs => self.draw_gallery_inputs_section(ui),
            GalleryCategory::Selection => self.draw_gallery_selection_section(ui),
            GalleryCategory::Cards => self.draw_gallery_cards_section(ui),
            GalleryCategory::Alerts => self.draw_gallery_alerts_section(ui),
            GalleryCategory::Feedback => self.draw_gallery_feedback_section(ui),
            GalleryCategory::Overlays => self.draw_gallery_overlays_section(ui),
            GalleryCategory::Navigation => self.draw_gallery_navigation_section(ui),
            GalleryCategory::Tables => self.draw_gallery_tables_section(ui),
            GalleryCategory::DevTools => self.draw_gallery_devtools_section(ui),
            GalleryCategory::DatabaseShell => self.draw_gallery_database_shell_section(ui),
            GalleryCategory::AgentUi => self.draw_gallery_agent_ui_section(ui),
        }
    }

    pub(super) fn draw_section_heading(&self, ui: &mut Ui, title: &str, subtitle: &str) {
        SectionHeader::new(title, self.theme).description(subtitle).show(ui);
        ui.add_space(SPACE_MD);
    }
}
