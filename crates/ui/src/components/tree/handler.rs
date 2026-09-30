use crate::DbProTheme;
use egui::{Color32, Pos2, Rect, Vec2};
use lucide_icons::Icon;

use super::config::{
    ALPHA_CHANNEL_MAX, CHEVRON_COLLAPSED_HIDE_PROGRESS, CHEVRON_EXPANDED_SHOW_PROGRESS, CHEVRON_HORIZONTAL_NUDGE,
    CHEVRON_SLOT, CONTROL_CENTER_OFFSET, DEFAULT_CONTENT_HEIGHT, DETAIL_INSET, HOVER_VISIBLE_THRESHOLD, ICON_SLOT,
    INDENT_STEP, LEFT_INSET, MIN_CLIP_HEIGHT, REVEAL_VISIBLE_THRESHOLD, ROW_HEIGHT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeNodeKind {
    Server,
    Database,
    Schema,
    Table,
    View,
    Column,
    PrimaryKey,
    ForeignKey,
    Index,
}

impl TreeNodeKind {
    pub fn icon(&self) -> Icon {
        icon_for_kind(*self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RowBackground {
    Selected,
    Hovered(f32),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RowLayout {
    pub chevron_pos: Pos2,
    pub icon_pos: Pos2,
    pub label_pos: Pos2,
    pub detail_pos: Pos2,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ChevronIconLayer {
    pub icon: Icon,
    pub alpha: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RevealClip {
    pub clip_height: f32,
    pub child_height: f32,
}

pub(crate) fn row_height() -> f32 {
    ROW_HEIGHT
}

pub(crate) fn row_background(selected: bool, hover: f32) -> RowBackground {
    if selected {
        return RowBackground::Selected;
    }

    if hover > HOVER_VISIBLE_THRESHOLD {
        RowBackground::Hovered(hover)
    } else {
        RowBackground::None
    }
}

pub(crate) fn row_layout(rect: Rect, depth: usize) -> RowLayout {
    // Tree rows always reserve the chevron slot, even for leaves, so labels keep a stable column
    // while sibling rows expand/collapse. The UI decides whether the chevron glyph is painted.
    let leading_x = rect.left() + depth as f32 * INDENT_STEP + LEFT_INSET;
    let center_y = rect.center().y;
    let icon_slot_x = leading_x + CHEVRON_SLOT;
    let label_x = icon_slot_x + ICON_SLOT;

    RowLayout {
        chevron_pos: Pos2::new(leading_x + CONTROL_CENTER_OFFSET + CHEVRON_HORIZONTAL_NUDGE, center_y),
        icon_pos: Pos2::new(icon_slot_x + CONTROL_CENTER_OFFSET, center_y),
        label_pos: Pos2::new(label_x, center_y),
        detail_pos: Pos2::new(rect.right() - DETAIL_INSET, center_y),
    }
}

pub(crate) fn hover_animation_enabled(hovered: bool, selected: bool) -> bool {
    hovered && !selected
}

pub(crate) fn should_toggle_expanded(clicked: bool, has_expander: bool) -> bool {
    clicked && has_expander
}

pub(crate) fn icon_for_kind(kind: TreeNodeKind) -> Icon {
    match kind {
        TreeNodeKind::Server => Icon::Server,
        TreeNodeKind::Database => Icon::Database,
        TreeNodeKind::Schema => Icon::Folder,
        TreeNodeKind::Table => Icon::Table2,
        TreeNodeKind::View => Icon::Eye,
        TreeNodeKind::Column => Icon::Columns3,
        TreeNodeKind::PrimaryKey => Icon::Key,
        TreeNodeKind::ForeignKey => Icon::Link,
        TreeNodeKind::Index => Icon::Zap,
    }
}

pub(crate) fn icon_color(kind: TreeNodeKind, selected: bool, theme: DbProTheme) -> Color32 {
    match kind {
        TreeNodeKind::PrimaryKey => theme.warning,
        TreeNodeKind::ForeignKey => theme.info,
        _ if selected => theme.accent,
        _ => theme.text_secondary,
    }
}

pub(crate) fn label_color(selected: bool, theme: DbProTheme) -> Color32 {
    if selected {
        theme.accent
    } else {
        theme.text_primary
    }
}

pub(crate) fn chevron_color(hovered: bool, selected: bool, theme: DbProTheme) -> Color32 {
    if hovered || selected {
        theme.text_secondary
    } else {
        theme.text_muted
    }
}

pub(crate) fn chevron_icon_layers(progress: f32) -> [Option<ChevronIconLayer>; 2] {
    let progress = progress.clamp(0.0, 1.0);
    [
        (progress < CHEVRON_COLLAPSED_HIDE_PROGRESS).then_some(ChevronIconLayer {
            icon: Icon::ChevronRight,
            alpha: alpha_from_unit(1.0 - progress),
        }),
        (progress > CHEVRON_EXPANDED_SHOW_PROGRESS).then_some(ChevronIconLayer {
            icon: Icon::ChevronDown,
            alpha: alpha_from_unit(progress),
        }),
    ]
}

pub(crate) fn reveal_clip(last_height: Option<f32>, progress: f32) -> Option<RevealClip> {
    if progress <= REVEAL_VISIBLE_THRESHOLD {
        return None;
    }

    let content_height = last_height.unwrap_or(DEFAULT_CONTENT_HEIGHT).max(MIN_CLIP_HEIGHT);
    let clip_height = (content_height * progress).max(MIN_CLIP_HEIGHT);
    Some(RevealClip {
        clip_height,
        child_height: content_height.max(clip_height),
    })
}

pub(crate) fn child_clip_rect(rect: Rect, width: f32, height: f32) -> Rect {
    Rect::from_min_size(rect.min, Vec2::new(width, height))
}

pub(crate) fn measured_child_height(height: f32) -> f32 {
    height.max(MIN_CLIP_HEIGHT)
}

fn alpha_from_unit(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * ALPHA_CHANNEL_MAX) as u8
}

#[cfg(test)]
mod tests {
    use super::super::config::{CHEVRON_ANIMATION_ID_SALT, CHILDREN_HEIGHT_ID_SALT, HOVER_ANIMATION_ID_SALT};
    use super::*;

    #[test]
    fn node_kind_icon_mapping_preserves_database_glyphs() {
        assert_eq!(char::from(TreeNodeKind::Server.icon()), char::from(Icon::Server));
        assert_eq!(char::from(TreeNodeKind::Database.icon()), char::from(Icon::Database));
        assert_eq!(char::from(TreeNodeKind::Schema.icon()), char::from(Icon::Folder));
        assert_eq!(char::from(TreeNodeKind::Table.icon()), char::from(Icon::Table2));
        assert_eq!(char::from(TreeNodeKind::View.icon()), char::from(Icon::Eye));
        assert_eq!(char::from(TreeNodeKind::Column.icon()), char::from(Icon::Columns3));
        assert_eq!(char::from(TreeNodeKind::PrimaryKey.icon()), char::from(Icon::Key));
        assert_eq!(char::from(TreeNodeKind::ForeignKey.icon()), char::from(Icon::Link));
        assert_eq!(char::from(TreeNodeKind::Index.icon()), char::from(Icon::Zap));
    }

    #[test]
    fn interaction_decisions_ignore_selected_hover_and_leaf_clicks() {
        assert!(hover_animation_enabled(true, false));
        assert!(!hover_animation_enabled(true, true));
        assert!(!hover_animation_enabled(false, false));

        assert_eq!(row_background(true, 0.0), RowBackground::Selected);
        assert_eq!(row_background(false, HOVER_VISIBLE_THRESHOLD), RowBackground::None);
        let visible_hover = HOVER_VISIBLE_THRESHOLD + 0.001;
        assert_eq!(
            row_background(false, visible_hover),
            RowBackground::Hovered(visible_hover)
        );

        assert!(should_toggle_expanded(true, true));
        assert!(!should_toggle_expanded(true, false));
        assert!(!should_toggle_expanded(false, true));
    }

    #[test]
    fn row_layout_preserves_legacy_positions() {
        let rect = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(300.0, ROW_HEIGHT));
        let layout = row_layout(rect, 2);

        assert_eq!(layout.chevron_pos, Pos2::new(52.0, 33.0));
        assert_eq!(layout.icon_pos, Pos2::new(67.0, 33.0));
        assert_eq!(layout.label_pos, Pos2::new(77.0, 33.0));
        assert_eq!(layout.detail_pos, Pos2::new(300.0, 33.0));
    }

    #[test]
    fn chevron_layers_crossfade_with_legacy_thresholds() {
        let collapsed = chevron_icon_layers(0.0);
        assert_eq!(
            collapsed[0].map(|layer| (char::from(layer.icon), layer.alpha)),
            Some((char::from(Icon::ChevronRight), 255))
        );
        assert!(collapsed[1].is_none());

        let mid = chevron_icon_layers(0.5);
        assert_eq!(
            mid[0].map(|layer| (char::from(layer.icon), layer.alpha)),
            Some((char::from(Icon::ChevronRight), 127))
        );
        assert_eq!(
            mid[1].map(|layer| (char::from(layer.icon), layer.alpha)),
            Some((char::from(Icon::ChevronDown), 127))
        );

        let expanded = chevron_icon_layers(1.0);
        assert!(expanded[0].is_none());
        assert_eq!(
            expanded[1].map(|layer| (char::from(layer.icon), layer.alpha)),
            Some((char::from(Icon::ChevronDown), 255))
        );
    }

    #[test]
    fn reveal_clip_sanitizes_missing_and_tiny_heights() {
        assert_eq!(reveal_clip(Some(48.0), REVEAL_VISIBLE_THRESHOLD), None);
        assert_eq!(
            reveal_clip(None, 0.5),
            Some(RevealClip {
                clip_height: 24.0,
                child_height: 48.0
            })
        );
        assert_eq!(
            reveal_clip(Some(-10.0), 0.5),
            Some(RevealClip {
                clip_height: 1.0,
                child_height: 1.0
            })
        );
        assert_eq!(measured_child_height(0.0), 1.0);
    }

    #[test]
    fn animation_id_salts_are_stable_for_existing_state() {
        assert_eq!(HOVER_ANIMATION_ID_SALT, "hover");
        assert_eq!(CHEVRON_ANIMATION_ID_SALT, "chev_anim");
        assert_eq!(CHILDREN_HEIGHT_ID_SALT, "content_h");
    }
}
