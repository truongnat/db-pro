use super::workspace_tab_primitives::{draw_workspace_tab_item, WorkspaceTabItem};
use super::*;

impl DbProApp {
    pub(super) fn draw_workspace_tabs(&mut self, ui: &mut egui::Ui) {
        let modifier = Self::primary_modifier_label();

        // Tab strip flush with CentralPanel; tiny top breath only.
        let tabs_width = ui.available_width();
        ui.set_min_width(tabs_width);
        egui::Frame {
            fill: self.theme.surface_panel,
            inner_margin: egui::Margin {
                left: SPACE_XS,
                right: SPACE_XS,
                top: SPACE_XXS,
                bottom: 0.0,
            },
            stroke: egui::Stroke::NONE,
            rounding: egui::Rounding::ZERO,
            outer_margin: egui::Margin::ZERO,
            ..Default::default()
        }
        .show(ui, |ui| {
            let inner = ui.max_rect();
            ui.set_min_size(egui::vec2(inner.width(), ui.min_rect().height().max(28.0)));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);

                egui::ScrollArea::horizontal()
                    .id_salt("workspace-tabs-scroll")
                    // Take full width; shrink height to the tab row (not the whole panel).
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);

                            let mut close_all_requested = false;
                            let mut close_welcome_requested = false;

                            // 1. Welcome Tab
                            if self.workspace.welcome_open {
                                let welcome_selected = self.workspace.active_tab == WorkspaceTab::Welcome;
                                let welcome_action = draw_workspace_tab_item(
                                    ui,
                                    self.theme,
                                    WorkspaceTabItem {
                                        selected: welcome_selected,
                                        icon: Icon::House,
                                        title: "Welcome",
                                        unsaved: false,
                                        show_close: true,
                                    },
                                    |ui, close_menu| {
                                        ui.label(
                                            RichText::new("Welcome")
                                                .font(font_ui_label())
                                                .strong()
                                                .color(self.theme.text_primary),
                                        );
                                        ui.separator();
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::X),
                                            "Close Tab",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            close_welcome_requested = true;
                                            *close_menu = true;
                                        }
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::Layers),
                                            "Close All Tabs",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            close_all_requested = true;
                                            *close_menu = true;
                                        }
                                    },
                                );
                                if welcome_action.close_clicked {
                                    close_welcome_requested = true;
                                } else if welcome_action.clicked {
                                    self.activate_welcome_tab();
                                }
                            }
                            if close_welcome_requested {
                                self.close_welcome_tab();
                            }

                            // 2. Query Documents Tabs
                            let documents: Vec<(usize, String, String)> = self
                                .query
                                .session
                                .documents
                                .iter()
                                .enumerate()
                                .map(|(index, doc)| (index, doc.title.clone(), doc.content().to_owned()))
                                .collect();

                            let mut switch_query_idx = None;
                            let mut close_query_idx = None;
                            let mut duplicate_query_idx = None;
                            let mut close_others_idx = None;
                            let mut close_right_idx = None;
                            let mut run_query_idx = None;

                            for (index, title, content) in &documents {
                                let idx = *index;
                                let selected = self.workspace.active_tab == WorkspaceTab::Query
                                    && self.query.session.active_document_index == idx;
                                let is_running =
                                    self.query.session.documents.get(idx).is_some_and(|doc| {
                                        matches!(doc.execution_state, QueryExecutionState::Running(_))
                                    });
                                let unsaved = self
                                    .query
                                    .session
                                    .documents
                                    .get(idx)
                                    .is_some_and(QueryDocument::is_dirty);
                                let icon = if is_running { Icon::Loader } else { Icon::FileCode2 };

                                let action = draw_workspace_tab_item(
                                    ui,
                                    self.theme,
                                    WorkspaceTabItem {
                                        selected,
                                        icon,
                                        title,
                                        unsaved,
                                        show_close: true,
                                    },
                                    |ui, close_menu| {
                                        ui.label(
                                            RichText::new(title)
                                                .font(font_ui_label())
                                                .strong()
                                                .color(self.theme.text_primary),
                                        );
                                        ui.separator();
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::X),
                                            &format!("Close Tab ({modifier}W)"),
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            close_query_idx = Some(idx);
                                            *close_menu = true;
                                        }
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::Layers),
                                            "Close Other Tabs",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            close_others_idx = Some(idx);
                                            *close_menu = true;
                                        }
                                        if idx + 1 < self.query.session.documents.len()
                                            && ctx_menu_item(
                                                ui,
                                                Some(Icon::ArrowRight),
                                                "Close Tabs to the Right",
                                                None,
                                                self.theme.text_primary,
                                                self.theme,
                                            )
                                            .clicked()
                                        {
                                            close_right_idx = Some(idx);
                                            *close_menu = true;
                                        }
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::Layers),
                                            "Close All Tabs",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            close_all_requested = true;
                                            *close_menu = true;
                                        }
                                        ui.separator();
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::Copy),
                                            "Duplicate Tab",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            duplicate_query_idx = Some(idx);
                                            *close_menu = true;
                                        }
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::FileCode2),
                                            "Copy SQL Content",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            ui.output_mut(|o| o.copied_text = content.clone());
                                            *close_menu = true;
                                        }
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::FileText),
                                            "Copy Title",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            ui.output_mut(|o| o.copied_text = title.to_string());
                                            *close_menu = true;
                                        }
                                        ui.separator();
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::Play),
                                            &format!("Run Query ({modifier}↵)"),
                                            None,
                                            self.theme.accent,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            run_query_idx = Some(idx);
                                            *close_menu = true;
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
                                self.workspace.active_tab = WorkspaceTab::Query;
                            }
                            if let Some(idx) = close_query_idx {
                                self.request_close_query_document(idx);
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
                                self.workspace.active_tab = WorkspaceTab::Query;
                                self.dispatch_query();
                            }
                            if close_all_requested {
                                self.close_all_tabs();
                            }

                            // 3. Table Tab
                            if let Some(table_name) = self.schema.explorer.selected_table.clone() {
                                let selected = self.workspace.active_tab == WorkspaceTab::Table;
                                let unsaved = !self.table.mutation.staged_changes.is_empty();
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
                                    |ui, close_menu| {
                                        ui.label(
                                            RichText::new(&table_name)
                                                .font(font_ui_label())
                                                .strong()
                                                .color(self.theme.text_primary),
                                        );
                                        ui.separator();
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::X),
                                            "Close Tab",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            close_table = true;
                                            *close_menu = true;
                                        }
                                        ui.separator();
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::RefreshCw),
                                            "Refresh Data",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            refresh_table = true;
                                            *close_menu = true;
                                        }
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::Copy),
                                            "Copy Table Name",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            ui.output_mut(|o| o.copied_text = table_name.clone());
                                            *close_menu = true;
                                        }
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::FileCode2),
                                            "Copy SELECT Query",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            ui.output_mut(|o| {
                                                o.copied_text = format!("SELECT * FROM {table_name} LIMIT 100;")
                                            });
                                            *close_menu = true;
                                        }
                                    },
                                );

                                if table_action.close_clicked || close_table {
                                    self.request_close_workspace_tab(WorkspaceTab::Table);
                                } else if table_action.clicked {
                                    self.workspace.active_tab = WorkspaceTab::Table;
                                }
                                if refresh_table {
                                    self.request_table_data();
                                }
                            }

                            // 4. Schema Object Tab (View, Trigger, Function)
                            if let Some(selection) = self.schema.explorer.selected_schema_object.clone() {
                                let (icon, name) = match &selection {
                                    SchemaObjectSelection::View(name) => (Icon::Eye, name.clone()),
                                    SchemaObjectSelection::Trigger(name) => (Icon::Zap, name.clone()),
                                    SchemaObjectSelection::Function { name, .. } => (Icon::Code2, name.clone()),
                                };
                                let selected = self.workspace.active_tab == WorkspaceTab::SchemaObject;
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
                                    |ui, close_menu| {
                                        ui.label(
                                            RichText::new(&name)
                                                .font(font_ui_label())
                                                .strong()
                                                .color(self.theme.text_primary),
                                        );
                                        ui.separator();
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::X),
                                            "Close Tab",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            close_obj = true;
                                            *close_menu = true;
                                        }
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::Copy),
                                            "Copy Name",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            ui.output_mut(|o| o.copied_text = name.clone());
                                            *close_menu = true;
                                        }
                                    },
                                );

                                if obj_action.close_clicked || close_obj {
                                    self.request_close_workspace_tab(WorkspaceTab::SchemaObject);
                                } else if obj_action.clicked {
                                    self.workspace.active_tab = WorkspaceTab::SchemaObject;
                                }
                            }

                            // 5. ER Diagram Tab
                            if self.workspace.active_tab == WorkspaceTab::Diagram {
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
                                    |ui, close_menu| {
                                        ui.label(
                                            RichText::new("ER Diagram")
                                                .font(font_ui_label())
                                                .strong()
                                                .color(self.theme.text_primary),
                                        );
                                        ui.separator();
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::X),
                                            "Close Tab",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            close_diagram = true;
                                            *close_menu = true;
                                        }
                                    },
                                );

                                if diagram_action.close_clicked || close_diagram {
                                    self.request_close_workspace_tab(WorkspaceTab::Diagram);
                                }
                            }

                            if self.workspace.active_tab == WorkspaceTab::SchemaWorkbench {
                                let mut close_wb = false;
                                let wb_action = draw_workspace_tab_item(
                                    ui,
                                    self.theme,
                                    WorkspaceTabItem {
                                        selected: true,
                                        icon: Icon::Boxes,
                                        title: "Schema Workbench",
                                        unsaved: false,
                                        show_close: true,
                                    },
                                    |ui, close_menu| {
                                        ui.label(
                                            RichText::new("Schema Workbench")
                                                .font(font_ui_label())
                                                .strong()
                                                .color(self.theme.text_primary),
                                        );
                                        ui.separator();
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::X),
                                            "Close Tab",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            close_wb = true;
                                            *close_menu = true;
                                        }
                                    },
                                );

                                if wb_action.close_clicked || close_wb {
                                    self.request_close_workspace_tab(WorkspaceTab::SchemaWorkbench);
                                }
                            }

                            if self.workspace.active_tab == WorkspaceTab::SchemaCompare {
                                let mut close_cmp = false;
                                let cmp_action = draw_workspace_tab_item(
                                    ui,
                                    self.theme,
                                    WorkspaceTabItem {
                                        selected: true,
                                        icon: Icon::GitCompare,
                                        title: "Schema Compare",
                                        unsaved: false,
                                        show_close: true,
                                    },
                                    |ui, close_menu| {
                                        ui.label(
                                            RichText::new("Schema Compare")
                                                .font(font_ui_label())
                                                .strong()
                                                .color(self.theme.text_primary),
                                        );
                                        ui.separator();
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::X),
                                            "Close Tab",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            close_cmp = true;
                                            *close_menu = true;
                                        }
                                    },
                                );
                                if cmp_action.close_clicked || close_cmp {
                                    self.request_close_workspace_tab(WorkspaceTab::SchemaCompare);
                                }
                            }

                            // 6. Component Gallery Tab
                            if self.workspace.active_tab == WorkspaceTab::ComponentGallery {
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
                                    |ui, close_menu| {
                                        ui.label(
                                            RichText::new("Components")
                                                .font(font_ui_label())
                                                .strong()
                                                .color(self.theme.text_primary),
                                        );
                                        ui.separator();
                                        if ctx_menu_item(
                                            ui,
                                            Some(Icon::X),
                                            "Close Tab",
                                            None,
                                            self.theme.text_primary,
                                            self.theme,
                                        )
                                        .clicked()
                                        {
                                            close_gallery = true;
                                            *close_menu = true;
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
}
