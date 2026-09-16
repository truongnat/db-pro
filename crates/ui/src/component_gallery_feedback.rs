use super::*;
use egui::{RichText, Ui};
use lucide_icons::Icon;

impl DbProApp {
    pub(super) fn draw_gallery_alerts_section(&mut self, ui: &mut Ui) {
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

    pub(super) fn draw_gallery_feedback_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Feedback, Progress & Shortcuts",
            "Progress bars, animated spinners, and keyboard shortcut badges.",
        );
        Card::new(theme).show(ui, |ui| {
            self.draw_gallery_progress(ui);
            self.draw_gallery_shortcut_bar(ui);
            self.draw_gallery_shortcut_dialog(ui);
            self.draw_gallery_toasts(ui);
            self.draw_gallery_empty_states(ui);
        });
    }

    fn draw_gallery_progress(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        ui.horizontal(|ui| {
            // Determinate Animated Progress
            ui.vertical(|ui| {
                ui.set_width(320.0);
                let pct = (self.gallery_state.progress_val * 100.0).round() as i32; // safe: progress is clamped to finite [0, 1].
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
    }

    fn draw_gallery_shortcut_bar(&mut self, ui: &mut Ui) {
        let theme = self.theme;
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
    }

    fn draw_gallery_shortcut_dialog(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        // Keyboard Shortcuts Dialog
        let mut shortcuts_open = self.gallery_state.shortcuts_dialog_open;
        Dialog::new(&mut shortcuts_open, "Keyboard Shortcuts Cheatsheet", theme)
            .id_salt("gallery_shortcuts_cheatsheet_dialog")
            .description(
                "Design reference for the shortcut vocabulary. The shipped build binds a subset \
                 of these — the live bindings are the ones the toolbar tooltips show.",
            )
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
    }

    fn draw_gallery_toasts(&mut self, ui: &mut Ui) {
        let theme = self.theme;
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
    }

    fn draw_gallery_empty_states(&mut self, ui: &mut Ui) {
        let theme = self.theme;
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
    }

    pub(super) fn draw_gallery_navigation_section(&mut self, ui: &mut Ui) {
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
}
