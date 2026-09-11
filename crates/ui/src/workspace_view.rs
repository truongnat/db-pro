use super::*;

impl DbProApp {
    pub(super) fn draw_workspace(&mut self, ui: &mut egui::Ui) {
        let toolbar_width = ui.max_rect().width();
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((toolbar_width - 24.0).max(0.0));
            ui.horizontal_wrapped(|ui| {
                let welcome = tab_frame(self.theme, self.active_tab == WorkspaceTab::Welcome).show(ui, |ui| {
                    ui.selectable_label(
                        self.active_tab == WorkspaceTab::Welcome,
                        icon_text(Icon::House, "Welcome", self.theme.text_primary),
                    )
                });
                if welcome.inner.clicked() {
                    self.active_tab = WorkspaceTab::Welcome;
                }
                let documents: Vec<(usize, String)> = self
                    .query_documents
                    .iter()
                    .enumerate()
                    .map(|(index, document)| (index, document.title.clone()))
                    .collect();
                for (index, title) in documents {
                    let selected = self.active_tab == WorkspaceTab::Query && self.active_query_document == index;
                    let can_close = self.query_documents.len() > 1;
                    let tab = tab_frame(self.theme, selected).show(ui, |ui| {
                        let mut tab_clicked = false;
                        let mut close_clicked = false;
                        ui.horizontal(|ui| {
                            tab_clicked = ui
                                .selectable_label(selected, icon_text(Icon::FileCode2, &title, self.theme.text_primary))
                                .clicked();
                            if can_close {
                                close_clicked = compact_icon_button(ui, Icon::X, self.theme)
                                    .on_hover_text("Close query")
                                    .clicked();
                            }
                        });
                        (tab_clicked, close_clicked)
                    });
                    if tab.inner.0 {
                        self.switch_query_document(index);
                        self.active_tab = WorkspaceTab::Query;
                    }
                    if tab.inner.1 {
                        self.close_query_document(index);
                    }
                }
                if let Some(table_name) = self.selected_table.clone() {
                    let selected = self.active_tab == WorkspaceTab::Table;
                    let table_tab = tab_frame(self.theme, selected).show(ui, |ui| {
                        let mut tab_clicked = false;
                        let mut close_clicked = false;
                        ui.horizontal(|ui| {
                            tab_clicked = ui
                                .selectable_label(
                                    selected,
                                    icon_text(Icon::Table2, &table_name, self.theme.text_primary),
                                )
                                .clicked();
                            close_clicked = compact_icon_button(ui, Icon::X, self.theme)
                                .on_hover_text("Close table")
                                .clicked();
                        });
                        (tab_clicked, close_clicked)
                    });
                    if table_tab.inner.0 {
                        self.active_tab = WorkspaceTab::Table;
                    }
                    if table_tab.inner.1 {
                        self.request_close_workspace_tab(WorkspaceTab::Table);
                    }
                }
                if let Some(selection) = self.selected_schema_object.clone() {
                    let (icon, name) = match selection {
                        SchemaObjectSelection::View(name) => (Icon::Eye, name),
                        SchemaObjectSelection::Trigger(name) => (Icon::Zap, name),
                        SchemaObjectSelection::Function(name) => (Icon::Code2, name),
                    };
                    let selected = self.active_tab == WorkspaceTab::SchemaObject;
                    let object_tab = tab_frame(self.theme, selected).show(ui, |ui| {
                        let mut tab_clicked = false;
                        let mut close_clicked = false;
                        ui.horizontal(|ui| {
                            tab_clicked = ui
                                .selectable_label(selected, icon_text(icon, &name, self.theme.text_primary))
                                .clicked();
                            close_clicked = compact_icon_button(ui, Icon::X, self.theme)
                                .on_hover_text("Close schema object")
                                .clicked();
                        });
                        (tab_clicked, close_clicked)
                    });
                    if object_tab.inner.0 {
                        self.active_tab = WorkspaceTab::SchemaObject;
                    }
                    if object_tab.inner.1 {
                        self.request_close_workspace_tab(WorkspaceTab::SchemaObject);
                    }
                }
                if self.active_tab == WorkspaceTab::Diagram {
                    let diagram_tab = tab_frame(self.theme, true).show(ui, |ui| {
                        let mut close_clicked = false;
                        ui.horizontal(|ui| {
                            let _ = ui.selectable_label(
                                true,
                                icon_text(Icon::ArrowRightLeft, "ER diagram", self.theme.text_primary),
                            );
                            close_clicked = compact_icon_button(ui, Icon::X, self.theme)
                                .on_hover_text("Close ER diagram")
                                .clicked();
                        });
                        close_clicked
                    });
                    if diagram_tab.inner {
                        self.request_close_workspace_tab(WorkspaceTab::Diagram);
                    }
                }
                if compact_icon_button(ui, Icon::Plus, self.theme)
                    .on_hover_text("New query")
                    .clicked()
                {
                    self.new_query_document();
                }
            });
        });
        ui.add_space(12.0);
        match self.active_tab {
            WorkspaceTab::Welcome => self.draw_welcome(ui),
            WorkspaceTab::Query => self.draw_query(ui),
            WorkspaceTab::Table => self.draw_table_workspace(ui),
            WorkspaceTab::SchemaObject => self.draw_schema_object_workspace(ui),
            WorkspaceTab::Diagram => self.draw_diagram(ui),
        }
    }
}
