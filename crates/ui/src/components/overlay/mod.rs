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

    #[test]
    fn outside_click_requires_an_interaction_position() {
        assert!(!should_close_on_outside_click(true, true, false));
        assert!(should_close_on_outside_click(true, true, true));
        assert!(!should_close_on_outside_click(false, true, true));
    }
}
