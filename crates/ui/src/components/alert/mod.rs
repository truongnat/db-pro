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

    #[test]
    fn dismiss_stays_at_trailing_edge_for_short_and_wrapped_copy() {
        for width in [280.0, 800.0] {
            for description in [
                "Short",
                "Correct the highlighted fields, then save again. ".repeat(5).as_str(),
            ] {
                let ctx = egui::Context::default();
                crate::DbProTheme::install_fonts(&ctx);
                let _ = crate::test_frame::frame(
                    &ctx,
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 600.0))),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            let right = ui.available_rect_before_wrap().right();
                            let dismiss = Alert::new("Attention", description, DbProTheme::light())
                                .dismissable(true)
                                .show(ui)
                                .unwrap();
                            let expected = right - super::config::ALERT_FRAME_PADDING_X;
                            assert!(
                                (dismiss.rect.right() - expected).abs() < 1.0,
                                "{width}: {:?}, expected {expected}",
                                dismiss.rect
                            );
                        });
                    },
                );
            }
        }
    }
}
