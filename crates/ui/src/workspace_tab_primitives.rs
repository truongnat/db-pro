use super::*;

pub(super) struct WorkspaceTabItem<'a> {
    pub(super) selected: bool,
    pub(super) icon: Icon,
    pub(super) title: &'a str,
    pub(super) unsaved: bool,
    pub(super) show_close: bool,
}

pub(super) struct TabChromeAction {
    pub(super) clicked: bool,
    pub(super) close_clicked: bool,
}

pub(super) const TAB_MIN_WIDTH: f32 = 72.0;
pub(super) const TAB_MAX_WIDTH: f32 = 220.0;
pub(super) const TAB_TITLE_MAX_WIDTH: f32 = 160.0;
pub(super) const TAB_HEIGHT: f32 = 28.0;

pub(super) fn truncate_tab_title(
    ui: &egui::Ui,
    title: &str,
    font_id: egui::FontId,
    color: egui::Color32,
    max_width: f32,
) -> String {
    let full_width = ui
        .painter()
        .layout_no_wrap(title.to_owned(), font_id.clone(), color)
        .size()
        .x;
    if full_width <= max_width {
        return title.to_owned();
    }

    let ellipsis_width = ui
        .painter()
        .layout_no_wrap("…".to_owned(), font_id.clone(), color)
        .size()
        .x;
    let mut visible = String::new();
    for character in title.chars() {
        let candidate = format!("{visible}{character}…");
        let candidate_width = ui
            .painter()
            .layout_no_wrap(candidate.clone(), font_id.clone(), color)
            .size()
            .x;
        if candidate_width > max_width.max(ellipsis_width) {
            break;
        }
        visible.push(character);
    }

    format!("{visible}…")
}

pub(super) fn draw_workspace_tab_item(
    ui: &mut egui::Ui,
    theme: DbProTheme,
    item: WorkspaceTabItem<'_>,
    context_menu: impl FnOnce(&mut egui::Ui, &mut bool),
) -> TabChromeAction {
    let font_id = if item.selected { font_ui_label() } else { font_body() };
    let text_color = if item.selected {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    let icon_color = if item.selected { theme.accent } else { theme.text_muted };

    let full_title_galley = ui
        .painter()
        .layout_no_wrap(item.title.to_owned(), font_id.clone(), text_color);
    let close_slot = if item.show_close { 22.0 } else { 0.0 };
    let unsaved_slot = if item.unsaved { 10.0 } else { 0.0 };
    let title_width = full_title_galley.size().x.min(TAB_TITLE_MAX_WIDTH);
    let item_width = (18.0 + title_width + unsaved_slot + close_slot + 18.0).clamp(TAB_MIN_WIDTH, TAB_MAX_WIDTH);
    let title_available_width = item_width - 18.0 - unsaved_slot - close_slot - 18.0;
    let display_title = truncate_tab_title(ui, item.title, font_id.clone(), text_color, title_available_width);
    let title_galley = ui.painter().layout_no_wrap(display_title, font_id, text_color);

    let (rect, resp) = ui.allocate_exact_size(egui::vec2(item_width, TAB_HEIGHT), egui::Sense::click());
    let resp = resp
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(item.title);
    let hovered = resp.hovered();

    let context_clicked = is_context_menu_triggered(&resp, ui);

    // Floating Context menu with Foreground Area z-index
    context_action_menu(ui, &resp, theme, context_menu);

    // Tab Background & Borders
    let rounding = egui::Rounding {
        nw: RADIUS_SM,
        ne: RADIUS_SM,
        sw: 0.0,
        se: 0.0,
    };

    if item.selected {
        ui.painter().rect(
            rect,
            rounding,
            theme.surface_app,
            egui::Stroke::new(STROKE_THIN, theme.border_subtle),
        );
        // Top accent indicator line
        let top_indicator_rect = egui::Rect::from_min_size(rect.left_top(), egui::vec2(rect.width(), 2.0));
        ui.painter()
            .rect_filled(top_indicator_rect, egui::Rounding::same(1.0), theme.accent);
    } else if hovered {
        ui.painter().rect_filled(rect, rounding, theme.surface_hover);
    }

    // Icon
    let icon_pos = egui::pos2(rect.left() + 8.0, rect.center().y);
    ui.painter().text(
        icon_pos,
        egui::Align2::LEFT_CENTER,
        char::from(item.icon).to_string(),
        egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())),
        icon_color,
    );

    // Title text
    let title_pos = egui::pos2(rect.left() + 24.0, rect.center().y - title_galley.size().y * 0.5);
    ui.painter().galley(title_pos, title_galley, text_color);

    // Unsaved dirty dot
    if item.unsaved {
        let dot_pos = egui::pos2(rect.right() - close_slot - 6.0, rect.center().y);
        ui.painter().circle_filled(dot_pos, 2.5, theme.accent);
    }

    // Close Button
    let mut close_clicked = false;
    if item.show_close {
        let close_rect =
            egui::Rect::from_center_size(egui::pos2(rect.right() - 12.0, rect.center().y), egui::vec2(16.0, 16.0));
        let pointer_pos = ui.input(|i| i.pointer.hover_pos().or(i.pointer.interact_pos()));
        let close_hovered = pointer_pos.is_some_and(|p| close_rect.contains(p));

        if close_hovered {
            ui.painter()
                .rect_filled(close_rect, egui::Rounding::same(RADIUS_SM), theme.surface_hover);
        }

        let close_color = if close_hovered {
            theme.danger
        } else if item.selected {
            theme.text_secondary
        } else {
            theme.text_muted
        };

        ui.painter().text(
            close_rect.center(),
            egui::Align2::CENTER_CENTER,
            char::from(Icon::X).to_string(),
            egui::FontId::new(10.5, egui::FontFamily::Name("lucide".into())),
            close_color,
        );

        if resp.clicked() && close_hovered {
            close_clicked = true;
        }
    }

    let middle_clicked = resp.middle_clicked();

    TabChromeAction {
        clicked: resp.clicked() && !close_clicked && !context_clicked,
        close_clicked: close_clicked || (middle_clicked && item.show_close),
    }
}
