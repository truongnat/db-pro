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
    DevTools,
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
            tree_server_expanded: true,
            tree_db_expanded: true,
            tree_schema_expanded: true,
            tree_table_expanded: true,
            tool_call_expanded: true,
            approval_status: None,
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
                    (GalleryCategory::Navigation, "Navigation"),
                    (GalleryCategory::Tables, "Data Tables"),
                    (GalleryCategory::DevTools, "Developer Tools"),
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

                // ── 8. NAVIGATION & TABS ────────────────────────────────────
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

                // ── 11. AI AGENT UI ─────────────────────────────────────────
                if cat == GalleryCategory::All || cat == GalleryCategory::AgentUi {
                    self.draw_gallery_agent_ui_section(ui);
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
                    .show(ui);
                ui.add_space(6.0);
                Button::new(theme)
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
                if Button::new(theme)
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
        });
    }

    fn draw_gallery_inputs_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Forms & Inputs",
            "Standard single-line text inputs, search fields, password toggles, and multiline textareas.",
        );

        Card::new(theme).show(ui, |ui| {
            ui.columns(2, |columns| {
                // Column 1
                let ui = &mut columns[0];
                Input::new(
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

                SearchInput::new(
                    &mut self.gallery_state.search_text,
                    "Search schemas, tables, indexes...",
                    theme,
                )
                .shortcut("⌘K")
                .show(ui);

                ui.add_space(14.0);

                PasswordInput::new(
                    &mut self.gallery_state.password_text,
                    "Enter database password...",
                    &mut self.gallery_state.show_password,
                    theme,
                )
                .label("Secret Password")
                .show(ui);

                // Column 2
                let ui = &mut columns[1];
                Input::new(&mut self.gallery_state.input_error_text, "URI string...", theme)
                    .label("PostgreSQL Connection URI")
                    .error_text("Invalid port format: expected integer between 1 and 65535.")
                    .leading_icon(Icon::AlertCircle)
                    .clearable(true)
                    .show(ui);

                ui.add_space(14.0);

                Textarea::new(&mut self.gallery_state.textarea_text, "Enter SQL query...", theme)
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

                // Column 3: Select & Slider
                let ui = &mut columns[2];
                ui.label(
                    RichText::new("Dropdown Select & Slider")
                        .size(13.0)
                        .strong()
                        .color(theme.text_secondary),
                );
                ui.add_space(8.0);
                Select::new(
                    "gallery_db_select",
                    &mut self.gallery_state.select_idx,
                    &self.gallery_state.select_options,
                    theme,
                )
                .label("Target Connection Profile")
                .show(ui);

                ui.add_space(16.0);
                Slider::new(&mut self.gallery_state.slider_val, 10.0..=500.0, theme)
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
            Alert::new(
                "Connection Established",
                "Successfully connected to PostgreSQL cluster xe_lac_hong_prod via localhost:5432.",
                theme,
            )
            .variant(AlertVariant::Success)
            .show(ui);

            ui.add_space(10.0);

            Alert::new(
                "Uncommitted Transaction",
                "You have active mutations in session 42. Closing this tab will trigger an automatic ROLLBACK.",
                theme,
            )
            .variant(AlertVariant::Warning)
            .dismissable(true)
            .show(ui);

            let ui = &mut columns[1];
            Alert::new(
                "Execution Interrupted",
                "Query execution cancelled by user: statement timeout exceeded (30,000 ms).",
                theme,
            )
            .variant(AlertVariant::Destructive)
            .dismissable(true)
            .show(ui);

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
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Progress Bar (72%)")
                            .size(12.5)
                            .strong()
                            .color(theme.text_secondary),
                    );
                    ui.add_space(6.0);
                    Progress::new(0.72, theme).height(8.0).show(ui);
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
                            Badge::new(row.1, theme).variant(BadgeVariant::Outline).show(ui);
                        }
                        2 => {
                            ui.label(RichText::new(row.2).monospace().size(12.0).color(theme.text_primary));
                        }
                        3 => {
                            ui.label(RichText::new(row.4).monospace().size(12.0).color(theme.text_secondary));
                        }
                        4 => {
                            Badge::new(row.7, theme).variant(row.6).dot(true).show(ui);
                        }
                        5 => {
                            ui.horizontal(|ui| {
                                Button::new(theme)
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

                    if self.gallery_state.tree_server_expanded {
                        DatabaseTreeNode::new("production_db", TreeNodeKind::Database, 1, theme)
                            .detail("UTF-8")
                            .expanded(&mut self.gallery_state.tree_db_expanded)
                            .show(ui);

                        if self.gallery_state.tree_db_expanded {
                            DatabaseTreeNode::new("public", TreeNodeKind::Schema, 2, theme)
                                .detail("12 tables")
                                .expanded(&mut self.gallery_state.tree_schema_expanded)
                                .show(ui);

                            if self.gallery_state.tree_schema_expanded {
                                DatabaseTreeNode::new("users", TreeNodeKind::Table, 3, theme)
                                    .detail("1,842,109 rows")
                                    .selected(true)
                                    .expanded(&mut self.gallery_state.tree_table_expanded)
                                    .show(ui);

                                if self.gallery_state.tree_table_expanded {
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
                                }

                                DatabaseTreeNode::new("orders", TreeNodeKind::Table, 3, theme)
                                    .detail("4,291,012 rows")
                                    .show(ui);

                                DatabaseTreeNode::new("v_active_bookings", TreeNodeKind::View, 3, theme)
                                    .detail("view")
                                    .show(ui);
                            }
                        }
                    }
                });
            });
        });
    }

    fn draw_gallery_agent_ui_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "AI Agent Workspace & Execution Components",
            "Context awareness, agent execution timeline, tool approvals, and safe execution boundaries.",
        );

        Card::new(theme).show(ui, |ui| {
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

            ui.add_space(20.0);

            // 3. Tool Calls & Execution Approvals
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
        });
    }
}
