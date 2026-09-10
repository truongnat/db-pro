use db_pro_ui::DbProApp;
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("DB Pro")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1024.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DB Pro",
        options,
        Box::new(|_creation_context| Ok(Box::new(DbProApp::default()))),
    )
}
