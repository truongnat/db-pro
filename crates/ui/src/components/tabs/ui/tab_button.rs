use egui::{text::LayoutJob, text::TextFormat, Color32, CornerRadius, FontId, Response, Sense, Ui};
use lucide_icons::Icon;

use super::super::config::{TAB_BUTTON_PAD_X, TAB_BUTTON_PAD_Y, TAB_BUTTON_WASH_INSET, TAB_BUTTON_WASH_RADIUS};
use crate::{components::common::icon_layout, tokens::FONT_SIZE_UI_LABEL, DbProTheme};

/// Content tab: active and hover share the `surface_hover` wash and primary text.
// cc-scan:allow TOO_MANY_PARAMS — context params passed through
pub fn tab_button(ui: &mut Ui, theme: DbProTheme, icon: Option<Icon>, label: &str, selected: bool) -> Response {
    fn job(icon: Option<Icon>, label: &str, color: Color32) -> LayoutJob {
        match icon {
            Some(i) => icon_layout(i, label, color),
            None => {
                let mut j = LayoutJob::default();
                j.append(
                    label,
                    0.0,
                    TextFormat {
                        font_id: FontId::proportional(FONT_SIZE_UI_LABEL),
                        color,
                        ..Default::default()
                    },
                );
                j
            }
        }
    }
    let pad = egui::vec2(TAB_BUTTON_PAD_X, TAB_BUTTON_PAD_Y);
    let probe = ui.fonts_mut(|f| f.layout_job(job(icon, label, theme.text_secondary)));
    let (rect, resp) = ui.allocate_exact_size(probe.size() + pad * 2.0, Sense::click());
    resp.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, ui.is_enabled(), selected, label)
    });
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);

    let color = if selected || resp.hovered() {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    if selected || resp.hovered() {
        ui.painter().rect_filled(
            rect.shrink(TAB_BUTTON_WASH_INSET),
            CornerRadius::same(TAB_BUTTON_WASH_RADIUS as u8),
            theme.surface_hover,
        );
    }
    let galley = ui.fonts_mut(|f| f.layout_job(job(icon, label, color)));
    ui.painter().galley(rect.min + pad, galley, color);
    resp
}
