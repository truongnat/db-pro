//! Optional Git adapter for the Files / IDE workspace (#255).
//!
//! Shells out to `git` when available. DB Pro stays fully usable without Git.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitFileStatus {
    pub path: String,
    /// Two-letter porcelain status (e.g. ` M`, `M `, `??`, `A `).
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitWorkspaceStatus {
    pub root: PathBuf,
    pub branch: Option<String>,
    pub available: bool,
    pub message: String,
    pub entries: Vec<GitFileStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitDiffResult {
    pub path: String,
    pub against: String,
    pub text: String,
}

fn run_git(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                "git is not installed — workspace still works without Git".to_owned()
            } else {
                format!("failed to spawn git: {err}")
            }
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if output.status.success() {
        Ok(stdout)
    } else {
        let detail = if stderr.trim().is_empty() {
            stdout.trim().to_owned()
        } else {
            stderr.trim().to_owned()
        };
        Err(if detail.is_empty() {
            format!("git {:?} failed", args)
        } else {
            detail
        })
    }
}

/// Probe whether `root` is inside a Git work tree and collect porcelain status.
pub fn probe_git_status(root: &Path) -> GitWorkspaceStatus {
    match run_git(root, &["rev-parse", "--is-inside-work-tree"]) {
        Ok(out) if out.trim() == "true" => {}
        Ok(_) => {
            return GitWorkspaceStatus {
                root: root.to_path_buf(),
                branch: None,
                available: false,
                message: "Not a Git work tree".into(),
                entries: Vec::new(),
            };
        }
        Err(message) => {
            return GitWorkspaceStatus {
                root: root.to_path_buf(),
                branch: None,
                available: false,
                message,
                entries: Vec::new(),
            };
        }
    }

    let branch = run_git(root, &["rev-parse", "--abbrev-ref", "HEAD"])
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());

    let porcelain = run_git(root, &["status", "--porcelain", "-uall"]).unwrap_or_default();
    let mut entries = Vec::new();
    for line in porcelain.lines() {
        if line.len() < 4 {
            continue;
        }
        let code = line[..2].to_owned();
        let path = line[3..].trim().to_owned();
        if path.is_empty() {
            continue;
        }
        entries.push(GitFileStatus { path, code });
    }
    entries.truncate(200);

    GitWorkspaceStatus {
        root: root.to_path_buf(),
        branch,
        available: true,
        message: format!("{} changed path(s)", entries.len()),
        entries,
    }
}

/// Diff a relative path against HEAD (or show working-tree content for untracked).
pub fn diff_against_head(root: &Path, relative: &str) -> Result<GitDiffResult, String> {
    let text = match run_git(root, &["diff", "HEAD", "--", relative]) {
        Ok(t) if !t.trim().is_empty() => t,
        Ok(_) => match run_git(root, &["diff", "--", relative]) {
            Ok(t) if !t.trim().is_empty() => t,
            _ => {
                let path = root.join(relative);
                let body = std::fs::read_to_string(&path).unwrap_or_default();
                if body.is_empty() {
                    "(no diff — file matches HEAD or is empty)".into()
                } else {
                    format!("--- /dev/null\n+++ b/{relative}\n{body}")
                }
            }
        },
        Err(err) => return Err(err),
    };
    Ok(GitDiffResult {
        path: relative.to_owned(),
        against: "HEAD".into(),
        text,
    })
}

pub fn stage_path(root: &Path, relative: &str) -> Result<(), String> {
    run_git(root, &["add", "--", relative]).map(|_| ())
}

pub fn unstage_path(root: &Path, relative: &str) -> Result<(), String> {
    run_git(root, &["restore", "--staged", "--", relative]).map(|_| ())
}

/// Explicit user-triggered commit. Never called automatically.
pub fn commit_paths(root: &Path, message: &str) -> Result<String, String> {
    let message = message.trim();
    if message.is_empty() {
        return Err("commit message is required".into());
    }
    let lower = message.to_ascii_lowercase();
    for needle in ["password=", "pwd=", "secret=", "private_key"] {
        if lower.contains(needle) {
            return Err("commit message looks like it contains secret material".into());
        }
    }
    run_git(root, &["commit", "-m", message])
}

/// Detect whether the on-disk file changed since `known_mtime` (secs since epoch).
pub fn disk_mtime_secs(path: &Path) -> Option<u64> {
    std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

/// Compare editor buffer to disk without overwriting. Returns true when disk differs.
pub fn disk_diverged_from_buffer(path: &Path, buffer: &str) -> bool {
    match std::fs::read_to_string(path) {
        Ok(disk) => disk != buffer,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
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
}
