//! Text layout helpers shared by labels, tabs, and nav chrome.

use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId};
use lucide_icons::Icon;

use crate::tokens::{font_icon, FONT_SIZE_UI_LABEL, ICON_SM};

/// Layout an icon glyph followed by a two-space-separated label as one job.
/// Empty `label` yields an icon-only job.
pub(crate) fn icon_layout(icon: Icon, label: &str, color: Color32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.append(
        &char::from(icon).to_string(),
        0.0,
        TextFormat {
            font_id: font_icon(ICON_SM),
            color,
            ..Default::default()
        },
    );
    if !label.is_empty() {
        job.append(
            &format!("  {label}"),
            0.0,
            TextFormat {
                font_id: FontId::proportional(FONT_SIZE_UI_LABEL),
                color,
                ..Default::default()
            },
        );
    }
    job
}

pub fn icon_text(icon: Icon, label: &str, color: Color32) -> LayoutJob {
    icon_layout(icon, label, color)
}
