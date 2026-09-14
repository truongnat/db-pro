#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ErViewport {
    pub pan: egui::Vec2,
    pub zoom: f32,
    pub screen_origin: egui::Pos2,
}

impl Default for ErViewport {
    fn default() -> Self {
        Self {
            pan: egui::Vec2::ZERO,
            zoom: 1.0,
            screen_origin: egui::Pos2::ZERO,
        }
    }
}

impl ErViewport {
    pub fn new(pan: egui::Vec2, zoom: f32, screen_origin: egui::Pos2) -> Self {
        Self {
            pan,
            zoom: zoom.clamp(0.5, 2.0),
            screen_origin,
        }
    }

    #[inline]
    pub fn world_to_screen_pos(&self, world_pos: egui::Pos2) -> egui::Pos2 {
        self.screen_origin + self.pan + world_pos.to_vec2() * self.zoom
    }

    #[inline]
    pub fn screen_to_world_pos(&self, screen_pos: egui::Pos2) -> egui::Pos2 {
        let delta = screen_pos - self.screen_origin - self.pan;
        egui::pos2(delta.x / self.zoom, delta.y / self.zoom)
    }

    pub fn world_to_screen_rect(&self, world_rect: egui::Rect) -> egui::Rect {
        let min = self.world_to_screen_pos(world_rect.min);
        let max = self.world_to_screen_pos(world_rect.max);
        egui::Rect::from_min_max(min, max)
    }

    pub fn screen_to_world_rect(&self, screen_rect: egui::Rect) -> egui::Rect {
        let min = self.screen_to_world_pos(screen_rect.min);
        let max = self.screen_to_world_pos(screen_rect.max);
        egui::Rect::from_min_max(min, max)
    }

    pub fn visible_world_rect(&self, screen_clip: egui::Rect, margin: f32) -> egui::Rect {
        let mut world_rect = self.screen_to_world_rect(screen_clip);
        if margin > 0.0 {
            world_rect = world_rect.expand(margin / self.zoom);
        }
        world_rect
    }
}
