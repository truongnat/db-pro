use super::*;

struct WorkspaceTabItem<'a> {
    selected: bool,
    icon: Icon,
    title: &'a str,
    unsaved: bool,
    show_close: bool,
}

struct TabChromeAction {
    clicked: bool,
    close_clicked: bool,
}

impl DbProApp {
    pub(super) fn draw_workspace_tabs(&mut self, ui: &mut egui::Ui) {
        let modifier = Self::primary_modifier_label();

        egui::Frame {
            fill: self.theme.surface_panel,
            inner_margin: egui::Margin {
                left: 4.0,
                right: 4.0,
                top: 4.0,
                bottom: 0.0,
            },
            stroke: egui::Stroke::NONE,
            rounding: egui::Rounding::ZERO,
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);

                egui::ScrollArea::horizontal()
                    .id_salt("workspace-tabs-scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);

                            // 1. Welcome Tab
                            let welcome_selected = self.active_tab == WorkspaceTab::Welcome;
                            let mut close_all_requested = false;
                            let welcome_action = draw_workspace_tab_item(
                                ui,
                                self.theme,
                                WorkspaceTabItem {
                                    selected: welcome_selected,
                                    icon: Icon::House,
                                    title: "Welcome",
                                    unsaved: false,
                                    show_close: false,
                                },
                                |ui| {
                                    ui.label(
                                        RichText::new("Welcome")
                                            .font(font_ui_label())
                                            .strong()
                                            .color(self.theme.text_primary),
                                    );
                                    ui.separator();
                                    if ui
                                        .button(icon_text(Icon::Layers, "Close All Tabs", self.theme.text_primary))
                                        .clicked()
                                    {
                                        close_all_requested = true;
                                        ui.close_menu();
                                    }
                                },
                            );
                            if welcome_action.clicked {
                                self.active_tab = WorkspaceTab::Welcome;
                            }
                            if close_all_requested {
                                self.close_all_tabs();
                            }

                            // 2. Query Documents Tabs
                            let documents: Vec<(usize, String, String)> = self
                                .query_documents
                                .iter()
                                .enumerate()
                                .map(|(index, doc)| (index, doc.title.clone(), doc.content.clone()))
                                .collect();

                            let mut switch_query_idx = None;
                            let mut close_query_idx = None;
                            let mut duplicate_query_idx = None;
                            let mut close_others_idx = None;
                            let mut close_right_idx = None;
                            let mut run_query_idx = None;

                            for (index, title, content) in &documents {
                                let idx = *index;
                                let selected =
                                    self.active_tab == WorkspaceTab::Query && self.active_query_document == idx;
                                let can_close = self.query_documents.len() > 1;
                                let unsaved = selected
                                    && self
                                        .query_documents
                                        .get(idx)
                                        .is_some_and(|doc| doc.content != self.query_text);

                                let action = draw_workspace_tab_item(
                                    ui,
                                    self.theme,
                                    WorkspaceTabItem {
                                        selected,
                                        icon: Icon::FileCode2,
                                        title,
                                        unsaved,
                                        show_close: can_close,
                                    },
                                    |ui| {
                                        ui.label(
                                            RichText::new(title)
                                                .font(font_ui_label())
                                                .strong()
                                                .color(self.theme.text_primary),
                                        );
                                        ui.separator();
                                        if ui
                                            .button(icon_text(
                                                Icon::X,
                                                &format!("Close Tab ({modifier}W)"),
                                                self.theme.text_primary,
                                            ))
                                            .clicked()
                                        {
                                            close_query_idx = Some(idx);
                                            ui.close_menu();
                                        }
                                        if ui
                                            .button(icon_text(
                                                Icon::Layers,
                                                "Close Other Tabs",
                                                self.theme.text_primary,
                                            ))
                                            .clicked()
                                        {
                                            close_others_idx = Some(idx);
                                            ui.close_menu();
                                        }
                                        if idx + 1 < self.query_documents.len()
                                            && ui
                                                .button(icon_text(
                                                    Icon::ArrowRight,
                                                    "Close Tabs to the Right",
                                                    self.theme.text_primary,
                                                ))
                                                .clicked()
                                        {
                                            close_right_idx = Some(idx);
                                            ui.close_menu();
                                        }
                                        if ui
                                            .button(icon_text(Icon::Layers, "Close All Tabs", self.theme.text_primary))
                                            .clicked()
                                        {
                                            close_all_requested = true;
                                            ui.close_menu();
                                        }
                                        ui.separator();
                                        if ui
                                            .button(icon_text(Icon::Copy, "Duplicate Tab", self.theme.text_primary))
                                            .clicked()
                                        {
                                            duplicate_query_idx = Some(idx);
                                            ui.close_menu();
                                        }
                                        if ui
                                            .button(icon_text(
                                                Icon::FileCode2,
                                                "Copy SQL Content",
                                                self.theme.text_primary,
                                            ))
                                            .clicked()
                                        {
                                            ui.output_mut(|o| o.copied_text = content.clone());
                                            ui.close_menu();
                                        }
                                        if ui
                                            .button(icon_text(Icon::FileText, "Copy Title", self.theme.text_primary))
                                            .clicked()
                                        {
                                            ui.output_mut(|o| o.copied_text = title.clone());
                                            ui.close_menu();
                                        }
                                        ui.separator();
                                        if ui
                                            .button(icon_text(
                                                Icon::Play,
                                                &format!("Run Query ({modifier}↵)"),
                                                self.theme.accent,
                                            ))
                                            .clicked()
                                        {
                                            run_query_idx = Some(idx);
                                            ui.close_menu();
                                        }
                                    },
                                );

                                if action.close_clicked {
                                    close_query_idx = Some(idx);
                                } else if action.clicked {
                                    switch_query_idx = Some(idx);
                                }
                            }

                            if let Some(idx) = switch_query_idx {
                                self.switch_query_document(idx);
                                self.active_tab = WorkspaceTab::Query;
                            }
                            if let Some(idx) = close_query_idx {
                                self.close_query_document(idx);
                            }
                            if let Some(idx) = duplicate_query_idx {
                                self.duplicate_query_document(idx);
                            }
                            if let Some(idx) = close_others_idx {
                                self.close_other_query_documents(idx);
                            }
                            if let Some(idx) = close_right_idx {
                                self.close_query_documents_to_right(idx);
                            }
                            if let Some(idx) = run_query_idx {
                                self.switch_query_document(idx);
                                self.active_tab = WorkspaceTab::Query;
                                self.dispatch_query();
                            }
                            if close_all_requested {
                                self.close_all_tabs();
                            }

                            // 3. Table Tab
                            if let Some(table_name) = self.selected_table.clone() {
                                let selected = self.active_tab == WorkspaceTab::Table;
                                let unsaved = !self.staged_changes.is_empty();
                                let mut close_table = false;
                                let mut refresh_table = false;
                                let table_action = draw_workspace_tab_item(
                                    ui,
                                    self.theme,
                                    WorkspaceTabItem {
                                        selected,
                                        icon: Icon::Table2,
                                        title: &table_name,
                                        unsaved,
                                        show_close: true,
                                    },
                                    |ui| {
                                        ui.label(
                                            RichText::new(&table_name)
                                                .font(font_ui_label())
                                                .strong()
                                                .color(self.theme.text_primary),
                                        );
                                        ui.separator();
                                        if ui
                                            .button(icon_text(Icon::X, "Close Tab", self.theme.text_primary))
                                            .clicked()
                                        {
                                            close_table = true;
                                            ui.close_menu();
                                        }
                                        ui.separator();
                                        if ui
                                            .button(icon_text(Icon::RefreshCw, "Refresh Data", self.theme.text_primary))
                                            .clicked()
                                        {
                                            refresh_table = true;
                                            ui.close_menu();
                                        }
                                        if ui
                                            .button(icon_text(Icon::Copy, "Copy Table Name", self.theme.text_primary))
                                            .clicked()
                                        {
                                            ui.output_mut(|o| o.copied_text = table_name.clone());
                                            ui.close_menu();
                                        }
                                        if ui
                                            .button(icon_text(
                                                Icon::FileCode2,
                                                "Copy SELECT Query",
                                                self.theme.text_primary,
                                            ))
                                            .clicked()
                                        {
                                            ui.output_mut(|o| {
                                                o.copied_text = format!("SELECT * FROM {table_name} LIMIT 100;")
                                            });
                                            ui.close_menu();
                                        }
                                    },
                                );

                                if table_action.close_clicked || close_table {
                                    self.request_close_workspace_tab(WorkspaceTab::Table);
                                } else if table_action.clicked {
                                    self.active_tab = WorkspaceTab::Table;
                                }
                                if refresh_table {
                                    self.request_table_data();
                                }
                            }

                            // 4. Schema Object Tab (View, Trigger, Function)
                            if let Some(selection) = self.selected_schema_object.clone() {
                                let (icon, name) = match &selection {
                                    SchemaObjectSelection::View(name) => (Icon::Eye, name.clone()),
                                    SchemaObjectSelection::Trigger(name) => (Icon::Zap, name.clone()),
                                    SchemaObjectSelection::Function(name) => (Icon::Code2, name.clone()),
                                };
                                let selected = self.active_tab == WorkspaceTab::SchemaObject;
                                let mut close_obj = false;
                                let obj_action = draw_workspace_tab_item(
                                    ui,
                                    self.theme,
                                    WorkspaceTabItem {
                                        selected,
                                        icon,
                                        title: &name,
                                        unsaved: false,
                                        show_close: true,
                                    },
                                    |ui| {
                                        ui.label(
                                            RichText::new(&name)
                                                .font(font_ui_label())
                                                .strong()
                                                .color(self.theme.text_primary),
                                        );
                                        ui.separator();
                                        if ui
                                            .button(icon_text(Icon::X, "Close Tab", self.theme.text_primary))
                                            .clicked()
                                        {
                                            close_obj = true;
                                            ui.close_menu();
                                        }
                                        if ui
                                            .button(icon_text(Icon::Copy, "Copy Name", self.theme.text_primary))
                                            .clicked()
                                        {
                                            ui.output_mut(|o| o.copied_text = name.clone());
                                            ui.close_menu();
                                        }
                                    },
                                );

                                if obj_action.close_clicked || close_obj {
                                    self.request_close_workspace_tab(WorkspaceTab::SchemaObject);
                                } else if obj_action.clicked {
                                    self.active_tab = WorkspaceTab::SchemaObject;
                                }
                            }

                            // 5. ER Diagram Tab
                            if self.active_tab == WorkspaceTab::Diagram {
                                let mut close_diagram = false;
                                let diagram_action = draw_workspace_tab_item(
                                    ui,
                                    self.theme,
                                    WorkspaceTabItem {
                                        selected: true,
                                        icon: Icon::ArrowRightLeft,
                                        title: "ER Diagram",
                                        unsaved: false,
                                        show_close: true,
                                    },
                                    |ui| {
                                        ui.label(
                                            RichText::new("ER Diagram")
                                                .font(font_ui_label())
                                                .strong()
                                                .color(self.theme.text_primary),
                                        );
                                        ui.separator();
                                        if ui
                                            .button(icon_text(Icon::X, "Close Tab", self.theme.text_primary))
                                            .clicked()
                                        {
                                            close_diagram = true;
                                            ui.close_menu();
                                        }
                                    },
                                );

                                if diagram_action.close_clicked || close_diagram {
                                    self.request_close_workspace_tab(WorkspaceTab::Diagram);
                                }
                            }

                            // 6. Component Gallery Tab
                            if self.active_tab == WorkspaceTab::ComponentGallery {
                                let mut close_gallery = false;
                                let gallery_action = draw_workspace_tab_item(
                                    ui,
                                    self.theme,
                                    WorkspaceTabItem {
                                        selected: true,
                                        icon: Icon::Palette,
                                        title: "Components",
                                        unsaved: false,
                                        show_close: true,
                                    },
                                    |ui| {
                                        ui.label(
                                            RichText::new("Components")
                                                .font(font_ui_label())
                                                .strong()
                                                .color(self.theme.text_primary),
                                        );
                                        ui.separator();
                                        if ui
                                            .button(icon_text(Icon::X, "Close Tab", self.theme.text_primary))
                                            .clicked()
                                        {
                                            close_gallery = true;
                                            ui.close_menu();
                                        }
                                    },
                                );

                                if gallery_action.close_clicked || close_gallery {
                                    self.request_close_workspace_tab(WorkspaceTab::ComponentGallery);
                                }
                            }

                            // 7. Plus Button for New Query
                            ui.add_space(2.0);
                            if compact_icon_button(ui, Icon::Plus, self.theme)
                                .on_hover_text(format!("New Query Tab ({modifier}N)"))
                                .clicked()
                            {
                                self.new_query_document();
                            }
                        });
                    });
            });
        });
    }

    pub(super) fn draw_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_tabs(ui);
        match self.active_tab {
            WorkspaceTab::Welcome => self.draw_welcome(ui),
            WorkspaceTab::Query => self.draw_query(ui),
            WorkspaceTab::Table => self.draw_table_workspace(ui),
            WorkspaceTab::SchemaObject => self.draw_schema_object_workspace(ui),
            WorkspaceTab::Diagram => self.draw_diagram(ui),
            WorkspaceTab::ComponentGallery => self.draw_component_gallery(ui),
        }
    }
}

fn draw_workspace_tab_item(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    item: WorkspaceTabItem<'_>,
    context_menu: impl FnOnce(&mut egui::Ui),
) -> TabChromeAction {
    let font_id = if item.selected { font_ui_label() } else { font_body() };
    let text_color = if item.selected {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    let icon_color = if item.selected { theme.accent } else { theme.text_muted };

    let title_galley = ui.painter().layout_no_wrap(item.title.to_string(), font_id, text_color);
    let close_slot = if item.show_close { 22.0 } else { 0.0 };
    let unsaved_slot = if item.unsaved { 10.0 } else { 0.0 };
    let item_width = (18.0 + title_galley.size().x + unsaved_slot + close_slot + 18.0).clamp(72.0, 220.0);
    let tab_height = 28.0;

    let (rect, resp) = ui.allocate_exact_size(egui::vec2(item_width, tab_height), egui::Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let hovered = resp.hovered();

    // Context menu
    resp.clone().context_menu(context_menu);

    // Tab Background & Borders
    let rounding = egui::Rounding {
        nw: RADIUS_SM,
        ne: RADIUS_SM,
        sw: 0.0,
        se: 0.0,
    };

    if item.selected {
        ui.painter().rect(
            rect,
            rounding,
            theme.surface_app,
            egui::Stroke::new(STROKE_THIN, theme.border_subtle),
        );
        // Top accent indicator line
        let top_indicator_rect = egui::Rect::from_min_size(rect.left_top(), egui::vec2(rect.width(), 2.0));
        ui.painter()
            .rect_filled(top_indicator_rect, egui::Rounding::same(1.0), theme.accent);
    } else if hovered {
        ui.painter().rect_filled(rect, rounding, theme.surface_hover);
    }

    // Icon
    let icon_pos = egui::pos2(rect.left() + 8.0, rect.center().y);
    ui.painter().text(
        icon_pos,
        egui::Align2::LEFT_CENTER,
        char::from(item.icon).to_string(),
        egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())),
        icon_color,
    );

    // Title text
    let title_pos = egui::pos2(rect.left() + 24.0, rect.center().y - title_galley.size().y * 0.5);
    ui.painter().galley(title_pos, title_galley, text_color);

    // Unsaved dirty dot
    if item.unsaved {
        let dot_pos = egui::pos2(rect.right() - close_slot - 6.0, rect.center().y);
        ui.painter().circle_filled(dot_pos, 2.5, theme.accent);
    }

    // Close Button
    let mut close_clicked = false;
    if item.show_close && (item.selected || hovered) {
        let close_rect =
            egui::Rect::from_center_size(egui::pos2(rect.right() - 12.0, rect.center().y), egui::vec2(16.0, 16.0));
        let close_id = resp.id.with("close_x");
        let close_resp = ui.interact(close_rect, close_id, egui::Sense::click());
        let close_hovered = close_resp.hovered();

        if close_hovered {
            ui.painter()
                .rect_filled(close_rect, egui::Rounding::same(RADIUS_SM), theme.surface_hover);
        }

        let close_color = if close_hovered {
            theme.danger
        } else if item.selected {
            theme.text_secondary
        } else {
            theme.text_muted
        };

        ui.painter().text(
            close_rect.center(),
            egui::Align2::CENTER_CENTER,
            char::from(Icon::X).to_string(),
            egui::FontId::new(10.5, egui::FontFamily::Name("lucide".into())),
            close_color,
        );

        if close_resp.clicked() {
            close_clicked = true;
        }
    }

    let middle_clicked = resp.middle_clicked();

    TabChromeAction {
        clicked: resp.clicked() && !close_clicked,
        close_clicked: close_clicked || (middle_clicked && item.show_close),
    }
}
