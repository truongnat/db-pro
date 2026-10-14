//! Tests for `../settings_model.rs` (moved out of the source file).
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
    fn keybinding_catalog_includes_new_connection_and_query() {
        let kb = KeybindingSettings::default();
        assert_eq!(kb.resolved("connection.new"), "mod+n");
        assert_eq!(kb.resolved("query.new"), "mod+t");
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

    #[test]
    fn saved_select_star_warning_turns_off_once() {
        let mut settings = AppSettings {
            version: 1,
            ..Default::default()
        };
        settings.editor.lint.select_star = true;
        let loaded = AppSettings::from_json(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert!(!loaded.editor.lint.select_star);
        assert_eq!(loaded.version, SETTINGS_VERSION);
    }