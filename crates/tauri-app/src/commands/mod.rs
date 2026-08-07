pub mod backup;
pub mod connection;
pub mod cross_connection;
pub mod export;
pub mod query;
pub mod schema;
pub mod table_data;
pub mod user_management;

pub use backup::*;
pub use connection::*;
pub use cross_connection::*;
pub use export::*;
pub use query::*;
pub use schema::*;
pub use table_data::*;
pub use user_management::*;

use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn close_splashscreen(app: AppHandle) {
    if let Some(splash) = app.get_webview_window("splashscreen") {
        let _ = splash.close();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
}
