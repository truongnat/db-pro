use super::{config, handler};
use crate::DbProTheme;
use egui::Ui;

pub struct ScrollArea {
    horizontal: bool,
    vertical: bool,
    auto_shrink: [bool; 2],
    max_height: Option<f32>,
    // Retained as part of the stable constructor contract; ScrollArea currently uses egui's
    // native visuals directly and has no theme-specific paint decisions.
    #[allow(dead_code)]
    theme: DbProTheme,
}

impl ScrollArea {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            horizontal: false,
            vertical: config::DEFAULT_VERTICAL,
            auto_shrink: config::DEFAULT_AUTO_SHRINK,
            max_height: None,
            theme,
        }
    }

    pub fn horizontal(mut self, enabled: bool) -> Self {
        self.horizontal = enabled;
        self
    }
    pub fn vertical(mut self, enabled: bool) -> Self {
        self.vertical = enabled;
        self
    }
    pub fn both(mut self) -> Self {
        self.horizontal = true;
        self.vertical = true;
        self
    }
    pub fn auto_shrink(mut self, shrink: [bool; 2]) -> Self {
        self.auto_shrink = shrink;
        self
    }
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        // The handler owns the axis-ordering decision; this layer only constructs and
        // renders egui's native scroll container with the selected presentation settings.
        let mut area = egui::ScrollArea::new(handler::scroll_axes(self.horizontal, self.vertical))
            .auto_shrink(self.auto_shrink)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded);
        if let Some(max_height) = self.max_height {
            area = area.max_height(max_height);
        }
        let output = area.show(ui, add_contents);
        output.inner
    }
}

/// Give a native `egui::ScrollArea`'s scroll bars an accessible name.
///
/// egui emits `ScrollBar` accesskit nodes for its native bars but never names
/// them, so audits and assistive tech see anonymous controls. The bar's node
/// id is `ScrollAreaOutput::id.with(d)` (d: 0 = horizontal, 1 = vertical) —
/// an egui-internal id derivation; if egui changes it, the call is a no-op
/// (`accesskit_node_builder` returns `None` for unknown ids), never a crash.
/// Call once after `ScrollArea::show`; safe to call when accesskit is off.
pub fn name_scroll_bars(ctx: &egui::Context, scroll_area_id: egui::Id, label: &str) {
    for (d, axis) in [(0usize, "horizontal"), (1, "vertical")] {
        ctx.accesskit_node_builder(scroll_area_id.with(d), |builder| {
            builder.set_label(format!("{label} · {axis} scroll bar"));
        });
    }
}
