//! Settings navigation presentation and section-selection intents.

use super::{font_body_sm, font_caption, font_icon, DbProTheme, SettingsSection, SPACE_LG, SPACE_MD, SPACE_SM, SPACE_XS};
use eframe::egui::{self, Align2, Color32, RichText, Sense};
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
            RichText::new("SETTINGS")
                .font(font_caption())
                .strong()
                .color(self.theme.text_muted),
        );
        ui.add_space(SPACE_LG);
        self.draw_grouped_section(
            ui,
            "Workspace",
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
            "Connections",
            &[(SettingsSection::Connections, Icon::Database), (SettingsSection::Ai, Icon::Bot)],
            &mut actions,
        );
        self.draw_grouped_section(
            ui,
            "System",
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
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), 34.0),
                Sense::click(),
            );
            let fill = if selected {
                self.theme.surface_active
            } else if response.hovered() {
                self.theme.surface_hover
            } else {
                Color32::TRANSPARENT
            };
            ui.painter().rect_filled(rect, egui::Rounding::same(6.0), fill);
            let color = if selected || response.hovered() {
                self.theme.text_primary
            } else {
                self.theme.text_secondary
            };
            ui.painter().text(
                egui::pos2(rect.left() + SPACE_SM, rect.center().y),
                Align2::LEFT_CENTER,
                char::from(*icon).to_string(),
                font_icon(16.0),
                color,
            );
            ui.painter().text(
                egui::pos2(rect.left() + SPACE_SM + 24.0, rect.center().y),
                Align2::LEFT_CENTER,
                section.label(),
                font_body_sm(),
                color,
            );
            if response.clicked() {
                actions.push(SettingsNavigationAction::SelectSection(*section));
            }
        }
    }
}
