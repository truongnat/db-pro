pub mod agent_primitives;
pub mod alert;
pub mod badge;
pub mod button;
pub mod card;
pub mod code;
pub mod diff;
pub mod feedback;
pub mod input;
pub mod legacy;
pub mod select;
pub mod selection;
pub mod table;
pub mod tabs;
pub mod tree;

pub use agent_primitives::{
    ContextChip, ContextChipKind, ExecutionApproval, ExecutionApprovalAction, RiskLevel, StatusBadge,
    StatusBadgeVariant, ToolCall, ToolCallStatus,
};
pub use alert::{Alert, AlertVariant, ShadcnAlert};
pub use badge::{Badge, BadgeVariant, ShadcnBadge};
pub use button::{Button, ButtonSize, ButtonVariant, ShadcnButton};
pub use card::{card_header, Card, MetricCard, ShadcnCard};
pub use code::{CodeBlock, InlineCode};
pub use diff::{DiffLine, DiffLineType, DiffViewer};
pub use feedback::{kbd_badge, separator_with_text, Progress, ShadcnProgress, ShadcnSpinner, Spinner};
pub use input::{
    Input, PasswordInput, SearchInput, ShadcnInput, ShadcnPasswordInput, ShadcnSearchInput, ShadcnTextarea, Textarea,
};
pub use legacy::*;
pub use select::{Select, ShadcnSelect};
pub use selection::{Checkbox, Radio, ShadcnCheckbox, ShadcnRadio, ShadcnSlider, ShadcnSwitch, Slider, Switch};
pub use table::{ShadcnTable, ShadcnTableColumn, Table, TableColumn, TableColumnAlign};
pub use tabs::{SegmentedTabs, UnderlineTabs};
pub use tree::{DatabaseTreeNode, TreeNodeKind};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbProTheme;

    #[test]
    fn test_button_builder_variants_and_sizes() {
        let theme = DbProTheme::light();
        let btn = Button::new(theme)
            .text("Click me")
            .variant(ButtonVariant::Destructive)
            .size(ButtonSize::Lg)
            .enabled(false)
            .loading(true)
            .full_width(true);

        assert_eq!(btn.variant, ButtonVariant::Destructive);
        assert_eq!(btn.size, ButtonSize::Lg);
        assert!(!btn.enabled);
        assert!(btn.loading);
        assert!(btn.full_width);
    }

    #[test]
    fn test_badge_variants() {
        let theme = DbProTheme::light();
        let badge = Badge::new("Active", theme).variant(BadgeVariant::Success).dot(true);

        assert_eq!(badge.variant, BadgeVariant::Success);
        assert!(badge.dot);
    }

    #[test]
    fn test_alert_variants() {
        let theme = DbProTheme::light();
        let alert = Alert::new("Title", "Description", theme)
            .variant(AlertVariant::Warning)
            .dismissable(true);

        assert_eq!(alert.variant, AlertVariant::Warning);
        assert!(alert.dismissable);
    }

    #[test]
    fn test_progress_fraction_clamping() {
        let theme = DbProTheme::light();
        let p1 = Progress::new(1.5, theme);
        assert_eq!(p1.fraction, 1.0);

        let p2 = Progress::new(-0.2, theme);
        assert_eq!(p2.fraction, 0.0);
    }

    #[test]
    fn test_table_column_builder() {
        let col = TableColumn::new("Status")
            .width(120.0)
            .align(TableColumnAlign::Center)
            .sortable(true);

        assert_eq!(col.title, "Status");
        assert_eq!(col.width, Some(120.0));
        assert_eq!(col.align, TableColumnAlign::Center);
        assert!(col.sortable);
    }

    #[test]
    fn test_checkbox_builder() {
        let theme = DbProTheme::light();
        let mut checked = false;
        let cb = Checkbox::new(&mut checked, "Enable SSL", theme)
            .description("Requires valid server certificate")
            .enabled(false);

        assert_eq!(cb.label, "Enable SSL");
        assert_eq!(cb.description, Some("Requires valid server certificate"));
        assert!(!cb.enabled);
    }

    #[test]
    fn test_slider_builder() {
        let theme = DbProTheme::light();
        let mut val = 50.0;
        let slider = Slider::new(&mut val, 0.0..=100.0, theme)
            .label("Volume")
            .show_value(true)
            .width(200.0);

        assert_eq!(slider.label, Some("Volume"));
        assert!(slider.show_value);
        assert_eq!(slider.width, Some(200.0));
    }

    #[test]
    fn test_diff_line_builders() {
        let ctx = DiffLine::context(10, 10, "SELECT 1;");
        assert_eq!(ctx.line_type, DiffLineType::Context);
        assert_eq!(ctx.old_line_num, Some(10));
        assert_eq!(ctx.new_line_num, Some(10));

        let add = DiffLine::added(11, "+ ADDED COLUMN");
        assert_eq!(add.line_type, DiffLineType::Added);
        assert_eq!(add.old_line_num, None);
        assert_eq!(add.new_line_num, Some(11));

        let rem = DiffLine::removed(12, "- REMOVED COLUMN");
        assert_eq!(rem.line_type, DiffLineType::Removed);
        assert_eq!(rem.old_line_num, Some(12));
        assert_eq!(rem.new_line_num, None);
    }

    #[test]
    fn test_context_chip_kind_icons() {
        assert_eq!(
            ContextChipKind::Connection.icon().to_string(),
            lucide_icons::Icon::Server.to_string()
        );
        assert_eq!(
            ContextChipKind::Database.icon().to_string(),
            lucide_icons::Icon::Database.to_string()
        );
        assert_eq!(
            ContextChipKind::Table.icon().to_string(),
            lucide_icons::Icon::Table.to_string()
        );
    }
}
