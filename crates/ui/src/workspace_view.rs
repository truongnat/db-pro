use super::*;

impl DbProApp {
    pub(super) fn draw_workspace_tabs_panel(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("workspace-tabs")
            .exact_height(36.0)
            .frame(egui::Frame {
                fill: self.theme.surface_app,
                inner_margin: egui::Margin::symmetric(10.0, 2.0),
                stroke: egui::Stroke::new(1.0, self.theme.border_subtle),
                ..Default::default()
            })
            .show(ctx, |ui| self.draw_workspace_tabs(ui));
    }

    pub(super) fn draw_workspace_tabs(&mut self, ui: &mut egui::Ui) {
        let tabs_width = ui.available_width().max(160.0);
        egui::Frame {
            fill: egui::Color32::TRANSPARENT,
            inner_margin: egui::Margin::symmetric(2.0, 1.0),
            stroke: egui::Stroke::NONE,
            rounding: egui::Rounding::ZERO,
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(tabs_width, 34.0),
                Layout::left_to_right(Align::Center),
                |ui| {
                    egui::ScrollArea::horizontal()
                        .id_salt("workspace-tabs-scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add_space(2.0);
                                self.draw_welcome_tab(ui);
                                self.draw_query_tabs(ui);
                                self.draw_table_tab(ui);
                                self.draw_schema_object_tab(ui);
                                self.draw_diagram_tab(ui);
                                if compact_icon_button(ui, Icon::Plus, self.theme)
                                    .on_hover_text("New query")
                                    .clicked()
                                {
                                    self.new_query_document();
                                }
                            });
                        });
                },
            );
        });
    }

    /// Static "Welcome" tab, always present at the start of the strip.
    fn draw_welcome_tab(&mut self, ui: &mut egui::Ui) {
        let selected = self.active_tab == WorkspaceTab::Welcome;
        let welcome = tab_frame(self.theme, selected).show(ui, |ui| {
            ui.selectable_label(selected, icon_text(Icon::House, "Welcome", self.theme.text_primary))
        });
        if selected {
            paint_tab_indicator(ui, welcome.response.rect, self.theme);
        }
        if welcome.inner.clicked() {
            self.active_tab = WorkspaceTab::Welcome;
        }
    }

    /// One tab per open query document, with unsaved marker and close affordance.
    fn draw_query_tabs(&mut self, ui: &mut egui::Ui) {
        let documents: Vec<(usize, String)> = self
            .query_documents
            .iter()
            .enumerate()
            .map(|(index, document)| (index, document.title.clone()))
            .collect();
        for (index, title) in documents {
            let selected = self.active_tab == WorkspaceTab::Query && self.active_query_document == index;
            let can_close = self.query_documents.len() > 1;
            let unsaved = selected
                && self
                    .query_documents
                    .get(index)
                    .is_some_and(|document| document.content != self.query_text);
            let tab_title = if unsaved {
                format!("{title}  •")
            } else {
                title.clone()
            };
            let tab = tab_frame(self.theme, selected).show(ui, |ui| {
                let mut tab_clicked = false;
                let mut close_clicked = false;
                ui.horizontal(|ui| {
                    let tab_response = ui.selectable_label(
                        selected,
                        icon_text(Icon::FileCode2, &tab_title, self.theme.text_primary),
                    );
                    tab_clicked = tab_response.clicked();
                    let tab_hovered = ui
                        .ctx()
                        .pointer_hover_pos()
                        .is_some_and(|position| tab_response.rect.expand2(egui::vec2(34.0, 4.0)).contains(position));
                    if can_close && (selected || tab_hovered) {
                        close_clicked = compact_icon_button(ui, Icon::X, self.theme)
                            .on_hover_text("Close query")
                            .clicked();
                    }
                });
                (tab_clicked, close_clicked)
            });
            if selected {
                paint_tab_indicator(ui, tab.response.rect, self.theme);
            }
            if tab.inner.0 {
                self.switch_query_document(index);
                self.active_tab = WorkspaceTab::Query;
            }
            if tab.inner.1 {
                self.close_query_document(index);
            }
        }
    }

    /// Tab for the currently selected table, if one is open.
    fn draw_table_tab(&mut self, ui: &mut egui::Ui) {
        let Some(table_name) = self.selected_table.clone() else {
            return;
        };
        let selected = self.active_tab == WorkspaceTab::Table;
        let (tab_clicked, close_clicked) = self.draw_closable_tab(
            ui,
            ClosableTab {
                selected,
                icon: Icon::Table2,
                label: &table_name,
                close_hint: "Close table",
            },
        );
        if tab_clicked {
            self.active_tab = WorkspaceTab::Table;
        }
        if close_clicked {
            self.request_close_workspace_tab(WorkspaceTab::Table);
        }
    }

    /// Tab for the currently selected view / trigger / function, if one is open.
    fn draw_schema_object_tab(&mut self, ui: &mut egui::Ui) {
        let Some(selection) = self.selected_schema_object.clone() else {
            return;
        };
        let (icon, name) = match selection {
            SchemaObjectSelection::View(name) => (Icon::Eye, name),
            SchemaObjectSelection::Trigger(name) => (Icon::Zap, name),
            SchemaObjectSelection::Function(name) => (Icon::Code2, name),
        };
        let selected = self.active_tab == WorkspaceTab::SchemaObject;
        let (tab_clicked, close_clicked) = self.draw_closable_tab(
            ui,
            ClosableTab {
                selected,
                icon,
                label: &name,
                close_hint: "Close schema object",
            },
        );
        if tab_clicked {
            self.active_tab = WorkspaceTab::SchemaObject;
        }
        if close_clicked {
            self.request_close_workspace_tab(WorkspaceTab::SchemaObject);
        }
    }

    /// ER diagram tab, rendered only while the diagram workspace is open.
    fn draw_diagram_tab(&mut self, ui: &mut egui::Ui) {
        if self.active_tab != WorkspaceTab::Diagram {
            return;
        }
        let diagram_tab = tab_frame(self.theme, true).show(ui, |ui| {
            let mut close_clicked = false;
            ui.horizontal(|ui| {
                // The ER diagram tab is always active while shown, so the label
                // is decorative; only the close button below is interactive.
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
        paint_tab_indicator(ui, diagram_tab.response.rect, self.theme);
        if diagram_tab.inner {
            self.request_close_workspace_tab(WorkspaceTab::Diagram);
        }
    }

    /// Renders a closable tab and paints its selection indicator.
    /// Returns `(tab_clicked, close_clicked)`.
    fn draw_closable_tab(&self, ui: &mut egui::Ui, tab: ClosableTab<'_>) -> (bool, bool) {
        let response = tab_frame(self.theme, tab.selected).show(ui, |ui| {
            let mut tab_clicked = false;
            let mut close_clicked = false;
            ui.horizontal(|ui| {
                tab_clicked = ui
                    .selectable_label(tab.selected, icon_text(tab.icon, tab.label, self.theme.text_primary))
                    .clicked();
                close_clicked = compact_icon_button(ui, Icon::X, self.theme)
                    .on_hover_text(tab.close_hint)
                    .clicked();
            });
            (tab_clicked, close_clicked)
        });
        if tab.selected {
            paint_tab_indicator(ui, response.response.rect, self.theme);
        }
        response.inner
    }

    pub(super) fn draw_workspace(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        match self.active_tab {
            WorkspaceTab::Welcome => self.draw_welcome(ui),
            WorkspaceTab::Query => self.draw_query(ui),
            WorkspaceTab::Table => self.draw_table_workspace(ui),
            WorkspaceTab::SchemaObject => self.draw_schema_object_workspace(ui),
            WorkspaceTab::Diagram => self.draw_diagram(ui),
        }
    }
}

/// A closable tab rendered in the workspace tab strip.
struct ClosableTab<'a> {
    selected: bool,
    icon: Icon,
    label: &'a str,
    close_hint: &'a str,
}

fn paint_tab_indicator(ui: &egui::Ui, rect: egui::Rect, theme: DbProTheme) {
    ui.painter().line_segment(
        [
            egui::pos2(rect.left() + 6.0, rect.bottom() - 1.0),
            egui::pos2(rect.right() - 6.0, rect.bottom() - 1.0),
        ],
        egui::Stroke::new(1.5, theme.accent),
    );
}
