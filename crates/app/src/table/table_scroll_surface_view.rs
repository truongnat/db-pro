//! Shared scroll-area geometry for table metadata panes.
use super::*;

pub(super) fn draw<F>(ui: &mut egui::Ui, id: &'static str, theme: DbProTheme, draw_contents: F)
where
    F: FnOnce(&mut egui::Ui),
{
    ScrollArea::new(theme)
        .id_salt(id)
        .auto_shrink([false, false])
        .show(ui, draw_contents);
}
