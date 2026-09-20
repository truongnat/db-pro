use super::*;

impl DbProApp {
    pub(super) fn draw_er_design_panel(&mut self, ui: &mut egui::Ui) {
        if !self.schema.diagram.design.enabled {
            return;
        }
        ui.add_space(SPACE_SM);
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "DESIGN DRAFT", self.theme);
            ui.label(
                RichText::new("Draft only · ObjectMutationService plan · fingerprint-gated apply")
                    .small()
                    .color(self.theme.text_muted),
            );
            let live_fp = {
                let names: Vec<String> = self
                    .schema
                    .explorer
                    .schema
                    .table_details
                    .iter()
                    .map(|t| format!("{}.{}", t.schema, t.name))
                    .collect();
                crate::diagram::design_mode::schema_fingerprint_from_names(&names)
            };
            if self.schema.diagram.design.fingerprint_stale(&live_fp) {
                ui.colored_label(
                    self.theme.warning,
                    "Live schema changed — re-open Design Mode or discard before apply",
                );
            }
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.schema.diagram.new_schema).hint_text("schema"));
                ui.add(egui::TextEdit::singleline(&mut self.schema.diagram.new_table).hint_text("table"));
                if secondary_button(ui, "Add draft table", self.theme).clicked() {
                    self.schema
                        .diagram
                        .design
                        .add_draft_table(&self.schema.diagram.new_schema, &self.schema.diagram.new_table);
                    self.schema.diagram.new_table.clear();
                }
                if ghost_button(ui, "Undo", self.theme).clicked() {
                    self.schema.diagram.design.undo();
                }
                if ghost_button(ui, "Redo", self.theme).clicked() {
                    self.schema.diagram.design.redo();
                }
                if danger_button(ui, "Discard", self.theme).clicked() {
                    self.schema.diagram.design.discard();
                }
            });
            for (idx, table) in self.schema.diagram.design.draft.tables.clone().into_iter().enumerate() {
                ui.label(
                    RichText::new(format!(
                        "draft {}.{} · {} cols",
                        table.schema,
                        table.name,
                        table.columns.len()
                    ))
                    .strong()
                    .monospace(),
                );
                for col in &table.columns {
                    ui.label(
                        RichText::new(format!(
                            "  {} {}{}{}",
                            col.name,
                            col.data_type,
                            if col.is_pk { " PK" } else { "" },
                            if col.is_unique { " UNIQUE" } else { "" }
                        ))
                        .small()
                        .monospace(),
                    );
                }
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.schema.diagram.column_name).hint_text("col"));
                    ui.add(egui::TextEdit::singleline(&mut self.schema.diagram.column_type).hint_text("type"));
                    if ghost_button(ui, "Add col", self.theme).clicked() {
                        self.schema.diagram.design.add_column(
                            idx,
                            crate::diagram::design_mode::DraftColumn {
                                name: self.schema.diagram.column_name.clone(),
                                data_type: self.schema.diagram.column_type.clone(),
                                nullable: true,
                                is_pk: false,
                                is_unique: false,
                            },
                        );
                        self.schema.diagram.column_name.clear();
                    }
                });
            }
            ui.add_space(SPACE_XS);
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.schema.diagram.foreign_key_name).hint_text("fk name"));
                ui.add(
                    egui::TextEdit::singleline(&mut self.schema.diagram.foreign_key_from)
                        .hint_text("from schema.table.col"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.schema.diagram.foreign_key_to)
                        .hint_text("to schema.table.col"),
                );
                if secondary_button(ui, "Add FK", self.theme).clicked() {
                    self.er_design_add_fk();
                }
            });
            for fk in &self.schema.diagram.design.draft.foreign_keys {
                ui.label(
                    RichText::new(format!(
                        "FK {} · {}.{}({}) → {}.{}({})",
                        fk.name,
                        fk.from_schema,
                        fk.from_table,
                        fk.from_columns.join(","),
                        fk.to_schema,
                        fk.to_table,
                        fk.to_columns.join(",")
                    ))
                    .small()
                    .monospace(),
                );
            }
            ui.horizontal(|ui| {
                if secondary_button(ui, "Preview mutation plan", self.theme).clicked() {
                    self.er_design_preview_plan();
                }
                if primary_button(ui, "Apply (confirm)", self.theme).clicked() {
                    self.er_design_apply_plan();
                }
            });
            if let Some(error) = &self.schema.diagram.design.error {
                ui.colored_label(self.theme.danger, error);
            }
            if !self.schema.diagram.design.preview_sql.is_empty() {
                ui.label(
                    RichText::new(format!(
                        "fingerprint {}",
                        self.schema.diagram.design.preview_fingerprint
                    ))
                    .small()
                    .color(self.theme.text_muted),
                );
                for effect in &self.schema.diagram.design.preview_effects {
                    ui.label(RichText::new(effect).small().color(self.theme.text_secondary));
                }
                egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                    ui.label(RichText::new(&self.schema.diagram.design.preview_sql).monospace());
                });
            }
        });
    }
}
