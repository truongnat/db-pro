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
    ui.add_space(SPACE_XS);
    let mut action = None;

    // Group 1: Core Navigation
    let group1 = [
        (Some(Activity::Explorer), Icon::Database, "Explorer"),
        (Some(Activity::Queries), Icon::FileCode2, "Queries"),
        (Some(Activity::Files), Icon::FolderOpen, "Files"),
        (Some(Activity::Data), Icon::Table2, "Data"),
    ];

    for (activity, icon, hint) in group1 {
        let active = activity.is_some_and(|value| context.activity == value)
            || (hint == "Queries" && context.active_tab == WorkspaceTab::Query);
        if draw_rail_icon_button(ui, icon, active, hint, context.theme) {
            action = activity_action(activity, hint);
        }
        ui.add_space(2.0);
    }

    ui.add_space(SPACE_XS);
    draw_rail_separator(ui, context.theme);
    ui.add_space(SPACE_XS);

    // Group 2: Tools & Management
    let group2 = [
        (Some(Activity::Diagram), Icon::ArrowRightLeft, "ER diagram"),
        (Some(Activity::Schema), Icon::Boxes, "Schema workbench"),
        (Some(Activity::Compare), Icon::GitCompare, "Schema compare"),
        (Some(Activity::History), Icon::History, "History"),
        (Some(Activity::Problems), Icon::TriangleAlert, "Problems"),
        (Some(Activity::Transfers), Icon::Upload, "Transfers"),
        (Some(Activity::Monitor), Icon::Gauge, "Monitor"),
        (Some(Activity::Security), Icon::Shield, "Security"),
        (Some(Activity::Tasks), Icon::ListTodo, "Saved tasks"),
    ];

    for (activity, icon, hint) in group2 {
        let active = activity.is_some_and(|value| context.activity == value);
        if draw_rail_icon_button(ui, icon, active, hint, context.theme) {
            action = activity_action(activity, hint);
        }
        ui.add_space(2.0);
    }

    ui.add_space(SPACE_XS);
    draw_rail_separator(ui, context.theme);
    ui.add_space(SPACE_XS);

    // Group 3: AI Copilot
    let agent_active = context.agent_open;
    if draw_rail_icon_button(ui, Icon::Bot, agent_active, "Agent (Copilot)", context.theme) {
        action = Some(ActivityBarAction::ToggleAgent);
    }

    action
}

fn draw_rail_separator(ui: &mut egui::Ui, theme: DbProTheme) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(24.0, 1.0), egui::Sense::hover());
    ui.painter().hline(
        rect.x_range(),
        rect.center().y,
        egui::Stroke::new(1.0, theme.border_subtle),
    );
}

fn draw_rail_icon_button(
    ui: &mut egui::Ui,
    icon: Icon,
    active: bool,
    hint: &str,
    theme: DbProTheme,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(36.0, 34.0), egui::Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(hint);
    let hovered = resp.hovered();

    if active {
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(RADIUS_SM),
            theme.surface_active,
        );
        // Left accent indicator pill on the panel edge
        let bar_left = ui.max_rect().left();
        let bar_rect = egui::Rect::from_min_max(
            egui::pos2(bar_left, rect.center().y - 8.0),
            egui::pos2(bar_left + 2.5, rect.center().y + 8.0),
        );
        ui.painter().rect_filled(bar_rect, egui::Rounding::same(1.25), theme.accent);
    } else if hovered {
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(RADIUS_SM),
            theme.surface_hover,
        );
    }

    let icon_color = if active {
        theme.accent
    } else if hovered {
        theme.text_primary
    } else {
        theme.text_muted
    };

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        char::from(icon).to_string(),
        font_icon(16.5),
        icon_color,
    );

    resp.clicked()
}

fn draw_settings_button(ui: &mut egui::Ui, context: &ActivityBarContext) -> bool {
    draw_rail_icon_button(
        ui,
        Icon::Settings2,
        context.activity == Activity::Settings,
        "Settings",
        context.theme,
    )
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
