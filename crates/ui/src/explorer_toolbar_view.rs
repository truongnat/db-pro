//! Schema explorer toolbar and empty state rendering.
use super::*;
use egui::{Align, FontFamily, Layout, RichText};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExplorerToolbarAction {
    RefreshSchema,
    NewConnection,
}

pub(super) struct ExplorerToolbarContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) search: &'a mut String,
    pub(super) filter: &'a mut ExplorerObjectFilter,
}

impl ExplorerToolbarContext<'_> {
    pub(super) fn draw_toolbar(&mut self, ui: &mut egui::Ui) -> Vec<ExplorerToolbarAction> {
        let mut actions = Vec::new();
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(ui.available_size(), Layout::right_to_left(Align::Center), |ui| {
                let mut refresh_schema = false;
                let refresh_button = Button::new(self.theme)
                    .icon(Icon::RotateCcw)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip(t!("explorer.refresh_schema"))
                    .access_label(t!("explorer.refresh_schema"))
                    .show(ui);
                refresh_button.context_menu(|ui| {
                    if ctx_menu_item(
                        ui,
                        Some(Icon::RotateCcw),
                        &t!("explorer.refresh_schema"),
                        Some("F5"),
                        self.theme.text_primary,
                        self.theme,
                    )
                    .clicked()
                    {
                        refresh_schema = true;
                        ui.close();
                    }
                });
                if refresh_button.clicked() || refresh_schema {
                    actions.push(ExplorerToolbarAction::RefreshSchema);
                }
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    SearchInput::new(self.search, &t!("explorer.filter_objects"), self.theme).show(ui);
                    self.draw_filter_workbench(ui);
                });
            });
        });
        actions
    }

    /// Object-filter workbench (spec 11): a popup next to the search box with
    /// the name-match mode and per-kind visibility toggles.
    // cc-scan:allow LONG_FUNCTION — linear pipeline — one cohesive pass
    fn draw_filter_workbench(&mut self, ui: &mut egui::Ui) {
        let popup_id = ui.make_persistent_id("explorer_object_filter_workbench");
        let filter_active = !self.filter.is_default();
        let button = Button::new(self.theme)
            .icon(Icon::ListFilter)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .tooltip(t!("explorer.filter_options"))
            .access_label(t!("explorer.filter_options"))
            .show(ui);
        if filter_active {
            ui.painter().circle_filled(
                egui::pos2(button.rect.right() - 5.0, button.rect.top() + 5.0),
                2.5,
                self.theme.accent,
            );
        }
        if button.clicked() {
            egui::Popup::toggle_id(ui.ctx(), popup_id);
        }
        let filter = &mut *self.filter;
        egui::Popup::new(popup_id, ui.ctx().clone(), &button, ui.layer_id())
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .open_memory(None)
            .show(|ui| {
                ui.set_min_width(180.0);
                ui.set_max_width(220.0);
                ui.label(
                    RichText::new("Name match")
                        .font(font_caption())
                        .strong()
                        .color(self.theme.text_secondary),
                );
                for mode in ExplorerMatchMode::ALL {
                    ui.radio_value(&mut filter.mode, mode, mode.label());
                }
                ui.add_space(SPACE_XS);
                ui.separator();
                ui.label(
                    RichText::new("Object kinds")
                        .font(font_caption())
                        .strong()
                        .color(self.theme.text_secondary),
                );
                ui.checkbox(&mut filter.tables, "Tables");
                ui.checkbox(&mut filter.views, "Views");
                ui.checkbox(&mut filter.functions, "Functions");
                ui.checkbox(&mut filter.triggers, "Triggers");
                ui.add_space(SPACE_XXS);
                if ui
                    .link(RichText::new("Reset").font(font_caption()).color(self.theme.accent))
                    .clicked()
                {
                    *filter = ExplorerObjectFilter::default();
                }
            },
        );
    }

    pub(super) fn draw_empty_state(&self, ui: &mut egui::Ui) -> Vec<ExplorerToolbarAction> {
        let mut actions = Vec::new();
        ui.add_space(36.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(char::from(Icon::Database).to_string())
                    .family(FontFamily::Name("lucide".into()))
                    .size(28.0)
                    .color(self.theme.text_muted),
            );
            ui.add_space(8.0);
            ui.label(RichText::new(t!("explorer.no_connections")).strong().color(self.theme.text_primary));
            ui.add_space(3.0);
            ui.label(
                RichText::new(t!("explorer.no_connections_desc"))
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.add_space(12.0);
            if Button::new(self.theme)
                .icon(Icon::Plus)
                .text(t!("explorer.new_connection"))
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                actions.push(ExplorerToolbarAction::NewConnection);
            }
        });
        actions
    }
}
