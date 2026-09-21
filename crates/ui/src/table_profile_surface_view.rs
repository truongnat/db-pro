//! Table-profile presentation and bounded page-level profiling.
use super::super::*;

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
    ui.label(
        RichText::new(format!(
            "Profiling {} loaded row(s) · not a full-table scan",
            result.rows.len()
        ))
        .small()
        .color(theme.text_muted),
    );
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
            for row in &result.rows {
                match row.get(col_idx) {
                    None | Some(UiCell::Null) => null_count += 1,
                    Some(cell) => values.push(cell_profile_text(cell)),
                }
            }
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
            }
        })
        .collect()
}

fn draw_profile_grid(theme: DbProTheme, ui: &mut egui::Ui, profiles: &[ColumnProfile]) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("column_profile_grid")
            .num_columns(6)
            .striped(true)
            .show(ui, |ui| {
                for heading in ["Column", "Type", "Nulls", "Distinct", "Min", "Max"] {
                    ui.label(RichText::new(heading).strong().color(theme.text_primary));
                }
                ui.end_row();
                for profile in profiles {
                    ui.label(&profile.name);
                    ui.label(&profile.data_type);
                    ui.label(format!("{} ({:.0}%)", profile.null_count, profile.null_rate * 100.0));
                    ui.label(format!(
                        "{} ({:.0}%)",
                        profile.distinct_count,
                        profile.distinct_rate * 100.0
                    ));
                    ui.label(profile.min.as_deref().unwrap_or("—"));
                    ui.label(profile.max.as_deref().unwrap_or("—"));
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
