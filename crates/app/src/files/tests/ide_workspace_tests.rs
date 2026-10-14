//! Tests for `../ide_workspace.rs` (moved out of the source file).
    use super::*;

    fn temp_dir() -> PathBuf {
        static CTR: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let count = CTR.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("time").as_nanos();
        let dir = std::env::temp_dir().join(format!("dbpro-workspace-{nanos}-{count}"));
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[test]
    fn multi_root_and_replace_require_trust() {
        let dir = temp_dir();
        std::fs::create_dir_all(dir.join("migrations")).unwrap();
        std::fs::write(dir.join("migrations/001_init.sql"), "CREATE TABLE users (id INT);").unwrap();
        let mut workspace = IdeWorkspaceState::default();
        workspace.open_root(dir.clone()).unwrap();
        assert!(workspace.apply_replace("users", "accounts").is_err());
        workspace.set_trusted(true);
        let count = workspace.apply_replace("users", "accounts").unwrap();
        assert!(count >= 1);
        let content = std::fs::read_to_string(dir.join("migrations/001_init.sql")).unwrap();
        assert!(content.contains("accounts"));
        let second = temp_dir();
        std::fs::write(second.join("extra.sql"), "SELECT 1;").unwrap();
        workspace.add_root(second.clone()).unwrap();
        assert_eq!(workspace.roots.len(), 2);
        let _ = std::fs::remove_dir_all(dir);
        let _ = std::fs::remove_dir_all(second);
    }

    #[test]
    fn migrations_and_diagnostics_scan() {
        let dir = temp_dir();
        std::fs::create_dir_all(dir.join("migrations")).unwrap();
        std::fs::write(dir.join("migrations/002_bad.sql"), "SELECT * FROM t;\nDROP TABLE t;\n").unwrap();
        let mut workspace = IdeWorkspaceState::default();
        workspace.open_root(dir.clone()).unwrap();
        let migrations = workspace.detect_migrations();
        assert_eq!(migrations.len(), 1);
        workspace.scan_diagnostics();
        assert!(workspace.workspace_diagnostics.len() >= 2);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn notebook_patch_sandbox_and_sql_tests_helpers() {
        let dir = temp_dir();
        let mut workspace = IdeWorkspaceState::default();
        workspace.open_root(dir.clone()).unwrap();
        workspace.set_trusted(true);
        let nb = dir.join("notes.dbpro-nb");
        let cells = vec![
            NotebookCell {
                kind: NotebookCellKind::Markdown,
                source: "# hello".to_owned(),
            },
            NotebookCell {
                kind: NotebookCellKind::Sql,
                source: "SELECT 1;".to_owned(),
            },
        ];
        IdeWorkspaceState::save_notebook(&nb, &cells).unwrap();
        let loaded = IdeWorkspaceState::load_notebook(&nb).unwrap();
        assert_eq!(loaded.len(), 2);
        workspace.apply_workspace_patch("patch.sql", "SELECT 2;").unwrap();
        assert!(dir.join("patch.sql").exists());
        let _ = workspace.rename_path("patch.sql", "patched.sql");
        let refs = workspace.find_references("SELECT", 10);
        assert!(!refs.is_empty());
        let tests = IdeWorkspaceState::parse_sql_tests("-- test: a\nSELECT 1;\n-- test: b\nSELECT 2;");
        assert_eq!(tests.len(), 2);
        let source_db = dir.join("demo.sqlite");
        std::fs::write(&source_db, b"sqlite").unwrap();
        let sandbox = workspace.create_sandbox_sqlite_copy(&source_db).unwrap();
        assert!(sandbox.exists());
        assert!(workspace.active_environment().is_some());
        let _ = IdeWorkspaceState::qualified_relative("root-1", "a.sql");
        let _ = std::fs::remove_dir_all(dir);
    }