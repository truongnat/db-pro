use super::*;
use egui::{RichText, Ui};

impl DbProApp {
    pub(super) fn draw_gallery_inputs_section(&mut self, ui: &mut Ui) {
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
            )
            .theme(theme)
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

    pub(super) fn draw_gallery_selection_section(&mut self, ui: &mut Ui) {
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
                )
                .theme(theme)
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

        ui.add_space(16.0);
        self.draw_section_heading(
            ui,
            "shadcn Primitives & Controls",
            "Accordion, Collapsible, Toggle, ToggleGroup, RadioGroup, DatePicker & Separator.",
        );

        Card::new(theme).show(ui, |ui| {
            ui.columns(3, |columns| {
                // Column 1: Toggle, ToggleGroup & DatePicker
                let ui = &mut columns[0];
                ui.label(
                    RichText::new("Toggle & ToggleGroup")
                        .font(DbProTheme::ui_medium_font(13.0))
                        .color(theme.text_secondary),
                );
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    Toggle::new(&mut self.gallery_state.toggle_single, theme)
                        .label("Grid View")
                        .icon(Icon::Grid)
                        .show(ui);

                    let mut italic_toggle = false;
                    Toggle::new(&mut italic_toggle, theme)
                        .icon(Icon::Italic)
                        .variant(ToggleVariant::Outline)
                        .show(ui);
                });

                ui.add_space(12.0);
                ui.label(
                    RichText::new("View Mode Group:")
                        .size(11.5)
                        .color(theme.text_muted),
                );
                ui.add_space(4.0);

                let tg = ToggleGroup::new(theme)
                    .item(ToggleGroupItem::new(1).icon(Icon::Table).label("Data"))
                    .item(ToggleGroupItem::new(2).icon(Icon::Layers).label("Structure"))
                    .item(ToggleGroupItem::new(3).icon(Icon::FileCode).label("DDL"));
                tg.show_single(ui, &mut self.gallery_state.toggle_group_val);

                ui.add_space(16.0);
                ui.label(
                    RichText::new("Calendar & DatePicker")
                        .font(DbProTheme::ui_medium_font(13.0))
                        .color(theme.text_secondary),
                );
                ui.add_space(8.0);
                DatePicker::new(
                    "gallery_datepicker",
                    &mut self.gallery_state.date_picker_val,
                    theme,
                )
                .show(ui);

                // Column 2: Accordion & Collapsible
                let ui = &mut columns[1];
                ui.label(
                    RichText::new("Collapsible & Accordion")
                        .font(DbProTheme::ui_medium_font(13.0))
                        .color(theme.text_secondary),
                );
                ui.add_space(8.0);

                Collapsible::new(&mut self.gallery_state.collapsible_open, theme)
                    .title("Advanced Connection Pool Settings")
                    .icon(Icon::Settings)
                    .badge("3 active")
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("Max Connections: 50 | Timeout: 30s | Idle: 10s")
                                .size(11.5)
                                .color(theme.text_secondary),
                        );
                    });

                ui.add_space(12.0);
                let acc = Accordion::new(theme);
                let acc_item_1 = AccordionItem::new("acc-1", "SSL / TLS Encryption")
                    .icon(Icon::ShieldCheck)
                    .badge("Enforced");
                acc.show_single(ui, acc_item_1, &mut self.gallery_state.accordion_open, true, |ui| {
                    ui.label(
                        RichText::new("Mode: verify-full\nCA: /etc/ssl/certs/db-root.crt")
                            .size(11.5)
                            .color(theme.text_secondary),
                    );
                });

                let acc_item_2 = AccordionItem::new("acc-2", "SSH Bastion Tunnel")
                    .icon(Icon::Server);
                acc.show_single(ui, acc_item_2, &mut self.gallery_state.accordion_open, true, |ui| {
                    ui.label(
                        RichText::new("Host: jump.internal:22 | User: deploy")
                            .size(11.5)
                            .color(theme.text_secondary),
                    );
                });

                // Column 3: RadioGroup & Separator
                let ui = &mut columns[2];
                ui.label(
                    RichText::new("Radio Group")
                        .font(DbProTheme::ui_medium_font(13.0))
                        .color(theme.text_secondary),
                );
                ui.add_space(8.0);

                let rg = RadioGroup::new(theme)
                    .option(
                        RadioGroupOption::new(1, "PostgreSQL")
                            .description("Recommended for relational workflows"),
                    )
                    .option(
                        RadioGroupOption::new(2, "MySQL")
                            .description("Popular standard OLTP database"),
                    )
                    .option(
                        RadioGroupOption::new(3, "SQLite")
                            .description("Local zero-config embedded file"),
                    );
                rg.show(ui, &mut self.gallery_state.radio_group_val);

                ui.add_space(14.0);
                Separator::horizontal(theme).label("OR").show(ui);
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Custom JDBC Driver")
                        .font(DbProTheme::ui_medium_font(12.0))
                        .color(theme.text_muted),
                );
            });
        });
    }
}
