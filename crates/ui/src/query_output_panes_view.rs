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
            egui::ScrollArea::vertical().show(ui, |ui| {
                for message in messages.iter().rev().take(20) {
                    // Producers tag failures inline ("Query failed · …", "Statement n failed · …")
                    let color = if message.contains("failed") {
                        context.theme.danger
                    } else {
                        context.theme.text_secondary
                    };
                    ui.label(RichText::new(message).small().color(color));
                }
            });
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
        // Y accepts numeric columns only; the select lists them by name and the
        // index maps back through `numeric_indexes`.
        let numeric_indexes: Vec<usize> = column_types
            .iter()
            .enumerate()
            .filter(|(_, data_type)| ChartEngine::is_numeric_column(data_type))
            .map(|(index, _)| index)
            .collect();

        if let Some(document) = context.session.documents.get_mut(document_index) {
            let config = &mut document.chart_config;
            ui.horizontal(|ui| {
                let select = |ui: &mut egui::Ui,
                              salt: &'static str,
                              selected: &mut usize,
                              options: &[String],
                              width: f32| {
                    crate::components::Select::new(salt, selected, options)
                        .theme(context.theme)
                        .size(crate::components::SelectSize::Sm)
                        .variant(crate::components::SelectVariant::Ghost)
                        .width(width)
                        .show(ui)
                };

                ui.label(RichText::new("Type").small().color(context.theme.text_secondary));
                const CHART_TYPES: [ChartType; 5] = [
                    ChartType::Bar,
                    ChartType::Line,
                    ChartType::Area,
                    ChartType::Scatter,
                    ChartType::Pie,
                ];
                let type_labels: Vec<String> = CHART_TYPES.iter().map(ToString::to_string).collect();
                let mut type_selected = CHART_TYPES.iter().position(|t| *t == config.chart_type).unwrap_or(0);
                let type_previous = type_selected;
                select(ui, "chart_type", &mut type_selected, &type_labels, 88.0);
                if type_selected != type_previous {
                    config.chart_type = CHART_TYPES[type_selected];
                }

                ui.label(RichText::new("X").small().color(context.theme.text_secondary));
                let mut x_selected = config.x_column.unwrap_or(0);
                let x_previous = x_selected;
                select(ui, "chart_x", &mut x_selected, &column_names, 120.0);
                if x_selected != x_previous {
                    config.x_column = Some(x_selected);
                }

                ui.label(RichText::new("Y").small().color(context.theme.text_secondary));
                let numeric_names: Vec<String> = numeric_indexes.iter().map(|i| column_names[*i].clone()).collect();
                if numeric_names.is_empty() {
                    ui.label(RichText::new("No numeric columns").small().color(context.theme.text_muted));
                } else {
                    let mut y_selected = config
                        .y_column
                        .and_then(|index| numeric_indexes.iter().position(|i| *i == index))
                        .unwrap_or(0);
                    let y_previous = y_selected;
                    select(ui, "chart_y", &mut y_selected, &numeric_names, 120.0);
                    if y_selected != y_previous {
                        config.y_column = Some(numeric_indexes[y_selected]);
                    }
                }

                ui.label(RichText::new("Agg").small().color(context.theme.text_secondary));
                const AGGREGATIONS: [ChartAggregation; 6] = [
                    ChartAggregation::None,
                    ChartAggregation::Count,
                    ChartAggregation::Sum,
                    ChartAggregation::Average,
                    ChartAggregation::Min,
                    ChartAggregation::Max,
                ];
                let agg_labels: Vec<String> = AGGREGATIONS.iter().map(ToString::to_string).collect();
                let mut agg_selected = AGGREGATIONS.iter().position(|a| *a == config.aggregation).unwrap_or(0);
                let agg_previous = agg_selected;
                select(ui, "chart_agg", &mut agg_selected, &agg_labels, 88.0);
                if agg_selected != agg_previous {
                    config.aggregation = AGGREGATIONS[agg_selected];
                }

                ui.label(RichText::new("Series").small().color(context.theme.text_secondary));
                let mut series_options = vec!["—".to_owned()];
                series_options.extend(column_names.iter().cloned());
                let mut series_selected = config.series_column.map(|index| index + 1).unwrap_or(0);
                let series_previous = series_selected;
                select(ui, "chart_series", &mut series_selected, &series_options, 120.0);
                if series_selected != series_previous {
                    config.series_column = (series_selected > 0).then_some(series_selected - 1);
                }
            });
        }

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
