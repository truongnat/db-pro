//! Tests for `../handler.rs` (moved out of the source file).
use super::{
    super::config::{CHEVRON_ANIMATION_ID_SALT, CHILDREN_HEIGHT_ID_SALT, HOVER_ANIMATION_ID_SALT},
    *,
};

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
