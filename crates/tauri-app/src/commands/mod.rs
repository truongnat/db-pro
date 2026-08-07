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

/// Show the main window first, then close the splash screen. Errors are
/// propagated to the frontend so it knows whether the handoff succeeded.
pub(crate) fn finish_startup_inner(app: &AppHandle) -> Result<(), String> {
    let main = app
        .get_webview_window("main")
        .ok_or_else(|| "main window missing".to_string())?;

    main.show().map_err(|e| e.to_string())?;
    main.set_focus().map_err(|e| e.to_string())?;

    if let Some(splash) = app.get_webview_window("splashscreen") {
        splash.close().map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn finish_startup(app: AppHandle) -> Result<(), String> {
    finish_startup_inner(&app)
}
