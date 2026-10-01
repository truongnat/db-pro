mod segmented;
mod underline;

use egui::{Key, Ui};

pub use segmented::SegmentedTabs;
pub use underline::UnderlineTabs;

fn navigation_keys(ui: &Ui) -> super::handler::NavigationKeys {
    ui.input(|input| super::handler::NavigationKeys {
        arrow_left: input.key_pressed(Key::ArrowLeft),
        arrow_up: input.key_pressed(Key::ArrowUp),
        arrow_right: input.key_pressed(Key::ArrowRight),
        arrow_down: input.key_pressed(Key::ArrowDown),
        activate: input.key_pressed(Key::Enter) || input.key_pressed(Key::Space),
    })
}
