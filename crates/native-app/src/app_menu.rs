use std::sync::mpsc::{self, Receiver, Sender};

use db_pro_ui::DbProApp;
use eframe::egui;
use muda::accelerator::{Accelerator, Code, Modifiers};
use muda::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};

const REFRESH_ID: &str = "db-pro.refresh-schema";
const REFRESH_CACHE_ID: &str = "db-pro.refresh-schema-cache";
const NEW_CONNECTION_ID: &str = "db-pro.new-connection";
const NEW_QUERY_ID: &str = "db-pro.new-query";
const HELP_ID: &str = "db-pro.help";
const EDIT_UNDO_ID: &str = "db-pro.edit-undo";
const EDIT_REDO_ID: &str = "db-pro.edit-redo";
const EDIT_CUT_ID: &str = "db-pro.edit-cut";
const EDIT_COPY_ID: &str = "db-pro.edit-copy";
const EDIT_PASTE_ID: &str = "db-pro.edit-paste";
const EDIT_SELECT_ALL_ID: &str = "db-pro.edit-select-all";
const DOCUMENTATION_URL: &str = "https://github.com/truongnat/db-pro#readme";

#[derive(Clone, Copy)]
enum AppMenuAction {
    NewConnection,
    NewQuery,
    RefreshSchema,
    RefreshSchemaCache,
    OpenHelp,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
}

fn app_submenu() -> muda::Result<Submenu> {
    Submenu::with_items(
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
    )
}

fn file_submenu() -> muda::Result<Submenu> {
    Submenu::with_items(
        "File",
        true,
        &[
            &MenuItem::with_id(NEW_CONNECTION_ID, "New Connection", true, None),
            &MenuItem::with_id(NEW_QUERY_ID, "New Query", true, None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::close_window(None),
        ],
    )
}

// Custom items, not PredefinedMenuItem: a predefined item's key equivalent is
// handled by macOS as an AppKit selector (copy:, selectAll:) that the winit
// view does not implement, so the keystroke was swallowed and egui never saw
// it. A plain MenuItem still intercepts the key but dispatches through
// muda's MenuEvent, which apply_pending_actions turns into a synthetic egui
// event.
fn edit_submenu() -> muda::Result<Submenu> {
    let cmd = Modifiers::META;
    let item = |id: &str, text: &str, mods: Modifiers, key: Code| {
        MenuItem::with_id(id, text, true, Some(Accelerator::new(mods, key)))
    };
    Submenu::with_items(
        "Edit",
        true,
        &[
            &item(EDIT_UNDO_ID, "Undo", cmd, Code::KeyZ),
            &item(EDIT_REDO_ID, "Redo", cmd | Modifiers::SHIFT, Code::KeyZ),
            &PredefinedMenuItem::separator(),
            &item(EDIT_CUT_ID, "Cut", cmd, Code::KeyX),
            &item(EDIT_COPY_ID, "Copy", cmd, Code::KeyC),
            &item(EDIT_PASTE_ID, "Paste", cmd, Code::KeyV),
            &item(EDIT_SELECT_ALL_ID, "Select All", cmd, Code::KeyA),
        ],
    )
}

fn view_submenu() -> muda::Result<Submenu> {
    Submenu::with_items(
        "View",
        true,
        &[
            &MenuItem::with_id(REFRESH_ID, "Refresh", true, None),
            &MenuItem::with_id(REFRESH_CACHE_ID, "Refresh Cache", true, None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::fullscreen(None),
        ],
    )
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
        let app_menu = app_submenu()?;
        let file_menu = file_submenu()?;
        let edit_menu = edit_submenu()?;
        let view_menu = view_submenu()?;
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
            if let Some(action) = menu_action_for(&event.id) {
                let _ = actions_tx.send(action);
                context.request_repaint();
            }
        }));
    }

    pub(crate) fn apply_pending_actions(&self, app: &mut DbProApp, ctx: &egui::Context) {
        while let Ok(action) = self.actions_rx.try_recv() {
            match action {
                AppMenuAction::NewConnection => app.open_new_connection_from_menu(),
                AppMenuAction::NewQuery => app.open_query_from_menu(),
                AppMenuAction::RefreshSchema => app.refresh_active_schema_from_menu(false),
                AppMenuAction::RefreshSchemaCache => app.refresh_active_schema_from_menu(true),
                AppMenuAction::OpenHelp => open_documentation(),
                action => push_edit_event(ctx, action),
            }
        }
    }
}

fn open_documentation() {
    if let Err(error) = std::process::Command::new("open").arg(DOCUMENTATION_URL).spawn() {
        tracing::warn!(%error, "could not open DB Pro documentation");
    }
}

/// Maps a menu item id to its action; `None` for ids the app ignores.
fn menu_action_for(id: &muda::MenuId) -> Option<AppMenuAction> {
    Some(match id.as_ref() {
        NEW_CONNECTION_ID => AppMenuAction::NewConnection,
        NEW_QUERY_ID => AppMenuAction::NewQuery,
        HELP_ID => AppMenuAction::OpenHelp,
        REFRESH_ID => AppMenuAction::RefreshSchema,
        REFRESH_CACHE_ID => AppMenuAction::RefreshSchemaCache,
        EDIT_UNDO_ID => AppMenuAction::Undo,
        EDIT_REDO_ID => AppMenuAction::Redo,
        EDIT_CUT_ID => AppMenuAction::Cut,
        EDIT_COPY_ID => AppMenuAction::Copy,
        EDIT_PASTE_ID => AppMenuAction::Paste,
        EDIT_SELECT_ALL_ID => AppMenuAction::SelectAll,
        _ => return None,
    })
}

/// Menu-triggered edit commands replay as synthetic egui events so the
/// focused widget handles them through the same path as real keystrokes.
fn push_edit_event(ctx: &egui::Context, action: AppMenuAction) {
    let command = egui::Modifiers {
        command: true,
        ..Default::default()
    };
    let key_event = |key: egui::Key, modifiers: egui::Modifiers| egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    };
    let event = match action {
        AppMenuAction::Undo => key_event(egui::Key::Z, command),
        AppMenuAction::Redo => key_event(egui::Key::Z, egui::Modifiers { shift: true, ..command }),
        AppMenuAction::Cut => egui::Event::Cut,
        AppMenuAction::Copy => egui::Event::Copy,
        AppMenuAction::Paste => {
            let Ok(mut clipboard) = arboard::Clipboard::new() else {
                return;
            };
            match clipboard.get_text() {
                Ok(text) if !text.is_empty() => egui::Event::Paste(text),
                _ => return,
            }
        }
        AppMenuAction::SelectAll => key_event(egui::Key::A, command),
        _ => return,
    };
    ctx.input_mut(|input| input.events.push(event));
    ctx.request_repaint();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_all_pushes_command_a_key_event() {
        let ctx = egui::Context::default();
        push_edit_event(&ctx, AppMenuAction::SelectAll);
        let pushed = ctx.input(|input| {
            input.events.iter().any(|event| {
                matches!(
                    event,
                    egui::Event::Key {
                        key: egui::Key::A,
                        pressed: true,
                        modifiers,
                        ..
                    } if modifiers.command
                )
            })
        });
        assert!(pushed, "Select All must inject a command+A key event");
    }

    #[test]
    fn copy_pushes_egui_copy_event() {
        let ctx = egui::Context::default();
        push_edit_event(&ctx, AppMenuAction::Copy);
        assert!(ctx.input(|input| input.events.iter().any(|event| matches!(event, egui::Event::Copy))));
    }

    #[test]
    fn redo_pushes_shift_command_z() {
        let ctx = egui::Context::default();
        push_edit_event(&ctx, AppMenuAction::Redo);
        let pushed = ctx.input(|input| {
            input.events.iter().any(|event| {
                matches!(
                    event,
                    egui::Event::Key {
                        key: egui::Key::Z,
                        pressed: true,
                        modifiers,
                        ..
                    } if modifiers.command && modifiers.shift
                )
            })
        });
        assert!(pushed, "Redo must inject shift+command+Z");
    }
}
