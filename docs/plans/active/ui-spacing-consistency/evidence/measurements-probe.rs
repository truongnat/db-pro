use db_pro_ui::{
    components::{Input, Label, PasswordInput, ResponsiveGrid, Select},
    DbProTheme,
};
use egui::{Color32, Rect, Shape};
use serde_json::{json, Value};
fn collect_rects(shape: &Shape, fill: Color32, out: &mut Vec<Rect>) {
    match shape {
        Shape::Rect(r) if r.fill == fill => out.push(r.rect),
        Shape::Vec(v) => {
            for s in v {
                collect_rects(s, fill, out)
            }
        }
        _ => {}
    }
}
fn main() {
    let mut results = Vec::<Value>::new();
    for context in ["normal", "gallery", "grid"] {
        let grid = context == "grid";
        for control in ["label", "input", "input-clearable", "password", "select"] {
            let ctx = egui::Context::default();
            let theme = DbProTheme::light();
            DbProTheme::install_fonts(&ctx);
            theme.apply(&ctx);
            let mut cell_spacing = egui::Vec2::ZERO;
            let mut label_rect = Rect::NOTHING;
            let mut value = String::from("database");
            let mut visible = false;
            let mut selected = 0;
            let options = vec!["PostgreSQL".to_owned()];
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0))),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.set_width(300.0);
                        ui.set_max_width(300.0);
                        if context == "gallery" {
                            ui.spacing_mut().item_spacing.x = 0.0;
                        }
                        let mut draw = |ui: &mut egui::Ui| {
                            cell_spacing = ui.spacing().item_spacing;
                            match control {
                                "label" => {
                                    label_rect = Label::new("Host", theme).required(true).show(ui).rect;
                                }
                                "input" | "input-clearable" => {
                                    Input::new(&mut value, "", theme)
                                        .clearable(control == "input-clearable")
                                        .width(300.0)
                                        .show(ui);
                                }
                                "password" => {
                                    PasswordInput::new(&mut value, "", &mut visible, theme)
                                        .width(300.0)
                                        .show(ui);
                                }
                                "select" => {
                                    Select::new("probe", &mut selected, &options)
                                        .theme(theme)
                                        .width(300.0)
                                        .show(ui);
                                }
                                _ => {}
                            }
                        };
                        if grid {
                            ResponsiveGrid::new(280.0)
                                .gap(16.0)
                                .max_columns(1)
                                .show(ui, [0], |cell, _| draw(cell));
                        } else {
                            draw(ui);
                        }
                    });
                },
            );
            let mut rects = Vec::new();
            for shape in &output.shapes {
                collect_rects(&shape.shape, theme.surface_editor, &mut rects);
            }
            let frame = rects.first().map(|r| json!({"width":r.width(),"height":r.height()}));
            let star = output.shapes.iter().find_map(|s| match &s.shape {
                Shape::Text(t) if t.galley.text() == "*" => Some(t.pos.x - label_rect.right()),
                _ => None,
            });
            results.push(json!({"context":context,"grid":grid,"control":control,"cell_spacing":[cell_spacing.x,cell_spacing.y],"frame":frame,"label_star_gap":star}));
        }
    }
    println!("{}", serde_json::to_string_pretty(&results).unwrap());
}
