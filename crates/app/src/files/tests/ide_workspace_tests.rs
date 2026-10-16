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

    #[test]
    fn workspace_paths_reject_traversal_and_absolute_paths() {
        let dir = temp_dir();
        let outside = temp_dir();
        let mut workspace = IdeWorkspaceState::default();
        workspace.open_root(dir.clone()).unwrap();
        let root_id = workspace.primary_root().unwrap().id.clone();

        assert!(workspace.absolute_for_relative("../outside.txt").is_none());
        assert!(workspace.absolute_for_relative("nested/../../outside.txt").is_none());
        assert!(workspace.absolute_for_relative(outside.join("outside.txt").to_str().unwrap()).is_none());
        assert!(workspace
            .absolute_for_relative(&IdeWorkspaceState::qualified_relative(&root_id, "../outside.txt"))
            .is_none());

        assert!(workspace.create_file("../", "outside.txt", "nope").is_err());
        assert!(workspace.create_file("", "../outside.txt", "nope").is_err());
        assert!(workspace.create_folder("", "../outside-dir").is_err());
        assert!(workspace.rename_path("../outside.txt", "renamed.txt").is_err());
        assert!(workspace.delete_path("../outside.txt").is_err());
        assert!(workspace.delete_path(".").is_err());
        workspace.set_trusted(true);
        assert!(workspace.apply_workspace_patch("../outside.txt", "nope").is_err());
        assert!(!outside.join("outside.txt").exists());

        let _ = std::fs::remove_dir_all(dir);
        let _ = std::fs::remove_dir_all(outside);
    }

    #[cfg(unix)]
    #[test]
    fn delete_workspace_symlink_does_not_delete_external_target() {
        use std::os::unix::fs::symlink;

        let dir = temp_dir();
        let outside = temp_dir();
        let target = outside.join("keep.txt");
        std::fs::write(&target, "keep me").unwrap();
        symlink(&target, dir.join("external-link.txt")).unwrap();

        let mut workspace = IdeWorkspaceState::default();
        workspace.open_root(dir.clone()).unwrap();
        workspace.delete_path("external-link.txt").unwrap();

        assert!(target.exists());
        assert!(!dir.join("external-link.txt").exists());
        let _ = std::fs::remove_dir_all(dir);
        let _ = std::fs::remove_dir_all(outside);
    }
