use super::*;
use egui::{RichText, Ui};
use lucide_icons::Icon;

impl DbProApp {
    pub(super) fn draw_gallery_overlays_section(&mut self, ui: &mut Ui) {
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

                ui.add_space(8.0);
                if Button::new(theme)
                    .text("Open AlertDialog")
                    .variant(ButtonVariant::Destructive)
                    .show(ui)
                    .clicked()
                {
                    self.gallery_state.alert_dialog_open = true;
                }

                ui.add_space(8.0);
                HoverCard::new("gallery_hover_card", theme).show(
                    ui,
                    |ui| {
                        Button::new(theme)
                            .text("@db-pro/core")
                            .variant(ButtonVariant::Link)
                            .show(ui)
                    },
                    |ui| {
                        ui.label(
                            RichText::new("db-pro-core crate")
                                .font(DbProTheme::ui_medium_font(13.0))
                                .color(theme.text_primary),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("Pure Rust database client and engine abstractions for PostgreSQL, MySQL, and SQLite.")
                                .size(11.5)
                                .color(theme.text_secondary),
                        );
                    },
                );
            });
        });

        ui.add_space(16.0);
        self.draw_section_heading(
            ui,
            "Command Palette Primitives",
            "CommandInput, CommandGroup, CommandItem, and CommandEmpty for quick actions.",
        );

        Card::new(theme).show(ui, |ui| {
            ui.set_max_width(520.0);
            CommandInput::new(&mut self.gallery_state.command_query, theme)
                .placeholder("Type a command or search actions...")
                .show(ui);

            CommandGroup::new("ACTIONS").show(ui, theme, |ui| {
                CommandItem::new("cmd_run", "Execute Current Query")
                    .icon(Icon::Play)
                    .shortcut("⌘Enter")
                    .show(ui, theme);
                CommandItem::new("cmd_explain", "Explain & Analyze Query")
                    .icon(Icon::Search)
                    .shortcut("⌥⌘E")
                    .show(ui, theme);
                CommandItem::new("cmd_format", "Format SQL Document")
                    .icon(Icon::AlignLeft)
                    .shortcut("⇧⌥F")
                    .show(ui, theme);
            });

            CommandGroup::new("NAVIGATION").show(ui, theme, |ui| {
                CommandItem::new("cmd_explorer", "Go to Database Explorer")
                    .icon(Icon::Database)
                    .shortcut("⌘1")
                    .show(ui, theme);
                CommandItem::new("cmd_settings", "Open Settings")
                    .icon(Icon::Settings)
                    .shortcut("⌘,")
                    .show(ui, theme);
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
                let actions = dialog_actions(
                    ui,
                    theme,
                    DialogActionLabels {
                        secondary: "Cancel",
                        primary: "Drop table",
                    },
                );
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

        let mut alert_open = self.gallery_state.alert_dialog_open;
        AlertDialog::new(
            "Delete Production Database",
            "Are you absolutely sure? This action cannot be undone. This will permanently delete the selected database schema and terminate all connected clients.",
            theme,
        )
        .confirm_label("Yes, delete database")
        .cancel_label("Cancel")
        .destructive(true)
        .show(ui.ctx(), &mut alert_open);
        self.gallery_state.alert_dialog_open = alert_open;
    }

    pub(super) fn draw_gallery_tables_section(&mut self, ui: &mut Ui) {
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
}
