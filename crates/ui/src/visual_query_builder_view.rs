//! Visual query builder panel UI (#247).

use super::*;
use crate::query::visual_builder::BuilderDialect;

impl DbProApp {
    pub(super) fn visual_builder_dialect(&self) -> BuilderDialect {
        visual_query_builder_state::VisualQueryBuilderState::dialect_for_driver(self.active_driver())
    }

    pub(super) fn draw_visual_query_builder(&mut self, ui: &mut egui::Ui) {
        let dialect = self.visual_builder_dialect();
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("SELECT-only · generates SQL until you Run · unsupported SQL stays in the text editor")
                .small()
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XS);
        ui.horizontal(|ui| {
            if secondary_button(ui, "Apply SQL → editor", self.theme).clicked() {
                self.apply_visual_builder_sql();
            }
            if ghost_button(ui, "Import from editor", self.theme).clicked() {
                self.import_visual_builder_from_editor();
            }
            if ghost_button(ui, "Clear", self.theme).clicked() {
                self.query.editor.visual_builder.clear();
            }
        });
        if let Some(error) = &self.query.editor.visual_builder.error {
            ui.colored_label(self.theme.warning, error);
        }

        ui.add_space(SPACE_SM);
        section_label(ui, "TABLES / VIEWS", self.theme);
        ui.add_space(SPACE_XS);
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("vqb_add_table")
                .selected_text(if self.query.editor.visual_builder.add_table.is_empty() {
                    "Select table…"
                } else {
                    &self.query.editor.visual_builder.add_table
                })
                .show_ui(ui, |ui| {
                    for table in &self.schema.explorer.schema.table_details {
                        let key = if table.schema.is_empty() {
                            table.name.clone()
                        } else {
                            format!("{}.{}", table.schema, table.name)
                        };
                        ui.selectable_value(
                            &mut self.query.editor.visual_builder.add_table,
                            key,
                            format!("{}.{}", table.schema, table.name),
                        );
                    }
                    for view in &self.schema.explorer.schema.views {
                        let key = if view.schema.is_empty() {
                            format!("view:{}", view.name)
                        } else {
                            format!("view:{}.{}", view.schema, view.name)
                        };
                        ui.selectable_value(
                            &mut self.query.editor.visual_builder.add_table,
                            key,
                            format!("view {}.{}", view.schema, view.name),
                        );
                    }
                });
            if secondary_button(ui, "Add", self.theme).clicked() {
                self.query.editor.visual_builder.add_selected_table(dialect);
            }
        });
        for (idx, table) in self
            .query
            .editor
            .visual_builder
            .model
            .tables
            .clone()
            .into_iter()
            .enumerate()
        {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} AS {}",
                        qualify_name(&table.schema, &table.name),
                        table.alias
                    ))
                    .monospace()
                    .small(),
                );
                if idx == 0 {
                    badge(ui, "FROM", self.theme.surface_active, self.theme.text_secondary);
                }
                if ghost_button(ui, "Remove", self.theme).clicked() {
                    self.query.editor.visual_builder.remove_table(idx, dialect);
                }
            });
        }

        ui.add_space(SPACE_SM);
        section_label(ui, "JOINS", self.theme);
        ui.add_space(SPACE_XS);
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.query.editor.visual_builder.join_table).hint_text("schema.table"),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.query.editor.visual_builder.join_left).hint_text("left_alias.col"),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.query.editor.visual_builder.join_right)
                    .hint_text("right_alias.col"),
            );
            if secondary_button(ui, "Add INNER JOIN", self.theme).clicked() {
                self.query.editor.visual_builder.add_join(dialect);
            }
            if ghost_button(ui, "FK join hint", self.theme).clicked() {
                self.query
                    .editor
                    .visual_builder
                    .suggest_fk_join(&self.schema.explorer.schema.table_details, dialect);
            }
        });
        for (idx, join) in self
            .query
            .editor
            .visual_builder
            .model
            .joins
            .clone()
            .into_iter()
            .enumerate()
        {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} JOIN {} AS {} ON {}.{} = {}.{}",
                        join.join_type,
                        qualify_name(&join.table.schema, &join.table.name),
                        join.table.alias,
                        join.left_alias,
                        join.left_column,
                        join.right_alias,
                        join.right_column
                    ))
                    .monospace()
                    .small(),
                );
                if ghost_button(ui, "×", self.theme).clicked() {
                    self.query.editor.visual_builder.remove_join(idx, dialect);
                }
            });
        }

        ui.add_space(SPACE_SM);
        section_label(ui, "COLUMNS", self.theme);
        ui.checkbox(&mut self.query.editor.visual_builder.model.select_star, "SELECT *");
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.query.editor.visual_builder.col_ref).hint_text("alias.column"));
            ui.add(egui::TextEdit::singleline(&mut self.query.editor.visual_builder.col_alias).hint_text("AS alias"));
            ui.add(egui::TextEdit::singleline(&mut self.query.editor.visual_builder.col_agg).hint_text("AGG optional"));
            if secondary_button(ui, "Add column", self.theme).clicked() {
                self.query.editor.visual_builder.add_column(dialect);
            }
        });
        for (idx, col) in self
            .query
            .editor
            .visual_builder
            .model
            .columns
            .clone()
            .into_iter()
            .enumerate()
        {
            ui.horizontal(|ui| {
                let mut label = if let Some(agg) = &col.aggregate {
                    format!("{}({}.{})", agg, col.table_alias, col.column)
                } else {
                    format!("{}.{}", col.table_alias, col.column)
                };
                if let Some(alias) = &col.alias {
                    label.push_str(&format!(" AS {alias}"));
                }
                ui.label(RichText::new(label).monospace().small());
                if ghost_button(ui, "×", self.theme).clicked() {
                    self.query.editor.visual_builder.remove_column(idx, dialect);
                }
            });
        }

        ui.add_space(SPACE_SM);
        section_label(ui, "WHERE / HAVING / ORDER / LIMIT", self.theme);
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.query.editor.visual_builder.where_left).hint_text("alias.col"));
            ui.add(egui::TextEdit::singleline(&mut self.query.editor.visual_builder.where_op).hint_text("="));
            ui.add(egui::TextEdit::singleline(&mut self.query.editor.visual_builder.where_value).hint_text("value"));
            if secondary_button(ui, "Add WHERE", self.theme).clicked() {
                self.query.editor.visual_builder.add_where(dialect);
            }
        });
        for (idx, pred) in self
            .query
            .editor
            .visual_builder
            .model
            .where_clauses
            .clone()
            .into_iter()
            .enumerate()
        {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{}.{} {} {}",
                        pred.left_alias, pred.left_column, pred.op, pred.value
                    ))
                    .monospace()
                    .small(),
                );
                if ghost_button(ui, "×", self.theme).clicked() {
                    self.query.editor.visual_builder.remove_predicate(idx, dialect);
                }
            });
        }
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.query.editor.visual_builder.order).hint_text("alias.col"));
            ui.checkbox(&mut self.query.editor.visual_builder.order_desc, "DESC");
            if secondary_button(ui, "Add ORDER", self.theme).clicked() {
                self.query.editor.visual_builder.add_order(dialect);
            }
            ui.label("LIMIT");
            ui.add(egui::TextEdit::singleline(&mut self.query.editor.visual_builder.limit).desired_width(60.0));
            ui.label("OFFSET");
            ui.add(egui::TextEdit::singleline(&mut self.query.editor.visual_builder.offset).desired_width(60.0));
            if ghost_button(ui, "Apply limit", self.theme).clicked() {
                self.query.editor.visual_builder.apply_limit_offset();
                self.query.editor.visual_builder.refresh_preview(dialect);
            }
        });

        ui.add_space(SPACE_SM);
        section_label(ui, "GENERATED SQL", self.theme);
        self.query.editor.visual_builder.refresh_preview(dialect);
        egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
            ui.label(RichText::new(&self.query.editor.visual_builder.sql_preview).monospace());
        });
    }

    fn apply_visual_builder_sql(&mut self) {
        let dialect = self.visual_builder_dialect();
        self.query.editor.visual_builder.refresh_preview(dialect);
        if self.query.editor.visual_builder.sql_preview.is_empty() {
            return;
        }
        self.set_active_query_text(self.query.editor.visual_builder.sql_preview.clone());
        self.feedback.runtime_message = "Visual builder SQL applied to editor (not executed)".into();
    }

    fn import_visual_builder_from_editor(&mut self) {
        let sql = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .map(|document| document.buffer.text().to_owned())
            .unwrap_or_default();
        let dialect = self.visual_builder_dialect();
        match self.query.editor.visual_builder.import_sql(&sql, dialect) {
            Ok(()) => {
                self.feedback.runtime_message = "Imported supported SELECT into visual builder".into();
            }
            Err(err) => {
                self.query.editor.visual_builder.error = Some(format!("Import refused (keeping text editor): {err}"));
            }
        }
    }
}

fn qualify_name(schema: &str, name: &str) -> String {
    if schema.is_empty() {
        name.to_owned()
    } else {
        format!("{schema}.{name}")
    }
}
