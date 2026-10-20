use super::*;
use std::collections::HashMap;

#[derive(Default)]
struct MemoryStorage(HashMap<String, String>);

impl eframe::Storage for MemoryStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.0.insert(key.to_owned(), value);
    }

    fn remove_string(&mut self, key: &str) {
        self.0.remove(key);
    }

    fn flush(&mut self) {}
}

fn render(app: &mut DbProApp, ctx: &egui::Context, events: Vec<egui::Event>) -> egui::FullOutput {
    crate::test_frame::frame(
        ctx,
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 800.0))),
            events,
            ..Default::default()
        },
        |ui| {
            // Match eframe: root Ui already exists when App::logic applies the theme.
            app.prepare_frame(ui.ctx());
            let frame_theme = app.theme;
            app.draw_shell(ui);
            assert_eq!(ui.visuals().dark_mode, frame_theme.dark_mode);
            assert_eq!(ui.visuals().panel_fill, frame_theme.surface_panel);
        },
    )
}

fn click(app: &mut DbProApp, ctx: &egui::Context, label: &str) {
    let output = render(app, ctx, Vec::new());
    let bounds = output
        .platform_output
        .accesskit_update
        .as_ref()
        .expect("accessible shell")
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing control: {label}"));
    let position = egui::pos2(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    let _ = render(app, ctx, vec![egui::Event::PointerMoved(position)]);
    for pressed in [true, false] {
        let _ = render(
            app,
            ctx,
            vec![egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            }],
        );
    }
}

fn assert_theme_persists(app: &mut DbProApp, ctx: &egui::Context, dark_mode: bool) {
    assert_eq!(app.preferences.dark_mode, dark_mode);
    assert_eq!(app.preferences.settings.appearance.dark_mode, dark_mode);
    for _ in 0..3 {
        let output = render(app, ctx, Vec::new());
        assert_eq!(app.theme.dark_mode, dark_mode);
        assert_eq!(
            ctx.theme(),
            if dark_mode {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            }
        );
        assert_eq!(app.preferences.settings.appearance.dark_mode, dark_mode);
        if app.workspace.activity == Activity::Settings {
            let selected = if dark_mode { "Dark" } else { "Light" };
            let node = output
                .platform_output
                .accesskit_update
                .as_ref()
                .expect("accessible settings")
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some(selected))
                .map(|(_, node)| node)
                .expect("selected appearance radio");
            assert_eq!(node.toggled(), Some(egui::accesskit::Toggled::True));
        }
        assert!(
            output.shapes.iter().any(|shape| matches!(&shape.shape,
            egui::epaint::Shape::Rect(rect) if rect.fill == app.theme.surface_app)),
            "shell must paint the selected theme's surface"
        );
    }
    let mut storage = MemoryStorage::default();
    eframe::App::save(app, &mut storage);
    assert_eq!(storage.0.get("dbpro.native.dark-mode"), Some(&dark_mode.to_string()));
    let restored = DbProApp::with_task_bridge_and_storage(TaskBridge::default(), Some(&storage));
    assert_eq!(restored.preferences.dark_mode, dark_mode);
    assert_eq!(restored.preferences.settings.appearance.dark_mode, dark_mode);
    assert_eq!(restored.theme.dark_mode, dark_mode);
}

#[test]
fn toolbar_and_settings_switch_rendered_theme_and_persist_the_same_choice() {
    let ctx = egui::Context::default();
    ctx.enable_accesskit();
    DbProTheme::install_fonts(&ctx);
    // Start with the opposite context style, as happens under a dark system theme.
    ctx.set_theme(egui::Theme::Dark);
    let mut app = DbProApp {
        initial_frames_count: 3,
        ..Default::default()
    };
    assert_theme_persists(&mut app, &ctx, false);

    click(&mut app, &ctx, "Switch to Dark Theme");
    assert_theme_persists(&mut app, &ctx, true);
    click(&mut app, &ctx, "Switch to Light Theme");
    assert_theme_persists(&mut app, &ctx, false);

    app.workspace.activity = Activity::Settings;
    app.preferences.section = settings_model::SettingsSection::Appearance;
    click(&mut app, &ctx, "Dark");
    assert_theme_persists(&mut app, &ctx, true);
    click(&mut app, &ctx, "Light");
    assert_theme_persists(&mut app, &ctx, false);
}

#[test]
fn shell_panel_boundaries_stay_borderless_in_both_themes() {
    for dark_mode in [false, true] {
        for output_open in [false, true] {
            let ctx = egui::Context::default();
            DbProTheme::install_fonts(&ctx);
            let mut app = DbProApp {
                initial_frames_count: 3,
                preferences: PreferencesState {
                    dark_mode,
                    ..Default::default()
                },
                workspace: WorkspaceFeatureState {
                    shell: WorkspaceShellState {
                        agent_open: true,
                        bottom_panel_open: output_open,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            for _ in 0..3 {
                let output = render(&mut app, &ctx, Vec::new());
                for shape in &output.shapes {
                    if let egui::epaint::Shape::Rect(rect) = &shape.shape {
                        let shell_boundary =
                            rect.rect.width() > 1200.0 || (rect.rect.height() > 400.0 && rect.rect.width() < 400.0);
                        if shell_boundary {
                            assert!(
                                rect.stroke.is_empty(),
                                "decorative shell boundary reappeared: {:?}, dark={dark_mode}, output={output_open}",
                                rect.rect
                            );
                        }
                    }
                    if let egui::epaint::Shape::LineSegment { points, stroke } = &shape.shape {
                        if points[0].distance(points[1]) > 400.0 {
                            assert!(stroke.is_empty(), "idle splitter must not paint a decorative line");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn borderless_sidebar_still_highlights_and_drags() {
    for dark_mode in [false, true] {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let mut app = DbProApp {
            initial_frames_count: 3,
            preferences: PreferencesState {
                dark_mode,
                ..Default::default()
            },
            ..Default::default()
        };
        for _ in 0..3 {
            let _ = render(&mut app, &ctx, Vec::new());
        }
        let original_width = app.workspace.sidebar_width;
        let grip = egui::pos2(
            crate::tokens::component::shell::ACTIVITY_RAIL_WIDTH + original_width,
            400.0,
        );
        let mut highlighted = false;
        for _ in 0..2 {
            let output = render(&mut app, &ctx, vec![egui::Event::PointerMoved(grip)]);
            highlighted = output.shapes.iter().any(|shape| {
                matches!(&shape.shape, egui::epaint::Shape::LineSegment { points, stroke }
                    if points[0].distance(points[1]) > 400.0 && stroke.color == app.theme.accent && !stroke.is_empty())
            });
        }
        assert!(highlighted, "hover must reveal the resize affordance");
        let _ = render(
            &mut app,
            &ctx,
            vec![egui::Event::PointerButton {
                pos: grip,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        let target = grip + egui::vec2(40.0, 0.0);
        let _ = render(&mut app, &ctx, vec![egui::Event::PointerMoved(target)]);
        let _ = render(
            &mut app,
            &ctx,
            vec![egui::Event::PointerButton {
                pos: target,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        assert!((app.workspace.sidebar_width - original_width - 40.0).abs() < 1.0);
    }
}
