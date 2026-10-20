// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dialog::Dialog;
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SaveAsDialogAction {
    Save,
    Cancel,
}

pub(super) struct SaveAsDialogContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) name: &'a mut String,
    pub(super) show_name_error: &'a mut bool,
}

impl SaveAsDialogContext<'_> {
    pub(super) fn draw(&mut self, ctx: &egui::Context) -> Option<SaveAsDialogAction> {
        let mut action = None;
        let mut open = true;
        Dialog::new(&mut open, "Save Query As", self.theme)
            .width(420.0)
            .id_salt("save_query_as_dialog")
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    let mut name_input = Input::new(self.name, "Query name", self.theme)
                        .label("Name")
                        .required(true);
                    if *self.show_name_error {
                        name_input = name_input.error_text("Enter a name for the saved query");
                    }
                    if name_input.show(ui).changed() && !self.name.trim().is_empty() {
                        *self.show_name_error = false;
                    }
                });
                frame.footer(|ui| {
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
        if !open && action.is_none() {
            action = Some(SaveAsDialogAction::Cancel);
        }
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
        let mut open = true;
        Dialog::new(&mut open, t!("query.unsaved_query_title").as_ref(), self.theme)
            .width(460.0)
            .id_salt("dirty_query_close_dialog")
            .show_framed_ctx(ctx, |frame| {
                frame.body(|ui| {
                    ui.label(t!("query.unsaved_query_prompt", title = self.title).into_owned());
                });
                frame.footer(|ui| {
                    if Button::new(self.theme)
                        .icon(Icon::Save)
                        .text(t!("common.save"))
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(DirtyCloseDialogAction::Save);
                    }
                    if Button::new(self.theme)
                        .icon(Icon::Trash2)
                        .text(t!("query.dont_save"))
                        .variant(ButtonVariant::Destructive)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(DirtyCloseDialogAction::Discard);
                    }
                    if Button::new(self.theme)
                        .icon(Icon::X)
                        .text(t!("common.cancel"))
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(DirtyCloseDialogAction::Cancel);
                    }
                });
            });
        if !open && action.is_none() {
            action = Some(DirtyCloseDialogAction::Cancel);
        }
        action
    }
}

#[cfg(test)]
#[path = "tests/query_save_dialog_surface_view_tests.rs"]
mod tests;

