use crate::components::animation::{self, hover_t, press_scale, press_t};
use crate::components::interact::{button_info, paint_focus_ring};
use crate::components::overlay::Tooltip;
use crate::tokens::{ICON_TEXT_GAP, RADIUS_BUTTON};
use crate::DbProTheme;
use egui::{
    text::{LayoutJob, TextFormat},
    Color32, FontFamily, FontId, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2,
};
use lucide_icons::Icon;
use std::borrow::Cow;

use crate::tokens::component::button as button_contract;

use super::super::config::LINK_UNDERLINE_WIDTH;
use super::super::handlers::{
    centered_content_pos, leading_content_x, ButtonPalette, ButtonSize, ButtonVariant, SizeTokens,
};

pub struct Button<'a> {
    pub(crate) label: Option<Cow<'a, str>>,
    pub(crate) icon: Option<Icon>,
    pub(crate) variant: ButtonVariant,
    pub(crate) size: ButtonSize,
    pub(crate) enabled: bool,
    pub(crate) loading: bool,
    pub(crate) full_width: bool,
    pub(crate) left_aligned: bool,
    pub(crate) access_label: Option<Cow<'a, str>>,
    pub(crate) tooltip: Option<Cow<'a, str>>,
    pub(crate) focusable: bool,
    pub(crate) theme: DbProTheme,
}

struct LoadingLayout {
    rect: Rect,
    fill_color: Color32,
    text_color: Color32,
    border_stroke: Stroke,
    content_w: f32,
    text_galley: Option<std::sync::Arc<egui::Galley>>,
    left_aligned: bool,
    horizontal_padding: f32,
}

struct InteractiveAllocation<'a> {
    button: &'a Button<'a>,
    width: f32,
    min_height: f32,
    accessible_name: &'a str,
}

struct InteractiveLayout<'a> {
    paint_rect: Rect,
    fill: Color32,
    stroke: Stroke,
    galley: &'a std::sync::Arc<egui::Galley>,
    text_color: Color32,
    left_aligned: bool,
    horizontal_padding: f32,
    underline: bool,
}

impl<'a> Button<'a> {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            label: None,
            icon: None,
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            enabled: true,
            loading: false,
            full_width: false,
            left_aligned: false,
            access_label: None,
            tooltip: None,
            focusable: true,
            theme,
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

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    /// Align the button's icon and label to its leading edge.
    pub fn left_aligned(mut self) -> Self {
        self.left_aligned = true;
        self
    }

    pub fn access_label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.access_label = Some(label.into());
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn accessible_name(&self) -> String {
        self.access_label
            .as_deref()
            .or(self.label.as_deref())
            .or(self.tooltip.as_deref())
            .unwrap_or("Button")
            .to_string()
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        // The caller supplies a theme and state flags for this frame. Resolve the
        // shared size first, then choose one response/render path for that state.
        assert!(
            self.icon.is_none()
                || self.label.as_deref().is_some_and(|label| !label.trim().is_empty())
                || self
                    .access_label
                    .as_deref()
                    .is_some_and(|label| !label.trim().is_empty()),
            "Icon-only buttons require an explicit access_label; tooltips are not stable accessible names."
        );
        let tokens = SizeTokens::from_size(self.size);

        // State precedence: disabled → loading. A disabled+loading button renders
        // its disabled state, not the spinner (see `button_contract::shows_loading`).
        if button_contract::shows_loading(self.enabled, self.loading) {
            self.show_loading(ui, &tokens)
        } else {
            self.show_interactive(ui, &tokens)
        }
    }

    fn show_loading(self, ui: &mut Ui, tokens: &SizeTokens) -> Response {
        // Loading uses the variant palette for contrast, but allocates hover-only
        // sense so the returned response cannot trigger the caller's action.
        let palette = ButtonPalette::from_variant(self.variant, self.theme);
        let (loading_text_color, fill_color, border_stroke) = palette.loading_colors(self.variant, self.theme);
        let text_color = if matches!(self.variant, ButtonVariant::Default | ButtonVariant::Destructive) {
            palette.text_on_fill(self.variant, fill_color, self.theme)
        } else {
            loading_text_color
        };

        let (content_w, text_galley) = if let Some(ref text) = self.label {
            let galley = ui.fonts(|fonts| {
                fonts.layout_no_wrap(text.to_string(), FontId::proportional(tokens.font_size), text_color)
            });
            let gap = ICON_TEXT_GAP;
            (tokens.icon_size + gap + galley.size().x, Some(galley))
        } else {
            (tokens.icon_size, None)
        };

        let width = tokens.calculate_width(content_w, self.full_width, ui.available_width());
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, tokens.min_height), Sense::hover());
        // Announce the transient state because a static reduced-motion cue
        // cannot communicate progress through movement.
        response.widget_info(|| button_info(false, &format!("{}, loading", self.accessible_name())));

        // Pass measured geometry to the painter; the spinner and optional label
        // use the same content width that determined the allocation above.
        let layout = LoadingLayout {
            rect,
            fill_color,
            text_color,
            border_stroke,
            content_w,
            text_galley,
            left_aligned: self.left_aligned,
            horizontal_padding: tokens.padding.x,
        };
        self.paint_loading(ui, &layout, tokens);

        response.on_hover_cursor(egui::CursorIcon::Wait)
    }

    fn paint_loading(&self, ui: &mut Ui, layout: &LoadingLayout, tokens: &SizeTokens) {
        let text_color = layout.text_color;
        let rounding = Rounding::same(RADIUS_BUTTON);
        ui.painter().rect_filled(layout.rect, rounding, layout.fill_color);
        if layout.border_stroke != Stroke::NONE {
            ui.painter().rect_stroke(layout.rect, rounding, layout.border_stroke);
        }
        let start_x = leading_content_x(
            layout.left_aligned,
            layout.rect.left(),
            layout.rect.center().x,
            layout.content_w,
            layout.horizontal_padding,
        );
        animation::paint_spinner(
            ui.painter(),
            Pos2::new(start_x + tokens.icon_size * 0.5, layout.rect.center().y),
            (tokens.icon_size - 2.0) * 0.5,
            1.8,
            text_color,
            text_color.linear_multiply(0.25),
            if self.theme.reduce_motion {
                0.0
            } else {
                animation::spinner_angle(ui)
            },
        );
        if let Some(galley) = &layout.text_galley {
            let gap = ICON_TEXT_GAP;
            let text_pos = centered_content_pos(
                start_x + tokens.icon_size + gap,
                layout.rect.center().y,
                galley.size().y,
            );
            ui.painter().galley(text_pos, std::sync::Arc::clone(galley), text_color);
        }
    }

    fn show_interactive(self, ui: &mut Ui, tokens: &SizeTokens) -> Response {
        // First resolve theme colors and measured content, then ask egui for the
        // response that supplies hover, press and focus events for this frame.
        let name = self.accessible_name();
        let palette = if self.enabled {
            ButtonPalette::from_variant(self.variant, self.theme)
        } else {
            ButtonPalette::disabled(self.theme)
        };
        let label_layout =
            ui.fonts(|fonts| fonts.layout_job(button_label_layout_job(&self, tokens, Color32::PLACEHOLDER)));
        let width = tokens.calculate_width(label_layout.size().x, self.full_width, ui.available_width());
        let (rect, response) = allocate_interactive_button(
            ui,
            InteractiveAllocation {
                button: &self,
                width,
                min_height: tokens.min_height,
                accessible_name: &name,
            },
        );
        // The core contract settles competing state flags. Animation uses that
        // decision; the handler turns progress into paint colors and geometry.
        let interaction_state = button_contract::resolve_interaction_state(
            self.enabled,
            self.loading,
            response.is_pointer_button_down_on(),
            response.has_focus(),
            response.hovered(),
        );
        let has_emphasis = matches!(
            interaction_state,
            button_contract::InteractionState::Active
                | button_contract::InteractionState::Focus
                | button_contract::InteractionState::Hover
        );
        let (hover, press) = interactive_animation_state(ui, &response, has_emphasis, self.theme.reduce_motion);

        let (fill, stroke) = palette.resolve_state(hover);
        // Paint the resolved state, then attach focus and tooltip treatment before
        // returning the response to the caller that owns the actual action.
        paint_interactive_surface(
            ui,
            InteractiveLayout {
                paint_rect: pressed_rect(rect, press),
                fill,
                stroke,
                galley: &label_layout,
                text_color: if self.enabled {
                    palette.text_on_fill(self.variant, fill, self.theme)
                } else {
                    palette.text_color
                },
                left_aligned: self.left_aligned,
                horizontal_padding: tokens.padding.x,
                underline: self.variant == ButtonVariant::Link && (hover > 0.01 || response.has_focus()),
            },
        );

        if response.has_focus() {
            paint_focus_ring(ui, rect, RADIUS_BUTTON, self.theme);
        }
        add_tooltip(response, self.tooltip.as_deref(), self.theme)
    }
}

fn allocate_interactive_button(ui: &mut Ui, allocation: InteractiveAllocation<'_>) -> (Rect, Response) {
    let button = allocation.button;
    // Disabled buttons may still be hovered for context, but cannot receive a
    // click or focus event that would activate a caller-owned action.
    let sense = if button.enabled {
        Sense {
            click: true,
            drag: false,
            focusable: button.focusable,
        }
    } else {
        Sense::hover()
    };
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(allocation.width, allocation.min_height), sense);
    response.widget_info(|| button_info(button.enabled, allocation.accessible_name));
    if button.enabled {
        response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    }
    (rect, response)
}

fn interactive_animation_state(ui: &Ui, response: &Response, enabled: bool, reduce_motion: bool) -> (f32, f32) {
    if reduce_motion {
        return (
            if enabled && (response.hovered() || response.has_focus()) {
                1.0
            } else {
                0.0
            },
            if enabled && response.is_pointer_button_down_on() {
                1.0
            } else {
                0.0
            },
        );
    }
    let hover = hover_t(
        ui.ctx(),
        response.id.with("hover"),
        enabled && (response.hovered() || response.has_focus()),
    );
    let press = press_t(
        ui.ctx(),
        response.id.with("press"),
        enabled && response.is_pointer_button_down_on(),
    );
    (hover, press)
}

fn pressed_rect(rect: Rect, press: f32) -> Rect {
    Rect::from_center_size(rect.center(), rect.size() * press_scale(press))
}

fn add_tooltip(response: Response, tooltip_text: Option<&str>, theme: DbProTheme) -> Response {
    let Some(tooltip_text) = tooltip_text else {
        return response;
    };
    Tooltip::new(tooltip_text, theme).show(&response)
}

fn paint_interactive_surface(ui: &mut Ui, layout: InteractiveLayout<'_>) {
    let rounding = Rounding::same(RADIUS_BUTTON);
    ui.painter().rect_filled(layout.paint_rect, rounding, layout.fill);
    if layout.stroke != Stroke::NONE {
        ui.painter().rect_stroke(layout.paint_rect, rounding, layout.stroke);
    }

    let text_x = leading_content_x(
        layout.left_aligned,
        layout.paint_rect.left(),
        layout.paint_rect.center().x,
        layout.galley.size().x,
        layout.horizontal_padding,
    );
    let text_pos = centered_content_pos(text_x, layout.paint_rect.center().y, layout.galley.size().y);
    ui.painter()
        .galley(text_pos, std::sync::Arc::clone(layout.galley), layout.text_color);
    if layout.underline {
        let underline_y = text_pos.y + layout.galley.size().y;
        ui.painter().line_segment(
            [
                Pos2::new(text_pos.x, underline_y),
                Pos2::new(text_pos.x + layout.galley.size().x, underline_y),
            ],
            Stroke::new(LINK_UNDERLINE_WIDTH, layout.text_color),
        );
    }
}

fn button_label_layout_job(button: &Button<'_>, tokens: &SizeTokens, text_color: Color32) -> LayoutJob {
    let mut label_layout = LayoutJob::default();
    if let Some(icon) = button.icon {
        label_layout.append(
            &char::from(icon).to_string(),
            0.0,
            TextFormat {
                font_id: FontId::new(tokens.icon_size, FontFamily::Name("lucide".into())),
                color: text_color,
                ..Default::default()
            },
        );
        if button.label.is_some() {
            label_layout.append("  ", 0.0, TextFormat::default());
        }
    }
    if let Some(ref text) = button.label {
        label_layout.append(
            text.as_ref(),
            0.0,
            TextFormat {
                font_id: FontId::proportional(tokens.font_size),
                color: text_color,
                ..Default::default()
            },
        );
    }
    label_layout
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Visible or explicit labels provide stable accessibility names; a tooltip is only a fallback.
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

    #[test]
    fn icon_only_button_requires_an_explicit_accessible_name_when_rendered() {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let theme = DbProTheme::dark();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let response = Button::new(theme).icon(Icon::X).access_label("Close panel").show(ui);
                assert!(response.rect.is_positive());
            });
        });
    }

    #[test]
    #[should_panic(expected = "Icon-only buttons require an explicit access_label")]
    fn icon_only_button_without_access_label_is_rejected_in_all_builds() {
        let ctx = egui::Context::default();
        let theme = DbProTheme::dark();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                Button::new(theme).icon(Icon::X).tooltip("Close panel").show(ui);
            });
        });
    }
}
