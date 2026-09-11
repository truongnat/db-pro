pub mod alert;
pub mod badge;
pub mod button;
pub mod card;
pub mod feedback;
pub mod input;
pub mod legacy;
pub mod select;
pub mod selection;
pub mod table;
pub mod tabs;

pub use alert::{AlertVariant, ShadcnAlert};
pub use badge::{BadgeVariant, ShadcnBadge};
pub use button::{ButtonSize, ButtonVariant, ShadcnButton};
pub use card::{card_header, MetricCard, ShadcnCard};
pub use feedback::{kbd_badge, separator_with_text, ShadcnProgress, ShadcnSpinner};
pub use input::{ShadcnInput, ShadcnPasswordInput, ShadcnSearchInput, ShadcnTextarea};
pub use legacy::*;
pub use select::ShadcnSelect;
pub use selection::{ShadcnCheckbox, ShadcnRadio, ShadcnSlider, ShadcnSwitch};
pub use table::{ShadcnTable, ShadcnTableColumn};
pub use tabs::{SegmentedTabs, UnderlineTabs};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DbProTheme;

    #[test]
    fn test_button_builder_variants_and_sizes() {
        let theme = DbProTheme::light();
        let btn = ShadcnButton::new(theme)
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
        let badge = ShadcnBadge::new("Active", theme)
            .variant(BadgeVariant::Success)
            .dot(true);

        assert_eq!(badge.variant, BadgeVariant::Success);
        assert!(badge.dot);
    }

    #[test]
    fn test_alert_variants() {
        let theme = DbProTheme::light();
        let alert = ShadcnAlert::new("Title", "Description", theme)
            .variant(AlertVariant::Warning)
            .dismissable(true);

        assert_eq!(alert.variant, AlertVariant::Warning);
        assert!(alert.dismissable);
    }

    #[test]
    fn test_progress_fraction_clamping() {
        let theme = DbProTheme::light();
        let p1 = ShadcnProgress::new(1.5, theme);
        assert_eq!(p1.fraction, 1.0);

        let p2 = ShadcnProgress::new(-0.2, theme);
        assert_eq!(p2.fraction, 0.0);
    }
}
