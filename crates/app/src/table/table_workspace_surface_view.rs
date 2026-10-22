// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TableWorkspaceSurfaceAction {
    AskAgent,
    NewQuery,
    Refresh,
    SelectView(TableView),
}

pub(super) struct TableWorkspaceSurfaceContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) connection_name: &'a str,
    pub(super) schema: &'a str,
    pub(super) table_name: &'a str,
    pub(super) active_view: TableView,
    pub(super) row_count: Option<u64>,
}

impl TableWorkspaceSurfaceContext<'_> {
    pub(super) fn draw(&self, ui: &mut egui::Ui) -> Vec<TableWorkspaceSurfaceAction> {
        let mut actions = self.draw_header(ui);
        ui.add_space(6.0);
        actions.extend(self.draw_view_tabs(ui));
        actions
    }

    fn draw_header(&self, ui: &mut egui::Ui) -> Vec<TableWorkspaceSurfaceAction> {
        let mut actions = Vec::new();
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                self.draw_breadcrumb(ui);
                if let Some(rows) = self.row_count {
                    Badge::new(format!("{rows} rows"), self.theme)
                        .variant(BadgeVariant::Secondary)
                        .compact(true)
                        .show(ui);
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    self.draw_header_actions(ui, &mut actions);
                });
            });
        });
        actions
    }

    fn draw_header_actions(&self, ui: &mut egui::Ui, actions: &mut Vec<TableWorkspaceSurfaceAction>) {
        let controls = [
            (
                Icon::Bot,
                "Ask Agent",
                "Open AI Assistant with table context",
                TableWorkspaceSurfaceAction::AskAgent,
            ),
            (
                Icon::FileCode2,
                "New Query",
                "Open SQL Editor for this table",
                TableWorkspaceSurfaceAction::NewQuery,
            ),
            (
                Icon::RotateCcw,
                "Refresh",
                "Reload table metadata and rows",
                TableWorkspaceSurfaceAction::Refresh,
            ),
        ];
        for (icon, label, tooltip, action) in controls {
            if Button::new(self.theme)
                .icon(icon)
                .text(label)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .tooltip(tooltip)
                .show(ui)
                .clicked()
            {
                actions.push(action);
            }
        }
    }

    fn draw_breadcrumb(&self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(char::from(Icon::Table2).to_string())
                .font(egui::FontId::new(14.0, egui::FontFamily::Name("lucide".into())))
                .color(self.theme.accent),
        );
        ui.label(
            RichText::new(self.connection_name)
                .font(font_caption())
                .color(self.theme.text_muted),
        );
        ui.label(
            RichText::new(char::from(Icon::ChevronRight).to_string())
                .font(egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())))
                .color(self.theme.text_muted),
        );
        ui.label(
            RichText::new(self.schema)
                .font(font_caption())
                .color(self.theme.text_secondary),
        );
        ui.label(
            RichText::new(char::from(Icon::ChevronRight).to_string())
                .font(egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())))
                .color(self.theme.text_muted),
        );
        ui.label(
            RichText::new(self.table_name)
                .font(font_subheading())
                .strong()
                .color(self.theme.text_primary),
        );
    }

    fn draw_view_tabs(&self, ui: &mut egui::Ui) -> Vec<TableWorkspaceSurfaceAction> {
        let mut actions = Vec::new();
        toolbar_frame(self.theme).show(ui, |ui| {
            ScrollArea::new(self.theme)
                .horizontal(true)
                .vertical(false)
                .id_salt("table-workspace-tabs")
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = crate::tokens::SPACE_XS;
                        for (view, icon, label) in table_view_tabs() {
                            let selected = self.active_view == view;
                            if tab_button(ui, self.theme, Some(icon), label, selected).clicked() && !selected {
                                actions.push(TableWorkspaceSurfaceAction::SelectView(view));
                            }
                        }
                    });
                });
        });
        actions
    }
}

fn table_view_tabs() -> [(TableView, Icon, &'static str); 8] {
    [
        (TableView::Data, Icon::Table2, "Data"),
        (TableView::Structure, Icon::Columns3, "Structure"),
        (TableView::Profile, Icon::ChartColumn, "Profile"),
        (TableView::Indexes, Icon::List, "Indexes"),
        (TableView::Relations, Icon::ArrowRightLeft, "Foreign Keys"),
        (TableView::Constraints, Icon::ShieldCheck, "Constraints"),
        (TableView::Dependencies, Icon::GitBranch, "Dependencies"),
        (TableView::Ddl, Icon::Code2, "DDL"),
    ]
}

pub(super) fn draw_metadata_filter_header(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    title: &str,
    search: &mut String,
    hint: &str,
    total: &str,
) {
    draw_metadata_filter_header_with_trailing(ui, theme, title, search, hint, |ui, _| {
        ui.label(RichText::new(total).font(font_ui_label()).color(theme.text_muted));
    });
}

/// Trailing controls (category tabs, counts) sit in a right-to-left strip when
/// the panel is wide enough. Below `narrow` they move to their own row so they
/// can never slide under the search input (#367).
pub(super) fn draw_metadata_filter_header_with_trailing(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    title: &str,
    search: &mut String,
    hint: &str,
    trailing: impl FnOnce(&mut egui::Ui, bool),
) {
    const TRAILING_MIN_WIDTH: f32 = 260.0;
    let trailing_width = ui.available_width() - 220.0 - SPACE_SM;
    if trailing_width < TRAILING_MIN_WIDTH {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_SM;
            ui.label(RichText::new(title).font(font_ui_label()).color(theme.text_secondary));
            crate::components::input::SearchInput::new(search, hint, theme)
                .width(220.0)
                .show(ui);
        });
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_SM;
            trailing(ui, false);
        });
        return;
    }
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_SM;
        ui.label(RichText::new(title).font(font_ui_label()).color(theme.text_secondary));
        crate::components::input::SearchInput::new(search, hint, theme)
            .width(220.0)
            .show(ui);
        ui.allocate_ui_with_layout(
            ui.available_size(),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                trailing(ui, true);
            },
        );
    });
}

#[cfg(test)]
#[path = "tests/table_workspace_surface_view_metadata_filter_header_tests.rs"]
mod metadata_filter_header_tests;
