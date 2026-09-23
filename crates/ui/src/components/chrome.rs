use crate::components::animation::pulse_alpha;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::DbProTheme;
use egui::{
    Align2, FontFamily, FontId, Frame, Margin, Pos2, Rect, Response, RichText, Rounding, Sense, Stroke, Ui,
    Vec2,
};
use lucide_icons::Icon;

const AVATAR_SM: f32 = 24.0;
const AVATAR_MD: f32 = 32.0;
const AVATAR_LG: f32 = 40.0;
const SKELETON_MIN_ALPHA: f32 = 0.35;
const SKELETON_MAX_ALPHA: f32 = 0.72;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarSize {
    Sm,
    Md,
    Lg,
}

impl AvatarSize {
    fn px(self) -> f32 {
        match self {
            Self::Sm => AVATAR_SM,
            Self::Md => AVATAR_MD,
            Self::Lg => AVATAR_LG,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarStatus {
    Online,
    Busy,
    Away,
    Offline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarShape {
    Circle,
    Rounded,
}

pub struct Avatar<'a> {
    initials: Option<&'a str>,
    icon: Option<Icon>,
    size: AvatarSize,
    shape: AvatarShape,
    status: Option<AvatarStatus>,
    theme: DbProTheme,
}

impl<'a> Avatar<'a> {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            initials: None,
            icon: None,
            size: AvatarSize::Md,
            shape: AvatarShape::Circle,
            status: None,
            theme,
        }
    }

    pub fn initials(mut self, initials: &'a str) -> Self {
        self.initials = Some(initials);
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    pub fn shape(mut self, shape: AvatarShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn status(mut self, status: AvatarStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let size = self.size.px();
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());

        let rounding = match self.shape {
            AvatarShape::Circle => Rounding::same(size * 0.5),
            AvatarShape::Rounded => Rounding::same(6.0),
        };

        ui.painter().rect_filled(rect, rounding, self.theme.surface_hover);
        ui.painter()
            .rect_stroke(rect, rounding, Stroke::new(1.0, self.theme.border_subtle));

        if let Some(initials) = self.initials {
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                initials,
                FontId::proportional((size * 0.38).max(10.0)),
                self.theme.text_primary,
            );
        } else if let Some(icon) = self.icon {
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                char::from(icon).to_string(),
                FontId::new(size * 0.45, FontFamily::Name("lucide".into())),
                self.theme.text_secondary,
            );
        }

        // Status indicator dot
        if let Some(status) = self.status {
            let dot_radius = match self.size {
                AvatarSize::Sm => 3.5,
                AvatarSize::Md => 4.5,
                AvatarSize::Lg => 5.5,
            };
            let dot_center = Pos2::new(
                rect.right() - dot_radius * 0.7,
                rect.bottom() - dot_radius * 0.7,
            );

            let dot_color = match status {
                AvatarStatus::Online => self.theme.success,
                AvatarStatus::Busy => self.theme.danger,
                AvatarStatus::Away => self.theme.warning,
                AvatarStatus::Offline => self.theme.text_disabled,
            };

            // White/surface border ring
            ui.painter().circle_filled(
                dot_center,
                dot_radius + 1.5,
                self.theme.surface_panel,
            );
            ui.painter().circle_filled(dot_center, dot_radius, dot_color);
        }

        response
    }
}

pub struct Skeleton {
    width: f32,
    height: f32,
    rounding: f32,
    shimmer: bool,
    theme: DbProTheme,
}

impl Skeleton {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            width: 160.0,
            height: 12.0,
            rounding: 6.0,
            shimmer: true,
            theme,
        }
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn rounding(mut self, rounding: f32) -> Self {
        self.rounding = rounding;
        self
    }

    pub fn shimmer(mut self, shimmer: bool) -> Self {
        self.shimmer = shimmer;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = if self.width <= 0.0 {
            ui.available_width()
        } else {
            self.width.min(ui.available_width())
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, self.height), Sense::hover());
        let alpha = pulse_alpha(ui, SKELETON_MIN_ALPHA, SKELETON_MAX_ALPHA);
        let fill = self.theme.surface_hover.linear_multiply(alpha);
        let rounding = Rounding::same(self.rounding);

        ui.painter().rect_filled(rect, rounding, fill);

        if self.shimmer {
            // Animated shimmer light wave moving horizontally
            let time = ui.input(|i| i.time);
            let cycle = (time * 0.8).fract() as f32;
            let shimmer_x = rect.left() + rect.width() * cycle;
            let shimmer_w = (rect.width() * 0.3).max(20.0);
            let shimmer_rect = Rect::from_min_size(
                Pos2::new(shimmer_x - shimmer_w * 0.5, rect.top()),
                Vec2::new(shimmer_w, self.height),
            );
            let clipped = rect.intersect(shimmer_rect);
            if clipped.is_positive() {
                let shimmer_fill = self.theme.surface_elevated.linear_multiply(0.25);
                ui.painter().rect_filled(clipped, rounding, shimmer_fill);
            }
            ui.ctx().request_repaint();
        }

        response
    }
}

pub struct EmptyState<'a> {
    icon: Icon,
    title: &'a str,
    description: &'a str,
    action_label: Option<&'a str>,
    theme: DbProTheme,
}

impl<'a> EmptyState<'a> {
    pub fn new(icon: Icon, title: &'a str, description: &'a str, theme: DbProTheme) -> Self {
        Self {
            icon,
            title,
            description,
            action_label: None,
            theme,
        }
    }

    pub fn action(mut self, label: &'a str) -> Self {
        self.action_label = Some(label);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Option<Response> {
        let mut action = None;
        ui.vertical_centered(|ui| {
            ui.add_space(12.0);
            ui.label(
                RichText::new(char::from(self.icon).to_string())
                    .font(FontId::new(20.0, FontFamily::Name("lucide".into())))
                    .color(self.theme.text_tertiary),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(self.title)
                    .size(15.0)
                    .strong()
                    .color(self.theme.text_primary),
            );
            ui.add_space(4.0);
            ui.add(
                egui::Label::new(
                    RichText::new(self.description)
                        .size(13.0)
                        .color(self.theme.text_secondary),
                )
                .wrap(),
            );
            if let Some(label) = self.action_label {
                ui.add_space(12.0);
                action = Some(Button::new(self.theme).text(label).show(ui));
            }
            ui.add_space(8.0);
        });
        action
    }
}

pub struct Toolbar {
    theme: DbProTheme,
}

impl Toolbar {
    pub fn new(theme: DbProTheme) -> Self {
        Self { theme }
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        Frame {
            fill: self.theme.surface_panel,
            stroke: Stroke::new(1.0, self.theme.border_subtle),
            inner_margin: Margin::symmetric(8.0, 4.0),
            rounding: Rounding::same(6.0),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);
                add_contents(ui)
            })
            .inner
        })
        .inner
    }
}

pub fn toolbar_button(ui: &mut Ui, tooltip: &str, icon: Icon, theme: DbProTheme) -> Response {
    Button::new(theme)
        .icon(icon)
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::IconSm)
        .tooltip(tooltip)
        .show(ui)
}
