//! Table-profile presentation and bounded page-level profiling.
use super::super::*;
use egui::{Pos2, Rect, Rounding, Vec2};

pub(super) fn draw_profile_pane(theme: DbProTheme, result: Option<&UiQueryResult>, ui: &mut egui::Ui) {
    let Some(result) = result else {
        empty_state(
            ui,
            Icon::ChartColumn,
            "No rows loaded",
            "Open the Data tab or wait for the current page to load, then return to Profile.",
            theme,
        );
        return;
    };
    if result.columns.is_empty() {
        empty_state(
            ui,
            Icon::ChartColumn,
            "No columns",
            "This result has no columns to profile.",
            theme,
        );
        return;
    }

    let profiles = profile_result_columns(result);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "Profiling {} loaded row(s) · {} column(s)",
                result.rows.len(),
                result.columns.len()
            ))
            .font(font_subheading())
            .strong()
            .color(theme.text_primary),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new("Page sample analysis")
                    .small()
                    .color(theme.text_muted),
            );
        });
    });
    ui.add_space(8.0);
    draw_profile_grid(theme, ui, &profiles);
}

pub(super) fn profile_result_columns(result: &UiQueryResult) -> Vec<ColumnProfile> {
    let row_count = result.rows.len().max(1) as f64;
    result
        .columns
        .iter()
        .enumerate()
        .map(|(col_idx, column)| {
            let mut null_count = 0usize;
            let mut values = Vec::new();
            let mut numeric_values = Vec::new();

            for row in &result.rows {
                match row.get(col_idx) {
                    None | Some(UiCell::Null) => null_count += 1,
                    Some(UiCell::Number(num_str)) => {
                        values.push(num_str.clone());
                        if let Ok(val) = num_str.parse::<f64>() {
                            numeric_values.push(val);
                        }
                    }
                    Some(cell) => values.push(cell_profile_text(cell)),
                }
            }

            let is_numeric = !numeric_values.is_empty() && numeric_values.len() + null_count == result.rows.len();
            let sum = if is_numeric && !numeric_values.is_empty() {
                Some(numeric_values.iter().sum())
            } else {
                None
            };
            let avg = if is_numeric && !numeric_values.is_empty() {
                Some(sum.unwrap_or(0.0) / numeric_values.len() as f64)
            } else {
                None
            };

            let distinct = values.iter().cloned().collect::<std::collections::BTreeSet<_>>();
            let min = values.iter().min().cloned();
            let max = values.iter().max().cloned();

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

fn draw_profile_grid(theme: DbProTheme, ui: &mut egui::Ui, profiles: &[ColumnProfile]) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("column_profile_grid")
            .num_columns(8)
            .striped(true)
            .spacing(Vec2::new(12.0, 8.0))
            .show(ui, |ui| {
                // Table header
                for heading in ["Column", "Type", "Fill / Null Ratio", "Distinct", "Quality", "Min", "Max", "Avg / Sum"] {
                    ui.label(RichText::new(heading).strong().color(theme.text_primary));
                }
                ui.end_row();

                for profile in profiles {
                    // 1. Column Name
                    ui.label(RichText::new(&profile.name).strong().color(theme.text_primary));

                    // 2. Type badge
                    ui.label(RichText::new(&profile.data_type).monospace().color(theme.text_secondary));

                    // 3. Fill / Null Ratio visual dual-color bar
                    ui.horizontal(|ui| {
                        let bar_w = 60.0;
                        let bar_h = 7.0;
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_w, bar_h), egui::Sense::hover());
                        let fill_pct = (1.0 - profile.null_rate).clamp(0.0, 1.0) as f32;
                        
                        // Background = Null (gray / muted)
                        ui.painter().rect_filled(rect, Rounding::same(2.0), theme.surface_active);
                        // Foreground = Populated (accent / green)
                        if fill_pct > 0.0 {
                            let fill_rect = Rect::from_min_size(rect.min, Vec2::new(bar_w * fill_pct, bar_h));
                            ui.painter().rect_filled(fill_rect, Rounding::same(2.0), theme.accent);
                        }

                        let null_label = if profile.null_count == 0 {
                            "100% full".to_owned()
                        } else {
                            format!("{:.0}% null", profile.null_rate * 100.0)
                        };
                        ui.label(RichText::new(null_label).small().color(theme.text_tertiary));
                    });

                    // 4. Distinct count & percentage
                    ui.label(format!(
                        "{} ({:.0}%)",
                        profile.distinct_count,
                        profile.distinct_rate * 100.0
                    ));

                    // 5. Uniqueness / Quality classification badge
                    ui.horizontal(|ui| {
                        if (profile.distinct_rate - 1.0).abs() < f64::EPSILON && profile.null_count == 0 {
                            let badge_g = ui.painter().layout_no_wrap("UNIQUE".to_owned(), font_caption(), theme.success);
                            let r = Rect::from_min_size(Pos2::new(ui.cursor().min.x, ui.cursor().min.y + 1.0), Vec2::new(badge_g.size().x + 6.0, 15.0));
                            ui.painter().rect_filled(r, Rounding::same(RADIUS_XS), theme.success_soft());
                            ui.painter().galley(Pos2::new(r.left() + 3.0, r.top() + 1.0), badge_g, egui::Color32::PLACEHOLDER);
                            ui.add_space(r.width() + 2.0);
                        } else if profile.distinct_count <= 1 && profile.null_count == 0 {
                            ui.label(RichText::new("Constant").small().color(theme.text_muted));
                        } else if profile.distinct_count <= 5 {
                            ui.label(RichText::new("Enum-like").small().color(theme.accent));
                        } else {
                            ui.label(RichText::new("Standard").small().color(theme.text_secondary));
                        }
                    });

                    // 6. Min
                    ui.label(profile.min.as_deref().unwrap_or("—"));

                    // 7. Max
                    ui.label(profile.max.as_deref().unwrap_or("—"));

                    // 8. Avg / Sum for numeric columns
                    if let (Some(avg), Some(sum)) = (profile.avg, profile.sum) {
                        ui.label(
                            RichText::new(format!("avg: {:.2} · sum: {:.1}", avg, sum))
                                .small()
                                .monospace()
                                .color(theme.text_secondary),
                        );
                    } else {
                        ui.label(RichText::new("—").small().color(theme.text_muted));
                    }

                    ui.end_row();
                }
            });
    });
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
}
