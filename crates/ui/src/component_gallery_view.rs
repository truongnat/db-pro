use super::*;
use crate::components::*;
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
    Navigation,
    Tables,
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
    pub table_search: String,
    pub table_selected_rows: std::collections::HashSet<usize>,
    pub table_sort_col: Option<usize>,
    pub table_sort_desc: bool,
}

impl Default for ComponentGalleryState {
    fn default() -> Self {
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
            select_options: vec![
                "PostgreSQL 16 (Production)".to_owned(),
                "PostgreSQL 15 (Staging)".to_owned(),
                "SQLite (Local Dev)".to_owned(),
                "MySQL 8.0 (Legacy)".to_owned(),
            ],
            select_idx: 0,
            segmented_tab_idx: 0,
            underline_tab_idx: 0,
            btn_loading: false,
            table_search: String::new(),
            table_selected_rows: [0, 2].into_iter().collect(),
            table_sort_col: Some(0),
            table_sort_desc: false,
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
                            ShadcnBadge::new("shadcn/ui inspired", theme)
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
                        if ShadcnButton::new(theme)
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
                        if ShadcnButton::new(theme)
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
                    (GalleryCategory::Navigation, "Navigation"),
                    (GalleryCategory::Tables, "Data Tables"),
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

                // ── 8. NAVIGATION & TABS ────────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::Navigation {
                    self.draw_gallery_navigation_section(ui);
                    ui.add_space(24.0);
                }

                // ── 9. DATA TABLES ──────────────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::Tables {
                    self.draw_gallery_tables_section(ui);
                    ui.add_space(32.0);
                }
            });
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

        ShadcnCard::new(theme).show(ui, |ui| {
            ui.label(
                RichText::new("Variants")
                    .size(13.0)
                    .strong()
                    .color(theme.text_secondary),
            );
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ShadcnButton::new(theme)
                    .text("Primary (Default)")
                    .variant(ButtonVariant::Default)
                    .show(ui);
                ui.add_space(6.0);
                ShadcnButton::new(theme)
                    .text("Secondary")
                    .variant(ButtonVariant::Secondary)
                    .show(ui);
                ui.add_space(6.0);
                ShadcnButton::new(theme)
                    .text("Outline")
                    .variant(ButtonVariant::Outline)
                    .show(ui);
                ui.add_space(6.0);
                ShadcnButton::new(theme)
                    .text("Ghost")
                    .variant(ButtonVariant::Ghost)
                    .show(ui);
                ui.add_space(6.0);
                ShadcnButton::new(theme)
                    .text("Destructive")
                    .variant(ButtonVariant::Destructive)
                    .icon(Icon::Trash2)
                    .show(ui);
                ui.add_space(6.0);
                ShadcnButton::new(theme)
                    .text("Link Action")
                    .variant(ButtonVariant::Link)
                    .show(ui);
            });

            ui.add_space(16.0);
            ui.label(RichText::new("Sizes").size(13.0).strong().color(theme.text_secondary));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ShadcnButton::new(theme)
                    .text("Small (Sm)")
                    .size(ButtonSize::Sm)
                    .show(ui);
                ui.add_space(6.0);
                ShadcnButton::new(theme)
                    .text("Default (Md)")
                    .size(ButtonSize::Default)
                    .show(ui);
                ui.add_space(6.0);
                ShadcnButton::new(theme)
                    .text("Large (Lg)")
                    .size(ButtonSize::Lg)
                    .show(ui);
                ui.add_space(10.0);
                ShadcnButton::new(theme)
                    .icon(Icon::Plus)
                    .size(ButtonSize::Icon)
                    .variant(ButtonVariant::Outline)
                    .show(ui);
                ui.add_space(6.0);
                ShadcnButton::new(theme)
                    .icon(Icon::Copy)
                    .size(ButtonSize::IconSm)
                    .variant(ButtonVariant::Secondary)
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
            ui.horizontal(|ui| {
                let loading = self.gallery_state.btn_loading;
                if ShadcnButton::new(theme)
                    .text(if loading { "Please wait..." } else { "Click to Load" })
                    .icon(Icon::Play)
                    .loading(loading)
                    .show(ui)
                    .clicked()
                {
                    self.gallery_state.btn_loading = true;
                }

                if loading {
                    ui.add_space(8.0);
                    if ShadcnButton::new(theme)
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
                ShadcnButton::new(theme).text("Disabled Button").enabled(false).show(ui);
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

        ShadcnCard::new(theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ShadcnBadge::new("Default Accent", theme)
                    .variant(BadgeVariant::Default)
                    .show(ui);
                ui.add_space(8.0);
                ShadcnBadge::new("Secondary Pill", theme)
                    .variant(BadgeVariant::Secondary)
                    .show(ui);
                ui.add_space(8.0);
                ShadcnBadge::new("Outline Tag", theme)
                    .variant(BadgeVariant::Outline)
                    .show(ui);
                ui.add_space(8.0);
                ShadcnBadge::new("Active Node", theme)
                    .variant(BadgeVariant::Success)
                    .dot(true)
                    .show(ui);
                ui.add_space(8.0);
                ShadcnBadge::new("Warning Alert", theme)
                    .variant(BadgeVariant::Warning)
                    .icon(Icon::AlertTriangle)
                    .show(ui);
                ui.add_space(8.0);
                ShadcnBadge::new("Connection Error", theme)
                    .variant(BadgeVariant::Destructive)
                    .dot(true)
                    .show(ui);
                ui.add_space(8.0);
                ShadcnBadge::new("PostgreSQL 16.2", theme)
                    .variant(BadgeVariant::Info)
                    .icon(Icon::Database)
                    .show(ui);
            });
        });
    }

    fn draw_gallery_inputs_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Forms & Inputs",
            "Standard single-line text inputs, search fields, password toggles, and multiline textareas.",
        );

        ShadcnCard::new(theme).show(ui, |ui| {
            ui.columns(2, |columns| {
                // Column 1
                let ui = &mut columns[0];
                ShadcnInput::new(
                    &mut self.gallery_state.input_text,
                    "Enter table or database name...",
                    theme,
                )
                .label("Database Connection Name")
                .helper_text("Used to identify this instance across your workspaces.")
                .leading_icon(Icon::Database)
                .clearable(true)
                .show(ui);

                ui.add_space(14.0);

                ShadcnSearchInput::new(
                    &mut self.gallery_state.search_text,
                    "Search schemas, tables, indexes...",
                    theme,
                )
                .shortcut("⌘K")
                .show(ui);

                ui.add_space(14.0);

                ShadcnPasswordInput::new(
                    &mut self.gallery_state.password_text,
                    "Enter database password...",
                    &mut self.gallery_state.show_password,
                    theme,
                )
                .label("Secret Password")
                .show(ui);

                // Column 2
                let ui = &mut columns[1];
                ShadcnInput::new(&mut self.gallery_state.input_error_text, "URI string...", theme)
                    .label("PostgreSQL Connection URI")
                    .error_text("Invalid port format: expected integer between 1 and 65535.")
                    .leading_icon(Icon::AlertCircle)
                    .clearable(true)
                    .show(ui);

                ui.add_space(14.0);

                ShadcnTextarea::new(&mut self.gallery_state.textarea_text, "Enter SQL query...", theme)
                    .label("Query Editor Scratchpad")
                    .min_rows(4)
                    .max_chars(500)
                    .show(ui);
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

        ShadcnCard::new(theme).show(ui, |ui| {
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
                ShadcnCheckbox::new(&mut self.gallery_state.checkbox_1, "Auto-commit queries", theme)
                    .description("Execute each SQL statement immediately in its own transaction.")
                    .show(ui);
                ui.add_space(8.0);
                ShadcnCheckbox::new(&mut self.gallery_state.checkbox_2, "Format SQL on save", theme)
                    .description("Automatically aligns keywords and clauses.")
                    .show(ui);
                ui.add_space(8.0);
                let mut disabled_check = true;
                ShadcnCheckbox::new(&mut disabled_check, "Enforce SSL encryption", theme)
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
                ShadcnSwitch::new(&mut self.gallery_state.switch_1, theme)
                    .label("Copilot Assistant")
                    .description("AI schema suggestions in real time.")
                    .show(ui);
                ui.add_space(10.0);
                ShadcnSwitch::new(&mut self.gallery_state.switch_2, theme)
                    .label("Query Execution Safety Guard")
                    .description("Confirm before destructive statements.")
                    .show(ui);

                ui.add_space(12.0);
                for (idx, label) in ["Read Committed", "Repeatable Read", "Serializable"].iter().enumerate() {
                    let is_sel = idx == self.gallery_state.radio_selected;
                    if ShadcnRadio::new(is_sel, label, theme).show(ui).clicked() {
                        self.gallery_state.radio_selected = idx;
                    }
                    ui.add_space(4.0);
                }

                // Column 3: Select & Slider
                let ui = &mut columns[2];
                ui.label(
                    RichText::new("Dropdown Select & Slider")
                        .size(13.0)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(8.0);
                ShadcnSelect::new(
                    "gallery_db_select",
                    &mut self.gallery_state.select_idx,
                    &self.gallery_state.select_options,
                    theme,
                )
                .label("Target Connection Profile")
                .show(ui);

                ui.add_space(16.0);
                ShadcnSlider::new(&mut self.gallery_state.slider_val, 10.0..=500.0, theme)
                    .label("Query Result Page Size (Rows)")
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

        ui.columns(4, |columns| {
            MetricCard::new("Active Connections", "8 / 10", theme)
                .change("+2 this session", true)
                .icon(Icon::Database)
                .show(&mut columns[0]);

            MetricCard::new("Total Queries Executed", "14,289", theme)
                .change("+18.4% today", true)
                .icon(Icon::Terminal)
                .show(&mut columns[1]);

            MetricCard::new("Slow Query Rate", "0.42%", theme)
                .change("-0.15% improvement", true)
                .icon(Icon::Gauge)
                .show(&mut columns[2]);

            MetricCard::new("Staged Mutations", "3 pending", theme)
                .change("Needs review", false)
                .icon(Icon::FileEdit)
                .show(&mut columns[3]);
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
            ShadcnAlert::new(
                "Connection Established",
                "Successfully connected to PostgreSQL cluster xe_lac_hong_prod via localhost:5432.",
                theme,
            )
            .variant(AlertVariant::Success)
            .show(ui);

            ui.add_space(10.0);

            ShadcnAlert::new(
                "Uncommitted Transaction",
                "You have active mutations in session 42. Closing this tab will trigger an automatic ROLLBACK.",
                theme,
            )
            .variant(AlertVariant::Warning)
            .dismissable(true)
            .show(ui);

            let ui = &mut columns[1];
            ShadcnAlert::new(
                "Execution Interrupted",
                "Query execution cancelled by user: statement timeout exceeded (30,000 ms).",
                theme,
            )
            .variant(AlertVariant::Destructive)
            .dismissable(true)
            .show(ui);

            ui.add_space(10.0);

            ShadcnAlert::new(
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

        ShadcnCard::new(theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Progress Bar (72%)")
                            .size(12.5)
                            .strong()
                            .color(theme.text_secondary),
                    );
                    ui.add_space(6.0);
                    ShadcnProgress::new(0.72, theme).height(8.0).show(ui);
                });

                ui.add_space(24.0);

                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Animated Canvas Spinner")
                            .size(12.5)
                            .strong()
                            .color(theme.text_secondary),
                    );
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ShadcnSpinner::new(theme).size(20.0).show(ui);
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
                ui.label(RichText::new("Quick Open:").size(12.0).color(theme.text_secondary));
                kbd_badge(ui, "⌘P", theme);
                ui.add_space(12.0);

                ui.label(RichText::new("Command Palette:").size(12.0).color(theme.text_secondary));
                kbd_badge(ui, "⇧⌘P", theme);
                ui.add_space(12.0);

                ui.label(RichText::new("Run Query:").size(12.0).color(theme.text_secondary));
                kbd_badge(ui, "⌘↵", theme);
                ui.add_space(12.0);

                ui.label(RichText::new("Format Code:").size(12.0).color(theme.text_secondary));
                kbd_badge(ui, "⌥⇧F", theme);
            });
        });
    }

    fn draw_gallery_navigation_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(ui, "Navigation & Tabs", "Segmented controls and underline tabs.");

        ShadcnCard::new(theme).show(ui, |ui| {
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
                |ui| ShadcnSearchInput::new(&mut self.gallery_state.table_search, "Filter objects...", theme).show(ui),
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
            ShadcnBadge::new(&badge_text, theme)
                .variant(BadgeVariant::Secondary)
                .show(ui);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ShadcnButton::new(theme)
                    .text("Export CSV")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .icon(Icon::Download)
                    .show(ui)
                    .clicked()
                {
                    // Noop demonstration
                }

                if ShadcnButton::new(theme)
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
            ShadcnTableColumn::new("Table Name").width(220.0).sortable(true),
            ShadcnTableColumn::new("Type").width(120.0).sortable(true),
            ShadcnTableColumn::new("Row Count")
                .width(130.0)
                .align(TableColumnAlign::Right)
                .sortable(true),
            ShadcnTableColumn::new("Size")
                .width(110.0)
                .align(TableColumnAlign::Right)
                .sortable(true),
            ShadcnTableColumn::new("Status")
                .width(120.0)
                .align(TableColumnAlign::Center)
                .sortable(true),
            ShadcnTableColumn::new("Actions").align(TableColumnAlign::Right),
        ];

        let visible_count = row_indices.len();
        let all_selected = visible_count > 0
            && row_indices
                .iter()
                .all(|idx| self.gallery_state.table_selected_rows.contains(idx));

        let sort_col = self.gallery_state.table_sort_col;
        let sort_desc = self.gallery_state.table_sort_desc;

        let mut toggle_all_target = None;
        let mut toggle_row_target = None;
        let mut toggle_sort_col = None;

        ShadcnTable::new(&columns, theme)
            .selectable(true, all_selected)
            .sort(sort_col, sort_desc)
            .row_height(38.0)
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
                            ShadcnBadge::new(row.1, theme).variant(BadgeVariant::Outline).show(ui);
                        }
                        2 => {
                            ui.label(RichText::new(row.2).monospace().size(12.0).color(theme.text_primary));
                        }
                        3 => {
                            ui.label(RichText::new(row.4).monospace().size(12.0).color(theme.text_secondary));
                        }
                        4 => {
                            ShadcnBadge::new(row.7, theme).variant(row.6).dot(true).show(ui);
                        }
                        5 => {
                            ui.horizontal(|ui| {
                                ShadcnButton::new(theme)
                                    .text("Inspect")
                                    .size(ButtonSize::Sm)
                                    .variant(ButtonVariant::Ghost)
                                    .icon(Icon::ExternalLink)
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
                if ShadcnButton::new(theme)
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
                ShadcnButton::new(theme)
                    .text("Next")
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm)
                    .icon(Icon::ChevronRight)
                    .enabled(false)
                    .show(ui);

                ui.add_space(4.0);
                ShadcnButton::new(theme)
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
}
