//! Agent primitive geometry that is owned by this component family.
//!
//! Shared palette, spacing, typography, and radius tokens remain in `DbProTheme`
//! and `components::tokens`; these values describe only the primitives' local
//! layout contracts.

pub const CHIP_HEIGHT: f32 = 24.0;
pub const CHIP_ICON_TEXT_GAP: f32 = 6.0;
pub const CHIP_HORIZONTAL_PADDING: f32 = 8.0;
pub const CHIP_REMOVABLE_WIDTH: f32 = 24.0;
pub const CHIP_RADIUS: f32 = 11.0;
pub const CHIP_TEXT_LEFT_OFFSET: f32 = 8.0;

pub const STATUS_DOT_WIDTH: f32 = 5.0;
pub const STATUS_DOT_RADIUS: f32 = 2.5;
pub const STATUS_TEXT_GAP: f32 = 4.0;
pub const STATUS_HORIZONTAL_PADDING: f32 = 8.0;
pub const STATUS_VERTICAL_PADDING: f32 = 2.0;
pub const STATUS_MIN_HEIGHT: f32 = 20.0;

pub const TOOL_HEADER_HEIGHT: f32 = 32.0;
pub const TOOL_HEADER_RIGHT_INSET: f32 = 10.0;
pub const TOOL_BADGE_HORIZONTAL_PADDING: f32 = 12.0;
pub const TOOL_BADGE_HEIGHT: f32 = 18.0;
pub const TOOL_BADGE_RADIUS: f32 = 9.0;
pub const TOOL_BADGE_TEXT_OFFSET_X: f32 = 6.0;
pub const TOOL_BADGE_TEXT_OFFSET_Y: f32 = 2.0;
pub const TOOL_DURATION_TEXT_Y_OFFSET: f32 = 6.0;
pub const TOOL_HEADER_ICON_OFFSET_X: f32 = 14.0;
pub const TOOL_NAME_ICON_OFFSET_X: f32 = 30.0;
pub const TOOL_NAME_TEXT_OFFSET_X: f32 = 44.0;
pub const TOOL_SECTION_INSET: f32 = 12.0;
pub const TOOL_SECTION_TOP_SPACE: f32 = 8.0;
pub const TOOL_SECTION_BOTTOM_SPACE: f32 = 8.0;
pub const TOOL_SECTION_LABEL_GAP: f32 = 2.0;
pub const TOOL_CODE_HORIZONTAL_INSET: f32 = 16.0;
pub const TOOL_CODE_PADDING: f32 = 8.0;
pub const TOOL_OUTPUT_GAP: f32 = 6.0;

pub const APPROVAL_RADIUS: f32 = 10.0;
pub const APPROVAL_PADDING: f32 = 14.0;
pub const APPROVAL_TITLE_SIZE: f32 = 13.5;
pub const APPROVAL_BODY_SIZE: f32 = 12.0;
pub const APPROVAL_CODE_RADIUS: f32 = 6.0;
pub const APPROVAL_CODE_PADDING: f32 = 10.0;
pub const APPROVAL_TITLE_GAP: f32 = 6.0;
pub const APPROVAL_CODE_GAP: f32 = 8.0;
pub const APPROVAL_ACTION_GAP: f32 = 12.0;
pub const APPROVAL_BUTTON_GAP: f32 = 4.0;

pub const THINKING_RADIUS: f32 = 8.0;
pub const THINKING_HORIZONTAL_PADDING: f32 = 10.0;
pub const THINKING_VERTICAL_PADDING: f32 = 6.0;
pub const THINKING_HEADER_HEIGHT: f32 = 24.0;
pub const THINKING_HEADER_RADIUS: f32 = 6.0;
pub const THINKING_CHEVRON_X: f32 = 8.0;
pub const THINKING_ICON_X: f32 = 22.0;
pub const THINKING_TITLE_X: f32 = 36.0;
pub const THINKING_BODY_GAP: f32 = 4.0;
pub const THINKING_BODY_RADIUS: f32 = 6.0;
pub const THINKING_BODY_PADDING: f32 = 8.0;
pub const THINKING_TEXT_SIZE: f32 = 11.5;

pub const PLAN_RADIUS: f32 = 8.0;
pub const PLAN_PADDING: f32 = 12.0;
pub const PLAN_TITLE_SIZE: f32 = 13.0;
pub const PLAN_META_SIZE: f32 = 11.5;
pub const PLAN_PROGRESS_GAP: f32 = 6.0;
pub const PLAN_PROGRESS_HEIGHT: f32 = 4.0;
pub const PLAN_PROGRESS_RADIUS: f32 = 2.0;
pub const PLAN_TASK_LIST_TOP_SPACE: f32 = 10.0;
pub const PLAN_TASK_GAP: f32 = 6.0;
pub const PLAN_TASK_ICON_SIZE: f32 = 13.0;
pub const PLAN_TASK_TITLE_SIZE: f32 = 12.0;
pub const PLAN_TASK_DETAIL_SIZE: f32 = 11.0;
pub const PLAN_TASK_ICON_TEXT_GAP: f32 = 2.0;

pub const TEXT_LABEL_SIZE: f32 = 10.5;
pub const TEXT_BODY_MONO_SIZE: f32 = 11.5;
pub const ICON_CONTEXT_SIZE: f32 = 13.0;
pub const ICON_SMALL_SIZE: f32 = 10.0;
pub const ICON_STATUS_SIZE: f32 = 12.0;
pub const ICON_TOOL_CHEVRON_SIZE: f32 = 10.5;
pub const ICON_TOOL_SIZE: f32 = 13.0;
pub const FRAME_BORDER_WIDTH: f32 = 1.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // This focused assertion makes the local-vs-shared token boundary explicit.
    fn local_geometry_constants_keep_the_legacy_chip_contract() {
        assert_eq!(CHIP_HEIGHT, 24.0);
        assert_eq!(CHIP_HORIZONTAL_PADDING, 8.0);
        assert_eq!(CHIP_REMOVABLE_WIDTH, 24.0);
    }
}
