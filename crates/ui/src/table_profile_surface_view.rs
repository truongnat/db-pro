// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
//! Table-profile presentation and bounded page-level profiling.
use super::super::*;
use crate::components::table::{Table, TableColumn};
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime};
use egui::{Rect, CornerRadius, Vec2};

pub(super) fn draw_profile_pane(theme: DbProTheme, result: Option<&UiQueryResult>, ui: &mut egui::Ui) {
    egui::Frame {
        fill: theme.surface_panel,
        inner_margin: egui::Margin::same(SPACE_2XL as i8),
        corner_radius: egui::CornerRadius::same(RADIUS_MD as u8),
        stroke: egui::Stroke::new(STROKE_THIN, theme.border_subtle),
        ..Default::default()
    }
    .show(ui, |ui| {
        let Some(result) = result else {
            EmptyState::new(
                Icon::ChartColumn,
                &t!("table.no_rows"),
                "Open the Data tab or wait for the current page to load, then return to Profile.",
                theme,
            )
            .show(ui);
            return;
        };
        if result.columns.is_empty() {
            EmptyState::new(
                Icon::ChartColumn,
                &t!("table.no_columns"),
                "This result has no columns to profile.",
                theme,
            )
            .show(ui);
            return;
        }
        if result.rows.is_empty() {
            EmptyState::new(
                Icon::ChartColumn,
                &t!("table.no_rows"),
                "Load a page of rows in the Data tab to calculate column profiles.",
                theme,
            )
            .show(ui);
            return;
        }

        let profiles = profile_result_columns(result);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Column profile")
                        .font(font_subheading())
                        .strong()
                        .color(theme.text_primary),
                );
                ui.add_space(SPACE_XS);
                ui.label(
                    RichText::new(format!(
                        "{} loaded rows · {} columns",
                        result.rows.len(),
                        result.columns.len()
                    ))
                    .small()
                    .color(theme.text_secondary),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                badge(ui, "Page sample", theme.surface_active, theme.text_secondary);
            });
        });
        ui.add_space(SPACE_2XL);
        draw_profile_grid(theme, ui, &profiles, result.rows.len());
    });
}

pub(super) fn profile_result_columns(result: &UiQueryResult) -> Vec<ColumnProfile> {
    let row_count = result.rows.len().max(1) as f64;
    result
        .columns
        .iter()
        .enumerate()
        .map(|(col_idx, column)| {
            let mut null_count = 0usize;
            let mut number_count = 0usize;
            let mut values = Vec::new();
            let mut numeric_values = Vec::new();

            for row in &result.rows {
                match row.get(col_idx) {
                    None | Some(UiCell::Null) => null_count += 1,
                    Some(UiCell::Number(num_str)) => {
                        number_count += 1;
                        values.push(num_str.clone());
                        if let Ok(val) = num_str.parse::<f64>() {
                            numeric_values.push(val);
                        }
                    }
                    Some(cell) => values.push(cell_profile_text(cell)),
                }
            }

            let numeric_cells_only = number_count > 0 && number_count + null_count == result.rows.len();
            let numeric_calculations_available = numeric_cells_only && numeric_values.len() == number_count;
            let sum = if numeric_calculations_available {
                Some(numeric_values.iter().sum())
            } else {
                None
            };
            let avg = if numeric_calculations_available {
                Some(sum.unwrap_or(0.0) / numeric_values.len() as f64)
            } else {
                None
            };

            let distinct = values.iter().cloned().collect::<std::collections::BTreeSet<_>>();
            let (min, max) = if number_count > 0 && values.len() != number_count {
                (None, None)
            } else {
                profile_extrema(&values, numeric_cells_only)
            };

            ColumnProfile {
                name: column.name.clone(),
                data_type: column.data_type.clone(),
                null_count,
                null_rate: null_count as f64 / row_count,
                distinct_count: distinct.len(),
                distinct_rate: distinct.len() as f64 / row_count,
                min,
                max,
                avg,
                sum,
            }
        })
        .collect()
}

fn profile_extrema(values: &[String], is_numeric: bool) -> (Option<String>, Option<String>) {
    if is_numeric {
        let parsed = values
            .iter()
            .map(|value| value.parse::<BigDecimal>().map(|number| (value, number)))
            .collect::<Result<Vec<_>, _>>();
        let Ok(parsed) = parsed else {
            return (None, None);
        };

        let min = parsed
            .iter()
            .min_by(|left, right| left.1.cmp(&right.1))
            .map(|(value, _)| (*value).clone());
        let max = parsed
            .iter()
            .max_by(|left, right| left.1.cmp(&right.1))
            .map(|(value, _)| (*value).clone());
        return (min, max);
    }

    (values.iter().min().cloned(), values.iter().max().cloned())
}

fn profile_text_width(ui: &egui::Ui, text: RichText) -> f32 {
    egui::WidgetText::from(text)
        .into_galley(
            ui,
            Some(egui::TextWrapMode::Extend),
            f32::INFINITY,
            egui::TextStyle::Body,
        )
        .size()
        .x
        .ceil()
}

fn profile_columns(
    ui: &egui::Ui,
    theme: DbProTheme,
    profiles: &[ColumnProfile],
    row_count: usize,
    numeric_summary: bool,
) -> Vec<TableColumn<'static>> {
    let mut headings = vec!["Column", "Completeness", "Distinct", "Pattern", "Range · Min / Max"];
    if numeric_summary {
        headings.push("Avg · Sum");
    }
    let mut widths = headings
        .iter()
        .map(|title| profile_text_width(ui, RichText::new(*title).font(font_ui_label())))
        .collect::<Vec<_>>();
    for profile in profiles {
        widths[0] = widths[0]
            .max(profile_text_width(
                ui,
                RichText::new(crate::components::truncate_ellipsis(&profile.name, 28)).strong(),
            ))
            .max(profile_text_width(
                ui,
                RichText::new(crate::components::truncate_ellipsis(&profile.data_type, 22))
                    .small()
                    .monospace(),
            ));
        widths[1] = widths[1].max(
            60.0 + SPACE_SM
                + profile_text_width(
                    ui,
                    RichText::new(format!("{:.0}% full", (1.0 - profile.null_rate) * 100.0)).small(),
                ),
        );
        widths[2] = widths[2].max(profile_text_width(
            ui,
            RichText::new(format!(
                "{} · {:.0}%",
                profile.distinct_count,
                profile.distinct_rate * 100.0
            ))
            .monospace(),
        ));
        let (pattern, _, _) = profile_pattern(theme, profile, row_count.saturating_sub(profile.null_count));
        widths[3] = widths[3].max(profile_text_width(ui, RichText::new(pattern).size(10.0)) + 12.0);
        if let (Some(min), Some(max)) = (profile.min.as_deref(), profile.max.as_deref()) {
            let range = format!(
                "{}  →  {}",
                format_profile_extreme(min, &profile.data_type),
                format_profile_extreme(max, &profile.data_type)
            );
            widths[4] = widths[4].max(profile_text_width(ui, RichText::new(range).small().monospace()));
        }
        if numeric_summary {
            if let (Some(avg), Some(sum)) = (profile.avg, profile.sum) {
                widths[5] = widths[5].max(profile_text_width(
                    ui,
                    RichText::new(format!("{avg:.2} / {sum:.1}")).monospace(),
                ));
            }
        }
    }
    headings
        .into_iter()
        .zip(widths)
        .map(|(title, width)| TableColumn::new(title).content_width(width))
        .collect()
}

fn draw_profile_grid(theme: DbProTheme, ui: &mut egui::Ui, profiles: &[ColumnProfile], row_count: usize) {
    let show_numeric_summary = profiles
        .iter()
        .any(|profile| profile.avg.is_some() || profile.sum.is_some());
    let columns = profile_columns(ui, theme, profiles, row_count, show_numeric_summary);
    let profile_scroll = egui::ScrollArea::horizontal()
        .id_salt("column-profile-grid-scroll")
        .auto_shrink([false, true])
        .show(ui, |ui| {
            // Row names come from the profile's real column names — AT users
            // hear the column the row describes, not "row 3".
            Table::new(&columns, theme)
                .row_height(48.0)
                .row_label(&|i| profiles[i].name.clone())
                .show(
                ui,
                profiles.len(),
                |_| false,
                |_| {},
                |_| {},
                |_| {},
                |ui, row_idx, col_idx| {
                    let profile = &profiles[row_idx];
                    match col_idx {
                        0 => {
                            ui.vertical(|ui| {
                                let column_name = crate::components::truncate_ellipsis(&profile.name, 28);
                                ui.add(
                                    egui::Label::new(RichText::new(column_name).strong().color(theme.text_primary))
                                        .truncate(),
                                )
                                .on_hover_text(&profile.name);
                                ui.add_space(SPACE_XS);
                                let data_type = crate::components::truncate_ellipsis(&profile.data_type, 22);
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(data_type).small().monospace().color(theme.text_muted),
                                    )
                                    .truncate(),
                                )
                                .on_hover_text(&profile.data_type);
                            });
                        }
                        1 => {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = SPACE_SM;
                                let bar_w = 60.0;
                                let bar_h = 6.0;
                                let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_w, bar_h), egui::Sense::hover());
                                let fill_pct = (1.0 - profile.null_rate).clamp(0.0, 1.0) as f32;

                                ui.painter()
                                    .rect_filled(rect, CornerRadius::same(2.0 as u8), theme.surface_active);
                                if fill_pct > 0.0 {
                                    let fill_rect = Rect::from_min_size(rect.min, Vec2::new(bar_w * fill_pct, bar_h));
                                    ui.painter().rect_filled(fill_rect, CornerRadius::same(2.0 as u8), theme.accent);
                                }

                                ui.add(
                                    egui::Label::new(
                                        RichText::new(format!("{:.0}% full", (1.0 - profile.null_rate) * 100.0))
                                            .small()
                                            .color(theme.text_secondary),
                                    )
                                    .truncate()
                                    .sense(egui::Sense::hover()),
                                )
                                .on_hover_text(format!(
                                    "{}% full · {} null of {} rows",
                                    (1.0 - profile.null_rate) * 100.0,
                                    profile.null_count,
                                    row_count
                                ));
                            });
                        }
                        2 => {
                            ui.label(
                                RichText::new(format!(
                                    "{} · {:.0}%",
                                    profile.distinct_count,
                                    profile.distinct_rate * 100.0
                                ))
                                .monospace()
                                .color(theme.text_secondary),
                            );
                        }
                        3 => {
                            let non_null_count = row_count.saturating_sub(profile.null_count);
                            draw_profile_pattern(theme, ui, profile, non_null_count);
                        }
                        4 => {
                            draw_profile_range(theme, ui, profile);
                        }
                        5 => {
                            if let (Some(avg), Some(sum)) = (profile.avg, profile.sum) {
                                let summary = format!("{avg:.2} / {sum:.1}");
                                ui.add(
                                    egui::Label::new(RichText::new(summary).monospace().color(theme.text_secondary))
                                        .extend(),
                                )
                                .on_hover_text(format!("Average: {avg:.2}\nSum: {sum:.1}"));
                            } else {
                                ui.label(RichText::new("—").small().color(theme.text_muted));
                            }
                        }
                        _ => {}
                    }
                },
            );
        });
    crate::components::name_scroll_bars(ui.ctx(), profile_scroll.id, "Column profile grid");
}

fn draw_profile_range(theme: DbProTheme, ui: &mut egui::Ui, profile: &ColumnProfile) {
    match (profile.min.as_deref(), profile.max.as_deref()) {
        (Some(min), Some(max)) => {
            let min = format_profile_extreme(min, &profile.data_type);
            let max = format_profile_extreme(max, &profile.data_type);
            let range = format!("{min}  →  {max}");
            ui.add(egui::Label::new(RichText::new(range).small().monospace().color(theme.text_primary)).extend())
                .on_hover_text(format!("Min: {min}\nMax: {max}"));
        }
        _ => {
            ui.label(RichText::new("—").small().color(theme.text_muted));
        }
    }
}

fn draw_profile_pattern(theme: DbProTheme, ui: &mut egui::Ui, profile: &ColumnProfile, non_null_count: usize) {
    let (label, fill, foreground) = profile_pattern(theme, profile, non_null_count);
    badge(ui, label, fill, foreground);
}

fn profile_pattern(
    theme: DbProTheme,
    profile: &ColumnProfile,
    non_null_count: usize,
) -> (&'static str, egui::Color32, egui::Color32) {
    let (label, fill, foreground) = if non_null_count == 0 {
        ("All null", theme.surface_active, theme.text_muted)
    } else if non_null_count == 1 {
        ("Single value", theme.surface_active, theme.text_secondary)
    } else if profile.null_count == 0 && profile.distinct_count == non_null_count {
        ("Unique", theme.success_soft(), theme.success)
    } else if profile.null_count == 0 && profile.distinct_count == 1 {
        ("Constant", theme.surface_active, theme.text_muted)
    } else if profile.distinct_count < non_null_count {
        ("Repeated", theme.accent_soft, theme.accent)
    } else {
        ("Varied", theme.surface_active, theme.text_secondary)
    };
    (label, fill, foreground)
}

fn format_profile_extreme(value: &str, data_type: &str) -> String {
    let data_type = data_type.to_ascii_lowercase();
    let is_date = data_type == "date" || data_type.starts_with("date ");
    let is_timestamp =
        data_type.contains("timestamp") || data_type.contains("datetime") || data_type.starts_with("time ");
    if !is_date && !is_timestamp {
        return value.to_owned();
    }

    if let Ok(datetime) = DateTime::parse_from_rfc3339(value) {
        return datetime.format("%Y-%m-%d %H:%M:%S%.f %:z").to_string();
    }
    for format in ["%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S%.f"] {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(value, format) {
            return datetime.format("%Y-%m-%d %H:%M:%S%.f").to_string();
        }
    }
    if is_date {
        if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
            return date.format("%Y-%m-%d").to_string();
        }
    }
    value.to_owned()
}

fn cell_profile_text(cell: &UiCell) -> String {
    match cell {
        UiCell::Null => String::new(),
        UiCell::Boolean(value) => value.to_string(),
        UiCell::Number(value) | UiCell::Text(value) | UiCell::Json(value) | UiCell::Bytes(value) => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UiColumn;

    #[test]
    fn long_numeric_summary_is_rendered_without_ellipsis_or_cell_clipping() {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let result = UiQueryResult {
            columns: vec![UiColumn {
                name: "id".into(),
                data_type: "BIGINT".into(),
                nullable: false,
            }],
            rows: vec![vec![UiCell::Number("1000000000000000000".into())]; 3],
            duration_ms: 0,
            row_count: 3,
        };
        let profiles = profile_result_columns(&result);
        let expected = format!("{:.2} / {:.1}", profiles[0].avg.unwrap(), profiles[0].sum.unwrap());
        let output = crate::test_frame::frame(&ctx, egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(2000.0, 500.0))),
            ..Default::default()
        }, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| draw_profile_grid(DbProTheme::light(), ui, &profiles, 3));
        });
        let summary = output
            .shapes
            .iter()
            .find_map(|clipped| {
                if let egui::Shape::Text(text) = &clipped.shape {
                    if text.galley.job.text == expected {
                        return Some((clipped.clip_rect, text));
                    }
                }
                None
            })
            .expect("full numeric summary must be rendered, without ellipsis");
        assert!(
            summary
                .0
                .contains_rect(egui::Rect::from_min_size(summary.1.pos, summary.1.galley.size())),
            "summary must fit its cell clip"
        );
    }

    #[test]
    fn test_profile_result_numeric_and_uniqueness() {
        let result = UiQueryResult {
            columns: vec![
                UiColumn {
                    name: "id".to_owned(),
                    data_type: "integer".to_owned(),
                    nullable: false,
                },
                UiColumn {
                    name: "amount".to_owned(),
                    data_type: "numeric".to_owned(),
                    nullable: true,
                },
            ],
            rows: vec![
                vec![UiCell::Number("1".to_owned()), UiCell::Number("10.5".to_owned())],
                vec![UiCell::Number("2".to_owned()), UiCell::Number("20.5".to_owned())],
                vec![UiCell::Number("3".to_owned()), UiCell::Null],
            ],
            duration_ms: 5,
            row_count: 3,
        };

        let profiles = profile_result_columns(&result);
        assert_eq!(profiles.len(), 2);

        // id column
        let id_prof = &profiles[0];
        assert_eq!(id_prof.distinct_count, 3);
        assert_eq!(id_prof.null_count, 0);
        assert_eq!(id_prof.sum, Some(6.0));
        assert_eq!(id_prof.avg, Some(2.0));

        // amount column
        let amount_prof = &profiles[1];
        assert_eq!(amount_prof.distinct_count, 2);
        assert_eq!(amount_prof.null_count, 1);
        assert_eq!(amount_prof.sum, Some(31.0));
        assert_eq!(amount_prof.avg, Some(15.5));
    }

    #[test]
    fn all_null_column_has_a_distinct_pattern_label() {
        let result = UiQueryResult {
            columns: vec![UiColumn {
                name: "optional_value".to_owned(),
                data_type: "text".to_owned(),
                nullable: true,
            }],
            rows: vec![vec![UiCell::Null], vec![UiCell::Null]],
            duration_ms: 0,
            row_count: 2,
        };
        let profile = &profile_result_columns(&result)[0];

        assert_eq!(profile.null_count, 2);
        assert_eq!(profile.distinct_count, 0);
        assert_eq!(profile_pattern(DbProTheme::dark(), profile, 0).0, "All null");
    }
}
