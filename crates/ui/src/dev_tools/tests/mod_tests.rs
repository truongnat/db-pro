//! Tests for `../mod.rs` (moved out of the source file).
use super::*;

#[test]
fn inspector_starts_closed_without_env() {
    std::env::remove_var(OPEN_ON_START_ENV);
    assert!(!DevToolsState::default().open);
}

#[test]
fn inspector_window_draws_and_closes() {
    let ctx = egui::Context::default();
    let mut state = open_inspector_state();
    let _ = crate::test_frame::frame(&ctx, egui::RawInput::default(), |ui| {
        state.draw(ui.ctx(), DbProTheme::dark());
    });
    assert!(state.open, "window must stay open while undismissed");

    let mut closed = DevToolsState {
        open: false,
        ..DevToolsState::default()
    };
    let out = crate::test_frame::frame(&ctx, egui::RawInput::default(), |ui| {
        closed.draw(ui.ctx(), DbProTheme::dark());
    });
    assert!(out.shapes.is_empty(), "closed inspector must paint nothing");
}

#[test]
fn pick_env_parses_xy() {
    std::env::set_var(PICK_ENV, "120.5, 64");
    assert_eq!(parse_pick_env(), Some(Pos2::new(120.5, 64.0)));
    std::env::set_var(PICK_ENV, "not-a-pair");
    assert_eq!(parse_pick_env(), None);
    std::env::remove_var(PICK_ENV);
}

/// Open-inspector state for tests — no pending pick, no audit cache.
fn open_inspector_state() -> DevToolsState {
    DevToolsState {
        open: true,
        selected: None,
        pending_pick: None,
        audit: None,
        prev_audit: None,
        audit_dumped: false,
        audit_frames: 0,
        pending_audit: None,
        fixture_painted: false,
        stable_streak: 0,
    }
}

fn plain_input(ctx_size: egui::Vec2) -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, ctx_size)),
        ..Default::default()
    }
}

#[test]
fn draw_populates_audit_report() {
    let ctx = egui::Context::default();
    let mut state = open_inspector_state();
    // Pass 1 registers widgets; pass 2 audits them.
    for _ in 0..2 {
        let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
            let _ = ui.button("target");
            state.draw(ui.ctx(), DbProTheme::dark());
        });
    }
    let report = state.audit.as_ref().expect("draw must run the audit");
    assert_eq!(report.rules.len(), 12, "every registered rule must report");
    assert!(report.widget_count > 0);
}

#[test]
fn audit_fixture_paints_broken_widgets() {
    // The intentional-error fixture must actually feed the engine:
    // overlap + zero-size widgets drawn via paint_audit_fixture land in
    // the next pass's snapshot and produce issues.
    let ctx = egui::Context::default();
    // Fixture paints via Area::show — must run inside a pass (fonts).
    for _ in 0..3 {
        let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
            paint_audit_fixture(ui.ctx());
        });
    }
    let snapshot = GeometrySnapshot::collect(&ctx, inspector_id());
    let report = audit::run_audit(&ctx, &snapshot, None);
    let overlap = report.issues.iter().any(|i| i.rule_id == "overlap.suspicious");
    let zero = report.issues.iter().any(|i| i.rule_id == "interactive.zero_size");
    assert!(overlap, "fixture must produce an overlap issue");
    assert!(zero, "fixture must produce a zero-size issue");
}

#[test]
fn semantic_tree_and_tab_focus_flow_through_capture() {
    // End-to-end: enable accesskit, paint two buttons, press Tab — the
    // captured TreeUpdate must carry nodes for both widgets, a resolvable
    // parent (root), and focus must land on an existing node (or root),
    // which is exactly what `focus.orphaned` audits.
    let ctx = egui::Context::default();
    AccessKitCapture::ensure_installed(&ctx);

    let paint = |ui: &mut egui::Ui| {
        let _ = ui.button("first");
        let _ = ui.button("second");
    };
    for _ in 0..3 {
        let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), paint);
    }
    // Tab keypress moves egui keyboard focus between focusable widgets.
    let _ = crate::test_frame::frame(
        &ctx,
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
            events: vec![egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        },
        paint,
    );

    let update = AccessKitCapture::latest(&ctx).expect("capture plugin must see a tree update");
    let sem = SemanticSnapshot::from_update(&update).expect("update carries nodes");
    // Buttons appear as semantic nodes joined to their widget ids.
    assert!(
        sem.nodes.values().any(|n| n.role == egui::accesskit::Role::Button),
        "buttons must emit Button nodes"
    );
    // Focus is always resolvable: root when nothing focused, else a node.
    if let Some(f) = sem.focus {
        assert!(sem.nodes.contains_key(&f), "focus {:?} must be a real node", f);
    }
    // The audit agrees: no orphaned focus, no missing names (both buttons
    // are labelled by their text).
    let report = audit::run_audit(&ctx, &GeometrySnapshot::collect(&ctx, inspector_id()), Some(&sem));
    assert!(
        report.issues.iter().all(|i| i.rule_id != "focus.orphaned"),
        "focus must not be orphaned"
    );
    assert!(
        report.issues.iter().all(|i| i.rule_id != "semantic.missing_name"),
        "named buttons must not flag"
    );
}

#[test]
fn named_scrollbar_and_textedit_nodes_report_accessible_names() {
    // P15 regression: egui emits anonymous ScrollBar nodes and a plain
    // multiline TextEdit carries no accessible name. The fixes name both;
    // the audit's semantic.missing_name must see the names.
    let ctx = egui::Context::default();
    AccessKitCapture::ensure_installed(&ctx);

    for _ in 0..3 {
        let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
            // Overflowing vertical scroll → a live scroll bar node.
            let scroll = egui::ScrollArea::vertical()
                .id_salt("p15-test-scroll")
                .max_height(50.0)
                .show(ui, |ui| {
                    for i in 0..20 {
                        ui.label(format!("row {i}"));
                    }
                });
            crate::components::scroll_area::name_scroll_bars(ui.ctx(), scroll.id, "Test list");

            let mut text = "select 1".to_owned();
            let resp = ui.add(egui::TextEdit::multiline(&mut text));
            ui.ctx().accesskit_node_builder(resp.id, |b| {
                b.set_label("DDL script editor");
            });
        });
    }

    let update = AccessKitCapture::latest(&ctx).expect("tree update");
    let sem = SemanticSnapshot::from_update(&update).expect("nodes");
    let report = audit::run_audit(&ctx, &GeometrySnapshot::collect(&ctx, inspector_id()), Some(&sem));

    // The named scrollbar + editor must not flag; nothing unnamed should remain.
    let unnamed: Vec<_> = report
        .issues
        .iter()
        .filter(|i| i.rule_id == "semantic.missing_name")
        .map(|i| i.evidence.as_str())
        .collect();
    assert!(unnamed.is_empty(), "named controls still flag: {unnamed:?}");
    // ScrollBar node exists AND carries the name (sanity — the test would
    // also pass if egui stopped emitting bar nodes at all).
    assert!(
        sem.nodes.values().any(|n| n.role == egui::accesskit::Role::ScrollBar
            && n.name.as_deref().is_some_and(|n| n.contains("Test list"))),
        "scroll bar must carry its accessible name"
    );
}

#[test]
fn pointer_press_pins_the_widget_under_the_click() {
    let ctx = egui::Context::default();
    let mut state = open_inspector_state();

    // Paint the button in the same pass the press lands — picks resolve
    // against the previous pass's widgets, so the target must exist there.
    let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
        ui.put(
            egui::Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(80.0, 24.0)),
            egui::Button::new("pick me"),
        );
    });

    // Pass 2: press inside the button while the inspector draws.
    let press = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
        events: vec![
            egui::Event::PointerMoved(Pos2::new(140.0, 112.0)),
            egui::Event::PointerButton {
                pos: Pos2::new(140.0, 112.0),
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
        ..Default::default()
    };
    let _ = crate::test_frame::frame(&ctx, press, |ui| {
        ui.put(
            egui::Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(80.0, 24.0)),
            egui::Button::new("pick me"),
        );
        state.draw(ui.ctx(), DbProTheme::dark());
    });
    let selected = state.selected.as_ref().expect("click must pin a widget");
    assert!(
        selected.picked.senses_click,
        "picked widget should be the clickable button"
    );
    assert!(selected.picked.rect.to_egui().contains(Pos2::new(140.0, 112.0)));
}

#[test]
fn selection_survives_pointer_move_and_escape_clears() {
    let ctx = egui::Context::default();
    let mut state = open_inspector_state();
    state.pending_pick = Some(Pos2::new(140.0, 112.0));

    // Pass 1 paints the target; the pending pick resolves on the next pass
    // once the widget exists in prev_pass.
    let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
        ui.put(
            egui::Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(80.0, 24.0)),
            egui::Button::new("pin me"),
        );
        state.draw(ui.ctx(), DbProTheme::dark());
    });
    assert!(
        state.pending_pick.is_some(),
        "pick must wait for the target to register"
    );
    let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
        ui.put(
            egui::Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(80.0, 24.0)),
            egui::Button::new("pin me"),
        );
        state.draw(ui.ctx(), DbProTheme::dark());
    });
    assert!(state.selected.is_some(), "pending pick must pin the widget");

    // Pass 2: pointer moved away — selection must stay.
    let _ = crate::test_frame::frame(
        &ctx,
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
            events: vec![egui::Event::PointerMoved(Pos2::new(400.0, 400.0))],
            ..Default::default()
        },
        |ui| {
            ui.put(
                egui::Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(80.0, 24.0)),
                egui::Button::new("pin me"),
            );
            state.draw(ui.ctx(), DbProTheme::dark());
        },
    );
    assert!(state.selected.is_some(), "pointer move must not drop the pin");

    // Pass 3: Escape clears the pin.
    let _ = crate::test_frame::frame(
        &ctx,
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
            events: vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            ..Default::default()
        },
        |ui| {
            ui.put(
                egui::Rect::from_min_size(Pos2::new(100.0, 100.0), egui::vec2(80.0, 24.0)),
                egui::Button::new("pin me"),
            );
            state.draw(ui.ctx(), DbProTheme::dark());
        },
    );
    assert!(state.selected.is_none(), "Escape must clear the pin");
}

#[test]
fn inspector_widgets_are_not_pickable() {
    let ctx = egui::Context::default();
    let mut state = open_inspector_state();

    // Pass 1 draws the inspector itself plus a background button.
    let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
        ui.put(
            egui::Rect::from_min_size(Pos2::new(20.0, 20.0), egui::vec2(40.0, 20.0)),
            egui::Button::new("bg"),
        );
        state.draw(ui.ctx(), DbProTheme::dark());
    });

    // Pass 2: pick inside the inspector window's area — selection must
    // not capture an inspector widget.
    state.pending_pick = Some(Pos2::new(200.0, 40.0));
    let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
        state.draw(ui.ctx(), DbProTheme::dark());
    });
    if let Some(sel) = &state.selected {
        assert!(
            sel.picked.layer.id != inspector_id(),
            "inspector widget leaked into the pick: {:?}",
            sel.picked.id_debug
        );
    }
}

#[test]
fn snapshot_picks_the_front_most_of_overlapping_widgets() {
    let ctx = egui::Context::default();

    let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
        ui.put(
            egui::Rect::from_min_size(Pos2::new(50.0, 50.0), egui::vec2(100.0, 100.0)),
            egui::Button::new("under"),
        );
        ui.put(
            egui::Rect::from_min_size(Pos2::new(80.0, 80.0), egui::vec2(100.0, 100.0)),
            egui::Button::new("over"),
        );
    });

    let snapshot = GeometrySnapshot::collect(&ctx, inspector_id());
    let picked = snapshot
        .pick(&ctx, Pos2::new(100.0, 100.0))
        .expect("overlap point must hit");

    // The "over" button was painted last → it wins the shared region.
    // `ui.put` also registers the child Ui on the same rect, so take the
    // last entry, not the first.
    let over = snapshot
        .layers
        .iter()
        .flat_map(|(_, ws)| ws.iter())
        .rfind(|w| w.rect.to_egui() == egui::Rect::from_min_size(Pos2::new(80.0, 80.0), egui::vec2(100.0, 100.0)));
    assert_eq!(Some(picked.id), over.map(|w| w.id), "front-most widget must win");
}

#[test]
fn foreground_popup_wins_over_background_widget() {
    let ctx = egui::Context::default();

    let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
        ui.put(
            egui::Rect::from_min_size(Pos2::new(50.0, 50.0), egui::vec2(200.0, 200.0)),
            egui::Button::new("background"),
        );
        egui::Area::new(egui::Id::new("test_popup"))
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::new(100.0, 100.0))
            .show(ui.ctx(), |ui| {
                ui.add(egui::Button::new("popup item").min_size(egui::vec2(80.0, 24.0)));
            });
    });

    let snapshot = GeometrySnapshot::collect(&ctx, inspector_id());
    let picked = snapshot
        .pick(&ctx, Pos2::new(120.0, 112.0))
        .expect("popup widget must be pickable");
    assert_eq!(
        picked.layer.order,
        egui::Order::Foreground,
        "foreground popup layer must beat the background widget, picked: {:?}",
        picked.layer_debug
    );
}

fn draw_test_scroller(ui: &mut Ui) {
    egui::ScrollArea::vertical()
        .id_salt("geom_scroll_test")
        .max_height(100.0)
        .show(ui, |ui| {
            for i in 0..40 {
                let _ = ui.button(format!("row {i}"));
            }
        });
}

/// The second visible clickable row — `widget_info`/`label` is only
/// recorded for interacted widgets, so locate by sense + position.
fn second_scrolled_row_center(ctx: &egui::Context) -> Pos2 {
    let snapshot = GeometrySnapshot::collect(ctx, inspector_id());
    let mut rows: Vec<_> = snapshot
        .layers
        .iter()
        .flat_map(|(_, ws)| ws.iter())
        .filter(|w| w.senses_click && w.enabled && w.rect.height() > 10.0 && w.rect.height() < 60.0 && !w.clipped)
        .collect();
    rows.sort_by(|a, b| {
        a.rect
            .min_y
            .partial_cmp(&b.rect.min_y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let row = rows.get(1).expect("a visible scrolled row must exist");
    row.to_global_rect(ctx).center()
}

#[test]
fn selection_follows_the_widget_after_scrolling() {
    let ctx = egui::Context::default();
    let mut state = open_inspector_state();

    // Pass 1: draw scroller + inspector.
    let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
        draw_test_scroller(ui);
        state.draw(ui.ctx(), DbProTheme::dark());
    });
    let center = second_scrolled_row_center(&ctx);
    state.pending_pick = Some(center);

    // Pass 2: the pending pick resolves against pass-1 widgets. Picking a
    // row's rect may pin the row's inner Ui or the row itself — the
    // contract is that *a* widget covering the point is pinned.
    let _ = crate::test_frame::frame(&ctx, plain_input(egui::vec2(800.0, 600.0)), |ui| {
        draw_test_scroller(ui);
        state.draw(ui.ctx(), DbProTheme::dark());
    });
    let picked = state.selected.as_ref().expect("pending pick must pin a widget");
    let to_local = ctx.layer_transform_from_global(picked.picked.layer).unwrap_or_default();
    let local_center = to_local * center;
    assert!(
        picked.picked.rect.to_egui().contains(local_center),
        "pinned widget must cover the picked point"
    );
    // Pass 3: scroll down; the pinned widget's live rect must move.
    let _ = crate::test_frame::frame(
        &ctx,
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0))),
            events: vec![egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(0.0, -60.0),
                modifiers: egui::Modifiers::NONE,
                phase: egui::TouchPhase::Move,
            }],
            ..Default::default()
        },
        |ui| {
            draw_test_scroller(ui);
            state.draw(ui.ctx(), DbProTheme::dark());
        },
    );

    if let Some(live) = state.selected.as_ref().and_then(|s| s.live.as_ref()) {
        assert!(
            (live.rect.min_y - state.selected.as_ref().unwrap().picked.rect.min_y).abs() > 1.0 || live.clipped,
            "scrolling must update the pinned widget's live geometry"
        );
    }
    // If the row scrolled out of `prev_pass` entirely, `live` is None —
    // which the detail view surfaces as "widget gone"; both are valid.
}
