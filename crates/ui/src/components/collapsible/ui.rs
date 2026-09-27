use super::config::{
    BADGE_TEXT_OFFSET_X, BADGE_TEXT_OFFSET_Y, CHEVRON_ADVANCE, CONTENT_MARGIN, HEADER_HEIGHT, HEADER_PAD_LEFT,
    HEADER_TITLE_FONT_SIZE, HOVER_ALPHA_MULTIPLIER, ICON_ADVANCE, OPEN_ANIMATION_SECONDS, OPEN_CONTENT_THRESHOLD,
};
use super::handler::{apply_header_click, calculate_badge_rect, chevron_icon, resolve_header_colors};
use crate::components::animation::hover_t;
use crate::components::interact::paint_focus_ring;
use crate::tokens::{font_icon, FONT_SIZE_BADGE, ICON_SM, ICON_TEXT_GAP, RADIUS_MD, RADIUS_SM};
use crate::DbProTheme;
use egui::{Align2, FontId, Id, Pos2, Rect, Response, Rounding, Sense, Ui, Vec2, WidgetInfo, WidgetType};
use lucide_icons::Icon;

/// An interactive disclosure widget that reveals or conceals custom nested content.
pub struct Collapsible<'a> {
    open: &'a mut bool,
    title: Option<&'a str>,
    icon: Option<Icon>,
    badge: Option<&'a str>,
    disabled: bool,
    stable_id: Option<Id>,
    theme: DbProTheme,
}

impl<'a> Collapsible<'a> {
    /// Creates a new `Collapsible` builder bound to a mutable boolean toggle state.
    pub fn new(open: &'a mut bool, theme: DbProTheme) -> Self {
        Self {
            open,
            title: None,
            icon: None,
            badge: None,
            disabled: false,
            stable_id: None,
            theme,
        }
    }

    /// Sets the label text displayed on the header row.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Sets an optional leading Lucide icon displayed between the chevron and title.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets an optional trailing badge pill text.
    pub fn badge(mut self, badge: &'a str) -> Self {
        self.badge = Some(badge);
        self
    }

    /// Disables user interaction and renders the header in muted disabled styling.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a unique ID for focus and animation state, recommended for dynamic lists.
    pub fn id(mut self, id: Id) -> Self {
        self.stable_id = Some(id);
        self
    }

    /// Shows the collapsible header and conditionally renders the content closure when open.
    ///
    /// Flow: allocate one header ID, translate pointer or focused keyboard input into the
    /// caller-owned open state, publish the accessible expanded state, then paint the header
    /// and reveal content while the open animation is active. A caller-supplied ID must be
    /// unique among sibling disclosures so focus and animation state do not collide.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> (Response, Option<R>) {
        let (id, rect) = self.allocate_header(ui);
        let response = self.interact_with_header(ui, id, rect);
        let open_anim_t = ui
            .ctx()
            .animate_bool_with_time(id.with("open_anim"), *self.open, OPEN_ANIMATION_SECONDS);
        let keyboard_toggle = if self.disabled || !response.has_focus() {
            false
        } else {
            ui.input_mut(|input| {
                [egui::Key::Space, egui::Key::Enter]
                    .into_iter()
                    .any(|key| input.consume_key(egui::Modifiers::NONE, key))
            })
        };
        apply_header_click(self.open, response.clicked() || keyboard_toggle, self.disabled);
        response.widget_info(|| {
            WidgetInfo::selected(
                WidgetType::CollapsingHeader,
                !self.disabled,
                *self.open,
                self.title.unwrap_or("Collapsible"),
            )
        });

        self.paint_header_state(ui, id, &response);
        let (icon_color, title_color) = resolve_header_colors(&self.theme, self.disabled, response.hovered());
        let layout = HeaderLayout {
            rect,
            center_y: rect.center().y,
            left_x: rect.left() + HEADER_PAD_LEFT,
            theme: self.theme,
            icon_color,
            title_color,
            icon: self.icon,
            title: self.title,
            badge: self.badge,
            open_anim_t,
        };
        layout.paint(ui);

        let content_res = self.paint_content(ui, open_anim_t, content);
        (response, content_res)
    }

    fn allocate_header(&self, ui: &mut Ui) -> (Id, egui::Rect) {
        let (allocated_id, rect) = ui.allocate_space(Vec2::new(ui.available_width(), HEADER_HEIGHT));
        (self.stable_id.unwrap_or(allocated_id), rect)
    }

    fn interact_with_header(&self, ui: &mut Ui, id: Id, rect: egui::Rect) -> Response {
        let sense = if self.disabled { Sense::hover() } else { Sense::click() };
        ui.interact(rect, id, sense)
    }

    fn paint_header_state(&self, ui: &mut Ui, id: Id, response: &Response) {
        let rect = response.rect;
        let hover = hover_t(ui.ctx(), id.with("hover"), response.hovered() && !self.disabled);
        if hover > 0.001 {
            let fill = self.theme.surface_hover.linear_multiply(hover * HOVER_ALPHA_MULTIPLIER);
            ui.painter().rect_filled(rect, Rounding::same(RADIUS_SM), fill);
        }
        if response.has_focus() && !self.disabled {
            paint_focus_ring(ui, rect, RADIUS_SM, self.theme);
        }
    }

    fn paint_content<R>(&self, ui: &mut Ui, open_anim_t: f32, content: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        if open_anim_t <= OPEN_CONTENT_THRESHOLD {
            return None;
        }
        let mut result = None;
        egui::Frame::none().inner_margin(CONTENT_MARGIN).show(ui, |ui| {
            ui.set_opacity(open_anim_t);
            result = Some(content(ui));
        });
        result
    }
}

struct HeaderLayout<'a> {
    rect: egui::Rect,
    center_y: f32,
    left_x: f32,
    theme: DbProTheme,
    icon_color: egui::Color32,
    title_color: egui::Color32,
    icon: Option<Icon>,
    title: Option<&'a str>,
    badge: Option<&'a str>,
    open_anim_t: f32,
}

impl<'a> HeaderLayout<'a> {
    fn paint(mut self, ui: &mut Ui) {
        self.paint_icon(ui, chevron_icon(self.open_anim_t), CHEVRON_ADVANCE);
        if let Some(icon) = self.icon {
            self.paint_icon(ui, icon, ICON_ADVANCE);
        }

        let badge_layout = self.badge.map(|badge| {
            let galley = ui.painter().layout_no_wrap(
                badge.to_owned(),
                FontId::proportional(FONT_SIZE_BADGE),
                self.theme.text_muted,
            );
            let rect = calculate_badge_rect(self.rect, self.center_y, galley.size().x);
            (rect, galley)
        });
        if let Some(title) = self.title {
            let title_right = badge_layout
                .as_ref()
                .map_or(self.rect.right() - HEADER_PAD_LEFT, |(badge_rect, _)| {
                    badge_rect.left() - ICON_TEXT_GAP
                });
            let title_clip = Rect::from_min_max(
                Pos2::new(self.left_x, self.rect.top()),
                Pos2::new(title_right.max(self.left_x), self.rect.bottom()),
            );
            ui.painter().with_clip_rect(title_clip).text(
                Pos2::new(self.left_x, self.center_y),
                Align2::LEFT_CENTER,
                title,
                DbProTheme::ui_medium_font(HEADER_TITLE_FONT_SIZE),
                self.title_color,
            );
        }
        if let Some((badge_rect, galley)) = badge_layout {
            self.paint_badge(ui, badge_rect, galley);
        }
    }

    fn paint_icon(&mut self, ui: &mut Ui, icon: Icon, advance: f32) {
        ui.painter().text(
            Pos2::new(self.left_x, self.center_y),
            Align2::LEFT_CENTER,
            char::from(icon).to_string(),
            font_icon(ICON_SM),
            self.icon_color,
        );
        self.left_x += advance;
    }

    fn paint_badge(&self, ui: &mut Ui, badge_rect: egui::Rect, galley: std::sync::Arc<egui::Galley>) {
        ui.painter().with_clip_rect(self.rect).rect_filled(
            badge_rect,
            Rounding::same(RADIUS_MD),
            self.theme.surface_hover,
        );
        ui.painter().with_clip_rect(badge_rect).galley(
            Pos2::new(
                badge_rect.left() + BADGE_TEXT_OFFSET_X,
                badge_rect.top() + BADGE_TEXT_OFFSET_Y,
            ),
            galley,
            self.theme.text_muted,
        );
    }
}
