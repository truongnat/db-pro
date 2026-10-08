// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use crate::components::animation::hover_t;
use crate::components::feedback::kbd_badge;
use crate::DbProTheme;
use egui::{
    Align2, Color32, FontFamily, FontId, Pos2, Response, Rounding, Sense, Stroke, Ui, UiBuilder, Vec2, WidgetInfo,
    WidgetType,
};
use lucide_icons::Icon;

use super::config::{
    EMPTY_STATE_FONT_SIZE, EMPTY_STATE_VERTICAL_SPACE, GROUP_CONTENT_GAP, GROUP_HEADING_SIZE, GROUP_TOP_GAP,
    INPUT_FONT_SIZE, INPUT_HEIGHT, INPUT_ICON_SIZE, INPUT_SEPARATOR_INSET, ITEM_HEIGHT, ITEM_ICON_ADVANCE,
    ITEM_ICON_SIZE, ITEM_LEFT_INSET, ITEM_ROUNDING, ITEM_SUBTITLE_GAP, ITEM_SUBTITLE_SIZE, ITEM_TITLE_SIZE,
};
use super::handler::{
    accessible_item_label, command_input_icon_x, command_input_text_rect, command_text_clip_rect, item_has_background,
    item_highlight_target, item_icon_color, item_is_actionable, item_title_color, shortcut_rect, subtitle_x,
};

/// Search field used above a command list.
pub struct CommandInput<'a> {
    /// Mutable query shared with the caller's filtering state.
    pub query: &'a mut String,
    /// Placeholder shown while the query is empty.
    pub placeholder: &'a str,
    /// Semantic theme used for text and separator colors.
    pub theme: DbProTheme,
}

impl<'a> CommandInput<'a> {
    pub fn new(query: &'a mut String, theme: DbProTheme) -> Self {
        Self {
            query,
            placeholder: "Type a command or search...",
            theme,
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, INPUT_HEIGHT), Sense::click());

        // The icon occupies a fixed leading slot so the text field starts consistently across palettes.
        let icon_x = command_input_icon_x(rect);
        let center_y = rect.center().y;
        ui.painter().text(
            Pos2::new(icon_x, center_y),
            Align2::LEFT_CENTER,
            char::from(Icon::Search).to_string(),
            FontId::new(INPUT_ICON_SIZE, FontFamily::Name("lucide".into())),
            self.theme.text_secondary,
        );

        let text_rect = command_input_text_rect(rect, icon_x);
        let mut child_ui = ui.new_child(UiBuilder::new().max_rect(text_rect));
        let edit_response = egui::TextEdit::singleline(self.query)
            .hint_text(self.placeholder)
            .font(DbProTheme::ui_medium_font(INPUT_FONT_SIZE))
            .text_color(self.theme.text_primary)
            .frame(false)
            .show(&mut child_ui)
            .response;

        let separator_y = rect.bottom() - INPUT_SEPARATOR_INSET;
        ui.painter().line_segment(
            [
                Pos2::new(rect.left(), separator_y),
                Pos2::new(rect.right(), separator_y),
            ],
            Stroke::new(crate::tokens::STROKE_THIN, self.theme.border_subtle),
        );

        // Union both responses so clicking the allocated row or editing its child remains observable.
        response.union(edit_response)
    }
}

/// A visually selectable command row; selection and command dispatch remain caller-owned.
pub struct CommandItem<'a> {
    /// Stable command metadata; it does not currently determine egui response identity.
    pub id: &'a str,
    /// Main command label.
    pub title: &'a str,
    /// Optional secondary text rendered after the title.
    pub subtitle: Option<&'a str>,
    /// Optional leading Lucide icon.
    pub icon: Option<Icon>,
    /// Optional keyboard shortcut badge rendered at the trailing edge.
    pub shortcut: Option<&'a str>,
    /// Disabled rows remain hoverable for context but cannot be clicked.
    pub disabled: bool,
    /// Selected controls visual emphasis only; it does not change response behavior.
    pub selected: bool,
}

impl<'a> CommandItem<'a> {
    pub fn new(id: &'a str, title: &'a str) -> Self {
        Self {
            id,
            title,
            subtitle: None,
            icon: None,
            shortcut: None,
            disabled: false,
            selected: false,
        }
    }

    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Prevents clicks while retaining hover sensing for row context.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Applies selected styling only; callers still own selection and activation state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn show(self, ui: &mut Ui, theme: DbProTheme) -> Response {
        let width = ui.available_width();
        let sense = if item_is_actionable(self.disabled) {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, ITEM_HEIGHT), sense);
        let accessible_label = accessible_item_label(self.title, self.subtitle, self.shortcut);
        response
            .widget_info(|| WidgetInfo::selected(WidgetType::Button, !self.disabled, self.selected, &accessible_label));
        let highlight_target = item_highlight_target(response.hovered(), self.selected, self.disabled);
        let hover_amount = hover_t(ui.ctx(), response.id.with("cmd_hover"), highlight_target);

        if item_has_background(self.selected, self.disabled, hover_amount) {
            let fill = if self.selected && !self.disabled {
                theme.surface_hover
            } else {
                theme.surface_hover.linear_multiply(hover_amount)
            };
            ui.painter().rect_filled(rect, Rounding::same(ITEM_ROUNDING), fill);
        }

        let mut left_x = rect.left() + ITEM_LEFT_INSET;
        let center_y = rect.center().y;
        let highlighted = self.selected || response.hovered();
        let text_clip = command_text_clip_rect(rect, left_x, self.shortcut.is_some());
        let text_painter = ui.painter().with_clip_rect(text_clip);

        if let Some(icon) = self.icon {
            ui.painter().text(
                Pos2::new(left_x, center_y),
                Align2::LEFT_CENTER,
                char::from(icon).to_string(),
                FontId::new(ITEM_ICON_SIZE, FontFamily::Name("lucide".into())),
                item_icon_color(&theme, self.disabled, highlighted),
            );
            left_x += ITEM_ICON_ADVANCE;
        }

        text_painter.text(
            Pos2::new(left_x, center_y),
            Align2::LEFT_CENTER,
            self.title,
            DbProTheme::ui_medium_font(ITEM_TITLE_SIZE),
            item_title_color(&theme, self.disabled, highlighted),
        );

        if let Some(subtitle) = self.subtitle {
            // The galley's color only participates in layout measurement; the painter uses the theme color below.
            let title_width = ui
                .painter()
                .layout_no_wrap(
                    self.title.to_string(),
                    DbProTheme::ui_medium_font(ITEM_TITLE_SIZE),
                    Color32::WHITE,
                )
                .size()
                .x;
            let subtitle_x = subtitle_x(left_x, title_width, ITEM_SUBTITLE_GAP);
            text_painter.text(
                Pos2::new(subtitle_x, center_y),
                Align2::LEFT_CENTER,
                subtitle,
                FontId::proportional(ITEM_SUBTITLE_SIZE),
                theme.text_muted,
            );
        }

        if let Some(shortcut) = self.shortcut {
            let mut shortcut_ui = ui.new_child(UiBuilder::new().max_rect(shortcut_rect(rect)));
            shortcut_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                kbd_badge(ui, shortcut, theme);
            });
        }

        if self.disabled {
            response
        } else {
            response.on_hover_cursor(egui::CursorIcon::PointingHand)
        }
    }
}

/// Adds a heading and vertical spacing around caller-provided command rows.
pub struct CommandGroup<'a> {
    /// Group heading text.
    pub heading: &'a str,
}

impl<'a> CommandGroup<'a> {
    pub fn new(heading: &'a str) -> Self {
        Self { heading }
    }

    pub fn show<R>(self, ui: &mut Ui, theme: DbProTheme, content: impl FnOnce(&mut Ui) -> R) -> R {
        ui.add_space(GROUP_TOP_GAP);
        ui.label(
            egui::RichText::new(self.heading)
                .font(DbProTheme::ui_medium_font(GROUP_HEADING_SIZE))
                .color(theme.text_muted),
        );
        ui.add_space(GROUP_CONTENT_GAP);
        content(ui)
    }
}

/// Message displayed when a command query has no matching rows.
pub struct CommandEmpty<'a> {
    /// Empty-state message text.
    pub text: &'a str,
    /// Semantic theme used for the message color.
    pub theme: DbProTheme,
}

impl<'a> CommandEmpty<'a> {
    pub fn new(theme: DbProTheme) -> Self {
        Self {
            text: "No results found.",
            theme,
        }
    }

    pub fn text(mut self, text: &'a str) -> Self {
        self.text = text;
        self
    }

    pub fn show(self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(EMPTY_STATE_VERTICAL_SPACE);
            ui.label(
                egui::RichText::new(self.text)
                    .font(DbProTheme::ui_medium_font(EMPTY_STATE_FONT_SIZE))
                    .color(self.theme.text_muted),
            );
            ui.add_space(EMPTY_STATE_VERTICAL_SPACE);
        });
    }
}
