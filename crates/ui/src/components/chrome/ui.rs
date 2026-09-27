use crate::components::animation::pulse_alpha;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::tokens::STROKE_THIN;
use crate::DbProTheme;
use egui::{
    Align2, FontFamily, FontId, Frame, Margin, Response, RichText, Rounding, Sense, Stroke, Ui, Vec2, WidgetInfo,
    WidgetType,
};
use lucide_icons::Icon;

use super::config::{
    EMPTY_STATE_ACTION_GAP, EMPTY_STATE_BOTTOM_SPACE, EMPTY_STATE_DESCRIPTION_SIZE, EMPTY_STATE_ICON_GAP,
    EMPTY_STATE_ICON_SIZE, EMPTY_STATE_TITLE_GAP, EMPTY_STATE_TITLE_SIZE, EMPTY_STATE_TOP_SPACE,
    SKELETON_DEFAULT_HEIGHT, SKELETON_DEFAULT_ROUNDING, SKELETON_DEFAULT_WIDTH, SKELETON_SHIMMER_ALPHA,
    TOOLBAR_ITEM_GAP, TOOLBAR_MARGIN_X, TOOLBAR_MARGIN_Y, TOOLBAR_ROUNDING,
};
use super::handler::{
    avatar_icon_font_size, avatar_initials_font_size, avatar_rounding_radius, avatar_status_color, avatar_status_dot,
    resolve_skeleton_height, resolve_skeleton_rounding, resolve_skeleton_width, skeleton_alpha_range,
    visible_skeleton_shimmer_rect, AvatarShape, AvatarSize, AvatarStatus,
};

pub struct Avatar<'a> {
    initials: Option<&'a str>,
    icon: Option<Icon>,
    size: AvatarSize,
    shape: AvatarShape,
    status: Option<AvatarStatus>,
    access_label: Option<&'a str>,
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
            access_label: None,
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

    pub fn access_label(mut self, label: &'a str) -> Self {
        self.access_label = Some(label);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let size = self.size.px();
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        let accessible_label = self
            .access_label
            .filter(|label| !label.trim().is_empty())
            .or(self.initials)
            .unwrap_or("Avatar");
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, accessible_label));
        let rounding = Rounding::same(avatar_rounding_radius(self.size, self.shape));

        ui.painter().rect_filled(rect, rounding, self.theme.surface_hover);
        ui.painter()
            .rect_stroke(rect, rounding, Stroke::new(STROKE_THIN, self.theme.border_subtle));

        // Initials stay primary when both forms are configured so avatars remain text-identifiable.
        if let Some(initials) = self.initials {
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                initials,
                FontId::proportional(avatar_initials_font_size(self.size)),
                self.theme.text_primary,
            );
        } else if let Some(icon) = self.icon {
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                char::from(icon).to_string(),
                FontId::new(avatar_icon_font_size(self.size), FontFamily::Name("lucide".into())),
                self.theme.text_secondary,
            );
        }

        if let Some(status) = self.status {
            let dot = avatar_status_dot(self.size, rect);
            let dot_color = avatar_status_color(status, &self.theme);

            // The surface-colored ring keeps the status dot legible on filled avatar backgrounds.
            ui.painter()
                .circle_filled(dot.center, dot.ring_radius, self.theme.surface_panel);
            ui.painter().circle_filled(dot.center, dot.radius, dot_color);
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
            width: SKELETON_DEFAULT_WIDTH,
            height: SKELETON_DEFAULT_HEIGHT,
            rounding: SKELETON_DEFAULT_ROUNDING,
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
        let width = resolve_skeleton_width(self.width, ui.available_width());
        let height = resolve_skeleton_height(self.height);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let alpha = skeleton_alpha_range();
        let fill = self
            .theme
            .surface_hover
            .linear_multiply(pulse_alpha(ui, alpha.min, alpha.max));
        let rounding = Rounding::same(resolve_skeleton_rounding(self.rounding));

        ui.painter().rect_filled(rect, rounding, fill);

        if self.shimmer {
            let time = ui.input(|input| input.time);
            if let Some(clipped) = visible_skeleton_shimmer_rect(rect, height, time) {
                let shimmer_fill = self.theme.surface_elevated.linear_multiply(SKELETON_SHIMMER_ALPHA);
                ui.painter().rect_filled(clipped, rounding, shimmer_fill);
            }
            // Animated placeholders must keep repainting even when no direct input arrives.
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
            ui.add_space(EMPTY_STATE_TOP_SPACE);
            ui.label(
                RichText::new(char::from(self.icon).to_string())
                    .font(FontId::new(EMPTY_STATE_ICON_SIZE, FontFamily::Name("lucide".into())))
                    .color(self.theme.text_tertiary),
            );
            ui.add_space(EMPTY_STATE_ICON_GAP);
            ui.label(
                RichText::new(self.title)
                    .size(EMPTY_STATE_TITLE_SIZE)
                    .strong()
                    .color(self.theme.text_primary),
            );
            ui.add_space(EMPTY_STATE_TITLE_GAP);
            ui.add(
                egui::Label::new(
                    RichText::new(self.description)
                        .size(EMPTY_STATE_DESCRIPTION_SIZE)
                        .color(self.theme.text_secondary),
                )
                .wrap(),
            );
            if let Some(label) = self.action_label {
                ui.add_space(EMPTY_STATE_ACTION_GAP);
                action = Some(Button::new(self.theme).text(label).show(ui));
            }
            ui.add_space(EMPTY_STATE_BOTTOM_SPACE);
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
            stroke: Stroke::new(STROKE_THIN, self.theme.border_subtle),
            inner_margin: Margin::symmetric(TOOLBAR_MARGIN_X, TOOLBAR_MARGIN_Y),
            rounding: Rounding::same(TOOLBAR_ROUNDING),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(TOOLBAR_ITEM_GAP, 0.0);
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
        .access_label(tooltip)
        .tooltip(tooltip)
        .show(ui)
}
