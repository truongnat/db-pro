//! IDE workspace model + operations for waves W01–W24 (#261–#284).
//!
//! Multi-root folders, file ops, search/replace, migrations, trust, env
//! profiles, diagnostics scan, rename refs, benchmarks, and sandbox helpers.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const WORKSPACE_MAX_ENTRIES: usize = 2_000;
pub const WORKSPACE_MAX_DEPTH: usize = 8;
pub const WORKSPACE_RECENT_MAX: usize = 12;

const INDEXABLE_EXTENSIONS: &[&str] = &["sql", "md", "txt", "toml", "json", "yml", "yaml", "dbpro-nb"];

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
    pub root_id: String,
    pub relative_path: String,
    pub is_sql: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceEnvironment {
    pub name: String,
    pub connection_id: Option<String>,
    pub database: Option<String>,
    pub schema: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationEntry {
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub version: String,
    pub status: MigrationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum MigrationStatus {
    Pending,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub root_id: String,
    pub relative_path: String,
    pub line: usize,
    pub preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplacePreview {
    pub root_id: String,
    pub relative_path: String,
    pub replacements: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceHit {
    pub root_id: String,
    pub relative_path: String,
    pub line: usize,
    pub preview: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDiagnostic {
    pub root_id: String,
    pub relative_path: String,
    pub line: usize,
    pub message: String,
    pub severity: WorkspaceDiagnosticSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceDiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRunResult {
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCase {
    pub name: String,
    pub sql: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkResult {
    pub name: String,
    pub runs: Vec<u64>,
    pub avg_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyEdge {
    pub from_file: String,
    pub object_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotebookCell {
    pub kind: NotebookCellKind,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotebookCellKind {
    Markdown,
    Sql,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotebookDocument {
    pub version: u32,
    pub cells: Vec<NotebookCellSerde>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotebookCellSerde {
    pub kind: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct WorkspaceRoot {
    pub id: String,
    pub path: PathBuf,
    pub tree: Vec<WorkspaceFileNode>,
    pub index: Vec<WorkspaceIndexEntry>,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct IdeWorkspaceState {
    pub roots: Vec<WorkspaceRoot>,
    pub active_root: usize,
    pub trust: WorkspaceTrust,
    pub recent_roots: Vec<PathBuf>,
    pub last_error: Option<String>,
    pub expanded: std::collections::BTreeSet<String>,
    pub environments: Vec<WorkspaceEnvironment>,
    pub active_environment: usize,
    pub workspace_diagnostics: Vec<WorkspaceDiagnostic>,
    pub last_task: Option<TaskRunResult>,
    pub schema_fingerprint: Option<u64>,
    pub schema_drift_message: Option<String>,
    pub last_drift_check: Option<Instant>,
}

impl IdeWorkspaceState {
    pub fn primary_root(&self) -> Option<&WorkspaceRoot> {
        self.roots.get(self.active_root)
    }

    pub fn primary_path(&self) -> Option<&Path> {
        self.primary_root().map(|root| root.path.as_path())
    }

    /// Back-compat for callers that still look at a single `root`.
    pub fn root(&self) -> Option<&PathBuf> {
        self.primary_root().map(|root| &root.path)
    }

    pub fn tree(&self) -> &[WorkspaceFileNode] {
        self.primary_root().map(|root| root.tree.as_slice()).unwrap_or(&[])
    }

    pub fn index(&self) -> Vec<WorkspaceIndexEntry> {
        self.roots.iter().flat_map(|root| root.index.clone()).collect()
    }

    pub fn open_root(&mut self, root: PathBuf) -> Result<(), String> {
        self.roots.clear();
        self.active_root = 0;
        self.add_root(root)
    }

    pub fn add_root(&mut self, root: PathBuf) -> Result<(), String> {
        if !root.is_dir() {
            return Err(format!("not a folder: {}", root.display()));
        }
        let canonical = root.canonicalize().unwrap_or(root);
        if self.roots.iter().any(|existing| existing.path == canonical) {
            return Ok(());
        }
        let id = format!("root-{}", self.roots.len() + 1);
        let mut count = 0usize;
        let tree = scan_directory(&canonical, &canonical, 0, &mut count)?;
        let index = flatten_index(&id, &tree);
        self.roots.push(WorkspaceRoot {
            id,
            path: canonical.clone(),
            tree,
            index,
        });
        self.active_root = self.roots.len() - 1;
        self.last_error = None;
        self.remember_recent(canonical);
        if self.environments.is_empty() {
            self.environments.push(WorkspaceEnvironment {
                name: "Development".to_owned(),
                connection_id: None,
                database: None,
                schema: None,
            });
            self.environments.push(WorkspaceEnvironment {
                name: "Staging".to_owned(),
                connection_id: None,
                database: None,
                schema: None,
            });
            self.environments.push(WorkspaceEnvironment {
                name: "Production".to_owned(),
                connection_id: None,
                database: None,
                schema: None,
            });
            self.active_environment = 0;
        }
        Ok(())
    }

    pub fn remove_active_root(&mut self) {
        if self.roots.is_empty() {
            return;
        }
        self.roots.remove(self.active_root);
        if self.active_root >= self.roots.len() {
            self.active_root = self.roots.len().saturating_sub(1);
        }
        if self.roots.is_empty() {
            self.close();
        }
    }

    pub fn close(&mut self) {
        self.roots.clear();
        self.active_root = 0;
        self.expanded.clear();
        self.last_error = None;
        self.trust = WorkspaceTrust::Untrusted;
        self.workspace_diagnostics.clear();
        self.last_task = None;
        self.schema_fingerprint = None;
        self.schema_drift_message = None;
    }

    pub fn set_trusted(&mut self, trusted: bool) {
        self.trust = if trusted {
            WorkspaceTrust::Trusted
        } else {
            WorkspaceTrust::Untrusted
        };
    }

    pub fn is_trusted(&self) -> bool {
        matches!(self.trust, WorkspaceTrust::Trusted)
    }

    pub fn refresh(&mut self) -> Result<(), String> {
        let roots = self.roots.clone();
        self.roots.clear();
        for root in roots {
            self.add_root(root.path)?;
        }
        Ok(())
    }

    pub fn root_label(&self) -> String {
        self.primary_root()
            .and_then(|root| root.path.file_name().map(|name| name.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "No folder".to_owned())
    }

    pub fn absolute_for_relative(&self, relative: &str) -> Option<PathBuf> {
        // Accept "root-id::path" or plain relative against active root.
        if let Some((root_id, path)) = relative.split_once("::") {
            let root = self.roots.iter().find(|root| root.id == root_id)?;
            return Some(root.path.join(path));
        }
        self.primary_root().map(|root| root.path.join(relative))
    }

    pub fn qualified_relative(root_id: &str, relative: &str) -> String {
        format!("{root_id}::{relative}")
    }

    fn remember_recent(&mut self, root: PathBuf) {
        self.recent_roots.retain(|item| item != &root);
        self.recent_roots.insert(0, root);
        if self.recent_roots.len() > WORKSPACE_RECENT_MAX {
            self.recent_roots.truncate(WORKSPACE_RECENT_MAX);
        }
    }

    pub fn create_file(&mut self, relative_dir: &str, name: &str, contents: &str) -> Result<PathBuf, String> {
        let root = self.primary_root().ok_or_else(|| "no workspace root".to_owned())?;
        let dir = if relative_dir.is_empty() {
            root.path.clone()
        } else {
            root.path.join(relative_dir)
        };
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(name);
        if path.exists() {
            return Err(format!("already exists: {}", path.display()));
        }
        std::fs::write(&path, contents).map_err(|e| e.to_string())?;
        self.refresh()?;
        Ok(path)
    }

    pub fn create_folder(&mut self, relative_dir: &str, name: &str) -> Result<PathBuf, String> {
        let root = self.primary_root().ok_or_else(|| "no workspace root".to_owned())?;
        let path = if relative_dir.is_empty() {
            root.path.join(name)
        } else {
            root.path.join(relative_dir).join(name)
        };
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
        self.refresh()?;
        Ok(path)
    }

    pub fn rename_path(&mut self, relative: &str, new_name: &str) -> Result<PathBuf, String> {
        let absolute = self
            .absolute_for_relative(relative)
            .ok_or_else(|| "missing path".to_owned())?;
        let parent = absolute.parent().ok_or_else(|| "no parent".to_owned())?;
        let dest = parent.join(new_name);
        std::fs::rename(&absolute, &dest).map_err(|e| e.to_string())?;
        self.refresh()?;
        Ok(dest)
    }

    pub fn delete_path(&mut self, relative: &str) -> Result<(), String> {
        let absolute = self
            .absolute_for_relative(relative)
            .ok_or_else(|| "missing path".to_owned())?;
        if absolute.is_dir() {
            std::fs::remove_dir_all(&absolute).map_err(|e| e.to_string())?;
        } else {
            std::fs::remove_file(&absolute).map_err(|e| e.to_string())?;
        }
        self.refresh()?;
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        let mut hits = Vec::new();
        if query.is_empty() {
            return hits;
        }
        let needle = query.to_lowercase();
        for root in &self.roots {
            for entry in &root.index {
                if hits.len() >= limit {
                    return hits;
                }
                let path = root.path.join(&entry.relative_path);
                let Ok(content) = std::fs::read_to_string(&path) else {
                    continue;
                };
                for (line_idx, line) in content.lines().enumerate() {
                    if line.to_lowercase().contains(&needle) {
                        hits.push(SearchHit {
                            root_id: root.id.clone(),
                            relative_path: entry.relative_path.clone(),
                            line: line_idx + 1,
                            preview: line.chars().take(160).collect(),
                        });
                        if hits.len() >= limit {
                            return hits;
                        }
                    }
                }
            }
        }
        hits
    }

    pub fn preview_replace(&self, query: &str, replacement: &str) -> Vec<ReplacePreview> {
        if query.is_empty() {
            return Vec::new();
        }
        let mut previews = Vec::new();
        for root in &self.roots {
            for entry in &root.index {
                let path = root.path.join(&entry.relative_path);
                let Ok(content) = std::fs::read_to_string(&path) else {
                    continue;
                };
                let count = content.matches(query).count();
                if count > 0 {
                    let _ = replacement; // preview only
                    previews.push(ReplacePreview {
                        root_id: root.id.clone(),
                        relative_path: entry.relative_path.clone(),
                        replacements: count,
                    });
                }
            }
        }
        previews
    }

    pub fn apply_replace(&mut self, query: &str, replacement: &str) -> Result<usize, String> {
        if !self.is_trusted() {
            return Err("trust the workspace before replace-in-files".to_owned());
        }
        if query.is_empty() {
            return Err("empty search".to_owned());
        }
        let mut total = 0usize;
        for root in &self.roots {
            for entry in &root.index {
                let path = root.path.join(&entry.relative_path);
                let Ok(content) = std::fs::read_to_string(&path) else {
                    continue;
                };
                if !content.contains(query) {
                    continue;
                }
                let updated = content.replace(query, replacement);
                let count = content.matches(query).count();
                std::fs::write(&path, updated).map_err(|e| e.to_string())?;
                total += count;
            }
        }
        self.refresh()?;
        Ok(total)
    }

    pub fn find_references(&self, symbol: &str, limit: usize) -> Vec<ReferenceHit> {
        self.search(symbol, limit)
            .into_iter()
            .map(|hit| ReferenceHit {
                root_id: hit.root_id,
                relative_path: hit.relative_path,
                line: hit.line,
                preview: hit.preview,
            })
            .collect()
    }

    pub fn scan_diagnostics(&mut self) {
        let mut diagnostics = Vec::new();
        for root in &self.roots {
            for entry in root.index.iter().filter(|entry| entry.is_sql) {
                let path = root.path.join(&entry.relative_path);
                let Ok(content) = std::fs::read_to_string(&path) else {
                    continue;
                };
                for (line_idx, line) in content.lines().enumerate() {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with("SELECT *") {
                        diagnostics.push(WorkspaceDiagnostic {
                            root_id: root.id.clone(),
                            relative_path: entry.relative_path.clone(),
                            line: line_idx + 1,
                            message: "avoid SELECT * in workspace SQL".to_owned(),
                            severity: WorkspaceDiagnosticSeverity::Warning,
                        });
                    }
                    if trimmed.contains("DROP TABLE") && !trimmed.contains("IF EXISTS") {
                        diagnostics.push(WorkspaceDiagnostic {
                            root_id: root.id.clone(),
                            relative_path: entry.relative_path.clone(),
                            line: line_idx + 1,
                            message: "DROP TABLE without IF EXISTS".to_owned(),
                            severity: WorkspaceDiagnosticSeverity::Error,
                        });
                    }
                }
            }
        }
        self.workspace_diagnostics = diagnostics;
    }

    pub fn detect_migrations(&self) -> Vec<MigrationEntry> {
        let mut entries = Vec::new();
        for root in &self.roots {
            for entry in root.index.iter().filter(|entry| entry.is_sql) {
                let lower = entry.relative_path.to_lowercase();
                if !(lower.contains("migration") || lower.contains("/migrate") || lower.starts_with("migrations/")) {
                    continue;
                }
                let version = Path::new(&entry.relative_path)
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().into_owned())
                    .unwrap_or_else(|| entry.relative_path.clone());
                entries.push(MigrationEntry {
                    relative_path: entry.relative_path.clone(),
                    absolute_path: root.path.join(&entry.relative_path),
                    version,
                    status: MigrationStatus::Unknown,
                });
            }
        }
        entries.sort_by(|a, b| a.version.cmp(&b.version));
        entries
    }

    pub fn run_task(&mut self, command_line: &str) -> Result<TaskRunResult, String> {
        if !self.is_trusted() {
            return Err("trust the workspace before running tasks/terminal commands".to_owned());
        }
        let cwd = self
            .primary_path()
            .ok_or_else(|| "open a workspace folder first".to_owned())?
            .to_path_buf();
        let started = Instant::now();
        #[cfg(target_os = "windows")]
        let output = Command::new("cmd")
            .args(["/C", command_line])
            .current_dir(&cwd)
            .output();
        #[cfg(not(target_os = "windows"))]
        let output = Command::new("sh")
            .args(["-lc", command_line])
            .current_dir(&cwd)
            .output();
        let output = output.map_err(|e| e.to_string())?;
        let result = TaskRunResult {
            command: command_line.to_owned(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            duration_ms: started.elapsed().as_millis() as u64,
        };
        self.last_task = Some(result.clone());
        Ok(result)
    }

    pub fn export_schema_snapshot(&self, schema_sql: &str) -> Result<PathBuf, String> {
        let root = self
            .primary_path()
            .ok_or_else(|| "open a workspace folder first".to_owned())?;
        let dir = root.join(".dbpro");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join("schema-snapshot.sql");
        std::fs::write(&path, schema_sql).map_err(|e| e.to_string())?;
        Ok(path)
    }

    pub fn rename_symbol_across_sql(&mut self, from: &str, to: &str) -> Result<usize, String> {
        if from.is_empty() || to.is_empty() {
            return Err("from/to required".to_owned());
        }
        if !self.is_trusted() {
            return Err("trust the workspace before refactoring".to_owned());
        }
        self.apply_replace(from, to)
    }

    pub fn active_environment(&self) -> Option<&WorkspaceEnvironment> {
        self.environments.get(self.active_environment)
    }

    pub fn set_active_environment(&mut self, index: usize) {
        if index < self.environments.len() {
            self.active_environment = index;
        }
    }

    pub fn update_schema_fingerprint(&mut self, fingerprint: u64) {
        if let Some(previous) = self.schema_fingerprint {
            if previous != fingerprint {
                self.schema_drift_message = Some(format!(
                    "Schema drift detected (fingerprint {previous} → {fingerprint})"
                ));
            }
        }
        self.schema_fingerprint = Some(fingerprint);
        self.last_drift_check = Some(Instant::now());
    }

    pub fn dependency_edges(&self) -> Vec<DependencyEdge> {
        let mut edges = Vec::new();
        for root in &self.roots {
            for entry in root.index.iter().filter(|entry| entry.is_sql) {
                let path = root.path.join(&entry.relative_path);
                let Ok(content) = std::fs::read_to_string(&path) else {
                    continue;
                };
                for token in extract_table_like_tokens(&content) {
                    edges.push(DependencyEdge {
                        from_file: format!("{}::{}", root.id, entry.relative_path),
                        object_name: token,
                    });
                }
            }
        }
        edges
    }

    pub fn create_sandbox_sqlite_copy(&self, source_db: &Path) -> Result<PathBuf, String> {
        if !self.is_trusted() {
            return Err("trust the workspace before creating a sandbox".to_owned());
        }
        let root = self
            .primary_path()
            .ok_or_else(|| "open a workspace folder first".to_owned())?;
        let sandbox_dir = root.join(".dbpro").join("sandbox");
        std::fs::create_dir_all(&sandbox_dir).map_err(|e| e.to_string())?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let dest = sandbox_dir.join(format!("sandbox-{stamp}.sqlite"));
        std::fs::copy(source_db, &dest).map_err(|e| e.to_string())?;
        Ok(dest)
    }

    pub fn load_notebook(path: &Path) -> Result<Vec<NotebookCell>, String> {
        let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let doc: NotebookDocument = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        Ok(doc
            .cells
            .into_iter()
            .map(|cell| NotebookCell {
                kind: if cell.kind.eq_ignore_ascii_case("sql") {
                    NotebookCellKind::Sql
                } else {
                    NotebookCellKind::Markdown
                },
                source: cell.source,
            })
            .collect())
    }

    pub fn save_notebook(path: &Path, cells: &[NotebookCell]) -> Result<(), String> {
        let doc = NotebookDocument {
            version: 1,
            cells: cells
                .iter()
                .map(|cell| NotebookCellSerde {
                    kind: match cell.kind {
                        NotebookCellKind::Markdown => "markdown".to_owned(),
                        NotebookCellKind::Sql => "sql".to_owned(),
                    },
                    source: cell.source.clone(),
                })
                .collect(),
        };
        let raw = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, raw).map_err(|e| e.to_string())
    }

    pub fn parse_sql_tests(content: &str) -> Vec<(String, String)> {
        // Minimal convention: -- test: name\nSQL until next test marker.
        let mut tests = Vec::new();
        let mut current_name: Option<String> = None;
        let mut current_sql = String::new();
        for line in content.lines() {
            if let Some(rest) = line.trim().strip_prefix("-- test:") {
                if let Some(name) = current_name.take() {
                    tests.push((name, current_sql.trim().to_owned()));
                    current_sql.clear();
                }
                current_name = Some(rest.trim().to_owned());
            } else if current_name.is_some() {
                current_sql.push_str(line);
                current_sql.push('\n');
            }
        }
        if let Some(name) = current_name {
            tests.push((name, current_sql.trim().to_owned()));
        }
        tests
    }

    pub fn apply_workspace_patch(&mut self, relative: &str, new_contents: &str) -> Result<(), String> {
        if !self.is_trusted() {
            return Err("trust the workspace before applying AI patches".to_owned());
        }
        let path = self
            .absolute_for_relative(relative)
            .ok_or_else(|| "missing file".to_owned())?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, new_contents).map_err(|e| e.to_string())?;
        self.refresh()?;
        Ok(())
    }
}

fn extract_table_like_tokens(sql: &str) -> Vec<String> {
    let mut out = Vec::new();
    let upper = sql.to_uppercase();
    for keyword in [" FROM ", " JOIN ", " UPDATE ", " INTO ", " TABLE "] {
        let mut rest = upper.as_str();
        while let Some(idx) = rest.find(keyword) {
            let after = &rest[idx + keyword.len()..];
            let token: String = after
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '.')
                .collect();
            if !token.is_empty() {
                out.push(token.to_lowercase());
            }
            rest = &after[token.len().min(after.len())..];
        }
    }
    out.sort();
    out.dedup();
    out
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
    let entries = std::fs::read_dir(dir).map_err(|error| format!("read {}: {error}", dir.display()))?;
    let mut nodes = Vec::new();
    let mut collected = Vec::new();
    for entry in entries {
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
        if name.starts_with('.') && name != ".env.example" && name != ".dbpro" && name != ".sqlfluff" {
            continue;
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

fn flatten_index(root_id: &str, nodes: &[WorkspaceFileNode]) -> Vec<WorkspaceIndexEntry> {
    let mut out = Vec::new();
    fn walk(root_id: &str, nodes: &[WorkspaceFileNode], out: &mut Vec<WorkspaceIndexEntry>) {
        for node in nodes {
            if node.is_dir {
                walk(root_id, &node.children, out);
            } else {
                out.push(WorkspaceIndexEntry {
                    root_id: root_id.to_owned(),
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
    walk(root_id, nodes, &mut out);
    out
}

/// Back-compat helper used by older call sites / tests.
#[allow(dead_code)]
pub fn search_workspace_files(
    root: &Path,
    index: &[WorkspaceIndexEntry],
    query: &str,
    limit: usize,
) -> Vec<(String, usize, String)> {
    let mut state = IdeWorkspaceState::default();
    let _ = state.open_root(root.to_path_buf());
    let _ = index;
    state
        .search(query, limit)
        .into_iter()
        .map(|hit| (hit.relative_path, hit.line, hit.preview))
        .collect()
}

pub fn fingerprint_schema_names(names: &[String]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    for name in names {
        name.hash(&mut hasher);
    }
    hasher.finish()
}

pub fn measure_local_benchmark(cases: &[BenchmarkCase], iterations: usize) -> Vec<BenchmarkResult> {
    cases
        .iter()
        .map(|case| {
            let mut runs = Vec::new();
            for _ in 0..iterations.max(1) {
                let started = Instant::now();
                // Local CPU stand-in: parse length / hash — real DB timing stays on QueryService.
                let _ = case.sql.len().saturating_mul(17);
                std::thread::sleep(Duration::from_millis(0));
                runs.push(started.elapsed().as_millis() as u64);
            }
            let avg = if runs.is_empty() {
                0
            } else {
                runs.iter().sum::<u64>() / runs.len() as u64
            };
            BenchmarkResult {
                name: case.name.clone(),
                runs,
                avg_ms: avg,
                error: None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("time").as_nanos();
        let dir = std::env::temp_dir().join(format!("dbpro-workspace-{nanos}"));
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
}
