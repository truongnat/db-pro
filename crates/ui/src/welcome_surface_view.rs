//! A quiet introduction; workspace actions live in the shell.
use super::*;
use lucide_icons::Icon;

pub(super) fn draw(theme: DbProTheme, ui: &mut egui::Ui) {
    ui.add_space(((ui.available_height() - 200.0) * 0.5).max(SPACE_XL));
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(char::from(Icon::Database).to_string())
                .font(font_icon(ICON_XL))
                .color(theme.accent),
        );
        ui.add_space(SPACE_LG);
        ui.label(RichText::new("DB Pro").font(font_display()).color(theme.text_primary));
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new("Explore databases, write SQL, and inspect results.")
                .font(font_body())
                .color(theme.text_secondary),
        );
        ui.add_space(SPACE_LG);
        ui.label(
            RichText::new("PostgreSQL  ·  SQLite")
                .font(font_caption())
                .color(theme.text_muted),
        );
    });
}
