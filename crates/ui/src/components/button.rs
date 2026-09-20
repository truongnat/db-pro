use crate::components::animation::{self, hover_t, lerp_color, press_scale, press_t};
use crate::components::interact::{button_info, paint_focus_ring};
use crate::components::overlay::Tooltip;
use crate::DbProTheme;
use egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontFamily, FontId, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2,
};
use lucide_icons::Icon;
use std::borrow::Cow;

const BUTTON_ROUNDING: f32 = 6.0;
const ICON_TEXT_GAP: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Default,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Sm,
    Default,
    Lg,
    Icon,
    IconSm,
}

pub struct Button<'a> {
    pub(crate) label: Option<Cow<'a, str>>,
    pub(crate) icon: Option<Icon>,
    pub(crate) variant: ButtonVariant,
    pub(crate) size: ButtonSize,
    pub(crate) enabled: bool,
    pub(crate) loading: bool,
    pub(crate) full_width: bool,
    pub(crate) access_label: Option<Cow<'a, str>>,
    pub(crate) tooltip: Option<Cow<'a, str>>,
    pub(crate) focusable: bool,
    pub(crate) theme: DbProTheme,
}

struct SizeTokens {
    min_height: f32,
    font_size: f32,
    icon_size: f32,
    padding: Vec2,
    default_width: f32,
}

impl SizeTokens {
    pub fn calculate_width(&self, content_width: f32, full_width: bool, available_width: f32) -> f32 {
        if full_width {
            available_width
        } else {
            self.default_width.max(content_width + self.padding.x * 2.0)
        }
    }
}

struct LoadingLayout {
    rect: Rect,
    fill_color: Color32,
    border_stroke: Stroke,
    content_w: f32,
    text_galley: Option<std::sync::Arc<egui::Galley>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonPalette {
    pub fill_rest: Color32,
    pub fill_hover: Color32,
    pub stroke_rest: Stroke,
    pub stroke_hover: Stroke,
    pub text_color: Color32,
}

impl ButtonPalette {
    pub fn from_variant(variant: ButtonVariant, theme: DbProTheme) -> Self {
        match variant {
            ButtonVariant::Default => Self {
                fill_rest: theme.accent,
                fill_hover: theme.accent_hover,
                stroke_rest: Stroke::NONE,
                stroke_hover: Stroke::NONE,
                text_color: theme.accent_foreground,
            },
            ButtonVariant::Secondary => Self {
                fill_rest: theme.surface_hover,
                fill_hover: theme.surface_active,
                stroke_rest: Stroke::NONE,
                stroke_hover: Stroke::NONE,
                text_color: theme.text_primary,
            },
            ButtonVariant::Outline => Self {
                fill_rest: Color32::TRANSPARENT,
                fill_hover: theme.surface_hover,
                stroke_rest: Stroke::new(1.0, theme.border_default),
                stroke_hover: Stroke::new(1.0, theme.border_strong),
                text_color: theme.text_primary,
            },
            ButtonVariant::Ghost => Self {
                fill_rest: Color32::TRANSPARENT,
                fill_hover: theme.surface_hover,
                stroke_rest: Stroke::NONE,
                stroke_hover: Stroke::new(1.0, theme.border_subtle),
                text_color: theme.text_secondary,
            },
            ButtonVariant::Destructive => Self {
                fill_rest: theme.danger,
                fill_hover: theme.danger.linear_multiply(0.85),
                stroke_rest: Stroke::NONE,
                stroke_hover: Stroke::NONE,
                text_color: theme.text_inverse,
            },
            ButtonVariant::Link => Self {
                fill_rest: Color32::TRANSPARENT,
                fill_hover: Color32::TRANSPARENT,
                stroke_rest: Stroke::NONE,
                stroke_hover: Stroke::NONE,
                text_color: theme.accent,
            },
        }
    }

    pub fn resolve_state(&self, hover: f32) -> (Color32, Stroke) {
        let fill = lerp_color(self.fill_rest, self.fill_hover, hover);
        let stroke_color = lerp_color(self.stroke_rest.color, self.stroke_hover.color, hover);
        let stroke = if self.stroke_rest == Stroke::NONE && self.stroke_hover == Stroke::NONE {
            Stroke::NONE
        } else {
            Stroke::new(1.0, stroke_color)
        };
        (fill, stroke)
    }

    pub fn loading_colors(&self, variant: ButtonVariant, theme: DbProTheme) -> (Color32, Color32, Stroke) {
        match variant {
            ButtonVariant::Default => (theme.accent_foreground, theme.accent, Stroke::NONE),
            ButtonVariant::Secondary => (theme.text_primary, theme.surface_hover, Stroke::NONE),
            ButtonVariant::Outline => (
                theme.text_primary,
                Color32::TRANSPARENT,
                Stroke::new(1.0, theme.border_default),
            ),
            ButtonVariant::Ghost => (theme.text_secondary, Color32::TRANSPARENT, Stroke::NONE),
            ButtonVariant::Destructive => (theme.text_inverse, theme.danger, Stroke::NONE),
            ButtonVariant::Link => (theme.accent, Color32::TRANSPARENT, Stroke::NONE),
        }
    }
}

impl<'a> Button<'a> {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            label: None,
            icon: None,
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            theme,
            enabled: true,
            tooltip: None,
            focusable: true,
            loading: false,
            full_width: false,
            access_label: None,
        }
    }

    pub fn text(mut self, text: impl Into<Cow<'a, str>>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    pub fn access_label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.access_label = Some(label.into());
        self
    }

    fn size_tokens(&self) -> SizeTokens {
        match self.size {
            ButtonSize::Sm => SizeTokens {
                min_height: 26.0,
                font_size: 11.5,
                icon_size: 13.0,
                padding: egui::vec2(8.0, 3.0),
                default_width: 0.0,
            },
            ButtonSize::Default => SizeTokens {
                min_height: 32.0,
                font_size: 13.0,
                icon_size: 14.5,
                padding: egui::vec2(12.0, 5.0),
                default_width: 0.0,
            },
            ButtonSize::Lg => SizeTokens {
                min_height: 38.0,
                font_size: 14.5,
                icon_size: 16.0,
                padding: egui::vec2(16.0, 7.0),
                default_width: 0.0,
            },
            ButtonSize::Icon => SizeTokens {
                min_height: 32.0,
                font_size: 13.0,
                icon_size: 16.0,
                padding: egui::vec2(6.0, 6.0),
                default_width: 32.0,
            },
            ButtonSize::IconSm => SizeTokens {
                min_height: 24.0,
                font_size: 11.5,
                icon_size: 13.0,
                padding: egui::vec2(4.0, 4.0),
                default_width: 24.0,
            },
        }
    }

    fn accessible_name(&self) -> Cow<'_, str> {
        self.access_label
            .as_deref()
            .or(self.label.as_deref())
            .or(self.tooltip.as_deref())
            .map(Cow::Borrowed)
            .unwrap_or(Cow::Borrowed("Button"))
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let tokens = self.size_tokens();
        if self.loading {
            return self.show_loading(ui, &tokens);
        }
        self.show_interactive(ui, &tokens)
    }

    fn show_loading(self, ui: &mut Ui, tokens: &SizeTokens) -> Response {
        let palette = ButtonPalette::from_variant(self.variant, self.theme);
        let (text_color, fill_color, border_stroke) = palette.loading_colors(self.variant, self.theme);

        let text_galley = self.label.as_ref().map(|txt| {
            ui.painter().layout_no_wrap(
                txt.as_ref().to_owned(),
                FontId::proportional(tokens.font_size),
                text_color,
            )
        });

        let gap = if text_galley.is_some() { ICON_TEXT_GAP } else { 0.0 };
        let text_w = text_galley.as_ref().map_or(0.0, |g| g.size().x);
        let content_w = tokens.icon_size + gap + text_w;
        let width = tokens.calculate_width(content_w, self.full_width, ui.available_width());

        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, tokens.min_height), Sense::hover());
        response.widget_info(|| button_info(false, &self.accessible_name()));

        let layout = LoadingLayout {
            rect,
            fill_color,
            border_stroke,
            content_w,
            text_galley,
        };
        self.paint_loading(ui, &layout, tokens);

        response.on_hover_cursor(egui::CursorIcon::Wait)
    }

    fn paint_loading(&self, ui: &mut Ui, layout: &LoadingLayout, tokens: &SizeTokens) {
        let palette = ButtonPalette::from_variant(self.variant, self.theme);
        let (text_color, _, _) = palette.loading_colors(self.variant, self.theme);
        let rounding = Rounding::same(BUTTON_ROUNDING);
        ui.painter().rect_filled(layout.rect, rounding, layout.fill_color);
        if layout.border_stroke != Stroke::NONE {
            ui.painter().rect_stroke(layout.rect, rounding, layout.border_stroke);
        }
        let start_x = layout.rect.center().x - layout.content_w * 0.5;
        animation::paint_spinner(
            ui.painter(),
            Pos2::new(start_x + tokens.icon_size * 0.5, layout.rect.center().y),
            (tokens.icon_size - 2.0) * 0.5,
            1.8,
            text_color,
            text_color.linear_multiply(0.25),
            animation::spinner_angle(ui),
        );
        if let Some(galley) = &layout.text_galley {
            let gap = ICON_TEXT_GAP;
            let text_pos = Pos2::new(
                start_x + tokens.icon_size + gap,
                layout.rect.center().y - galley.size().y * 0.5,
            );
            ui.painter()
                .galley(text_pos, std::sync::Arc::clone(galley), Color32::PLACEHOLDER);
        }
    }

    fn show_interactive(self, ui: &mut Ui, tokens: &SizeTokens) -> Response {
        let name = self.accessible_name();
        let palette = ButtonPalette::from_variant(self.variant, self.theme);
        let job = content_job(&self, tokens, palette.text_color);
        let galley = ui.fonts(|fonts| fonts.layout_job(job));
        let width = tokens.calculate_width(galley.size().x, self.full_width, ui.available_width());

        let sense = if self.enabled {
            Sense {
                click: true,
                drag: false,
                focusable: self.focusable,
            }
        } else {
            Sense::hover()
        };
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, tokens.min_height), sense);
        response.widget_info(|| button_info(self.enabled, &name));
        if self.enabled {
            response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
        }

        let hover = hover_t(
            ui.ctx(),
            response.id.with("hover"),
            self.enabled && (response.hovered() || response.has_focus()),
        );
        let press = press_t(
            ui.ctx(),
            response.id.with("press"),
            self.enabled && response.is_pointer_button_down_on(),
        );

        let (fill, stroke) = palette.resolve_state(hover);
        let scale = press_scale(press);
        let paint_rect = Rect::from_center_size(rect.center(), rect.size() * scale);

        paint_interactive_surface(ui, paint_rect, fill, stroke, &galley, palette.text_color);

        if response.has_focus() {
            paint_focus_ring(ui, rect, BUTTON_ROUNDING, self.theme);
        }

        if let Some(ref tooltip_text) = self.tooltip {
            response = Tooltip::new(tooltip_text.as_ref(), self.theme).show(&response);
        }

        response
    }
}

fn paint_interactive_surface(
    ui: &mut Ui,
    paint_rect: Rect,
    fill: Color32,
    stroke: Stroke,
    galley: &std::sync::Arc<egui::Galley>,
    text_color: Color32,
) {
    let rounding = Rounding::same(BUTTON_ROUNDING);
    ui.painter().rect_filled(paint_rect, rounding, fill);
    if stroke != Stroke::NONE {
        ui.painter().rect_stroke(paint_rect, rounding, stroke);
    }

    let text_pos = Pos2::new(
        paint_rect.center().x - galley.size().x * 0.5,
        paint_rect.center().y - galley.size().y * 0.5,
    );
    ui.painter().galley(text_pos, std::sync::Arc::clone(galley), text_color);
}

fn content_job(button: &Button<'_>, tokens: &SizeTokens, text_color: Color32) -> LayoutJob {
    let mut job = LayoutJob::default();
    if let Some(icon) = button.icon {
        job.append(
            &char::from(icon).to_string(),
            0.0,
            TextFormat {
                font_id: FontId::new(tokens.icon_size, FontFamily::Name("lucide".into())),
                color: text_color,
                ..Default::default()
            },
        );
        if button.label.is_some() {
            job.append("  ", 0.0, TextFormat::default());
        }
    }
    if let Some(ref text) = button.label {
        job.append(
            text.as_ref(),
            0.0,
            TextFormat {
                font_id: FontId::proportional(tokens.font_size),
                color: text_color,
                ..Default::default()
            },
        );
    }
    job
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Icon-only controls carry no visible text, so the name a screen reader receives
    /// comes from `access_label`, the visible label, or the tooltip. The tooltip arm is
    /// what stops the app's toolbar from announcing every control as "Button".
    #[test]
    fn accessible_name_prefers_access_label_then_label_then_tooltip() {
        let theme = DbProTheme::dark();

        let icon_only_with_tooltip = Button::new(theme).icon(Icon::X).tooltip("Close Agent");
        assert_eq!(icon_only_with_tooltip.accessible_name(), "Close Agent");

        let labelled = Button::new(theme).icon(Icon::X).text("Close").tooltip("Close Agent");
        assert_eq!(labelled.accessible_name(), "Close");

        let explicit = Button::new(theme)
            .icon(Icon::X)
            .text("Close")
            .access_label("Dismiss the agent panel")
            .tooltip("Close Agent");
        assert_eq!(explicit.accessible_name(), "Dismiss the agent panel");
    }

    #[test]
    fn a_button_with_no_name_at_all_still_has_one() {
        let theme = DbProTheme::dark();
        assert_eq!(Button::new(theme).accessible_name(), "Button");
    }
}
