use crate::components::button::ButtonVariant;
use crate::DbProTheme;
use egui::{Color32, Pos2, Rect, Vec2};
use lucide_icons::Icon;

use super::{
    config, AgentSqlActionKind, AgentTaskItem, AgentTaskStatus, ContextChipKind, RiskLevel, StatusBadgeVariant,
    ToolCallStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StatusBadgePalette {
    pub(crate) dot: Color32,
    pub(crate) text: Color32,
    pub(crate) background: Color32,
}

#[derive(Debug)]
pub(crate) struct TaskVisual {
    pub(crate) icon: Icon,
    pub(crate) icon_color: Color32,
    pub(crate) title_color: Color32,
}

#[derive(Debug)]
pub(crate) struct ThinkingHeader {
    pub(crate) icon: Icon,
    pub(crate) title: String,
}

pub(crate) fn context_chip_icon(kind: ContextChipKind) -> Icon {
    match kind {
        ContextChipKind::Connection => Icon::Server,
        ContextChipKind::Database => Icon::Database,
        ContextChipKind::Schema => Icon::Folder,
        ContextChipKind::Table => Icon::Table,
        ContextChipKind::Editor => Icon::Code,
        ContextChipKind::File => Icon::FileText,
    }
}

pub(crate) fn context_chip_size(icon_size: Vec2, text_size: Vec2, removable: bool) -> Vec2 {
    let removable_width = if removable { config::CHIP_REMOVABLE_WIDTH } else { 0.0 };
    Vec2::new(
        icon_size.x
            + config::CHIP_ICON_TEXT_GAP
            + text_size.x
            + removable_width
            + config::CHIP_HORIZONTAL_PADDING * 2.0,
        config::CHIP_HEIGHT,
    )
}

pub(crate) fn context_chip_icon_pos(rect: Rect, icon_size: Vec2) -> Pos2 {
    Pos2::new(
        rect.left() + config::CHIP_HORIZONTAL_PADDING,
        rect.center().y - icon_size.y * 0.5,
    )
}

pub(crate) fn context_chip_text_pos(rect: Rect, icon_size: Vec2, text_size: Vec2) -> Pos2 {
    Pos2::new(
        rect.left() + config::CHIP_TEXT_LEFT_OFFSET + icon_size.x + config::CHIP_ICON_TEXT_GAP,
        rect.center().y - text_size.y * 0.5,
    )
}

pub(crate) fn context_chip_remove_rect(rect: Rect) -> Rect {
    Rect::from_min_size(
        Pos2::new(rect.right() - config::CHIP_REMOVABLE_WIDTH, rect.top()),
        Vec2::new(config::CHIP_REMOVABLE_WIDTH, rect.height()),
    )
}

pub(crate) fn status_badge_palette(theme: DbProTheme, variant: StatusBadgeVariant) -> StatusBadgePalette {
    match variant {
        StatusBadgeVariant::Active | StatusBadgeVariant::Success => StatusBadgePalette {
            dot: theme.success,
            text: theme.success,
            background: theme.success_soft(),
        },
        StatusBadgeVariant::Running => StatusBadgePalette {
            dot: theme.info,
            text: theme.info,
            background: theme.info_soft(),
        },
        StatusBadgeVariant::Warning => StatusBadgePalette {
            dot: theme.warning,
            text: theme.warning,
            background: theme.warning_soft(),
        },
        StatusBadgeVariant::Destructive => StatusBadgePalette {
            dot: theme.danger,
            text: theme.danger,
            background: theme.danger_soft(),
        },
        StatusBadgeVariant::Archived | StatusBadgeVariant::Draft => StatusBadgePalette {
            dot: theme.text_tertiary,
            text: theme.text_secondary,
            background: theme.surface_hover,
        },
    }
}

pub(crate) fn status_badge_size(text_size: Vec2) -> Vec2 {
    Vec2::new(
        config::STATUS_HORIZONTAL_PADDING * 2.0 + config::STATUS_DOT_WIDTH + config::STATUS_TEXT_GAP + text_size.x,
        (text_size.y + config::STATUS_VERTICAL_PADDING * 2.0).max(config::STATUS_MIN_HEIGHT),
    )
}

pub(crate) fn status_badge_dot_pos(rect: Rect) -> Pos2 {
    Pos2::new(
        rect.left() + config::STATUS_HORIZONTAL_PADDING + config::STATUS_DOT_WIDTH * 0.5,
        rect.center().y,
    )
}

pub(crate) fn status_badge_text_pos(rect: Rect, text_size: Vec2) -> Pos2 {
    Pos2::new(
        rect.left() + config::STATUS_HORIZONTAL_PADDING + config::STATUS_DOT_WIDTH + config::STATUS_TEXT_GAP,
        rect.center().y - text_size.y * 0.5,
    )
}

pub(crate) fn tool_status_label(status: ToolCallStatus) -> &'static str {
    match status {
        ToolCallStatus::Running => "running",
        ToolCallStatus::Success => "done",
        ToolCallStatus::Failed => "failed",
    }
}

pub(crate) fn tool_status_fill(theme: DbProTheme, status: ToolCallStatus) -> Color32 {
    let color = match status {
        ToolCallStatus::Running => theme.info,
        ToolCallStatus::Success => theme.success,
        ToolCallStatus::Failed => theme.danger,
    };
    color.linear_multiply(0.15)
}

pub(crate) fn tool_badge_size(text_size: Vec2) -> Vec2 {
    Vec2::new(
        text_size.x + config::TOOL_BADGE_HORIZONTAL_PADDING,
        config::TOOL_BADGE_HEIGHT,
    )
}

pub(crate) fn tool_badge_rect(header_rect: Rect, badge_size: Vec2) -> Rect {
    let right = header_rect.right() - config::TOOL_HEADER_RIGHT_INSET;
    Rect::from_min_size(
        Pos2::new(right - badge_size.x, header_rect.center().y - badge_size.y * 0.5),
        badge_size,
    )
}

pub(crate) fn tool_badge_text_pos(rect: Rect) -> Pos2 {
    Pos2::new(
        rect.left() + config::TOOL_BADGE_TEXT_OFFSET_X,
        rect.top() + config::TOOL_BADGE_TEXT_OFFSET_Y,
    )
}

pub(crate) fn tool_duration_pos(header_rect: Rect, right_cursor: &mut f32, duration_size: Vec2) -> Pos2 {
    *right_cursor -= duration_size.x + config::TOOL_BADGE_HORIZONTAL_PADDING;
    Pos2::new(
        *right_cursor,
        header_rect.center().y - config::TOOL_DURATION_TEXT_Y_OFFSET,
    )
}

pub(crate) fn risk_badge(risk: RiskLevel) -> (&'static str, StatusBadgeVariant) {
    match risk {
        RiskLevel::Low => ("Low Risk", StatusBadgeVariant::Success),
        RiskLevel::Medium => ("Medium Risk", StatusBadgeVariant::Warning),
        RiskLevel::High => ("High Risk", StatusBadgeVariant::Destructive),
        RiskLevel::Destructive => ("Destructive", StatusBadgeVariant::Destructive),
    }
}

pub(crate) fn approval_button_variants(risk: RiskLevel) -> (ButtonVariant, ButtonVariant) {
    match risk {
        RiskLevel::High | RiskLevel::Destructive => (ButtonVariant::Destructive, ButtonVariant::Default),
        RiskLevel::Low | RiskLevel::Medium => (ButtonVariant::Default, ButtonVariant::Outline),
    }
}

pub(crate) fn approval_action(
    run_clicked: bool,
    preview_clicked: bool,
    cancel_clicked: bool,
) -> Option<super::ExecutionApprovalAction> {
    if run_clicked {
        return Some(super::ExecutionApprovalAction::Run);
    }
    if preview_clicked {
        return Some(super::ExecutionApprovalAction::Preview);
    }
    if cancel_clicked {
        return Some(super::ExecutionApprovalAction::Cancel);
    }
    None
}

pub(crate) fn disclosure_activation(clicked: bool, focused: bool, keyboard_activation: bool) -> bool {
    clicked || (focused && keyboard_activation)
}

pub(crate) fn toggle_expanded(expanded: &mut bool, activated: bool) {
    if activated {
        *expanded = !*expanded;
    }
}

pub(crate) fn thinking_header(active: bool, duration: Option<&str>, step_count: Option<usize>) -> ThinkingHeader {
    if active {
        return ThinkingHeader {
            icon: Icon::LoaderCircle,
            title: "Thinking...".to_owned(),
        };
    }

    let mut title = String::from("Thought");
    if let Some(duration) = duration {
        title.push_str(" for ");
        title.push_str(duration);
    }
    if let Some(steps) = step_count {
        title.push_str(&format!(" ({} step{})", steps, if steps > 1 { "s" } else { "" }));
    }
    ThinkingHeader {
        icon: Icon::Brain,
        title,
    }
}

pub(crate) fn plan_progress(tasks: &[AgentTaskItem]) -> (usize, usize, f32) {
    let total = tasks.len();
    let completed = tasks
        .iter()
        .filter(|task| task.status == AgentTaskStatus::Completed)
        .count();
    let ratio = if total == 0 {
        0.0
    } else {
        completed as f32 / total as f32
    };
    (completed, total, ratio)
}

pub(crate) fn task_visual(theme: DbProTheme, status: AgentTaskStatus) -> TaskVisual {
    match status {
        AgentTaskStatus::Completed => TaskVisual {
            icon: Icon::CheckCircle2,
            icon_color: theme.success,
            title_color: theme.text_primary,
        },
        AgentTaskStatus::Running => TaskVisual {
            icon: Icon::LoaderCircle,
            icon_color: theme.info,
            title_color: theme.text_primary,
        },
        AgentTaskStatus::Pending => TaskVisual {
            icon: Icon::Circle,
            icon_color: theme.text_tertiary,
            title_color: theme.text_secondary,
        },
        AgentTaskStatus::Failed => TaskVisual {
            icon: Icon::AlertCircle,
            icon_color: theme.danger,
            title_color: theme.danger,
        },
        AgentTaskStatus::Skipped => TaskVisual {
            icon: Icon::MinusCircle,
            icon_color: theme.text_muted,
            title_color: theme.text_disabled,
        },
    }
}

pub(crate) fn sql_action_label(kind: AgentSqlActionKind) -> &'static str {
    match kind {
        AgentSqlActionKind::Explain => "Explain Query",
        AgentSqlActionKind::Optimize => "Optimize Query",
        AgentSqlActionKind::FixError => "Fix Error",
        AgentSqlActionKind::GenerateMigration => "Generate Migration",
        AgentSqlActionKind::DescribeSchema => "Describe Schema",
        AgentSqlActionKind::ConvertDialect => "Convert Dialect",
    }
}

pub(crate) fn sql_action_icon(kind: AgentSqlActionKind) -> Icon {
    match kind {
        AgentSqlActionKind::Explain => Icon::ChartNoAxesCombined,
        AgentSqlActionKind::Optimize => Icon::Gauge,
        AgentSqlActionKind::FixError => Icon::Wrench,
        AgentSqlActionKind::GenerateMigration => Icon::GitFork,
        AgentSqlActionKind::DescribeSchema => Icon::FileSpreadsheet,
        AgentSqlActionKind::ConvertDialect => Icon::RefreshCw,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Compare glyph identities through their rendered character because Icon has no Eq implementation.
    fn context_kind_mapping_preserves_icons() {
        assert_eq!(
            char::from(context_chip_icon(ContextChipKind::Connection)),
            char::from(Icon::Server)
        );
        assert_eq!(
            char::from(context_chip_icon(ContextChipKind::Database)),
            char::from(Icon::Database)
        );
        assert_eq!(
            char::from(context_chip_icon(ContextChipKind::File)),
            char::from(Icon::FileText)
        );
    }

    #[test]
    fn removable_chip_target_occupies_its_trailing_hit_area() {
        let chip = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(120.0, config::CHIP_HEIGHT));
        let remove = context_chip_remove_rect(chip);
        assert_eq!(remove.min.x, chip.right() - config::CHIP_REMOVABLE_WIDTH);
        assert_eq!(remove.max.x, chip.right());
        assert_eq!(remove.height(), chip.height());
    }

    #[test]
    fn status_mapping_keeps_success_and_neutral_semantics() {
        let theme = DbProTheme::light();
        let success = status_badge_palette(theme, StatusBadgeVariant::Success);
        assert_eq!(success.dot, theme.success);
        assert_eq!(success.text, theme.success);
        assert_eq!(success.background, theme.success_soft());

        let draft = status_badge_palette(theme, StatusBadgeVariant::Draft);
        assert_eq!(draft.dot, theme.text_tertiary);
        assert_eq!(draft.text, theme.text_secondary);
    }

    #[test]
    fn risk_mapping_keeps_destructive_actions_distinct_in_label() {
        assert_eq!(risk_badge(RiskLevel::Low), ("Low Risk", StatusBadgeVariant::Success));
        assert_eq!(
            risk_badge(RiskLevel::Destructive),
            ("Destructive", StatusBadgeVariant::Destructive)
        );
        assert_eq!(
            approval_button_variants(RiskLevel::Destructive),
            (ButtonVariant::Destructive, ButtonVariant::Default)
        );
        assert_eq!(
            approval_button_variants(RiskLevel::Medium),
            (ButtonVariant::Default, ButtonVariant::Outline)
        );
    }

    #[test]
    fn thinking_titles_preserve_duration_and_pluralization() {
        assert_eq!(thinking_header(true, Some("2s"), Some(3)).title, "Thinking...");
        assert_eq!(
            thinking_header(false, Some("2s"), Some(1)).title,
            "Thought for 2s (1 step)"
        );
        assert_eq!(thinking_header(false, None, Some(2)).title, "Thought (2 steps)");
    }

    #[test]
    // Empty plans stay deterministic while partial plans expose a bounded ratio.
    fn progress_ratio_handles_empty_and_partial_plans() {
        let empty: Vec<AgentTaskItem> = Vec::new();
        assert_eq!(plan_progress(&empty), (0, 0, 0.0));
        let tasks = vec![
            AgentTaskItem::new("done", AgentTaskStatus::Completed),
            AgentTaskItem::new("waiting", AgentTaskStatus::Pending),
        ];
        assert_eq!(plan_progress(&tasks), (1, 2, 0.5));
    }

    #[test]
    fn disclosure_requires_focus_for_keyboard_activation() {
        assert!(disclosure_activation(false, true, true));
        assert!(!disclosure_activation(false, false, true));
        assert!(disclosure_activation(true, false, false));
    }

    #[test]
    fn action_mapping_has_deterministic_button_priority() {
        assert_eq!(
            approval_action(true, true, true),
            Some(super::super::ExecutionApprovalAction::Run)
        );
        assert_eq!(
            approval_action(false, false, true),
            Some(super::super::ExecutionApprovalAction::Cancel)
        );
        assert_eq!(approval_action(false, false, false), None);
    }
}
