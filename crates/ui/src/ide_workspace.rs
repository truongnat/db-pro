//! Local IDE workspace foundation (#261–#264, trust stub #271).
//!
//! Provides a single-root folder model, recursive file tree, and a lightweight
//! path index for Quick Open / Problems follow-ups. Later waves (#265–#284)
//! build on this state rather than inventing parallel stores.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Soft caps so a huge repo cannot freeze the UI on open/refresh.
pub const WORKSPACE_MAX_ENTRIES: usize = 2_000;
pub const WORKSPACE_MAX_DEPTH: usize = 8;
pub const WORKSPACE_RECENT_MAX: usize = 12;

const INDEXABLE_EXTENSIONS: &[&str] = &["sql", "md", "txt", "toml", "json", "yml", "yaml"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WorkspaceTrust {
    #[default]
    Untrusted,
    Trusted,
    Restricted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceFileNode {
    pub name: String,
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<WorkspaceFileNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceIndexEntry {
    pub relative_path: String,
    pub is_sql: bool,
}

#[derive(Debug, Clone, Default)]
pub struct IdeWorkspaceState {
    pub root: Option<PathBuf>,
    pub trust: WorkspaceTrust,
    pub tree: Vec<WorkspaceFileNode>,
    pub index: Vec<WorkspaceIndexEntry>,
    pub recent_roots: Vec<PathBuf>,
    pub last_error: Option<String>,
    pub expanded: std::collections::BTreeSet<String>,
}

impl IdeWorkspaceState {
    pub fn open_root(&mut self, root: PathBuf) -> Result<(), String> {
        if !root.is_dir() {
            return Err(format!("not a folder: {}", root.display()));
        }
        let canonical = root.canonicalize().unwrap_or(root);
        self.root = Some(canonical.clone());
        self.trust = WorkspaceTrust::Untrusted;
        self.last_error = None;
        self.remember_recent(canonical);
        self.refresh()
    }

    pub fn close(&mut self) {
        self.root = None;
        self.tree.clear();
        self.index.clear();
        self.expanded.clear();
        self.last_error = None;
        self.trust = WorkspaceTrust::Untrusted;
    }

    pub fn set_trusted(&mut self, trusted: bool) {
        self.trust = if trusted {
            WorkspaceTrust::Trusted
        } else {
            WorkspaceTrust::Untrusted
        };
    }

    pub fn refresh(&mut self) -> Result<(), String> {
        let Some(root) = self.root.clone() else {
            self.tree.clear();
            self.index.clear();
            return Ok(());
        };
        let mut count = 0usize;
        match scan_directory(&root, &root, 0, &mut count) {
            Ok(tree) => {
                self.tree = tree;
                self.index = flatten_index(&self.tree);
                self.last_error = None;
                Ok(())
            }
            Err(error) => {
                self.last_error = Some(error.clone());
                Err(error)
            }
        }
    }

    pub fn root_label(&self) -> String {
        self.root
            .as_ref()
            .and_then(|path| path.file_name().map(|name| name.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "No folder".to_owned())
    }

    pub fn absolute_for_relative(&self, relative: &str) -> Option<PathBuf> {
        self.root.as_ref().map(|root| root.join(relative))
    }

    fn remember_recent(&mut self, root: PathBuf) {
        self.recent_roots.retain(|item| item != &root);
        self.recent_roots.insert(0, root);
        if self.recent_roots.len() > WORKSPACE_RECENT_MAX {
            self.recent_roots.truncate(WORKSPACE_RECENT_MAX);
        }
    }
}

fn is_indexable(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
        INDEXABLE_EXTENSIONS
            .iter()
            .any(|allowed| ext.eq_ignore_ascii_case(allowed))
    })
}

fn should_skip_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | "node_modules" | "target" | ".idea" | ".vscode" | "dist" | "build" | ".next"
    )
}

fn scan_directory(root: &Path, dir: &Path, depth: usize, count: &mut usize) -> Result<Vec<WorkspaceFileNode>, String> {
    if depth > WORKSPACE_MAX_DEPTH || *count >= WORKSPACE_MAX_ENTRIES {
        return Ok(Vec::new());
    }
    let mut entries = std::fs::read_dir(dir).map_err(|error| format!("read {}: {error}", dir.display()))?;
    let mut nodes = Vec::new();
    let mut collected = Vec::new();
    while let Some(entry) = entries.next() {
        let entry = entry.map_err(|error| format!("read {}: {error}", dir.display()))?;
        collected.push(entry);
    }
    collected.sort_by_key(|entry| {
        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        (!is_dir, entry.file_name().to_string_lossy().to_lowercase())
    });

    for entry in collected {
        if *count >= WORKSPACE_MAX_ENTRIES {
            break;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') && name != ".env.example" {
            // Keep tree tidy; still allow explicit SQL files later via refresh filters.
            if name != ".sqlfluff" {
                continue;
            }
        }
        let path = entry.path();
        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        if is_dir {
            if should_skip_dir(&name) {
                continue;
            }
            *count += 1;
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let children = scan_directory(root, &path, depth + 1, count)?;
            nodes.push(WorkspaceFileNode {
                name,
                relative_path: relative,
                absolute_path: path,
                is_dir: true,
                children,
            });
        } else if is_indexable(&path) {
            *count += 1;
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            nodes.push(WorkspaceFileNode {
                name,
                relative_path: relative,
                absolute_path: path,
                is_dir: false,
                children: Vec::new(),
            });
        }
    }
    Ok(nodes)
}

fn flatten_index(nodes: &[WorkspaceFileNode]) -> Vec<WorkspaceIndexEntry> {
    let mut out = Vec::new();
    fn walk(nodes: &[WorkspaceFileNode], out: &mut Vec<WorkspaceIndexEntry>) {
        for node in nodes {
            if node.is_dir {
                walk(&node.children, out);
            } else {
                out.push(WorkspaceIndexEntry {
                    is_sql: node
                        .relative_path
                        .rsplit('.')
                        .next()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("sql")),
                    relative_path: node.relative_path.clone(),
                });
            }
        }
    }
    walk(nodes, &mut out);
    out
}

/// Best-effort full-text hit list for Find-in-Files foundation (#267).
pub fn search_workspace_files(
    root: &Path,
    index: &[WorkspaceIndexEntry],
    query: &str,
    limit: usize,
) -> Vec<(String, usize, String)> {
    if query.is_empty() || limit == 0 {
        return Vec::new();
    }
    let needle = query.to_lowercase();
    let mut hits = Vec::new();
    for entry in index {
        if hits.len() >= limit {
            break;
        }
        let path = root.join(&entry.relative_path);
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (line_idx, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&needle) {
                hits.push((
                    entry.relative_path.clone(),
                    line_idx + 1,
                    line.chars().take(160).collect(),
                ));
                if hits.len() >= limit {
                    break;
                }
            }
        }
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("time").as_nanos();
        let dir = std::env::temp_dir().join(format!("dbpro-workspace-{nanos}"));
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[test]
    fn open_root_indexes_sql_files_and_skips_target() {
        let dir = temp_dir();
        std::fs::create_dir_all(dir.join("migrations")).unwrap();
        std::fs::create_dir_all(dir.join("target/debug")).unwrap();
        std::fs::write(dir.join("migrations/001_init.sql"), "CREATE TABLE t (id INT);").unwrap();
        std::fs::write(dir.join("readme.md"), "# demo").unwrap();
        std::fs::write(dir.join("target/debug/noise.sql"), "SELECT 1;").unwrap();

        let mut workspace = IdeWorkspaceState::default();
        workspace.open_root(dir.clone()).expect("open");
        let paths: Vec<_> = workspace
            .index
            .iter()
            .map(|entry| entry.relative_path.as_str())
            .collect();
        assert!(paths.contains(&"migrations/001_init.sql"));
        assert!(paths.contains(&"readme.md"));
        assert!(!paths.iter().any(|path| path.contains("target/")));

        let hits = search_workspace_files(workspace.root.as_ref().unwrap(), &workspace.index, "create table", 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, "migrations/001_init.sql");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn recent_roots_are_mru_capped() {
        let mut workspace = IdeWorkspaceState::default();
        for index in 0..(WORKSPACE_RECENT_MAX + 3) {
            workspace.remember_recent(PathBuf::from(format!("/tmp/ws-{index}")));
        }
        assert_eq!(workspace.recent_roots.len(), WORKSPACE_RECENT_MAX);
        assert_eq!(
            workspace.recent_roots[0],
            PathBuf::from(format!("/tmp/ws-{}", WORKSPACE_RECENT_MAX + 2))
        );
    }
}
