mod config;
mod handler;
mod ui;

pub use ui::{Alert, AlertDialog, AlertDialogAction, AlertVariant};

#[cfg(test)]
mod tests {
    use super::{Alert, AlertDialog, AlertVariant};
    use crate::DbProTheme;

    #[test]
    fn exposes_all_alert_variants() {
        assert_ne!(AlertVariant::Info, AlertVariant::Success);
        assert_eq!(AlertVariant::Default, AlertVariant::Default);
    }

    #[test]
    fn destructive_confirmation_is_opt_in_and_backdrop_cannot_cancel_it() {
        let dialog = AlertDialog::new("Delete", "Confirm deletion", DbProTheme::light());
        assert!(!dialog.destructive);
        assert!(super::handler::should_close_from_backdrop(false, true));
        assert!(!super::handler::should_close_from_backdrop(true, true));
    }

    #[test]
    fn alert_builder_keeps_non_dismissable_default() {
        let alert = Alert::title_only("Saved", DbProTheme::light()).variant(AlertVariant::Success);
        assert!(!alert.dismissable);
        assert_eq!(alert.variant, AlertVariant::Success);
    }
}
