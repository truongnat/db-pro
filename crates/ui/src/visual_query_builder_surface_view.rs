//! Visual query-builder presentation and typed cross-feature intents.
use super::super::visual_query_builder_state::VisualQueryBuilderState;
use super::super::*;
use crate::query::visual_builder::BuilderDialect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VisualQueryBuilderAction {
    ApplySql,
    ImportFromEditor,
    Clear,
}

pub(super) struct VisualQueryBuilderContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut VisualQueryBuilderState,
    pub(super) table_details: &'a [UiTableSummary],
    pub(super) views: &'a [UiViewSummary],
    pub(super) dialect: BuilderDialect,
}

impl VisualQueryBuilderContext<'_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Option<VisualQueryBuilderAction> {
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("SELECT-only · generates SQL until you Run · unsupported SQL stays in the text editor")
                .small()
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XS);
        let mut action = None;
        ui.horizontal(|ui| {
            if secondary_button(ui, "Apply SQL → editor", self.theme).clicked() {
                action = Some(VisualQueryBuilderAction::ApplySql);
            }
            if ghost_button(ui, "Import from editor", self.theme).clicked() {
                action = Some(VisualQueryBuilderAction::ImportFromEditor);
            }
            if ghost_button(ui, "Clear", self.theme).clicked() {
                action = Some(VisualQueryBuilderAction::Clear);
            }
        });
        if let Some(error) = &self.state.error {
            ui.colored_label(self.theme.warning, error);
        }

        self.draw_tables(ui);
        self.draw_joins(ui);
        self.draw_columns(ui);
        self.draw_filters(ui);
        self.draw_preview(ui);
        action
    }

    fn draw_tables(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_SM);
        section_label(ui, "TABLES / VIEWS", self.theme);
        ui.add_space(SPACE_XS);
        self.draw_table_picker(ui);
        self.draw_selected_tables(ui);
    }

    fn draw_table_picker(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("vqb_add_table")
                .selected_text(if self.state.add_table.is_empty() {
                    "Select table…"
                } else {
                    &self.state.add_table
                })
                .show_ui(ui, |ui| {
                    for table in self.table_details {
                        let key = if table.schema.is_empty() {
                            table.name.clone()
                        } else {
                            format!("{}.{}", table.schema, table.name)
                        };
                        ui.selectable_value(
                            &mut self.state.add_table,
                            key,
                            format!("{}.{}", table.schema, table.name),
                        );
                    }
                    for view in self.views {
                        let key = if view.schema.is_empty() {
                            format!("view:{}", view.name)
                        } else {
                            format!("view:{}.{}", view.schema, view.name)
                        };
                        ui.selectable_value(
                            &mut self.state.add_table,
                            key,
                            format!("view {}.{}", view.schema, view.name),
                        );
                    }
                });
            if secondary_button(ui, "Add", self.theme).clicked() {
                self.state.add_selected_table(self.dialect);
            }
        });
    }

    fn draw_selected_tables(&mut self, ui: &mut egui::Ui) {
        for (index, table) in self.state.model.tables.clone().into_iter().enumerate() {
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
                if index == 0 {
                    badge(ui, "FROM", self.theme.surface_active, self.theme.text_secondary);
                }
                if ghost_button(ui, "Remove", self.theme).clicked() {
                    self.state.remove_table(index, self.dialect);
                }
            });
        }
    }

    fn draw_joins(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_SM);
        section_label(ui, "JOINS", self.theme);
        ui.add_space(SPACE_XS);
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.state.join_table).hint_text("schema.table"));
            ui.add(egui::TextEdit::singleline(&mut self.state.join_left).hint_text("left_alias.col"));
            ui.add(egui::TextEdit::singleline(&mut self.state.join_right).hint_text("right_alias.col"));
            if secondary_button(ui, "Add INNER JOIN", self.theme).clicked() {
                self.state.add_join(self.dialect);
            }
            if ghost_button(ui, "FK join hint", self.theme).clicked() {
                self.state.suggest_fk_join(self.table_details, self.dialect);
            }
        });
        for (index, join) in self.state.model.joins.clone().into_iter().enumerate() {
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
                    self.state.remove_join(index, self.dialect);
                }
            });
        }
    }

    fn draw_columns(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_SM);
        section_label(ui, "COLUMNS", self.theme);
        ui.checkbox(&mut self.state.model.select_star, "SELECT *");
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.state.col_ref).hint_text("alias.column"));
            ui.add(egui::TextEdit::singleline(&mut self.state.col_alias).hint_text("AS alias"));
            ui.add(egui::TextEdit::singleline(&mut self.state.col_agg).hint_text("AGG optional"));
            if secondary_button(ui, "Add column", self.theme).clicked() {
                self.state.add_column(self.dialect);
            }
        });
        for (index, column) in self.state.model.columns.clone().into_iter().enumerate() {
            ui.horizontal(|ui| {
                let mut label = if let Some(aggregate) = &column.aggregate {
                    format!("{}({}.{})", aggregate, column.table_alias, column.column)
                } else {
                    format!("{}.{}", column.table_alias, column.column)
                };
                if let Some(alias) = &column.alias {
                    label.push_str(&format!(" AS {alias}"));
                }
                ui.label(RichText::new(label).monospace().small());
                if ghost_button(ui, "×", self.theme).clicked() {
                    self.state.remove_column(index, self.dialect);
                }
            });
        }
    }

    fn draw_filters(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_SM);
        section_label(ui, "WHERE / HAVING / ORDER / LIMIT", self.theme);
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.state.where_left).hint_text("alias.col"));
            ui.add(egui::TextEdit::singleline(&mut self.state.where_op).hint_text("="));
            ui.add(egui::TextEdit::singleline(&mut self.state.where_value).hint_text("value"));
            if secondary_button(ui, "Add WHERE", self.theme).clicked() {
                self.state.add_where(self.dialect);
            }
        });
        for (index, predicate) in self.state.model.where_clauses.clone().into_iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{}.{} {} {}",
                        predicate.left_alias, predicate.left_column, predicate.op, predicate.value
                    ))
                    .monospace()
                    .small(),
                );
                if ghost_button(ui, "×", self.theme).clicked() {
                    self.state.remove_predicate(index, self.dialect);
                }
            });
        }
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.state.order).hint_text("alias.col"));
            ui.checkbox(&mut self.state.order_desc, "DESC");
            if secondary_button(ui, "Add ORDER", self.theme).clicked() {
                self.state.add_order(self.dialect);
            }
            ui.label("LIMIT");
            ui.add(egui::TextEdit::singleline(&mut self.state.limit).desired_width(60.0));
            ui.label("OFFSET");
            ui.add(egui::TextEdit::singleline(&mut self.state.offset).desired_width(60.0));
            if ghost_button(ui, "Apply limit", self.theme).clicked() {
                self.state.apply_limit_offset();
                self.state.refresh_preview(self.dialect);
            }
        });
    }

    fn draw_preview(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACE_SM);
        section_label(ui, "GENERATED SQL", self.theme);
        self.state.refresh_preview(self.dialect);
        egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
            ui.label(RichText::new(&self.state.sql_preview).monospace());
        });
    }
}

fn qualify_name(schema: &str, name: &str) -> String {
    if schema.is_empty() {
        name.to_owned()
    } else {
        format!("{schema}.{name}")
    }
}
