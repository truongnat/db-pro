//! Typed native settings model with versioned persistence (#205).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const SETTINGS_STORAGE_KEY: &str = "dbpro.native.settings-v1";
pub(crate) const SETTINGS_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SettingsSection {
    #[default]
    General,
    Appearance,
    Editor,
    DataGrid,
    Connections,
    Ai,
    Keybindings,
    Backup,
    Security,
    Advanced,
}

impl SettingsSection {
    pub(crate) fn label_key(self) -> &'static str {
        match self {
            SettingsSection::General => "settings.section_general",
            SettingsSection::Appearance => "settings.section_appearance",
            SettingsSection::Editor => "settings.section_editor",
            SettingsSection::DataGrid => "settings.section_data_grid",
            SettingsSection::Connections => "settings.section_connections",
            SettingsSection::Ai => "settings.section_ai",
            SettingsSection::Keybindings => "settings.section_keybindings",
            SettingsSection::Backup => "settings.section_backup",
            SettingsSection::Security => "settings.section_security",
            SettingsSection::Advanced => "settings.section_advanced",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct AppSettings {
    pub version: u32,
    pub general: GeneralSettings,
    pub appearance: AppearanceSettings,
    pub editor: EditorSettings,
    pub data_grid: DataGridSettings,
    pub connections: ConnectionSettings,
    pub ai: AiSettings,
    pub keybindings: KeybindingSettings,
    pub security: SecuritySettings,
    pub advanced: AdvancedSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            general: GeneralSettings::default(),
            appearance: AppearanceSettings::default(),
            editor: EditorSettings::default(),
            data_grid: DataGridSettings::default(),
            connections: ConnectionSettings::default(),
            ai: AiSettings::default(),
            keybindings: KeybindingSettings::default(),
            security: SecuritySettings::default(),
            advanced: AdvancedSettings::default(),
        }
    }
}

impl AppSettings {
    /// Migrate older blobs (or missing version) up to [`SETTINGS_VERSION`].
    pub(crate) fn migrate(mut self) -> Self {
        if self.version < 2 {
            // SELECT * is how you look at a table. The warning used to be on by default.
            self.editor.lint.select_star = false;
        }
        self.version = SETTINGS_VERSION;
        self.editor.font_size = self.editor.font_size.clamp(10.0, 24.0);
        self.editor.tab_width = self.editor.tab_width.clamp(2, 8);
        self.data_grid.page_size = self.data_grid.page_size.clamp(25, 1_000);
        self
    }

    pub(crate) fn from_json(raw: &str) -> Option<Self> {
        serde_json::from_str::<AppSettings>(raw).ok().map(Self::migrate)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct GeneralSettings {
    pub confirm_destructive_queries: bool,
    pub restore_tabs_on_startup: bool,
    #[serde(default)]
    pub language: crate::UiLanguage,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            confirm_destructive_queries: true,
            restore_tabs_on_startup: true,
            language: crate::UiLanguage::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct AppearanceSettings {
    pub dark_mode: bool,
    pub reduce_motion: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct EditorSettings {
    pub font_size: f32,
    pub tab_width: u8,
    pub format_on_save: bool,
    pub completion_enabled: bool,
    /// Serialized PredictionMode label: off | subtle | eager
    pub prediction_mode: String,
    /// Deterministic SQL lint rules (#257).
    #[serde(default)]
    pub lint: SqlLintSettings,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            tab_width: 2,
            format_on_save: false,
            completion_enabled: true,
            prediction_mode: "off".to_owned(),
            lint: SqlLintSettings::default(),
        }
    }
}

/// Per-rule SQL lint toggles and suppressions (#257).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SqlLintSettings {
    /// Master switch — when false, no lint diagnostics are emitted.
    pub enabled: bool,
    pub select_star: bool,
    pub null_compare: bool,
    pub delete_no_where: bool,
    pub update_no_where: bool,
    pub order_by_ordinal: bool,
    pub comma_join: bool,
    pub duplicate_alias: bool,
    /// Explicitly suppressed rule codes (e.g. `lint.select-star`).
    #[serde(default)]
    pub suppressed_codes: BTreeSet<String>,
}

impl Default for SqlLintSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            select_star: false,
            null_compare: true,
            delete_no_where: true,
            update_no_where: true,
            order_by_ordinal: true,
            comma_join: true,
            duplicate_alias: true,
            suppressed_codes: BTreeSet::new(),
        }
    }
}

impl SqlLintSettings {
    pub(crate) fn allows(&self, code: &str) -> bool {
        if !self.enabled || self.suppressed_codes.contains(code) {
            return false;
        }
        match code {
            "lint.select-star" => self.select_star,
            "lint.null-compare" => self.null_compare,
            "lint.delete-no-where" => self.delete_no_where,
            "lint.update-no-where" => self.update_no_where,
            "lint.order-by-ordinal" => self.order_by_ordinal,
            "lint.comma-join" => self.comma_join,
            "lint.duplicate-alias" => self.duplicate_alias,
            _ => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct DataGridSettings {
    pub page_size: u32,
    pub show_row_numbers: bool,
    pub wrap_cell_text: bool,
}

impl Default for DataGridSettings {
    fn default() -> Self {
        Self {
            page_size: 100,
            show_row_numbers: true,
            wrap_cell_text: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct ConnectionSettings {
    pub auto_connect_last: bool,
    pub default_ssl_prefer: bool,
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            auto_connect_last: false,
            default_ssl_prefer: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct AiSettings {
    pub enabled: bool,
    pub auto_run_read_only: bool,
    /// Provider label only — API keys stay in secret storage.
    pub provider_label: String,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_run_read_only: false,
            provider_label: "Offline draft".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct KeybindingSettings {
    /// command_id → shortcut token (e.g. "mod+k"). Empty map means all defaults.
    pub overrides: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SecuritySettings {
    pub redact_secrets_in_logs: bool,
    pub lock_secret_export: bool,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            redact_secrets_in_logs: true,
            lock_secret_export: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct AdvancedSettings {
    pub verbose_runtime_log: bool,
    pub experimental_features: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KeybindingCommand {
    pub id: &'static str,
    pub title: &'static str,
    pub default_shortcut: &'static str,
}

pub(crate) fn default_keybinding_catalog() -> &'static [KeybindingCommand] {
    &[
        KeybindingCommand {
            id: "palette.quick_open",
            title: "Quick Open",
            default_shortcut: "mod+p",
        },
        KeybindingCommand {
            id: "palette.commands",
            title: "Command Palette",
            default_shortcut: "mod+shift+p",
        },
        KeybindingCommand {
            id: "palette.quick_open_alt",
            title: "Quick Open (alternate)",
            default_shortcut: "mod+k",
        },
        KeybindingCommand {
            id: "view.toggle_sidebar",
            title: "Toggle Sidebar",
            default_shortcut: "mod+b",
        },
        KeybindingCommand {
            id: "editor.find",
            title: "Find in Editor",
            default_shortcut: "mod+f",
        },
        KeybindingCommand {
            id: "query.save",
            title: "Save Query",
            default_shortcut: "mod+s",
        },
        KeybindingCommand {
            id: "query.save_as",
            title: "Save Query As",
            default_shortcut: "mod+shift+s",
        },
        KeybindingCommand {
            id: "query.run",
            title: "Run Query",
            // Ctrl/Cmd+Enter is the IDE-conventional run chord (toolbar advertises
            // it); F5 stays a hardcoded alias in `handle_catalog_shortcuts`.
            default_shortcut: "mod+enter",
        },
        KeybindingCommand {
            id: "query.run_all",
            title: "Run Script",
            default_shortcut: "mod+shift+enter",
        },
        KeybindingCommand {
            id: "query.format",
            title: "Format Query",
            default_shortcut: "mod+shift+f",
        },
        KeybindingCommand {
            id: "connection.new",
            title: "New Connection",
            default_shortcut: "mod+n",
        },
        KeybindingCommand {
            id: "query.new",
            title: "New Query",
            default_shortcut: "mod+t",
        },
        KeybindingCommand {
            id: "table.open",
            title: "Open Table",
            default_shortcut: "mod+o",
        },
        KeybindingCommand {
            id: "result.focus",
            title: "Focus Results",
            default_shortcut: "mod+shift+r",
        },
        KeybindingCommand {
            id: "editor.comment",
            title: "Toggle Comment",
            default_shortcut: "mod+/",
        },
    ]
}

impl KeybindingSettings {
    pub(crate) fn resolved(&self, command_id: &str) -> String {
        if let Some(over) = self.overrides.get(command_id) {
            return over.clone();
        }
        default_keybinding_catalog()
            .iter()
            .find(|cmd| cmd.id == command_id)
            .map(|cmd| cmd.default_shortcut.to_owned())
            .unwrap_or_default()
    }

    pub(crate) fn reset_all(&mut self) {
        self.overrides.clear();
    }

    pub(crate) fn reset_one(&mut self, command_id: &str) {
        self.overrides.remove(command_id);
    }

    /// Saves one override. An empty draft or the catalog default clears it.
    /// A token the matcher cannot parse is refused and left unchanged.
    pub(crate) fn save_override(&mut self, command_id: &str, default_shortcut: &str, draft: &str) -> bool {
        let draft = draft.trim().to_ascii_lowercase();
        if draft.is_empty() || draft == default_shortcut {
            self.reset_one(command_id);
            return true;
        }
        if shortcut_parts(&draft).is_none() {
            return false;
        }
        self.overrides.insert(command_id.to_owned(), draft);
        true
    }

    /// Returns shortcut tokens that map to more than one command.
    pub(crate) fn conflicts(&self) -> Vec<(String, Vec<String>)> {
        // Group by canonical parsed chord (key + modifiers) instead of raw string
        // so aliases like "ctrl+k" vs "mod+k" or "cmd+k" detect conflicts (#453).
        type CanonicalChord = (egui::Key, bool, bool, bool);
        type ConflictGroup = (String, Vec<String>);
        let mut by_canonical: BTreeMap<CanonicalChord, ConflictGroup> = BTreeMap::new();
        for cmd in default_keybinding_catalog() {
            let shortcut = self.resolved(cmd.id);
            let Some(parts) = shortcut_parts(&shortcut) else {
                continue;
            };
            let key = (parts.key, parts.want_mod, parts.want_shift, parts.want_alt);
            by_canonical
                .entry(key)
                .or_insert_with(|| (shortcut.to_ascii_lowercase(), Vec::new()))
                .1
                .push(cmd.id.to_owned());
        }
        by_canonical
            .into_values()
            .filter(|(_, ids)| ids.len() > 1)
            .collect()
    }

    pub(crate) fn conflict_ids(&self) -> BTreeSet<String> {
        self.conflicts().into_iter().flat_map(|(_, ids)| ids).collect()
    }
}

struct ShortcutParts {
    key: egui::Key,
    want_mod: bool,
    want_shift: bool,
    want_alt: bool,
}

fn shortcut_parts(token: &str) -> Option<ShortcutParts> {
    let token = token.trim().to_ascii_lowercase();
    if token.is_empty() {
        return None;
    }
    let parts: Vec<&str> = token.split('+').map(str::trim).collect();
    let mut want_mod = false;
    let mut want_shift = false;
    let mut want_alt = false;
    let mut key_name = "";
    for part in parts {
        match part {
            "mod" | "cmd" | "ctrl" | "command" => want_mod = true,
            "shift" => want_shift = true,
            "alt" => want_alt = true,
            other => key_name = other,
        }
    }
    Some(ShortcutParts {
        key: parse_key_name(key_name)?,
        want_mod,
        want_shift,
        want_alt,
    })
}

fn modifiers_match(modifiers: egui::Modifiers, parts: &ShortcutParts) -> bool {
    let primary = modifiers.command || modifiers.ctrl;
    parts.want_mod == primary && parts.want_shift == modifiers.shift && parts.want_alt == modifiers.alt
}

/// Match a compact shortcut token (`mod+k`, `mod+shift+p`, `f5`) against egui input.
///
/// Reads modifiers from the key event, not `input.modifiers`: a fast tap can
/// deliver key-down, key-up and `ModifiersChanged(NONE)` in the same frame, so
/// the frame-level modifier state no longer reflects the press.
pub(crate) fn match_shortcut_token(input: &egui::InputState, token: &str) -> bool {
    input.events.iter().any(|event| event_matches_shortcut(event, token))
}

pub(crate) fn event_matches_shortcut(event: &egui::Event, token: &str) -> bool {
    let egui::Event::Key {
        key,
        pressed: true,
        modifiers,
        ..
    } = event
    else {
        return false;
    };
    let Some(parts) = shortcut_parts(token) else {
        return false;
    };
    *key == parts.key && modifiers_match(*modifiers, &parts)
}

fn parse_key_name(name: &str) -> Option<egui::Key> {
    Some(match name {
        "a" => egui::Key::A,
        "b" => egui::Key::B,
        "c" => egui::Key::C,
        "d" => egui::Key::D,
        "e" => egui::Key::E,
        "f" => egui::Key::F,
        "g" => egui::Key::G,
        "h" => egui::Key::H,
        "i" => egui::Key::I,
        "j" => egui::Key::J,
        "k" => egui::Key::K,
        "l" => egui::Key::L,
        "m" => egui::Key::M,
        "n" => egui::Key::N,
        "o" => egui::Key::O,
        "p" => egui::Key::P,
        "q" => egui::Key::Q,
        "r" => egui::Key::R,
        "s" => egui::Key::S,
        "t" => egui::Key::T,
        "u" => egui::Key::U,
        "v" => egui::Key::V,
        "w" => egui::Key::W,
        "x" => egui::Key::X,
        "y" => egui::Key::Y,
        "z" => egui::Key::Z,
        "/" | "slash" => egui::Key::Slash,
        "0" => egui::Key::Num0,
        "1" => egui::Key::Num1,
        "2" => egui::Key::Num2,
        "3" => egui::Key::Num3,
        "4" => egui::Key::Num4,
        "5" => egui::Key::Num5,
        "6" => egui::Key::Num6,
        "7" => egui::Key::Num7,
        "8" => egui::Key::Num8,
        "9" => egui::Key::Num9,
        "f1" => egui::Key::F1,
        "f2" => egui::Key::F2,
        "f3" => egui::Key::F3,
        "f4" => egui::Key::F4,
        "f5" => egui::Key::F5,
        "f6" => egui::Key::F6,
        "f7" => egui::Key::F7,
        "f8" => egui::Key::F8,
        "f9" => egui::Key::F9,
        "f10" => egui::Key::F10,
        "f11" => egui::Key::F11,
        "f12" => egui::Key::F12,
        "delete" | "del" => egui::Key::Delete,
        "enter" | "return" => egui::Key::Enter,
        "escape" | "esc" => egui::Key::Escape,
        _ => return None,
    })
}

#[cfg(test)]
#[path = "tests/settings_model_tests.rs"]
mod tests;

