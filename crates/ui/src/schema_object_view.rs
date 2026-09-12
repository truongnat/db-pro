use super::*;

impl DbProApp {
    pub(super) fn draw_schema_object_workspace(&mut self, ui: &mut egui::Ui) {
        let Some(selection) = self.selected_schema_object.clone() else {
            self.active_tab = WorkspaceTab::Welcome;
            return;
        };
        let is_view = matches!(selection, SchemaObjectSelection::View(_));

        let (icon, kind, name, schema, definition, metadata, query) = match selection {
            SchemaObjectSelection::View(name) => {
                let Some(view) = self.schema.views.iter().find(|view| view.name == name).cloned() else {
                    return;
                };
                (
                    Icon::Eye,
                    "VIEW".to_owned(),
                    view.name.clone(),
                    view.schema.clone(),
                    view.definition.clone(),
                    None,
                    format!("SELECT *\nFROM \"{}\".\"{}\"\nLIMIT 100;", view.schema, view.name),
                )
            }
            SchemaObjectSelection::Trigger(name) => {
                let Some(trigger) = self
                    .schema
                    .triggers
                    .iter()
                    .find(|trigger| trigger.name == name)
                    .cloned()
                else {
                    return;
                };
                (
                    Icon::Zap,
                    "TRIGGER".to_owned(),
                    trigger.name.clone(),
                    trigger.schema.clone(),
                    trigger.definition.clone(),
                    Some(format!(
                        "{} · {} · {}",
                        trigger.table_name, trigger.timing, trigger.event
                    )),
                    trigger.definition.clone(),
                )
            }
            SchemaObjectSelection::Function(name) => {
                let Some(function) = self
                    .schema
                    .functions
                    .iter()
                    .find(|function| function.name == name)
                    .cloned()
                else {
                    return;
                };
                (
                    Icon::Code2,
                    function.routine_type,
                    function.name.clone(),
                    function.schema.clone(),
                    function.definition.clone(),
                    Some(format!("returns {}", function.data_type)),
                    function.definition.clone(),
                )
            }
        };

        toolbar_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 24.0).max(0.0));
            ui.horizontal_wrapped(|ui| {
                ui.label(icon_text(icon, &kind, self.theme.text_primary));
                ui.label(icon_text(Icon::ChevronRight, "", self.theme.text_muted));
                ui.label(
                    RichText::new(format!("{schema}.{name}"))
                        .strong()
                        .color(self.theme.accent),
                );
                if let Some(metadata) = metadata.as_deref() {
                    badge(ui, metadata, self.theme.surface_active, self.theme.text_secondary);
                }
                if is_view {
                    for (view, icon, label) in [
                        (SchemaObjectView::Definition, Icon::Code2, "Definition"),
                        (SchemaObjectView::Data, Icon::Table2, "Data"),
                    ] {
                        let selected = self.schema_object_view == view;
                        let tab = tab_frame(self.theme, selected).show(ui, |ui| {
                            ui.selectable_label(selected, icon_text(icon, label, self.theme.text_primary))
                        });
                        if tab.inner.clicked() {
                            self.schema_object_view = view;
                            if view == SchemaObjectView::Data {
                                self.table_data_result = None;
                                self.table_data_total_rows = None;
                                self.table_data_error = None;
                                self.table_data_offset = 0;
                                self.request_table_data();
                            }
                        }
                    }
                }
                if secondary_button_with_icon(ui, Icon::FileCode2, "Open in Query", self.theme).clicked() {
                    self.query_text = query;
                    self.active_tab = WorkspaceTab::Query;
                }
            });
        });
        ui.add_space(SPACE_MD);
        if is_view && self.schema_object_view == SchemaObjectView::Data {
            if self.table_data_result.is_none() && self.table_data_request.is_none() && self.table_data_error.is_none()
            {
                self.request_table_data();
            }
            self.draw_table_data(ui, &name);
        } else {
            self.draw_schema_definition(ui, &kind, &definition);
        }
    }

    fn draw_schema_definition(&self, ui: &mut egui::Ui, kind: &str, definition: &str) {
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            section_label(ui, format!("{kind} DEFINITION"), self.theme);
            ui.add_space(SPACE_SM);
            CodeBlock::new(definition, self.theme).language("sql").show(ui);
        });
    }
}
