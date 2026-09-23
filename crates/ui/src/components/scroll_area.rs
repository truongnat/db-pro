use crate::DbProTheme;
use egui::Ui;

pub struct ScrollArea {
    horizontal: bool,
    vertical: bool,
    auto_shrink: [bool; 2],
    max_height: Option<f32>,
    #[allow(dead_code)]
    theme: DbProTheme,
}

impl ScrollArea {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            horizontal: false,
            vertical: true,
            auto_shrink: [false, false],
            max_height: None,
            theme,
        }
    }

    pub fn horizontal(mut self, enabled: bool) -> Self {
        self.horizontal = enabled;
        self
    }

    pub fn vertical(mut self, enabled: bool) -> Self {
        self.vertical = enabled;
        self
    }

    pub fn both(mut self) -> Self {
        self.horizontal = true;
        self.vertical = true;
        self
    }

    pub fn auto_shrink(mut self, shrink: [bool; 2]) -> Self {
        self.auto_shrink = shrink;
        self
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        let mut area = egui::ScrollArea::new([self.horizontal, self.vertical])
            .auto_shrink(self.auto_shrink)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded);

        if let Some(mh) = self.max_height {
            area = area.max_height(mh);
        }

        // Apply sleek shadcn scrollbar aesthetics
        let prev_style = ui.style().visuals.clone();
        let mut new_style = prev_style.clone();
        new_style.clip_rect_margin = 0.0;
        ui.style_mut().visuals = new_style;

        let output = area.show(ui, add_contents);

        ui.style_mut().visuals = prev_style;
        output.inner
    }
}
