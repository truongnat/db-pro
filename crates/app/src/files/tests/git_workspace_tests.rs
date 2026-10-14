//! Tests for `../git_workspace.rs` (moved out of the source file).
    use super::*;
    use std::fs;

    #[test]
    fn probe_non_repo_is_graceful() {
        let dir = std::env::temp_dir().join(format!("db-pro-git-none-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let status = probe_git_status(&dir);
        assert!(!status.available);
        assert!(!status.message.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn commit_rejects_secret_looking_message() {
        let err = commit_paths(Path::new("."), "password=hunter2").unwrap_err();
        assert!(err.contains("secret"));
    }

    #[test]
    fn disk_mtime_and_divergence() {
        let path = std::env::temp_dir().join(format!("db-pro-mtime-{}.txt", std::process::id()));
        fs::write(&path, "alpha").unwrap();
        assert!(disk_mtime_secs(&path).is_some());
        assert!(!disk_diverged_from_buffer(&path, "alpha"));
        assert!(disk_diverged_from_buffer(&path, "beta"));
        let _ = fs::remove_file(&path);
    }