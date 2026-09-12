use super::*;

struct SchemaObjectDetails {
    icon: Icon,
    kind: String,
    name: String,
    schema: String,
    definition: String,
    metadata: Option<String>,
    query: String,
}

impl DbProApp {
    pub(super) fn draw_schema_object_workspace(&mut self, ui: &mut egui::Ui) {
        let Some(selection) = self.selected_schema_object.clone() else {
            self.active_tab = WorkspaceTab::Welcome;
            return;
        };
        let is_view = matches!(selection, SchemaObjectSelection::View(_));
        let Some(details) = self.resolve_schema_object(&selection) else {
            return;
        };

        toolbar_frame(self.theme).show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 24.0).max(0.0));
            ui.horizontal_wrapped(|ui| {
                self.draw_schema_object_breadcrumb(
                    ui,
                    details.icon,
                    &details.kind,
                    &details.schema,
                    &details.name,
                    details.metadata.as_deref(),
                );
                if is_view {
                    self.draw_schema_object_view_tabs(ui);
                }
                if secondary_button_with_icon(ui, Icon::FileCode2, "Open in Query", self.theme).clicked() {
                    self.query_text = details.query;
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
            self.draw_table_data(ui, &details.name);
        } else {
            self.draw_schema_definition(ui, &details.kind, &details.definition);
        }
    }

    fn resolve_schema_object(&self, selection: &SchemaObjectSelection) -> Option<SchemaObjectDetails> {
        match selection {
            SchemaObjectSelection::View(name) => {
                let view = self.schema.views.iter().find(|view| &view.name == name)?.clone();
                Some(SchemaObjectDetails {
                    icon: Icon::Eye,
                    kind: "VIEW".to_owned(),
                    name: view.name.clone(),
                    schema: view.schema.clone(),
                    definition: view.definition,
                    metadata: None,
                    query: format!("SELECT *\nFROM \"{}\".\"{}\"\nLIMIT 100;", view.schema, view.name),
                })
            }
            SchemaObjectSelection::Trigger(name) => {
                let trigger = self
                    .schema
                    .triggers
                    .iter()
                    .find(|trigger| &trigger.name == name)?
                    .clone();
                Some(SchemaObjectDetails {
                    icon: Icon::Zap,
                    kind: "TRIGGER".to_owned(),
                    name: trigger.name,
                    schema: trigger.schema,
                    definition: trigger.definition.clone(),
                    metadata: Some(format!(
                        "{} · {} · {}",
                        trigger.table_name, trigger.timing, trigger.event
                    )),
                    query: trigger.definition,
                })
            }
            SchemaObjectSelection::Function(name) => {
                let function = self
                    .schema
                    .functions
                    .iter()
                    .find(|function| &function.name == name)?
                    .clone();
                Some(SchemaObjectDetails {
                    icon: Icon::Code2,
                    kind: function.routine_type,
                    name: function.name,
                    schema: function.schema,
                    definition: function.definition.clone(),
                    metadata: Some(format!("returns {}", function.data_type)),
                    query: function.definition,
                })
            }
        }
    }

    fn draw_schema_object_breadcrumb(
        &self,
        ui: &mut egui::Ui,
        icon: Icon,
        kind: &str,
        schema: &str,
        name: &str,
        metadata: Option<&str>,
    ) {
        ui.label(icon_text(icon, kind, self.theme.text_primary));
        ui.label(icon_text(Icon::ChevronRight, "", self.theme.text_muted));
        ui.label(
            RichText::new(format!("{schema}.{name}"))
                .strong()
                .color(self.theme.accent),
        );
        if let Some(metadata) = metadata {
            badge(ui, metadata, self.theme.surface_active, self.theme.text_secondary);
        }
    }

    fn draw_schema_object_view_tabs(&mut self, ui: &mut egui::Ui) {
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

    fn draw_schema_definition(&self, ui: &mut egui::Ui, kind: &str, definition: &str) {
        card_frame(self.theme).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            section_label(ui, format!("{kind} DEFINITION"), self.theme);
            ui.add_space(SPACE_SM);
            CodeBlock::new(definition, self.theme).language("sql").show(ui);
        });
    }
}
