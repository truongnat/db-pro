//! Settings navigation presentation and section-selection intents.

use super::{font_caption, Button, ButtonSize, ButtonVariant, DbProTheme, SettingsSection, SPACE_LG, SPACE_MD, SPACE_XS};
use eframe::egui::{self, RichText};
use lucide_icons::Icon;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SettingsNavigationAction {
    SelectSection(SettingsSection),
}

pub(crate) struct SettingsNavigationContext {
    pub(crate) theme: DbProTheme,
    pub(crate) selected: SettingsSection,
}

impl SettingsNavigationContext {
    pub(crate) fn draw(&self, ui: &mut egui::Ui) -> Vec<SettingsNavigationAction> {
        let mut actions = Vec::new();
        ui.set_width(ui.available_width());
        ui.label(
            RichText::new(t!("app.settings"))
                .font(font_caption())
                .strong()
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_LG);
        self.draw_grouped_section(
            ui,
            t!("settings.group_workspace").as_ref(),
            &[
                (SettingsSection::General, Icon::Settings2),
                (SettingsSection::Appearance, Icon::Palette),
                (SettingsSection::Editor, Icon::Code2),
                (SettingsSection::DataGrid, Icon::Table2),
            ],
            &mut actions,
        );
        self.draw_grouped_section(
            ui,
            t!("settings.group_connections").as_ref(),
            &[(SettingsSection::Connections, Icon::Database), (SettingsSection::Ai, Icon::Bot)],
            &mut actions,
        );
        self.draw_grouped_section(
            ui,
            t!("settings.group_system").as_ref(),
            &[
                (SettingsSection::Keybindings, Icon::Keyboard),
                (SettingsSection::Backup, Icon::Archive),
                (SettingsSection::Security, Icon::ShieldCheck),
                (SettingsSection::Advanced, Icon::SlidersHorizontal),
            ],
            &mut actions,
        );
        actions
    }

    fn draw_grouped_section(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        sections: &[(SettingsSection, Icon)],
        actions: &mut Vec<SettingsNavigationAction>,
    ) {
        ui.label(
            RichText::new(title)
                .font(font_caption())
                .strong()
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_XS);
        self.draw_group(ui, sections, actions);
        ui.add_space(SPACE_MD);
    }

    fn draw_group(
        &self,
        ui: &mut egui::Ui,
        sections: &[(SettingsSection, Icon)],
        actions: &mut Vec<SettingsNavigationAction>,
    ) {
        for (section, icon) in sections {
            let selected = self.selected == *section;
            let response = Button::new(self.theme)
                .text(t!(section.label_key()).as_ref())
                .icon(*icon)
                .variant(if selected {
                    ButtonVariant::Secondary
                } else {
                    ButtonVariant::Ghost
                })
                .size(ButtonSize::Default)
                .full_width(true)
                .left_aligned()
                .show(ui);
            if response.clicked() {
                actions.push(SettingsNavigationAction::SelectSection(*section));
            }
            if selected {
                // The rail's fill equals the light-theme Secondary rest fill, so the
                // active section is marked by the shared accent pill, not the fill.
                crate::components::nav::paint_selection_pill(ui, response.rect, self.theme);
            }
            ui.add_space(SPACE_XS);
        }
    }
}
