use std::sync::mpsc::{self, Receiver, Sender};

use db_pro_ui::DbProApp;
use muda::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};

const REFRESH_ID: &str = "db-pro.refresh-schema";
const REFRESH_CACHE_ID: &str = "db-pro.refresh-schema-cache";
const NEW_CONNECTION_ID: &str = "db-pro.new-connection";
const NEW_QUERY_ID: &str = "db-pro.new-query";
const HELP_ID: &str = "db-pro.help";
const DOCUMENTATION_URL: &str = "https://github.com/truongnat/db-pro#readme";

#[derive(Clone, Copy)]
enum AppMenuAction {
    NewConnection,
    NewQuery,
    RefreshSchema,
    RefreshSchemaCache,
    OpenHelp,
}

/// Owns the native menu for the lifetime of the application and forwards its
/// database actions to the UI thread.
pub(crate) struct AppMenu {
    _menu: Menu,
    actions_tx: Sender<AppMenuAction>,
    actions_rx: Receiver<AppMenuAction>,
}

impl AppMenu {
    pub(crate) fn install() -> muda::Result<Self> {
        let app_menu = Submenu::with_items(
            "DB Pro",
            true,
            &[
                &PredefinedMenuItem::about(
                    None,
                    Some(AboutMetadata {
                        name: Some("DB Pro".to_owned()),
                        version: Some(env!("CARGO_PKG_VERSION").to_owned()),
                        ..Default::default()
                    }),
                ),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::hide(None),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(None),
            ],
        )?;

        let new_connection = MenuItem::with_id(NEW_CONNECTION_ID, "New Connection", true, None);
        let new_query = MenuItem::with_id(NEW_QUERY_ID, "New Query", true, None);
        let file_menu = Submenu::with_items(
            "File",
            true,
            &[
                &new_connection,
                &new_query,
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::close_window(None),
            ],
        )?;
        let edit_menu = Submenu::with_items(
            "Edit",
            true,
            &[
                &PredefinedMenuItem::undo(None),
                &PredefinedMenuItem::redo(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::cut(None),
                &PredefinedMenuItem::copy(None),
                &PredefinedMenuItem::paste(None),
                &PredefinedMenuItem::select_all(None),
            ],
        )?;

        let refresh = MenuItem::with_id(REFRESH_ID, "Refresh", true, None);
        let refresh_cache = MenuItem::with_id(REFRESH_CACHE_ID, "Refresh Cache", true, None);
        let view_menu = Submenu::with_items(
            "View",
            true,
            &[
                &refresh,
                &refresh_cache,
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::fullscreen(None),
            ],
        )?;
        let window_menu = Submenu::with_items(
            "Window",
            true,
            &[
                &PredefinedMenuItem::minimize(None),
                &PredefinedMenuItem::zoom(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::bring_all_to_front(None),
            ],
        )?;
        let help_item = MenuItem::with_id(HELP_ID, "DB Pro Documentation", true, None);
        let help_menu = Submenu::with_items("Help", true, &[&help_item])?;

        let menu = Menu::with_items(&[&app_menu, &file_menu, &edit_menu, &view_menu, &window_menu, &help_menu])?;
        menu.init_for_nsapp();
        window_menu.set_as_windows_menu_for_nsapp();
        help_menu.set_as_help_menu_for_nsapp();

        let (actions_tx, actions_rx) = mpsc::channel();
        Ok(Self {
            _menu: menu,
            actions_tx,
            actions_rx,
        })
    }

    pub(crate) fn connect_to_ui(&self, context: &eframe::egui::Context) {
        let context = context.clone();
        let actions_tx = self.actions_tx.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let action = if event.id == NEW_CONNECTION_ID {
                Some(AppMenuAction::NewConnection)
            } else if event.id == NEW_QUERY_ID {
                Some(AppMenuAction::NewQuery)
            } else if event.id == HELP_ID {
                Some(AppMenuAction::OpenHelp)
            } else if event.id == REFRESH_ID {
                Some(AppMenuAction::RefreshSchema)
            } else if event.id == REFRESH_CACHE_ID {
                Some(AppMenuAction::RefreshSchemaCache)
            } else {
                None
            };
            if let Some(action) = action {
                let _ = actions_tx.send(action);
                context.request_repaint();
            }
        }));
    }

    pub(crate) fn apply_pending_actions(&self, app: &mut DbProApp) {
        while let Ok(action) = self.actions_rx.try_recv() {
            match action {
                AppMenuAction::NewConnection => app.open_new_connection_from_menu(),
                AppMenuAction::NewQuery => app.open_query_from_menu(),
                AppMenuAction::RefreshSchema => app.refresh_active_schema_from_menu(false),
                AppMenuAction::RefreshSchemaCache => app.refresh_active_schema_from_menu(true),
                AppMenuAction::OpenHelp => {
                    if let Err(error) = std::process::Command::new("open").arg(DOCUMENTATION_URL).spawn() {
                        tracing::warn!(%error, "could not open DB Pro documentation");
                    }
                }
            }
        }
    }
}
