//! Query output panes that only depend on query output/session state.
use super::*;
use egui::RichText;
use lucide_icons::Icon;

pub(super) struct QueryOutputPanesContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) session: &'a mut QuerySessionState,
}

pub(super) fn draw_messages_pane(context: &mut QueryOutputPanesContext<'_>, ui: &mut egui::Ui) {
    let output_width = ui.available_width();
    let messages = context.session.active_messages();
    card_frame(context.theme).show(ui, |ui| {
        ui.set_min_width(output_width.max(0.0));
        if messages.is_empty() {
            empty_state(
                ui,
                Icon::MessageSquareText,
                "No messages yet",
                "Query notices and execution details will appear here.",
                context.theme,
            );
        } else {
            for message in messages.iter().rev().take(20) {
                ui.label(RichText::new(message).small().color(context.theme.text_secondary));
            }
        }
    });
}

pub(super) fn draw_chart_pane(
    context: &mut QueryOutputPanesContext<'_>,
    ui: &mut egui::Ui,
    result: Option<&UiQueryResult>,
) {
    use crate::{ChartAggregation, ChartEngine, ChartRenderer, ChartType};

    let output_width = ui.available_width();
    card_frame(context.theme).show(ui, |ui| {
        ui.set_min_width(output_width.max(0.0));
        let Some(result) = result else {
            empty_state(
                ui,
                Icon::BarChart3,
                "No result to chart",
                "Run a query that returns rows, then open Chart.",
                context.theme,
            );
            return;
        };
        if result.columns.is_empty() || result.rows.is_empty() {
            empty_state(
                ui,
                Icon::BarChart3,
                "No data to chart",
                "The current result has no rows.",
                context.theme,
            );
            return;
        }

        let document_index = context.session.active_document_index;
        let column_names: Vec<String> = result.columns.iter().map(|column| column.name.clone()).collect();
        let column_types: Vec<String> = result.columns.iter().map(|column| column.data_type.clone()).collect();

        ui.horizontal(|ui| {
            ui.label(RichText::new("Type").small().color(context.theme.text_secondary));
            if let Some(document) = context.session.documents.get_mut(document_index) {
                egui::ComboBox::from_id_salt("chart_type")
                    .selected_text(document.chart_config.chart_type.to_string())
                    .show_ui(ui, |ui| {
                        for chart_type in [
                            ChartType::Bar,
                            ChartType::Line,
                            ChartType::Area,
                            ChartType::Scatter,
                            ChartType::Pie,
                        ] {
                            ui.selectable_value(
                                &mut document.chart_config.chart_type,
                                chart_type,
                                chart_type.to_string(),
                            );
                        }
                    });
            }

            ui.label(RichText::new("X").small().color(context.theme.text_secondary));
            if let Some(document) = context.session.documents.get_mut(document_index) {
                let x_label = document
                    .chart_config
                    .x_column
                    .and_then(|index| column_names.get(index))
                    .cloned()
                    .unwrap_or_else(|| column_names.first().cloned().unwrap_or_default());
                egui::ComboBox::from_id_salt("chart_x")
                    .selected_text(x_label)
                    .show_ui(ui, |ui| {
                        for (index, name) in column_names.iter().enumerate() {
                            ui.selectable_value(&mut document.chart_config.x_column, Some(index), name);
                        }
                    });
            }

            ui.label(RichText::new("Y").small().color(context.theme.text_secondary));
            if let Some(document) = context.session.documents.get_mut(document_index) {
                let numeric_indexes: Vec<usize> = column_types
                    .iter()
                    .enumerate()
                    .filter(|(_, data_type)| ChartEngine::is_numeric_column(data_type))
                    .map(|(index, _)| index)
                    .collect();
                let y_label = document
                    .chart_config
                    .y_column
                    .and_then(|index| column_names.get(index))
                    .cloned()
                    .unwrap_or_else(|| {
                        numeric_indexes
                            .first()
                            .and_then(|index| column_names.get(*index))
                            .cloned()
                            .unwrap_or_else(|| "—".to_owned())
                    });
                egui::ComboBox::from_id_salt("chart_y")
                    .selected_text(y_label)
                    .show_ui(ui, |ui| {
                        if numeric_indexes.is_empty() {
                            ui.label("No numeric columns");
                        }
                        for index in numeric_indexes {
                            ui.selectable_value(&mut document.chart_config.y_column, Some(index), &column_names[index]);
                        }
                    });
            }

            ui.label(RichText::new("Agg").small().color(context.theme.text_secondary));
            if let Some(document) = context.session.documents.get_mut(document_index) {
                egui::ComboBox::from_id_salt("chart_agg")
                    .selected_text(document.chart_config.aggregation.to_string())
                    .show_ui(ui, |ui| {
                        for aggregation in [
                            ChartAggregation::None,
                            ChartAggregation::Count,
                            ChartAggregation::Sum,
                            ChartAggregation::Average,
                            ChartAggregation::Min,
                            ChartAggregation::Max,
                        ] {
                            ui.selectable_value(
                                &mut document.chart_config.aggregation,
                                aggregation,
                                aggregation.to_string(),
                            );
                        }
                    });
            }

            ui.label(RichText::new("Series").small().color(context.theme.text_secondary));
            if let Some(document) = context.session.documents.get_mut(document_index) {
                let series_label = document
                    .chart_config
                    .series_column
                    .and_then(|index| column_names.get(index))
                    .cloned()
                    .unwrap_or_else(|| "—".to_owned());
                egui::ComboBox::from_id_salt("chart_series")
                    .selected_text(series_label)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut document.chart_config.series_column, None, "—");
                        for (index, name) in column_names.iter().enumerate() {
                            ui.selectable_value(&mut document.chart_config.series_column, Some(index), name);
                        }
                    });
            }
        });

        ui.add_space(8.0);
        let config = context
            .session
            .documents
            .get(document_index)
            .map(|document| document.chart_config.clone())
            .unwrap_or_default();
        let projection = if config.chart_type == ChartType::Pie {
            ChartEngine::project_pie(&result.columns, &result.rows, &config)
        } else {
            ChartEngine::project(&result.columns, &result.rows, &config)
        };
        ui.allocate_ui(egui::vec2(ui.available_width(), 280.0), |ui| {
            ChartRenderer::draw(ui, &projection.points, &config, &context.theme);
        });
        let mut footer = format!("{} points (max {})", projection.points.len(), config.max_points.max(1));
        if projection.skipped_null_y > 0 {
            footer.push_str(&format!(" · skipped {} null/non-numeric Y", projection.skipped_null_y));
        }
        if projection.x_fallback_to_index > 0 {
            let x_note = if config.chart_type == ChartType::Pie {
                format!(" · {} X nulls labeled NULL", projection.x_fallback_to_index)
            } else {
                format!(" · {} X nulls mapped to row index", projection.x_fallback_to_index)
            };
            footer.push_str(&x_note);
        }
        ui.label(RichText::new(footer).small().color(context.theme.text_muted));
    });
}
