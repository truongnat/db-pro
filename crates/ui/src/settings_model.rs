//! Typed native settings model with versioned persistence (#205).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const SETTINGS_STORAGE_KEY: &str = "dbpro.native.settings-v1";
pub(crate) const SETTINGS_VERSION: u32 = 1;

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
    pub(crate) fn all() -> &'static [SettingsSection] {
        &[
            SettingsSection::General,
            SettingsSection::Appearance,
            SettingsSection::Editor,
            SettingsSection::DataGrid,
            SettingsSection::Connections,
            SettingsSection::Ai,
            SettingsSection::Keybindings,
            SettingsSection::Backup,
            SettingsSection::Security,
            SettingsSection::Advanced,
        ]
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            SettingsSection::General => "General",
            SettingsSection::Appearance => "Appearance",
            SettingsSection::Editor => "Editor",
            SettingsSection::DataGrid => "Data Grid",
            SettingsSection::Connections => "Connections",
            SettingsSection::Ai => "AI Providers",
            SettingsSection::Keybindings => "Keybindings",
            SettingsSection::Backup => "Backup / Restore",
            SettingsSection::Security => "Security",
            SettingsSection::Advanced => "Advanced",
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
        if self.version == 0 {
            self.version = SETTINGS_VERSION;
        }
        if self.version > SETTINGS_VERSION {
            // Future blob: keep fields we know, clamp version for honesty.
            self.version = SETTINGS_VERSION;
        }
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
            prediction_mode: "eager".to_owned(),
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
            select_star: true,
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
            default_shortcut: "f5",
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

    /// Returns shortcut tokens that map to more than one command.
    pub(crate) fn conflicts(&self) -> Vec<(String, Vec<String>)> {
        let mut by_shortcut: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for cmd in default_keybinding_catalog() {
            let shortcut = self.resolved(cmd.id).to_ascii_lowercase();
            if shortcut.is_empty() {
                continue;
            }
            by_shortcut.entry(shortcut).or_default().push(cmd.id.to_owned());
        }
        by_shortcut.into_iter().filter(|(_, ids)| ids.len() > 1).collect()
    }

    pub(crate) fn conflict_ids(&self) -> BTreeSet<String> {
        self.conflicts().into_iter().flat_map(|(_, ids)| ids).collect()
    }
}

/// Match a compact shortcut token (`mod+k`, `mod+shift+p`, `f5`) against egui input.
pub(crate) fn match_shortcut_token(input: &egui::InputState, token: &str) -> bool {
    let token = token.trim().to_ascii_lowercase();
    if token.is_empty() {
        return false;
    }
    let parts: Vec<&str> = token.split('+').map(str::trim).collect();
    let mut want_mod = false;
    let mut want_shift = false;
    let mut key_name = "";
    for part in parts {
        match part {
            "mod" | "cmd" | "ctrl" | "command" => want_mod = true,
            "shift" => want_shift = true,
            other => key_name = other,
        }
    }
    let Some(key) = parse_key_name(key_name) else {
        return false;
    };
    if !input.key_pressed(key) {
        return false;
    }
    let primary = input.modifiers.command || input.modifiers.ctrl;
    if want_mod != primary {
        return false;
    }
    if want_shift != input.modifiers.shift {
        return false;
    }
    true
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
        "f5" => egui::Key::F5,
        "enter" | "return" => egui::Key::Enter,
        "escape" | "esc" => egui::Key::Escape,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_clamps_and_sets_version() {
        let settings = AppSettings {
            version: 0,
            editor: EditorSettings {
                font_size: 99.0,
                ..EditorSettings::default()
            },
            data_grid: DataGridSettings {
                page_size: 5,
                ..DataGridSettings::default()
            },
            ..AppSettings::default()
        };
        let migrated = settings.migrate();
        assert_eq!(migrated.version, SETTINGS_VERSION);
        assert_eq!(migrated.editor.font_size, 24.0);
        assert_eq!(migrated.data_grid.page_size, 25);
    }

    #[test]
    fn keybinding_conflicts_detect_duplicate_overrides() {
        let mut kb = KeybindingSettings::default();
        kb.overrides.insert("palette.quick_open".into(), "mod+k".into());
        // mod+k is also the alternate quick open default → conflict
        let conflicts = kb.conflicts();
        assert!(
            conflicts
                .iter()
                .any(|(shortcut, ids)| { shortcut == "mod+k" && ids.len() >= 2 }),
            "expected mod+k conflict, got {conflicts:?}"
        );
    }

    #[test]
    fn settings_round_trip_json() {
        let mut settings = AppSettings::default();
        settings.appearance.dark_mode = true;
        settings
            .keybindings
            .overrides
            .insert("query.run".into(), "mod+enter".into());
        let json = serde_json::to_string(&settings).unwrap();
        let loaded = AppSettings::from_json(&json).unwrap();
        assert!(loaded.appearance.dark_mode);
        assert_eq!(loaded.keybindings.resolved("query.run"), "mod+enter");
    }

    #[test]
    fn sql_lint_settings_persist_and_suppress() {
        let mut settings = AppSettings::default();
        settings.editor.lint.select_star = false;
        settings.editor.lint.suppressed_codes.insert("lint.comma-join".into());
        let json = serde_json::to_string(&settings).unwrap();
        let loaded = AppSettings::from_json(&json).unwrap();
        assert!(!loaded.editor.lint.select_star);
        assert!(!loaded.editor.lint.allows("lint.select-star"));
        assert!(!loaded.editor.lint.allows("lint.comma-join"));
        assert!(loaded.editor.lint.allows("lint.null-compare"));
    }
}
