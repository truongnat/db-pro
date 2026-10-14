//! Tests for `../handler.rs` (moved out of the source file).
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
