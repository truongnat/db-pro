use super::config::{ITEM_HEIGHT, MAX_VISIBLE_ITEMS, MENU_PAD};
use egui::{Pos2, Rect};

pub fn dropdown_should_open_above(space_below: f32, space_above: f32, menu_h: f32) -> bool {
    space_below < menu_h && space_above > space_below
}

pub struct DropdownGeometry {
    pub menu_pos: Pos2,
    pub menu_width: f32,
    pub max_height: f32,
}

pub fn calculate_menu_geometry(
    screen: Rect,
    parent_rect: Rect,
    option_count: usize,
    has_more: bool,
) -> DropdownGeometry {
    let extra = if has_more { 1 } else { 0 };
    let visible = option_count.saturating_add(extra).min(MAX_VISIBLE_ITEMS);
    let menu_h = visible as f32 * ITEM_HEIGHT + MENU_PAD * 2.0;
    let space_below = (screen.bottom() - parent_rect.bottom()).max(0.0);
    let space_above = (parent_rect.top() - screen.top()).max(0.0);
    let open_up = dropdown_should_open_above(space_below, space_above, menu_h);
    let menu_width = parent_rect.width().max(160.0).min((screen.width() - 16.0).max(80.0));
    let menu_left = parent_rect.left().clamp(
        screen.left() + 8.0,
        (screen.right() - menu_width - 8.0).max(screen.left() + 8.0),
    );
    let menu_pos = if open_up {
        Pos2::new(menu_left, parent_rect.top() - 4.0 - menu_h)
    } else {
        Pos2::new(menu_left, parent_rect.bottom() + 4.0)
    };

    DropdownGeometry {
        menu_pos,
        menu_width,
        max_height: MAX_VISIBLE_ITEMS as f32 * ITEM_HEIGHT,
    }
}
