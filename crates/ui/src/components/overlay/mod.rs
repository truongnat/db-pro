mod config;
mod handler;
mod toast;
mod tooltip;
mod ui;

pub use toast::{Toast, ToastItem, ToastManager, ToastPosition, ToastResponse, ToastVariant};
pub use tooltip::{Tooltip, TooltipPosition};
pub use ui::{
    context_action_menu, ctx_menu_item, floating_surface, is_context_menu_triggered, DropdownItem, DropdownMenu,
    Popover,
};

pub(crate) use ui::screen_rect;

#[cfg(test)]
mod tests {
    use super::handler::should_close_on_outside_click;
    use super::ui::context_action_menu;
    use crate::DbProTheme;

    #[test]
    fn outside_click_requires_an_interaction_position() {
        assert!(!should_close_on_outside_click(true, true, false));
        assert!(should_close_on_outside_click(true, true, true));
        assert!(!should_close_on_outside_click(false, true, true));
    }

    fn frame_with_row(ctx: &egui::Context, input: egui::RawInput, theme: DbProTheme, menu_rendered: &mut bool) {
        let _ = crate::test_frame::frame(&ctx, input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let (_rect, response) = ui.allocate_exact_size(egui::vec2(300.0, 26.0), egui::Sense::click());
                context_action_menu(ui, &response, theme, |ui, _close_menu| {
                    *menu_rendered = true;
                    let _ = ui.label("Menu item");
                });
            });
        });
    }

    fn button_event(pos: egui::Pos2, button: egui::PointerButton, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button,
            pressed,
            modifiers: egui::Modifiers::default(),
        }
    }

    #[test]
    fn context_menu_opens_on_secondary_click() {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let theme = DbProTheme::dark();
        let click_pos = egui::pos2(60.0, 13.0);
        let mut menu_rendered = false;

        frame_with_row(&ctx, Default::default(), theme, &mut menu_rendered);
        assert!(!menu_rendered);

        for events in [
            vec![
                egui::Event::PointerMoved(click_pos),
                button_event(click_pos, egui::PointerButton::Secondary, true),
            ],
            vec![button_event(click_pos, egui::PointerButton::Secondary, false)],
            vec![],
        ] {
            let input = egui::RawInput {
                events,
                ..Default::default()
            };
            frame_with_row(&ctx, input, theme, &mut menu_rendered);
        }

        assert!(menu_rendered, "menu body should render after right-click");
    }
}
