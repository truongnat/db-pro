use crate::components::animation::pulse_alpha;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::DbProTheme;
use egui::{Align2, FontFamily, FontId, Frame, Margin, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2};
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

pub struct Avatar<'a> {
    initials: Option<&'a str>,
    icon: Option<Icon>,
    size: AvatarSize,
    theme: DbProTheme,
}

impl<'a> Avatar<'a> {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            initials: None,
            icon: None,
            size: AvatarSize::Md,
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

    pub fn show(self, ui: &mut Ui) -> Response {
        let size = self.size.px();
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        ui.painter()
            .circle_filled(rect.center(), size * 0.5, self.theme.surface_hover);
        ui.painter()
            .circle_stroke(rect.center(), size * 0.5, Stroke::new(1.0, self.theme.border_subtle));
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
        response
    }
}

pub struct Skeleton {
    width: f32,
    height: f32,
    rounding: f32,
    theme: DbProTheme,
}

impl Skeleton {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            width: 160.0,
            height: 12.0,
            rounding: 6.0,
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

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = if self.width <= 0.0 {
            ui.available_width()
        } else {
            self.width.min(ui.available_width())
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, self.height), Sense::hover());
        let alpha = pulse_alpha(ui, SKELETON_MIN_ALPHA, SKELETON_MAX_ALPHA);
        let fill = self.theme.surface_hover.linear_multiply(alpha);
        ui.painter().rect_filled(rect, Rounding::same(self.rounding), fill);
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
            rounding: Rounding::same(8.0),
            ..Default::default()
        }
        .show(ui, |ui| ui.horizontal(|ui| add_contents(ui)).inner)
        .inner
    }
}

pub fn toolbar_button(ui: &mut Ui, label: &str, icon: Icon, theme: DbProTheme) -> Response {
    Button::new(theme)
        .text(label)
        .icon(icon)
        .size(ButtonSize::Sm)
        .variant(ButtonVariant::Ghost)
        .show(ui)
}
