use super::*;
use egui::{Response, RichText, Ui};

const FORM_TWO_COLUMN_MIN_WIDTH: f32 = 680.0;
const FORM_INLINE_ACTIONS_MIN_WIDTH: f32 = 520.0;

impl DbProApp {
    pub(super) fn draw_gallery_inputs_section(&mut self, ui: &mut Ui) {
        let theme = self.theme;
        self.draw_section_heading(
            ui,
            "Form Handling & Validation",
            "Responsive connection form with explicit labels, keyboard navigation, validation summary, and inline recovery.",
        );

        Card::new(theme).show(ui, |ui| {
            self.draw_gallery_form_header(ui);
            ui.add_space(SPACE_LG);

            if let Some(error) = self.gallery_state.form_error.clone() {
                if let Some(dismiss) = Alert::new("Connection details need attention", &error, theme)
                    .variant(AlertVariant::Destructive)
                    .dismissable(true)
                    .show(ui)
                {
                    if dismiss.clicked() {
                        self.gallery_state.form_error = None;
                    }
                }
                ui.add_space(SPACE_MD);
            }

            self.draw_gallery_connection_fields(ui);

            ui.add_space(SPACE_LG);
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
            .label("Cluster pool")
            .has_more(has_more)
            .load_more(&mut load_more)
            .show(ui);
            if load_more {
                self.gallery_state.select_loaded = (loaded + 12).min(self.gallery_state.select_options.len());
            }

            ui.add_space(SPACE_LG);
            Textarea::new(&mut self.gallery_state.form_notes, "Optional notes…", theme)
                .label("Connection notes")
                .min_rows(3)
                .max_chars(500)
                .show(ui);

            ui.add_space(SPACE_LG);
            ui.separator();
            ui.add_space(SPACE_MD);
            self.draw_gallery_form_actions(ui);
        });
    }

    fn draw_gallery_form_header(&self, ui: &mut Ui) {
        let theme = self.theme;
        let draw_title = |ui: &mut Ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Connection profile")
                        .font(font_ui_label())
                        .color(theme.text_primary),
                );
                ui.add_space(SPACE_XXS);
                ui.label(
                    RichText::new("Fields resize with the workspace; errors appear next to the affected control.")
                        .font(font_caption())
                        .color(theme.text_secondary),
                );
            });
        };

        if ui.available_width() >= FORM_TWO_COLUMN_MIN_WIDTH {
            ui.horizontal(|ui| {
                draw_title(ui);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    Badge::new("Required fields marked *", theme)
                        .variant(BadgeVariant::Secondary)
                        .show(ui);
                });
            });
        } else {
            draw_title(ui);
            ui.add_space(SPACE_SM);
            Badge::new("Required fields marked *", theme)
                .variant(BadgeVariant::Secondary)
                .show(ui);
        }
    }

    fn draw_gallery_connection_fields(&mut self, ui: &mut Ui) {
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_LG;
            if ui.available_width() >= FORM_TWO_COLUMN_MIN_WIDTH {
                ui.columns(2, |columns| {
                    self.draw_gallery_name_field(&mut columns[0]);
                    self.draw_gallery_database_field(&mut columns[1]);
                });
                ui.add_space(SPACE_LG);
                ui.columns(2, |columns| {
                    self.draw_gallery_host_field(&mut columns[0]);
                    self.draw_gallery_password_field(&mut columns[1]);
                });
                ui.add_space(SPACE_LG);
                ui.columns(2, |columns| {
                    self.draw_gallery_port_field(&mut columns[0]);
                    Switch::new(&mut self.gallery_state.form_ssl, self.theme)
                        .label("Require SSL / TLS")
                        .description("Refuse unencrypted plaintext connections.")
                        .show(&mut columns[1]);
                });
            } else {
                self.draw_gallery_name_field(ui);
                ui.add_space(SPACE_LG);
                self.draw_gallery_host_field(ui);
                ui.add_space(SPACE_LG);
                self.draw_gallery_port_field(ui);
                ui.add_space(SPACE_LG);
                self.draw_gallery_database_field(ui);
                ui.add_space(SPACE_LG);
                self.draw_gallery_password_field(ui);
                ui.add_space(SPACE_LG);
                Switch::new(&mut self.gallery_state.form_ssl, self.theme)
                    .label("Require SSL / TLS")
                    .description("Refuse unencrypted plaintext connections.")
                    .show(ui);
            }
        });
    }

    fn draw_gallery_name_field(&mut self, ui: &mut Ui) {
        let error = self.gallery_state.form_state.get_error("form_name");
        let show_error = self.gallery_state.form_state.should_show_error("form_name");
        let mut field = FormField::new(
            "Display name",
            &mut self.gallery_state.form_name,
            "Production replica",
            self.theme,
        )
        .required(self.gallery_state.form_state.is_required("form_name"))
        .helper_text("Shown in the sidebar and command palette.");
        if show_error {
            if let Some(error) = error {
                field = field.error_text(error);
            }
        }
        let response = field.show(ui);
        Self::track_gallery_form_field(
            &mut self.gallery_state.form_state,
            "form_name",
            &self.gallery_state.form_name,
            &response,
        );
    }

    fn draw_gallery_host_field(&mut self, ui: &mut Ui) {
        let error = self.gallery_state.form_state.get_error("form_host");
        let show_error = self.gallery_state.form_state.should_show_error("form_host");
        let mut field = FormField::new("Host", &mut self.gallery_state.form_host, "db.internal", self.theme)
            .required(self.gallery_state.form_state.is_required("form_host"))
            .helper_text("Domain name or IPv4/IPv6 address.");
        if show_error {
            if let Some(error) = error {
                field = field.error_text(error);
            }
        }
        let response = field.show(ui);
        Self::track_gallery_form_field(
            &mut self.gallery_state.form_state,
            "form_host",
            &self.gallery_state.form_host,
            &response,
        );
    }

    fn draw_gallery_port_field(&mut self, ui: &mut Ui) {
        let error = self.gallery_state.form_state.get_error("form_port");
        let show_error = self.gallery_state.form_state.should_show_error("form_port");
        let mut field = FormField::new("Port", &mut self.gallery_state.form_port, "5432", self.theme)
            .required(self.gallery_state.form_state.is_required("form_port"))
            .helper_text("TCP port 1–65535.");
        if show_error {
            if let Some(error) = error {
                field = field.error_text(error);
            }
        }
        let response = field.show(ui);
        Self::track_gallery_form_field(
            &mut self.gallery_state.form_state,
            "form_port",
            &self.gallery_state.form_port,
            &response,
        );
    }

    fn draw_gallery_database_field(&mut self, ui: &mut Ui) {
        let error = self.gallery_state.form_state.get_error("form_database");
        let show_error = self.gallery_state.form_state.should_show_error("form_database");
        let mut field = FormField::new(
            "Database",
            &mut self.gallery_state.form_database,
            "app_prod",
            self.theme,
        )
        .required(self.gallery_state.form_state.is_required("form_database"))
        .helper_text("Default catalog database.");
        if show_error {
            if let Some(error) = error {
                field = field.error_text(error);
            }
        }
        let response = field.show(ui);
        Self::track_gallery_form_field(
            &mut self.gallery_state.form_state,
            "form_database",
            &self.gallery_state.form_database,
            &response,
        );
    }

    fn draw_gallery_password_field(&mut self, ui: &mut Ui) {
        let error = self.gallery_state.form_state.get_error("form_password");
        let show_error = self.gallery_state.form_state.should_show_error("form_password");
        let mut field = PasswordInput::new(
            &mut self.gallery_state.password_text,
            "Enter password…",
            &mut self.gallery_state.show_password,
            self.theme,
        )
        .id_salt("gallery.form.password")
        .label("Password")
        .required(self.gallery_state.form_state.is_required("form_password"))
        .helper_text("Stored securely; minimum 6 characters.");
        if show_error {
            if let Some(error) = error {
                field = field.error_text(error);
            }
        }
        let response = field.show(ui);
        Self::track_gallery_form_field(
            &mut self.gallery_state.form_state,
            "form_password",
            &self.gallery_state.password_text,
            &response,
        );
    }

    fn track_gallery_form_field(
        form_state: &mut FormState,
        field_name: &str,
        value: &str,
        response: &Response,
    ) {
        if response.changed() {
            form_state.set_dirty(field_name);
            form_state.validate_field(field_name, value);
        }
        if response.lost_focus() {
            form_state.touch(field_name);
            form_state.validate_field(field_name, value);
        }
    }

    fn draw_gallery_form_actions(&mut self, ui: &mut Ui) {
        let full_width = ui.available_width() < FORM_INLINE_ACTIONS_MIN_WIDTH;
        let (save, test, clear) = if full_width {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = SPACE_MD;
                self.draw_gallery_form_action_buttons(ui, true)
            })
            .inner
        } else {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = SPACE_MD;
                self.draw_gallery_form_action_buttons(ui, false)
            })
            .inner
        };

        if save {
            let fields = [
                ("form_name", self.gallery_state.form_name.as_str()),
                ("form_host", self.gallery_state.form_host.as_str()),
                ("form_port", self.gallery_state.form_port.as_str()),
                ("form_database", self.gallery_state.form_database.as_str()),
                ("form_password", self.gallery_state.password_text.as_str()),
            ];
            if self.gallery_state.form_state.handle_submit(&fields, || {}) {
                self.gallery_state.form_error = None;
                self.gallery_state
                    .toasts
                    .success("Connection validated and saved", ToastPosition::BottomRight);
            } else {
                self.gallery_state.form_error =
                    Some("Correct the highlighted fields, then save again.".to_owned());
            }
        }

        if test {
            self.gallery_state.toasts.show_with_action(
                "Reached host in 38 ms · TLS verified",
                ToastVariant::Default,
                ToastPosition::BottomRight,
                "Details",
            );
        }

        if clear {
            self.gallery_state.form_name.clear();
            self.gallery_state.form_host.clear();
            self.gallery_state.form_port.clear();
            self.gallery_state.form_database.clear();
            self.gallery_state.password_text.clear();
            self.gallery_state.form_notes.clear();
            self.gallery_state.form_error = None;
            self.gallery_state.form_state.reset();
        }
    }

    fn draw_gallery_form_action_buttons(&self, ui: &mut Ui, full_width: bool) -> (bool, bool, bool) {
        let save = Button::new(self.theme)
            .text("Save connection")
            .size(ButtonSize::Sm)
            .full_width(full_width)
            .show(ui)
            .clicked();
        let test = Button::new(self.theme)
            .text("Test connection")
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Sm)
            .full_width(full_width)
            .show(ui)
            .clicked();
        let clear = Button::new(self.theme)
            .text("Clear form")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .full_width(full_width)
            .show(ui)
            .clicked();
        (save, test, clear)
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
