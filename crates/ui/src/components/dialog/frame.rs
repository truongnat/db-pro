use egui::Ui;

pub struct DialogFrame<'a> {
    pub ui: &'a mut Ui,
    pub max_content_height: f32,
    pub inner_width: f32,
}

impl<'a> DialogFrame<'a> {
    /// Renders the scrollable body of the dialog.
    pub fn body<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        let inner_w = self.inner_width;
        let max_h = self.max_content_height;
        egui::ScrollArea::vertical()
            .max_width(inner_w)
            .max_height(max_h)
            .auto_shrink([false, true])
            .show(self.ui, |body_ui| {
                body_ui.set_max_width(inner_w);
                add_contents(body_ui)
            })
            .inner
    }

    /// Renders a fixed sticky footer at the bottom of the card, outside the scroll area.
    pub fn footer<F>(&mut self, add_footer: impl FnOnce(&mut Ui) -> F) -> F {
        self.ui.add_space(8.0);
        self.ui.separator();
        self.ui.add_space(8.0);
        add_footer(self.ui)
    }
}
