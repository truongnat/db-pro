//! Left activity rail (48px): Explorer / Files / Query / … settings.
use super::*;

pub(super) enum ActivityBarAction {
    SelectActivity(Activity),
    OpenQuery,
    OpenDiagram,
    OpenSchemaCompare,
    OpenSchemaWorkbench,
    OpenSettings,
    ToggleAgent,
}

pub(super) struct ActivityBarContext {
    pub(super) theme: DbProTheme,
    pub(super) activity: Activity,
    pub(super) active_tab: WorkspaceTab,
    pub(super) agent_open: bool,
}

pub(super) fn draw_activity_bar(ctx: &egui::Context, context: &ActivityBarContext) -> Option<ActivityBarAction> {
    let mut action = None;
    egui::SidePanel::left("activity_bar")
        .resizable(false)
        .exact_width(48.0)
        .show_separator_line(false)
        .frame(egui::Frame {
            fill: context.theme.surface_panel,
            inner_margin: egui::Margin::ZERO,
            outer_margin: egui::Margin::ZERO,
            stroke: egui::Stroke::NONE,
            rounding: egui::Rounding::ZERO,
            shadow: egui::Shadow::NONE,
        })
        .show(ctx, |ui| {
            let full = ui.max_rect();
            ui.painter()
                .rect_filled(full, egui::Rounding::ZERO, context.theme.surface_panel);
            let line_x = ui.painter().round_to_pixel_center(full.right() - 1.0);
            ui.painter().vline(
                line_x,
                full.y_range(),
                egui::Stroke::new(1.0, context.theme.border_subtle),
            );

            ui.vertical_centered(|ui| {
                action = draw_activity_buttons(ui, context);
                ui.add_space((ui.available_height() - 44.0).max(0.0));
                if draw_settings_button(ui, context) {
                    action = Some(ActivityBarAction::OpenSettings);
                }
                ui.add_space(SPACE_SM);
            });

            ui.allocate_rect(full, egui::Sense::hover());
        });
    action
}

fn draw_activity_buttons(ui: &mut egui::Ui, context: &ActivityBarContext) -> Option<ActivityBarAction> {
    ui.add_space(SPACE_SM);
    let mut action = None;
    for (activity, icon, hint) in [
        (Some(Activity::Explorer), Icon::Database, "Explorer"),
        (Some(Activity::Files), Icon::FolderOpen, "Files"),
        (Some(Activity::Queries), Icon::FileCode2, "Queries"),
        (Some(Activity::Data), Icon::Table2, "Data"),
        (Some(Activity::History), Icon::History, "History"),
        (Some(Activity::Problems), Icon::TriangleAlert, "Problems"),
        (Some(Activity::Transfers), Icon::Upload, "Transfers"),
        (Some(Activity::Monitor), Icon::Gauge, "Monitor"),
        (Some(Activity::Security), Icon::Shield, "Security"),
        (Some(Activity::Diagram), Icon::ArrowRightLeft, "ER diagram"),
        (Some(Activity::Schema), Icon::Boxes, "Schema workbench"),
        (Some(Activity::Compare), Icon::GitCompare, "Schema compare"),
        (Some(Activity::Tasks), Icon::ListTodo, "Saved tasks"),
        (None, Icon::Bot, "Agent (Copilot)"),
    ] {
        let active = activity.is_some_and(|value| context.activity == value)
            || (hint == "Queries" && context.active_tab == WorkspaceTab::Query)
            || (hint == "Agent (Copilot)" && context.agent_open);
        let response = icon_button(ui, icon, active, context.theme);
        if response.on_hover_text(hint).clicked() {
            action = activity_action(activity, hint);
        }
        ui.add_space(SPACE_XS);
    }
    action
}

fn draw_settings_button(ui: &mut egui::Ui, context: &ActivityBarContext) -> bool {
    icon_button(
        ui,
        Icon::Settings2,
        context.activity == Activity::Settings,
        context.theme,
    )
    .on_hover_text("Settings")
    .clicked()
}

fn activity_action(activity: Option<Activity>, hint: &str) -> Option<ActivityBarAction> {
    match (activity, hint) {
        (Some(Activity::Queries), _) => Some(ActivityBarAction::OpenQuery),
        (Some(Activity::Diagram), _) => Some(ActivityBarAction::OpenDiagram),
        (Some(Activity::Schema), _) => Some(ActivityBarAction::OpenSchemaWorkbench),
        (Some(Activity::Compare), _) => Some(ActivityBarAction::OpenSchemaCompare),
        (Some(value), _) => Some(ActivityBarAction::SelectActivity(value)),
        (None, "Agent (Copilot)") => Some(ActivityBarAction::ToggleAgent),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activity_clicks_emit_navigation_intents_without_mutating_state() {
        assert!(matches!(
            activity_action(Some(Activity::Queries), "Queries"),
            Some(ActivityBarAction::OpenQuery)
        ));
        assert!(matches!(
            activity_action(Some(Activity::Diagram), "ER diagram"),
            Some(ActivityBarAction::OpenDiagram)
        ));
        assert!(matches!(
            activity_action(Some(Activity::Schema), "Schema workbench"),
            Some(ActivityBarAction::OpenSchemaWorkbench)
        ));
        assert!(matches!(
            activity_action(Some(Activity::Explorer), "Explorer"),
            Some(ActivityBarAction::SelectActivity(Activity::Explorer))
        ));
        assert!(matches!(
            activity_action(None, "Agent (Copilot)"),
            Some(ActivityBarAction::ToggleAgent)
        ));
    }
}
