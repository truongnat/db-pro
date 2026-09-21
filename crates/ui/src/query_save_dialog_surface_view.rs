use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SaveAsDialogAction {
    Save,
    Cancel,
}

pub(super) struct SaveAsDialogContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) name: &'a mut String,
}

impl SaveAsDialogContext<'_> {
    pub(super) fn draw(&mut self, ctx: &egui::Context) -> Option<SaveAsDialogAction> {
        let mut action = None;
        egui::Window::new("Save Query As")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Name");
                ui.text_edit_singleline(self.name);
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .icon(Icon::Save)
                        .text("Save")
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(SaveAsDialogAction::Save);
                    }
                    if Button::new(self.theme)
                        .icon(Icon::X)
                        .text("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(SaveAsDialogAction::Cancel);
                    }
                });
            });
        action
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DirtyCloseDialogAction {
    Save,
    Discard,
    Cancel,
}

pub(super) struct DirtyCloseDialogContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) title: &'a str,
}

impl DirtyCloseDialogContext<'_> {
    pub(super) fn draw(&self, ctx: &egui::Context) -> Option<DirtyCloseDialogAction> {
        let mut action = None;
        egui::Window::new("Unsaved query")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("Save changes to {} before closing?", self.title));
                ui.horizontal(|ui| {
                    if Button::new(self.theme)
                        .icon(Icon::Save)
                        .text("Save")
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(DirtyCloseDialogAction::Save);
                    }
                    if Button::new(self.theme)
                        .icon(Icon::Trash2)
                        .text("Don't Save")
                        .variant(ButtonVariant::Destructive)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(DirtyCloseDialogAction::Discard);
                    }
                    if Button::new(self.theme)
                        .icon(Icon::X)
                        .text("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(DirtyCloseDialogAction::Cancel);
                    }
                });
            });
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_dialog_actions_are_distinct() {
        assert_ne!(DirtyCloseDialogAction::Save, DirtyCloseDialogAction::Discard);
        assert_ne!(SaveAsDialogAction::Save, SaveAsDialogAction::Cancel);
    }
}
