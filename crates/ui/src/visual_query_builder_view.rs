//! Visual query builder panel UI (#247).

use super::*;
use crate::query::visual_builder::{
    try_import_select, BuilderColumn, BuilderDialect, BuilderJoin, BuilderPredicate, BuilderTable,
};

impl DbProApp {
    pub(super) fn visual_builder_dialect(&self) -> BuilderDialect {
        if self.active_driver().to_ascii_lowercase().contains("sqlite") {
            BuilderDialect::Sqlite
        } else {
            BuilderDialect::Postgres
        }
    }

    pub(super) fn draw_visual_query_builder(&mut self, ui: &mut egui::Ui) {
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
                self.query_editor.visual_query_model.clear();
                self.query_editor.visual_query_error = None;
                self.query_editor.visual_query_sql_preview.clear();
            }
        });
        if let Some(error) = &self.query_editor.visual_query_error {
            ui.colored_label(self.theme.warning, error);
        }

        ui.add_space(SPACE_SM);
        section_label(ui, "TABLES / VIEWS", self.theme);
        ui.add_space(SPACE_XS);
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("vqb_add_table")
                .selected_text(if self.query_editor.visual_query_add_table.is_empty() {
                    "Select table…"
                } else {
                    &self.query_editor.visual_query_add_table
                })
                .show_ui(ui, |ui| {
                    for table in &self.schema_explorer.schema.table_details {
                        let key = if table.schema.is_empty() {
                            table.name.clone()
                        } else {
                            format!("{}.{}", table.schema, table.name)
                        };
                        ui.selectable_value(
                            &mut self.query_editor.visual_query_add_table,
                            key,
                            format!("{}.{}", table.schema, table.name),
                        );
                    }
                    for view in &self.schema_explorer.schema.views {
                        let key = if view.schema.is_empty() {
                            format!("view:{}", view.name)
                        } else {
                            format!("view:{}.{}", view.schema, view.name)
                        };
                        ui.selectable_value(
                            &mut self.query_editor.visual_query_add_table,
                            key,
                            format!("view {}.{}", view.schema, view.name),
                        );
                    }
                });
            if secondary_button(ui, "Add", self.theme).clicked() {
                self.visual_builder_add_selected_table();
            }
        });

        for (idx, table) in self
            .query_editor
            .visual_query_model
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
                    self.query_editor.visual_query_model.tables.remove(idx);
                    self.refresh_visual_builder_preview();
                }
            });
        }

        ui.add_space(SPACE_SM);
        section_label(ui, "JOINS", self.theme);
        ui.add_space(SPACE_XS);
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.query_editor.visual_query_join_table).hint_text("schema.table"),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.query_editor.visual_query_join_left).hint_text("left_alias.col"),
            );
            ui.add(
                egui::TextEdit::singleline(&mut self.query_editor.visual_query_join_right).hint_text("right_alias.col"),
            );
            if secondary_button(ui, "Add INNER JOIN", self.theme).clicked() {
                self.visual_builder_add_join();
            }
            if ghost_button(ui, "FK join hint", self.theme).clicked() {
                self.visual_builder_suggest_fk_join();
            }
        });
        for (idx, join) in self
            .query_editor
            .visual_query_model
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
                    self.query_editor.visual_query_model.joins.remove(idx);
                    self.refresh_visual_builder_preview();
                }
            });
        }

        ui.add_space(SPACE_SM);
        section_label(ui, "COLUMNS", self.theme);
        ui.checkbox(&mut self.query_editor.visual_query_model.select_star, "SELECT *");
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.query_editor.visual_query_col_ref).hint_text("alias.column"));
            ui.add(egui::TextEdit::singleline(&mut self.query_editor.visual_query_col_alias).hint_text("AS alias"));
            ui.add(egui::TextEdit::singleline(&mut self.query_editor.visual_query_col_agg).hint_text("AGG optional"));
            if secondary_button(ui, "Add column", self.theme).clicked() {
                self.visual_builder_add_column();
            }
        });
        for (idx, col) in self
            .query_editor
            .visual_query_model
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
                    self.query_editor.visual_query_model.columns.remove(idx);
                    self.refresh_visual_builder_preview();
                }
            });
        }

        ui.add_space(SPACE_SM);
        section_label(ui, "WHERE / HAVING / ORDER / LIMIT", self.theme);
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.query_editor.visual_query_where_left).hint_text("alias.col"));
            ui.add(egui::TextEdit::singleline(&mut self.query_editor.visual_query_where_op).hint_text("="));
            ui.add(egui::TextEdit::singleline(&mut self.query_editor.visual_query_where_value).hint_text("value"));
            if secondary_button(ui, "Add WHERE", self.theme).clicked() {
                self.visual_builder_add_where();
            }
        });
        for (idx, pred) in self
            .query_editor
            .visual_query_model
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
                    self.query_editor.visual_query_model.where_clauses.remove(idx);
                    self.refresh_visual_builder_preview();
                }
            });
        }
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.query_editor.visual_query_order).hint_text("alias.col"));
            ui.checkbox(&mut self.query_editor.visual_query_order_desc, "DESC");
            if secondary_button(ui, "Add ORDER", self.theme).clicked() {
                self.visual_builder_add_order();
            }
            ui.label("LIMIT");
            ui.add(egui::TextEdit::singleline(&mut self.query_editor.visual_query_limit).desired_width(60.0));
            ui.label("OFFSET");
            ui.add(egui::TextEdit::singleline(&mut self.query_editor.visual_query_offset).desired_width(60.0));
            if ghost_button(ui, "Apply limit", self.theme).clicked() {
                // allow: parse error means limit/offset is not a valid integer — model retains None (no
                // LIMIT/OFFSET clause) and SQL preview updates immediately for user correction.
                self.query_editor.visual_query_model.limit = self.query_editor.visual_query_limit.parse().ok();
                self.query_editor.visual_query_model.offset = self.query_editor.visual_query_offset.parse().ok();
                self.refresh_visual_builder_preview();
            }
        });

        ui.add_space(SPACE_SM);
        section_label(ui, "GENERATED SQL", self.theme);
        self.refresh_visual_builder_preview();
        egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
            ui.label(RichText::new(&self.query_editor.visual_query_sql_preview).monospace());
        });
    }

    fn visual_builder_add_selected_table(&mut self) {
        let raw = self.query_editor.visual_query_add_table.clone();
        if raw.is_empty() {
            return;
        }
        let (schema, name) = if let Some(rest) = raw.strip_prefix("view:") {
            split_schema_table(rest)
        } else {
            split_schema_table(&raw)
        };
        self.query_editor.visual_query_model.add_table(&schema, &name);
        self.query_editor.visual_query_add_table.clear();
        self.refresh_visual_builder_preview();
    }

    fn visual_builder_add_join(&mut self) {
        let (schema, name) = split_schema_table(&self.query_editor.visual_query_join_table);
        let Some((left_alias, left_column)) = split_alias_col(&self.query_editor.visual_query_join_left) else {
            self.query_editor.visual_query_error = Some("join left must be alias.column".into());
            return;
        };
        let Some((right_alias, right_column)) = split_alias_col(&self.query_editor.visual_query_join_right) else {
            self.query_editor.visual_query_error = Some("join right must be alias.column".into());
            return;
        };
        let alias = self.query_editor.visual_query_model.next_alias(&name);
        self.query_editor.visual_query_model.joins.push(BuilderJoin {
            table: BuilderTable {
                schema,
                name,
                alias: alias.clone(),
            },
            join_type: "INNER".into(),
            left_alias,
            left_column,
            right_alias: if right_alias.is_empty() { alias } else { right_alias },
            right_column,
        });
        self.refresh_visual_builder_preview();
    }

    fn visual_builder_suggest_fk_join(&mut self) {
        let Some(primary) = self.query_editor.visual_query_model.tables.first().cloned() else {
            self.query_editor.visual_query_error = Some("add a primary table first".into());
            return;
        };
        let Some(detail) = self
            .schema_explorer
            .schema
            .table_details
            .iter()
            .find(|t| t.name == primary.name && (primary.schema.is_empty() || t.schema == primary.schema))
        else {
            self.query_editor.visual_query_error = Some("primary table metadata not loaded".into());
            return;
        };
        let Some(fk) = detail.foreign_keys.first() else {
            self.query_editor.visual_query_error = Some("no FK discovered on primary table".into());
            return;
        };
        let alias = self.query_editor.visual_query_model.next_alias(&fk.to_table);
        self.query_editor.visual_query_model.joins.push(BuilderJoin {
            table: BuilderTable {
                schema: fk.to_schema.clone(),
                name: fk.to_table.clone(),
                alias: alias.clone(),
            },
            join_type: "INNER".into(),
            left_alias: primary.alias,
            left_column: fk.from_columns.first().cloned().unwrap_or_default(),
            right_alias: alias,
            right_column: fk.to_columns.first().cloned().unwrap_or_default(),
        });
        self.query_editor.visual_query_error = None;
        self.refresh_visual_builder_preview();
    }

    fn visual_builder_add_column(&mut self) {
        let Some((table_alias, column)) = split_alias_col(&self.query_editor.visual_query_col_ref) else {
            self.query_editor.visual_query_error = Some("column must be alias.column".into());
            return;
        };
        let alias = if self.query_editor.visual_query_col_alias.trim().is_empty() {
            None
        } else {
            Some(self.query_editor.visual_query_col_alias.trim().to_owned())
        };
        let aggregate = if self.query_editor.visual_query_col_agg.trim().is_empty() {
            None
        } else {
            Some(self.query_editor.visual_query_col_agg.trim().to_ascii_uppercase())
        };
        self.query_editor.visual_query_model.columns.push(BuilderColumn {
            table_alias,
            column,
            alias,
            aggregate,
        });
        self.query_editor.visual_query_model.select_star = false;
        self.refresh_visual_builder_preview();
    }

    fn visual_builder_add_where(&mut self) {
        let Some((left_alias, left_column)) = split_alias_col(&self.query_editor.visual_query_where_left) else {
            self.query_editor.visual_query_error = Some("WHERE left must be alias.column".into());
            return;
        };
        let op = if self.query_editor.visual_query_where_op.trim().is_empty() {
            "=".to_owned()
        } else {
            self.query_editor.visual_query_where_op.trim().to_owned()
        };
        let is_null_check = op.eq_ignore_ascii_case("IS NULL") || op.eq_ignore_ascii_case("IS NOT NULL");
        self.query_editor
            .visual_query_model
            .where_clauses
            .push(BuilderPredicate {
                left_alias,
                left_column,
                op,
                value: self.query_editor.visual_query_where_value.clone(),
                is_null_check,
            });
        self.refresh_visual_builder_preview();
    }

    fn visual_builder_add_order(&mut self) {
        let Some((alias, col)) = split_alias_col(&self.query_editor.visual_query_order) else {
            self.query_editor.visual_query_error = Some("ORDER BY must be alias.column".into());
            return;
        };
        self.query_editor
            .visual_query_model
            .order_by
            .push((alias, col, self.query_editor.visual_query_order_desc));
        self.refresh_visual_builder_preview();
    }

    fn refresh_visual_builder_preview(&mut self) {
        match self
            .query_editor
            .visual_query_model
            .generate_sql(self.visual_builder_dialect())
        {
            Ok(sql) => {
                self.query_editor.visual_query_sql_preview = sql;
                self.query_editor.visual_query_error = None;
            }
            Err(err) => {
                self.query_editor.visual_query_sql_preview.clear();
                if self.query_editor.visual_query_model.tables.is_empty() {
                    self.query_editor.visual_query_error = None;
                } else {
                    self.query_editor.visual_query_error = Some(err);
                }
            }
        }
    }

    fn apply_visual_builder_sql(&mut self) {
        self.refresh_visual_builder_preview();
        if self.query_editor.visual_query_sql_preview.is_empty() {
            return;
        }
        self.set_active_query_text(self.query_editor.visual_query_sql_preview.clone());
        self.runtime_message = "Visual builder SQL applied to editor (not executed)".into();
    }

    fn import_visual_builder_from_editor(&mut self) {
        let sql = self
            .query_session_state
            .documents
            .get(self.query_session_state.active_document_index)
            .map(|d| d.buffer.text().to_owned())
            .unwrap_or_default();
        match try_import_select(&sql, self.visual_builder_dialect()) {
            Ok(model) => {
                self.query_editor.visual_query_model = model;
                self.query_editor.visual_query_error = None;
                self.refresh_visual_builder_preview();
                self.runtime_message = "Imported supported SELECT into visual builder".into();
            }
            Err(err) => {
                self.query_editor.visual_query_error = Some(format!("Import refused (keeping text editor): {err}"));
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

fn split_schema_table(raw: &str) -> (String, String) {
    if let Some((schema, table)) = raw.split_once('.') {
        (schema.to_owned(), table.to_owned())
    } else {
        (String::new(), raw.to_owned())
    }
}

fn split_alias_col(raw: &str) -> Option<(String, String)> {
    let (alias, col) = raw.split_once('.')?;
    if alias.is_empty() || col.is_empty() {
        return None;
    }
    Some((alias.to_owned(), col.to_owned()))
}
