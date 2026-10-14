//! Tests for `../mod.rs` (moved out of the source file).
use super::*;

#[test]
fn public_status_and_task_variants_remain_stable() {
    assert_eq!(StatusBadgeVariant::Active, StatusBadgeVariant::Active);
    assert_eq!(ToolCallStatus::Failed, ToolCallStatus::Failed);
    assert_eq!(AgentTaskStatus::Skipped, AgentTaskStatus::Skipped);
}

#[test]
fn task_builder_preserves_title_status_and_optional_detail() {
    let task = AgentTaskItem::new("Inspect schema", AgentTaskStatus::Running).with_detail("42ms");
    assert_eq!(task.title, "Inspect schema");
    assert_eq!(task.status, AgentTaskStatus::Running);
    assert_eq!(task.detail.as_deref(), Some("42ms"));
}
