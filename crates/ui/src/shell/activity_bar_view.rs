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
        .frame(activity_bar_frame(context.theme))
        .show(ui, |ui| {
            let full = ui.max_rect();

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
    // `Secondary` is the shared selected treatment (same as the diagram mode
    // toggles); `Ghost` is the quiet rest state.
    let variant = if active {
        ButtonVariant::Secondary
    } else {
        ButtonVariant::Ghost
    };
    Button::new(theme)
        .icon(icon)
        .variant(variant)
        .size(ButtonSize::IconSm)
        .tooltip(hint)
        .access_label(hint)
        .show(ui)
        .clicked()
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
