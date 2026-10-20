use super::*;

pub(super) fn draw_diagram_sidebar(ui: &mut egui::Ui, theme: DbProTheme, schema: &UiSchemaSummary) -> bool {
    draw_schema_map_section(ui, theme, schema);
    ui.add_space(SPACE_MD);
    Separator::horizontal(theme).show(ui);
    ui.add_space(SPACE_MD);
    ui.vertical(|ui| {
        section_label(ui, t!("diagram.section_navigation").as_ref(), theme);
        ui.add_space(SPACE_SM);
        ui.label(RichText::new(t!("diagram.pan_hint")).small().color(theme.text_secondary));
        ui.add_space(SPACE_SM);
        let back = secondary_button_with_icon(ui, Icon::Database, t!("diagram.back_to_explorer").as_ref(), theme);
        back.clicked()
    })
    .inner
}

fn draw_schema_map_section(ui: &mut egui::Ui, theme: DbProTheme, schema: &UiSchemaSummary) {
    let tables = schema.table_details.len();
    let relationships = schema
        .table_details
        .iter()
        .map(|table| table.foreign_keys.len())
        .sum::<usize>();
    ui.vertical(|ui| {
        section_label(ui, t!("diagram.section_schema_map").as_ref(), theme);
        ui.add_space(SPACE_SM);
        ui.label(RichText::new(t!("diagram.schema_map_summary")).color(theme.text_primary));
        ui.add_space(SPACE_XS);
        ui.label(
            RichText::new(format!(
                "{} · {}",
                pluralize(tables, "diagram.count_tables"),
                pluralize(relationships, "diagram.count_relationships"),
            ))
            .small()
            .color(theme.text_muted),
        );
    });
}

/// Locale-aware count label: `{key}_one` for n=1, `{key}_other` otherwise.
/// Non-plural locales (vi/ja/ko/zh) define both keys to the same noun, so the
/// split is invisible there and only English gets real singular/plural forms.
fn pluralize(n: usize, key: &'static str) -> String {
    let key = if n == 1 {
        format!("{key}_one")
    } else {
        format!("{key}_other")
    };
    rust_i18n::t!(&key, n = n).to_string()
}
