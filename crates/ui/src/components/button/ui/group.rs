use crate::tokens::SPACE_XS;
use crate::DbProTheme;
use egui::{Ui, Vec2};

pub struct ButtonGroup {
    // Retained to preserve the existing constructor API.
    #[allow(dead_code)]
    theme: DbProTheme,
}

impl ButtonGroup {
    pub fn new(theme: DbProTheme) -> Self {
        Self { theme }
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        // Keep the compact gap scoped to this row; child buttons retain their
        // own theme, state resolution and returned responses.
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(SPACE_XS, 0.0);
            add_contents(ui)
        })
        .inner
    }
}
