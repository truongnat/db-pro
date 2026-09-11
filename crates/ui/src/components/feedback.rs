use crate::DbProTheme;
use egui::{Color32, Frame, Margin, Pos2, Rect, RichText, Rounding, Stroke, Ui, Vec2};

pub struct Progress {
    pub(crate) fraction: f32, // 0.0 to 1.0
    pub(crate) height: f32,
    pub(crate) theme: DbProTheme,
}

pub type ShadcnProgress = Progress;

impl Progress {
    pub fn new(fraction: f32, theme: DbProTheme) -> Self {
        Self {
            fraction: fraction.clamp(0.0, 1.0),
            height: 6.0,
            theme,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, self.height), egui::Sense::hover());

        let rounding = Rounding::same(self.height * 0.5);

        // Track
        ui.painter().rect_filled(rect, rounding, self.theme.surface_hover);

        // Progress fill
        if self.fraction > 0.0 {
            let fill_width = rect.width() * self.fraction;
            let fill_rect = Rect::from_min_size(rect.left_top(), Vec2::new(fill_width, self.height));
            ui.painter().rect_filled(fill_rect, rounding, self.theme.accent);
        }
    }
}

pub struct Spinner {
    size: f32,
    color: Option<Color32>,
    theme: DbProTheme,
}

pub type ShadcnSpinner = Spinner;

impl Spinner {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            size: 18.0,
            color: None,
            theme,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn show(self, ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(self.size), egui::Sense::hover());
        ui.ctx().request_repaint();

        let time = ui.input(|i| i.time);
        let center = rect.center();
        let radius = (self.size - 3.0) * 0.5;
        let color = self.color.unwrap_or(self.theme.accent);

        // Draw subtle track circle
        ui.painter()
            .circle_stroke(center, radius, Stroke::new(2.0, self.theme.border_subtle));

        // Draw rotating arc
        let angle_start = (time * 7.0) as f32;
        let sweep = std::f32::consts::PI * 1.3;
        let n_points = 24;
        let mut arc_points = Vec::with_capacity(n_points);
        for i in 0..n_points {
            let t = i as f32 / (n_points - 1) as f32;
            let a = angle_start + t * sweep;
            arc_points.push(Pos2::new(center.x + a.cos() * radius, center.y + a.sin() * radius));
        }

        ui.painter()
            .add(egui::epaint::PathShape::line(arc_points, Stroke::new(2.0, color)));
    }
}

pub fn kbd_badge(ui: &mut Ui, shortcut: &str, theme: DbProTheme) {
    Frame {
        fill: theme.surface_panel,
        stroke: Stroke::new(1.0, theme.border_default),
        inner_margin: Margin::symmetric(5.0, 2.0),
        rounding: Rounding::same(4.0),
        shadow: egui::epaint::Shadow {
            offset: egui::vec2(0.0, 1.0),
            blur: 0.0,
            spread: 0.0,
            color: theme.border_strong,
        },
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.label(
            RichText::new(shortcut)
                .size(11.0)
                .monospace()
                .color(theme.text_secondary),
        );
    });
}

pub fn separator_with_text(ui: &mut Ui, text: &str, theme: DbProTheme) {
    ui.horizontal(|ui| {
        let text_layout =
            ui.painter()
                .layout_no_wrap(text.to_owned(), egui::FontId::proportional(11.0), theme.text_muted);
        let text_width = text_layout.size().x;
        let total_width = ui.available_width();
        let line_width = ((total_width - text_width - 16.0) * 0.5).max(10.0);

        let (rect_left, _) = ui.allocate_exact_size(Vec2::new(line_width, 14.0), egui::Sense::hover());
        ui.painter().hline(
            rect_left.x_range(),
            rect_left.center().y,
            Stroke::new(1.0, theme.border_default),
        );

        ui.label(RichText::new(text).size(11.0).color(theme.text_muted));

        let (rect_right, _) = ui.allocate_exact_size(Vec2::new(line_width, 14.0), egui::Sense::hover());
        ui.painter().hline(
            rect_right.x_range(),
            rect_right.center().y,
            Stroke::new(1.0, theme.border_default),
        );
    });
}
