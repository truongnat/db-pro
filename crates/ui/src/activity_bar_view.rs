// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
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

pub(super) fn draw_activity_bar(ui: &mut egui::Ui, context: &ActivityBarContext) -> Option<ActivityBarAction> {
    let mut action = None;
    egui::Panel::left("activity_bar")
        .resizable(false)
        .exact_size(48.0)
        .show_separator_line(false)
        .frame(egui::Frame {
            fill: context.theme.surface_panel,
            inner_margin: egui::Margin::ZERO,
            outer_margin: egui::Margin::ZERO,
            stroke: egui::Stroke::NONE,
            corner_radius: egui::CornerRadius::ZERO,
            shadow: egui::Shadow::NONE,
        })
        .show(ui, |ui| {
            let full = ui.max_rect();
            ui.painter()
                .rect_filled(full, egui::CornerRadius::ZERO, context.theme.surface_panel);
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

    // 1. Database Navigator (Explorer)
    let explorer_active = context.activity == Activity::Explorer;
    if draw_rail_icon_button(ui, Icon::Database, explorer_active, "Database Navigator", context.theme) {
        action = activity_action(Some(Activity::Explorer), "Database Navigator");
    }
    ui.add_space(2.0);

    // 2. SQL Scripts & Projects
    let queries_active = context.activity == Activity::Queries
        || context.activity == Activity::Files
        || context.active_tab == WorkspaceTab::Query;
    if draw_rail_icon_button(ui, Icon::FileCode2, queries_active, "SQL Scripts & Projects", context.theme) {
        action = activity_action(Some(Activity::Queries), "SQL Scripts & Projects");
    }
    ui.add_space(2.0);

    // 3. Tools & Administration Hub
    let tools_active = matches!(
        context.activity,
        Activity::Tools
            | Activity::Diagram
            | Activity::Schema
            | Activity::Compare
            | Activity::History
            | Activity::Problems
            | Activity::Transfers
            | Activity::Monitor
            | Activity::Security
            | Activity::Tasks
            | Activity::Data
    );
    if draw_rail_icon_button(ui, Icon::Boxes, tools_active, "Tools & Management Hub", context.theme) {
        action = activity_action(Some(Activity::Tools), "Tools & Management Hub");
    }

    ui.add_space(SPACE_SM);
    draw_rail_separator(ui, context.theme);
    ui.add_space(SPACE_SM);

    // 4. AI Copilot
    let agent_active = context.agent_open;
    if draw_rail_icon_button(ui, Icon::Sparkles, agent_active, "AI Copilot (Agent)", context.theme) {
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
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), hint));
    let hovered = resp.hovered();

    if active {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(crate::tokens::RADIUS_BUTTON as u8),
            theme.surface_active,
        );
        // Left accent indicator pill on the panel edge (Stitch 2px active pill)
        let bar_left = ui.max_rect().left();
        let bar_rect = egui::Rect::from_min_max(
            egui::pos2(bar_left, rect.center().y - 9.0),
            egui::pos2(bar_left + 2.0, rect.center().y + 9.0),
        );
        ui.painter().rect_filled(bar_rect, egui::CornerRadius::same(1.0 as u8), theme.accent);
    } else if hovered {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(crate::tokens::RADIUS_BUTTON as u8),
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
        (None, "AI Copilot (Agent)" | "Agent (Copilot)") => Some(ActivityBarAction::ToggleAgent),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activity_clicks_emit_navigation_intents_without_mutating_state() {
        assert!(matches!(
            activity_action(Some(Activity::Queries), "SQL Scripts & Projects"),
            Some(ActivityBarAction::OpenQuery)
        ));
        assert!(matches!(
            activity_action(Some(Activity::Tools), "Tools & Management Hub"),
            Some(ActivityBarAction::SelectActivity(Activity::Tools))
        ));
        assert!(matches!(
            activity_action(Some(Activity::Explorer), "Database Navigator"),
            Some(ActivityBarAction::SelectActivity(Activity::Explorer))
        ));
        assert!(matches!(
            activity_action(None, "AI Copilot (Agent)"),
            Some(ActivityBarAction::ToggleAgent)
        ));
    }
}
